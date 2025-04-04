//! Implementation of [`PageTableEntry`] and [`PageTable`].

use crate::config::{KERNEL_PGNUM_OFFSET, PAGE_SIZE_BITS};
// use crate::hal;
use super::address::KernelAddr;
use super::{frame_alloc, FrameTracker, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum, KERNEL_SPACE};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
// use riscv::register::satp;

// TODO(COW标志位可以设置在这里)
bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    ///PTE
    pub bits: usize,
}

impl PageTableEntry {
    ///Create a PTE from ppn
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    ///Return an empty PTE
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    ///Return 44bit ppn
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    ///Return 10bit flag
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    ///Check PTE valid
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }
    ///Check PTE readable
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    ///Check PTE writable
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    ///Check PTE executable
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

pub unsafe fn switch_pgtable(page_table_token: usize) {
    // unimplemented!()
    // satp::write(page_table_token);
    // asm!("sfence.vma");
    crate::arch::satp_write(page_table_token);
    crate::arch::sfence();
    // hal::arch::switch_pagetable(page_table_token);
}

pub struct PageTable {
    pub root_ppn: PhysPageNum,
    pub frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    pub fn new_from_kernel() -> Self {
        let frame = frame_alloc().unwrap();
        let kernel_page_table = &KERNEL_SPACE.lock().page_table;
        let kernel_root_ppn = kernel_page_table.root_ppn;
        // 第一级页表
        let index = VirtPageNum::from(KERNEL_PGNUM_OFFSET).indexes()[0];
        frame.ppn.get_pte_array()[index..].copy_from_slice(&kernel_root_ppn.get_pte_array()[index..]);
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    #[allow(unused)]
    /// 建立虚拟地址和物理地址的映射
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    #[allow(unused)]
    /// 解除映射
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    /// 根据虚拟地址找到物理地址，翻译
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            //println!("translate_va:va = {:?}", va);
            let aligned_pa: PhysAddr = pte.ppn().into();
            //println!("translate_va:pa_align = {:?}", aligned_pa);
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }
    /// 获取根页表 ppn
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
    pub unsafe fn switch(&self) {
        switch_pgtable(self.token());
    }
}
/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();

    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();

        // 翻译虚拟页号
        let ppn = match page_table.translate(vpn) {
            Some(pte) => pte.ppn(),
            _ => {
                println!("[kernel] mm: 0x{:x} not mapped", start);
                // TODO: 实现lazy分配后，这里是否需要修改
                vpn.step();
                start = vpn.into(); // 跳过未映射的页
                continue;
            }
        };
        vpn.step();
        // 计算当前页的结束地址
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        // 获取字节数组切片
        let slice_start = start_va.page_offset();
        let slice_end = if end_va.page_offset() == 0 {
            ppn.get_bytes_array().len()
        } else {
            end_va.page_offset()
        };
        v.push(&mut ppn.get_bytes_array()[slice_start..slice_end]);

        // 更新起始地址
        start = end_va.into();
    }

    v
}
/// tdranslate a pointer to a const u8 Vec end with `\0` through page table to a `String`
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let ptr: *mut u8 = KernelAddr::from(
        page_table
            .translate_va(VirtAddr::from(ptr as usize))
            .unwrap(),
    )
    .0 as _;
    let mut len: isize = 0;
    unsafe {
        while *ptr.offset(len) != 0 {
            len += 1;
        }
        String::from_utf8(core::slice::from_raw_parts_mut(ptr, len as usize).to_vec()).unwrap()
    }
}
/// translate a generic through page table and return a mutable reference
/// 将一个用户的指针转化为一个可变引用，就能直接操作这个指针指向的内存了
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    //println!("into translated_refmut!");
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;

    // 检查指针
    assert!(ptr as usize % core::mem::align_of::<T>() == 0, "[translated_refmut] ptr not aligned");
    assert!(!ptr.is_null(), "[translated_refmut] ptr is null");

    KernelAddr::from(page_table
        .translate_va(VirtAddr::from(va))
        .expect("[translated_refmut] translate failed"))
        .get_mut()
}
/// 通过token，将一个指针转化为 特定的数据结构
pub fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T {
    //println!("into translated_refmut!");
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;

    KernelAddr::from(page_table
        .translate_va(VirtAddr::from(va))
        .unwrap())
        .get_ref()
}