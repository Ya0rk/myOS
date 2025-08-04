//! Implementation of [`PageTableEntry`] and [`PageTable`].

use crate::board::{MEMORY_END, MMIO};
use crate::console::print;
use crate::hal::arch::{kernel_token_write, sfence};
use crate::hal::entry::boot::print_checkpoint;
use crate::mm::{Direct, PageNum};
use core::arch::asm;
use core::ops::Range;
// use crate::mm::address::kva_d2pg;
use super::address::KernelAddr;
use super::memory_space::vm_area::MapPerm;
use super::{frame_alloc, FrameTracker, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use crate::hal::config::{KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET};
use crate::hal::mem::page_table::*;
use crate::sync::SpinNoIrqLock;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use log::info;
use spin::Mutex;


pub unsafe fn switch_user_page_table(user_token: usize) {
    // unimplemented!()
    // satp::write(page_table_token);
    // asm!("sfence.vma");
    crate::hal::arch::user_token_write(user_token);
    crate::hal::arch::sfence();
}

// TODO: 优化结构
lazy_static! {
    pub static ref KERNEL_PAGE_TABLE: Arc<SpinNoIrqLock<PageTable>> =
        Arc::new(SpinNoIrqLock::new(PageTable::init_kernel_page_table()));
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
        // info!("[map_leaf] {:#x} to {:#x}", vpn.0, ppn.0);
        let pte = self.find_pte_create(vpn).unwrap();
        debug_assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
                                                                                  // TODO: to avoid exposed flag bits
        *pte = PageTableEntry::new(ppn, flags);
        // info!("[map_leaf] pte is {:#x}", pte.bits);
    }
    pub fn map_leaf_force(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        // assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // 避免重复映射
        // TODO: to avoid exposed flag bits
        *pte = PageTableEntry::new(ppn, flags);
    }

    pub fn map_kernel_range(&mut self, range_va: Range<VirtAddr>, perm: MapPerm) {
        // println!("[map kernel range] range_va:{:?}", range_va);
        let range_vpn = range_va.start.paged_va().floor()..range_va.end.paged_va().ceil();
        println!(
            "[map_kernel_range] map area:{:#x}..{:#x}",
            range_va.start.paged_va().0,
            range_va.end.paged_va().0
        );
        for vpn in range_vpn {
            let ppn = vpn.ppn();
            // println!("[map vpn] vpn:{:#x}, ppn:{:#x}", vpn.0, ppn.0);
            self.map_leaf(vpn, ppn, perm.into());
        }
    }

    pub fn map_kernel_huge_page(&mut self, base_pa: PhysAddr, perm: MapPerm) {
        // TODO: to avoid exposed flag bits
        assert!(perm.intersects(MapPerm::RWX));
        let base_vpn: VirtPageNum = base_pa.paged_va().into();
        let pte = &mut self.root_ppn.get_pte_array()[base_vpn.indexes()[0]];
        // TODO: to avoid exposed flag bits
        *pte = PageTableEntry::new(base_pa.into(), perm.into());
    }
    pub fn unmap_kernel_range(&mut self, range_va: Range<VirtAddr>) {
        let range_vpn: Range<VirtPageNum> =
            range_va.start.paged_va().floor()..range_va.end.paged_va().ceil();
        info!(
            "[unmap_kernel_range] unmap area:{:#x}..{:#x}",
            range_va.start.paged_va().0,
            range_va.end.paged_va().0
        );
        for vpn in range_vpn {
            let ppn = vpn.ppn();
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
    print_checkpoint(3);
    kernel_token_write(KERNEL_PAGE_TABLE.lock().token());
    print_checkpoint(4);
    sfence();
    print_checkpoint(5);

}
