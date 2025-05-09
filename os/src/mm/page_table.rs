//! Implementation of [`PageTableEntry`] and [`PageTable`].

use core::arch::asm;
use core::ops::Range;
use crate::board::{MEMORY_END, MMIO};
use crate::hal::arch::kernel_token_write;
use super::address::{kaddr_p2v, kpn_v2p, KernelAddr};
use super::memory_space::vm_area::MapPerm;
use crate::hal::config::{KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET};
use crate::hal::mem::page_table::*;
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
// bitflags! {
//     #[derive(Debug, Copy, Clone, PartialEq, Eq)]
//     pub struct PTEFlags: u16 {
//         const V = 1 << 0;
//         const R = 1 << 1;
//         const W = 1 << 2;
//         const X = 1 << 3;
//         const U = 1 << 4;
//         const G = 1 << 5;
//         const A = 1 << 6;
//         const D = 1 << 7;
//         const COW = 1 << 8;
//     }
// }

// #[derive(Copy, Clone)]
// #[repr(C)]
// /// page table entry structure
// pub struct PageTableEntry {
//     ///PTE
//     pub bits: usize,
// }

// impl PageTableEntry {
//     ///Create a PTE from ppn
//     pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
//         PageTableEntry {
//             bits: ppn.0 << 10 | flags.bits() as usize,
//         }
//     }
//     ///Return an empty PTE
//     pub fn empty() -> Self {
//         PageTableEntry { bits: 0 }
//     }
//     ///Return 44bit ppn
//     pub fn ppn(&self) -> PhysPageNum {
//         (self.bits >> 10 & ((1usize << 44) - 1)).into()
//     }
//     ///Return 10bit flag
//     pub fn flags(&self) -> PTEFlags {
//         PTEFlags::from_bits((self.bits & ((1 << 9) - 1)) as u16).unwrap()
//     }
//     ///Check PTE valid
//     pub fn is_valid(&self) -> bool {
//         self.flags().contains(PTEFlags::V)
//     }
//     ///Check PTE readable
//     pub fn readable(&self) -> bool {
//         self.flags().contains(PTEFlags::R)
//     }
//     ///Check PTE writable
//     pub fn writable(&self) -> bool {
//         self.flags().contains(PTEFlags::W)
//     }
//     ///Check PTE executable
//     pub fn executable(&self) -> bool {
//         self.flags().contains(PTEFlags::X)
//     }
//     pub fn set_flags(&mut self, flags: PTEFlags) {
//         self.bits = ((self.bits >> 10) << 10) | flags.bits() as usize;
//     }
// }

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




pub unsafe fn switch_user_page_table(user_token: usize) {
    // unimplemented!()
    // satp::write(page_table_token);
    // asm!("sfence.vma");
    crate::hal::arch::user_token_write(user_token);
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


    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            // TODO: to adapt la
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    // TODO: to refactor: whether pte is created during fn call or not is undefined
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
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::new_valid());
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    // TODO: to refactor
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
    pub fn map_leaf(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        debug_assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
        // TODO: to avoid exposed flag bits
        *pte = PageTableEntry::new(ppn, flags);
    }
    pub fn map_leaf_force(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        // assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
        // TODO: to avoid exposed flag bits
        *pte = PageTableEntry::new(ppn, flags);
    }

    pub fn map_kernel_range(&mut self, range_va: Range<VirtAddr>, perm: MapPerm) {
        // println!("[map kernel range] range_va:{:?}", range_va);
        let range_vpn = range_va.start.floor()..range_va.end.ceil();
        println!("[map_kernel_range] map area:{:#x}..{:#x}", range_va.start.0, range_va.end.0);
        for vpn in range_vpn {
            let ppn = kpn_v2p(vpn);
            // println!("[map vpn] vpn:{:#x}, ppn:{:#x}", vpn.0, ppn.0);
            self.map_leaf(vpn, ppn, perm.into());
        }
    }    

    pub fn map_kernel_huge_page(&mut self, base_pa: PhysAddr, perm: MapPerm) {
        // TODO: to avoid exposed flag bits
        assert!(perm.intersects(MapPerm::RWX));
        let base_vpn: VirtPageNum = kaddr_p2v(base_pa).into();
        let pte = &mut self.root_ppn.get_pte_array()[base_vpn.indexes()[0]];
        // TODO: to avoid exposed flag bits
        *pte = PageTableEntry::new(base_pa.into(), perm.into());
    }
    pub fn unmap_kernel_range(&mut self, range_va: Range<VirtAddr>) {
        let range_vpn: Range<VirtPageNum> = range_va.start.floor()..range_va.end.ceil();
        info!("[unmap_kernel_range] unmap area:{:#x}..{:#x}", range_va.start.0, range_va.end.0);
        for vpn in range_vpn {
            let ppn = kpn_v2p(vpn);
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


    pub unsafe fn enable(&self) {
        switch_user_page_table(self.token());
    }

}



/// NOTE: should be used no more than init phase
pub fn enable_kernel_pgtable() {
    // unsafe { KERNEL_PAGE_TABLE.lock().enable(); }
    kernel_token_write( KERNEL_PAGE_TABLE.lock().token() );

}
// TODO: all below is to be discarded
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

