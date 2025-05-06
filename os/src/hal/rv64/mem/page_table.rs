/// 更换页表，刷新TLB，开启内存屏障
/// 传入的是satp的值
// pub fn switch_pagetable(satp: usize) {
//     unsafe {
//         satp::write(satp);
//         core::arch::asm!("sfence.vma");
//     }
// }



use crate::board::MEMORY_END;
use crate::mm::memory_space::vm_area::MapPerm;
use crate::mm::{PhysPageNum, VirtAddr, VirtPageNum, PhysAddr, PageTable};

use crate::hal::config::{PPN_SHIFT, PPN_LEN, PA_LEN, KERNEL_ADDR_OFFSET, KERNEL_PGNUM_OFFSET};
// use paste::paste;
use crate::{impl_flag_checker, impl_flag_setter};


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

// macro_rules! impl_flag_check {
//     ($class:ident, $flag:ident) => {
//         paste!{
//             impl $class {
//                 pub fn [<is_ $flag>](&self) -> bool {
//                     self.contains(Self::$flag)
//                 }
//             }
//         }
//     };
// }

// macro_rules! impl_flag_set {
//     ($class:ident, $flag:ident) => {
//         paste!{   
//             impl $class {
//                 pub fn [<set_ $flag>](&mut self, val: bool) -> &mut Self {
//                     self.set(Self::$flag, val);
//                     self
//                 }
//             }
//         }
//     };
// }

impl PTEFlags {

    impl_flag_checker!(
        pub U,
        pub V,
        pub R,
        pub W,
        pub X,
        pub G,
        pub COW
    );

    impl_flag_setter!(
        pub U,
        pub V,
        pub R,
        pub W,
        pub X,
        pub G,
        pub COW
    );

    pub fn new_valid() -> Self {
        let flags = Self::V;
        flags
    }
    
}



impl From<MapPerm> for PTEFlags {
    fn from(perm: MapPerm) -> Self {
        *Self::new_valid()
            .set_U( perm.contains(MapPerm::U) )
            .set_G(!perm.contains(MapPerm::U) )
            .set_R( perm.contains(MapPerm::R) )
            .set_W( perm.contains(MapPerm::W) )
            .set_X( perm.contains(MapPerm::X) )
        

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
            MapPerm::RX,
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
            MapPerm::RW,
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
            MapPerm::RW,
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
        kernel_page_table.map_kernel_range(
            (ekernel as usize).into()..(MEMORY_END).into(),
            MapPerm::RW,
        );
        // ffff_ffc0_8020_0000
        // ffff_ffc0_8800_0000
        println!("mapping devices");
        // 映射两个巨页，0x0000_0000~0x8000_0000，作为设备保留区
        // TODO: to adapt la
        kernel_page_table.map_kernel_huge_page(
            (0x0000_0000).into(),
            MapPerm::RW
        );
        kernel_page_table.map_kernel_huge_page(
            (0x4000_0000).into(),
            MapPerm::RW
        );
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
}




// /// translate kernel virtual addr into physical addr
// pub fn kaddr_v2p(va: VirtAddr) -> PhysAddr {
//     (va.0 - KERNEL_ADDR_OFFSET).into()
// }

// /// translate kernel virtual page number into physical page number
// pub fn kpn_v2p(vpn: VirtPageNum) -> PhysPageNum {
//     (vpn.0 - KERNEL_PGNUM_OFFSET).into()
// }

// /// translate physical addr into kernel virtual addr
// pub fn kaddr_p2v(pa: PhysAddr) -> VirtAddr {
//     (pa.0 + KERNEL_ADDR_OFFSET).into()
// }

// /// translate physical page number into kernel virtual page number
// pub fn kpn_p2v(ppn: PhysPageNum) -> VirtPageNum {
//     (ppn.0 + KERNEL_PGNUM_OFFSET).into()
// }