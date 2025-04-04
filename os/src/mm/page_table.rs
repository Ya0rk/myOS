//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::arch::asm;
use core::ops::Range;
use crate::board::{MEMORY_END, MMIO};
use crate::config::{KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET};
use super::address::{kernel_map_vpn_to_ppn, KernelAddr};
use super::{frame_alloc, FrameTracker, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use log::info;
use spin::Mutex;
// use riscv::register::satp;
// use crate::utils::flags::{AccessFlags, AccessFlagsMut, UserAccessFlags};

// TODO(COW标志位可以设置在这里)
bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct PTEFlags: u16 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
        const COW = 1 << 8;
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
        PTEFlags::from_bits((self.bits & ((1 << 9) - 1)) as u16).unwrap()
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
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.bits = ((self.bits >> 10) << 10) | flags.bits() as usize;
    }
}

// impl AccessFlags for PTEFlags {
//     fn readable(&self) -> bool {
//         self.contains(PTEFlags::R)
//     }
//     fn writable(&self) -> bool {
//         self.contains(PTEFlags::W)
//     }
//     fn executable(&self) -> bool {
//         self.contains(PTEFlags::X)
//     }
//     fn into<T: AccessFlagsMut + AccessFlagsInit>(&self) -> T {
//         let mut flags = T::new();
//         flags.set_readable(self.readable());
//         flags.set_writable(self.writable());
//         flags.set_executable(self.executable());
//         flags
//     }
// }

// impl AccessFlagsMut for PTEFlags {
//     fn set_readable(&mut self, readable: bool) {
//         self.set(PTEFlags::R, readable);
//     }
//     fn set_writable(&mut self, writable: bool) {
//         self.set(PTEFlags::W, writable);
//     }
//     fn set_executable(&mut self, executable: bool) {
//         self.set(PTEFlags::X, executable);
//     }
    
// }

// impl UserAccessFlags for PTEFlags {
//     fn user_accessible(&self) -> bool {
//         self.contains(PTEFlags::U)
//     }
//     fn set_user_accessible(&mut self, user_accessible: bool) {
//         self.set(PTEFlags::U, user_accessible);
//     }
// }




pub unsafe fn switch_pgtable(page_table_token: usize) {
    // unimplemented!()
    // satp::write(page_table_token);
    // asm!("sfence.vma");
    crate::hal::arch::satp_write(page_table_token);
    crate::hal::arch::sfence();
    // hal::arch::switch_pagetable(page_table_token);

}


// TODO: 优化结构

// pub static mut KERNEL_PAGE_TABLE: Option<Arc<Mutex<PageTable>>> = None;
lazy_static! {
    pub static ref KERNEL_PAGE_TABLE: Arc<Mutex<PageTable>> = Arc::new(Mutex::new(PageTable::init_kernel_page_table()));
}

