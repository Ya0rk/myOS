use crate::mm::address::*;


bitflags!{
    pub struct PTEFlags: usize {
        /// valid
        const V = 1 << 0;

        /// dirty
        const D = 1 << 1;

        /// privilege
        const PLV_USER = 0b11 << 2;
        const PLV_KERN = 0b00 << 2;

        /// cache type
        const MAT_NOCACHE = 0b01 << 4;

        /// Designates a global mapping OR Whether the page is huge page.
        const GH = 1 << 6;

        /// Page is existing.
        const P = 1 << 7;
        /// Page is writeable.
        const W = 1 << 8;
        /// Mapping to this page is copied but not yet the page itself
        const COW = 1 << 9;
        /// Is a Global Page if using huge page(GH bit).
        const G = 1 << 12;
        /// Page is not readable.
        const NR = 1 << 61;
        /// Page is not executable.
        /// FIXME: Is it just for a huge page?
        /// Linux related url: https://github.com/torvalds/linux/blob/master/arch/loongarch/include/asm/pgtable-bits.h
        const NX = 1 << 62;
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = 1 << 63;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    ///PTE
    pub bits: usize,
}

const static PPN_SHIFT: usize = 12;
const static PA_LEN: usize = 56;
const static PPN_LEN: usize = 44;

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
        !self.flags().contains(PTEFlags::NR)
    }
    ///Check PTE writable
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    ///Check PTE executable
    pub fn executable(&self) -> bool {
        !self.flags().contains(PTEFlags::NX)
    }
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.bits = (self.ppn() << PPN_SHIFT) | flags.bits() as usize;
    }
}
