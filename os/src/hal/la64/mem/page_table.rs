
use crate::mm::memory_space::vm_area::MapPerm;
use crate::mm::{PhysPageNum, VirtAddr, VirtPageNum, PhysAddr, PageTable};

use crate::hal::config::{KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET};
// use paste::paste;
use crate::{impl_flag_checker, impl_flag_setter};

/*

TODO:
- kernel page table
- tlb refill
- compatibility check
- asid
- 




*/



bitflags!{
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct PTEFlags: usize {
        /// valid
        const V = 1 << 0;

        /// dirty
        const D = 1 << 1;

        /// privilege 
        const PLV_USER = 0b11 << 2;
        /// should never be used
        const PLV_KERN = 0b00 << 2;

        /// cache type: coherent cached
        const MAT_CC = 0b01 << 4;

        /// cache type: strongly-ordered uncached
        const MAT_SUC = 0b00 << 4;

        /// cached type: weakly-ordered uncached
        /// NOTE: should never be used
        const MAT_WUC = 0b10 << 4;

        /// cached type: preserved
        /// NOTE: used to clear MAT flag bits
        const MAT_P = 0b11 << 4;

        /// Designates a global mapping OR Whether the page is huge page.
        const G = 1 << 6;

        /// Page is existing.
        const P = 1 << 7;
        /// Page is writeable.
        const W = 1 << 8;
        /// Mapping to this page is copied but not yet the page itself
        const COW = 1 << 9;
        /// Is a Global Page if using huge page(GH bit).
        // const G = 1 << 12;
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


impl PTEFlags {

    impl_flag_checker!(
        PLV_USER,
        pub V,
        NR,
        pub W,
        NX,
        pub G,
        pub D,
        pub COW,
        pub RPLV
    );

    impl_flag_setter!(
        PLV_USER,
        MAT_CC,
        MAT_SUC,
        MAT_WUC,
        pub V,
        NR,
        pub W,
        NX,
        pub G,
        pub D,
        pub COW,
        pub RPLV
    );
    pub fn new_valid() -> Self {
        let flags = Self::V;
        flags
    }
    pub fn is_U(&self) -> bool {
        self.is_PLV_USER()
    }
    pub fn is_R(&self) -> bool {
        !self.is_NR()
    }
    pub fn is_X(&self) -> bool {
        !self.is_NX()
    }
    pub fn set_U(&mut self, val: bool) -> &mut Self {
        self.set_PLV_USER(val)
    }
    pub fn set_R(&mut self, val: bool) -> &mut Self {
        self.set_NR(!val)
    }
    pub fn set_X(&mut self, val: bool) -> &mut Self {
        self.set_NX(!val)
    }


    /// Must clear bits before set
    pub fn clear_MAT(&mut self) -> &mut Self {
        self.remove(Self::MAT_P);
        self
    }
    pub fn enable_MAT_CC(&mut self) -> &mut Self {
        self.clear_MAT().set_MAT_CC(true)
    }
    pub fn enable_MAT_SUC(&mut self) -> &mut Self {
        self.clear_MAT().set_MAT_SUC(true)
    }
    pub fn enable_MAT_WUC(&mut self) -> &mut Self {
        self.clear_MAT().set_MAT_WUC(true)
    }
}

// impl_flag_set!(PTEFlags, PLV_USER);
// impl_flag_set!(PTEFlags, V);
// impl_flag_set!(PTEFlags, NR);
// impl_flag_set!(PTEFlags, W);
// impl_flag_set!(PTEFlags, NX);
// impl_flag_set!(PTEFlags, GH);
// impl_flag_set!(PTEFlags, COW);
// impl_flag_check!(PTEFlags, PLV_USER);
// impl_flag_check!(PTEFlags, V);
// impl_flag_check!(PTEFlags, NR);
// impl_flag_check!(PTEFlags, W);
// impl_flag_check!(PTEFlags, NX);
// impl_flag_check!(PTEFlags, GH);
// impl_flag_check!(PTEFlags, COW);


impl From<MapPerm> for PTEFlags {
    fn from(perm: MapPerm) -> Self {
        *Self::new_valid()
            .set_U( perm.contains(MapPerm::U) )
            .set_G(!perm.contains(MapPerm::U) )
            .set_R( perm.contains(MapPerm::R) )
            .set_W( perm.contains(MapPerm::W) )
            .set_D( perm.contains(MapPerm::W) )
            .set_X( perm.contains(MapPerm::X) )
            .set_RPLV( false )
        

        // ret |= PTEFlags::V;
        // if perm.contains(MapPerm::U) {
        //     ret |= PTEFlags::U;
        // } else {
        //     ret |= PTEFlags::G;
        // }
        // if perm.contains(MapPerm::R) {
        //     ret |= PTEFlags::R;
        // }
        // if perm.contains(MapPerm::W) {
        //     ret |= PTEFlags::W;
        // }
        // if perm.contains(MapPerm::X) {
        //     ret |= PTEFlags::X;
        // }
        // ret
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    ///PTE
    pub bits: usize,
}

const PPN_SHIFT: usize = 12;
const PA_LEN: usize = 56;
const PPN_LEN: usize = 44;

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
        self.flags().is_V()
    }
    ///Check PTE readable
    pub fn readable(&self) -> bool {
        self.flags().is_R()
    }
    ///Check PTE writable
    pub fn writable(&self) -> bool {
        self.flags().is_W()
    }
    ///Check PTE executable
    pub fn executable(&self) -> bool {
        self.flags().is_X()
    }
    pub fn set_flags(&mut self, flags: PTEFlags) {
        self.bits = ((self.bits >> PPN_SHIFT) << PPN_SHIFT) | flags.bits() as usize;
    }
}

impl PageTable {
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
        // println!("aaa");
        // TODO: to avoid exposed flag bits
        kernel_page_table.map_kernel_range(
            (stext as usize).into()..(etext as usize).into(),
            MapPerm::R | MapPerm::X,
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
        // TODO: to avoid exposed flag bits
        kernel_page_table.map_kernel_range(
            (srodata as usize).into()..(erodata as usize).into(),
            MapPerm::R,
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
        // TODO: to avoid exposed flag bits
        kernel_page_table.map_kernel_range(
            (sdata as usize).into()..(edata as usize).into(),
            MapPerm::R | MapPerm::W,
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
        // TODO: to avoid exposed flag bits
        kernel_page_table.map_kernel_range(
            (sbss_with_stack as usize).into()..(ebss as usize).into(),
            MapPerm::R | MapPerm::W,
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
        // TODO: la: use direct mapping instead of page mapping
        // TODO: to avoid exposed flag bits
        // kernel_page_table.map_kernel_range(
        //     (ekernel as usize).into()..(MEMORY_END).into(),
        //     PTEFlags::R | PTEFlags::W,
        // );
        // ffff_ffc0_8020_0000
        // ffff_ffc0_8800_0000


        // println!("mapping devices");
        // NOTE: LA架构不使用巨页，甚至不使用页表实现直接映射，而是使用0x8000_0000_0000_0000的直接映射窗口，其为强序非缓存的。
        // 映射两个巨页，0x0000_0000~0x8000_0000，作为设备保留区

        // SCRIPT: remove device space mapping from pgtbl, which has been taken over by direct mapping currently

        // kernel_page_table.map_kernel_huge_page(
        //     (0x0000_0000).into(),
        //     PTEFlags::R | PTEFlags::W
        // );
        // kernel_page_table.map_kernel_huge_page(
        //     (0x4000_0000).into(),
        //     PTEFlags::R | PTEFlags::W
        // );
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
        // for pair in MMIO {
        //     let base = (*pair).0 + KERNEL_ADDR_OFFSET;
        //     kernel_page_table.map_kernel_range(
        //         base.into()..(base + (*pair).1).into(),
        //         PTEFlags::R | PTEFlags::W,
        //     );
        // }
        println!("kernel memory set initialized");
        kernel_page_table
    }


    // NOTE: user pagetable should be initialized as a bare one
    pub fn new_user() -> Self {
        Self::new()
    }
    /// 获取根页表 ppn
    pub fn token(&self) -> usize {
        // todo!("to adapt la");
        self.root_ppn.0
    }
}