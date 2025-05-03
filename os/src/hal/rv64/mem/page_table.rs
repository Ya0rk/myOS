/// 更换页表，刷新TLB，开启内存屏障
/// 传入的是satp的值
// pub fn switch_pagetable(satp: usize) {
//     unsafe {
//         satp::write(satp);
//         core::arch::asm!("sfence.vma");
//     }
// }


use crate::mm::{PhysPageNum, VirtAddr, VirtPageNum, PhysAddr};

use crate::hal::config::{PPN_SHIFT, PPN_LEN, PA_LEN, KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET};

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct PTEFlags: usize {
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

impl PTEFlags {
    pub fn RW() -> Self {
        Self::W | Self::R
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
            bits: ppn.0 << PPN_SHIFT | flags.bits() as usize,
        }
    }
    ///Return an empty PTE
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    ///Return 44bit ppn
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> PPN_SHIFT & ((1usize << PPN_LEN) - 1)).into()
    }
    ///Return 10bit flag
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits)
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
        self.bits = ((self.bits >> PPN_SHIFT) << PPN_SHIFT) | flags.bits() as usize;
    }
}

/// translate kernel virtual addr into physical addr
pub fn kaddr_v2p(va: VirtAddr) -> PhysAddr {
    (va.0 - KERNEL_ADDR_OFFSET).into()
}

/// translate kernel virtual page number into physical page number
pub fn kpn_v2p(vpn: VirtPageNum) -> PhysPageNum {
    (vpn.0 - KERNEL_PGNUM_OFFSET).into()
}

/// translate physical addr into kernel virtual addr
pub fn kaddr_p2v(pa: PhysAddr) -> VirtAddr {
    (pa.0 + KERNEL_ADDR_OFFSET).into()
}

/// translate physical page number into kernel virtual page number
pub fn kpn_p2v(ppn: PhysPageNum) -> VirtPageNum {
    (ppn.0 + KERNEL_PGNUM_OFFSET).into()
}