// pub fn get_kernel_page_table() -> &'static Arc<Mutex<PageTable>> {
//     unsafe {KERNEL_PAGE_TABLE.as_ref()}
// }


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
        let kernel_page_table = KERNEL_PAGE_TABLE.lock();
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
                let frame = frame_alloc().expect("no free space to allocate!");
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        // info!("[find_pte] walk vpn {:#x}", vpn.0);
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            // info!("[find_pte] layer {}, pte {:#x}, ppn {:#x}", i, pte.bits, pte.ppn().0);
            if !pte.is_valid() {
                return None;
            }
            if i == 2 {
                result = Some(pte);
                break;
            }
            ppn = pte.ppn();
        }
        result
    }
    // #[allow(unused)]
    /// 建立虚拟地址和物理地址的映射
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        debug_assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn map_force(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        // assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn map_kernel_range(&mut self, range_va: Range<VirtAddr>, flags: PTEFlags) {
        // println!("[map kernel range] range_va:{:?}", range_va);
        let range_vpn = range_va.start.floor()..range_va.end.ceil();
        println!("[map_kernel_range] map area:{:#x}..{:#x}", range_va.start.0, range_va.end.0);
        for vpn in range_vpn {
            let ppn = kernel_map_vpn_to_ppn(vpn);
            // println!("[map vpn] vpn:{:#x}, ppn:{:#x}", vpn.0, ppn.0);
            self.map(vpn, ppn, flags);
        }
    }    
    pub fn unmap_kernel_range(&mut self, range_va: Range<VirtAddr>) {
        let range_vpn = range_va.start.floor()..range_va.end.ceil();
        info!("[unmap_kernel_range] unmap area:{:#x}..{:#x}", range_va.start.0, range_va.end.0);
        for vpn in range_vpn {
            let ppn = kernel_map_vpn_to_ppn(vpn);
            self.unmap(vpn);
        }
    }
    #[allow(unused)]
    /// 解除映射
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).expect("leaf pte is not valid");
        debug_assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
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

    pub fn init_kernel_page_table() -> Self {
        extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn sdata();
            fn edata();
            fn sbss_with_stack();
            fn ebss();
            fn ekernel();
        }
        let mut kernel_page_table = Self::new();
        println!("kernel satp : {:#x}", kernel_page_table.token());
        // map trampoline
        // memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        // memory_set.push(
        //     MapArea::new(
        //         (stext as usize).into(),
        //         (etext as usize).into(),
        //         MapType::Direct,
        //         MapPermission::R | MapPermission::X,
        //     ),
        //     None,
        // );
        println!("aaa");
        kernel_page_table.map_kernel_range(
            (stext as usize).into()..(etext as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
        println!("mapping .rodata section");
        // memory_set.push(
        //     MapArea::new(
        //         (srodata as usize).into(),
        //         (erodata as usize).into(),
        //         MapType::Direct,
        //         MapPermission::R,
        //     ),
        //     None,
        // );
        
        kernel_page_table.map_kernel_range(
            (srodata as usize).into()..(erodata as usize).into(),
            PTEFlags::R,
        );
        println!("mapping .data section");
        // memory_set.push(
        //     MapArea::new(
        //         (sdata as usize).into(),
        //         (edata as usize).into(),
        //         MapType::Direct,
        //         MapPermission::R | MapPermission::W,
        //     ),
        //     None,
        // );
        
        kernel_page_table.map_kernel_range(
            (sdata as usize).into()..(edata as usize).into(),
            PTEFlags::R | PTEFlags::W,
        );
        println!("mapping .bss section");
        // memory_set.push(
        //     MapArea::new(
        //         (sbss_with_stack as usize).into(),
        //         (ebss as usize).into(),
        //         MapType::Direct,
        //         MapPermission::R | MapPermission::W,
        //     ),
        //     None,
        // );
        
        kernel_page_table.map_kernel_range(
            (sbss_with_stack as usize).into()..(ebss as usize).into(),
            PTEFlags::R | PTEFlags::W,
        );
        println!("mapping physical memory");
        // memory_set.push(
        //     MapArea::new(
        //         (ekernel as usize).into(),
        //         MEMORY_END.into(),
        //         MapType::Direct,
        //         MapPermission::R | MapPermission::W,
        //     ),
        //     None,
        // );
        
        kernel_page_table.map_kernel_range(
            (ekernel as usize).into()..(MEMORY_END).into(),
            PTEFlags::R | PTEFlags::W,
        );
        println!("mapping memory-mapped registers");
        // for pair in MMIO {
        //     memory_set.push(
        //         MapArea::new(
        //             ((*pair).0 + KERNEL_ADDR_OFFSET).into(),
        //             ((*pair).0 + KERNEL_ADDR_OFFSET + (*pair).1).into(),
        //             MapType::Direct,
        //             MapPermission::R | MapPermission::W,
        //         ),
        //         None,
        //     );
        // }
        for pair in MMIO {
            let base = (*pair).0 + KERNEL_ADDR_OFFSET;
            kernel_page_table.map_kernel_range(
                base.into()..(base + (*pair).1).into(),
                PTEFlags::R | PTEFlags::W,
            );
        }
        println!("kernel memory set initialized");
        kernel_page_table
    }
}

pub fn switch_to_kernel_pgtable() {
    unsafe { KERNEL_PAGE_TABLE.lock().switch(); }
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
///translate a generic through page table and return a mutable reference
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    //println!("into translated_refmut!");
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;

    KernelAddr::from(page_table
        .translate_va(VirtAddr::from(va))
        .unwrap())
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

