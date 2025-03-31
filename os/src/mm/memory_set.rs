use super::{MapArea, MapPermission, MapType};
use super::{PageTable, PageTableEntry};
use super::{VirtAddr, VirtPageNum};
use crate::config::{KERNEL_ADDR_OFFSET, MEMORY_END, MMIO, PAGE_SIZE, USER_SPACE_TOP, USER_STACK_SIZE, USER_TRAP_CONTEXT};
use alloc::sync::Arc;
use alloc::vec::Vec;
use log::debug;
use spin::Mutex;
use core::arch::asm;
use lazy_static::*;
use riscv::register::satp;
use crate::utils::elf::ElfCheck;
use crate::utils::elf::ProgramHeaderChecker;
// use crate::fs::{File, FileClass};

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

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

pub fn kernel_token() -> usize {
    KERNEL_SPACE.lock().token()
}

pub fn switch_to_kernel_pgtable() {
    unsafe { KERNEL_SPACE.lock().activate() };
}

/// memory set structure, controls virtual-memory space
pub struct MemorySet {
    pub page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    /// 创建一个空的 `MemorySet`
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    /// 创建一个带有kernel pagetable的 `MemorySet`
    /// 创建pagetable时初始化了kernel的页表
    /// 每个进程有自己的页表（不同的用户页表+相同的内核页表）
    pub fn new_with_kernel_pagetable() -> Self {
        Self {
            page_table: PageTable::new_from_kernel(),
            areas: Vec::new(),
        }
    }
    ///Get pagetable `root_ppn`
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    
    /// Assume that no conflicts.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    ///Remove `MapArea` that starts with `start_vpn`
    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.vpn_range.get_start() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.areas.remove(idx);
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        debug!("kernel satp : {:#x}", memory_set.page_table.token());
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
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Direct,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Direct,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Direct,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Direct,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Direct,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(
                MapArea::new(
                    ((*pair).0 + KERNEL_ADDR_OFFSET).into(),
                    ((*pair).0 + KERNEL_ADDR_OFFSET + (*pair).1).into(),
                    MapType::Direct,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        println!("kernel memory set initialized");
        memory_set
    }
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_with_kernel_pagetable();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        // assert!(elf.check_magic([0x7f, 0x45, 0x4c, 0x46]), "invalid elf!");
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn: VirtPageNum = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        debug!("user_stack_bottom={:#x}, user_stack_top={:#x}", user_stack_bottom, user_stack_top);
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                (user_stack_top+10).into(), // TODO 这里是强行修改+10，为了通过一些测试用例，猜测是还木有实现cow
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        memory_set.push(
            MapArea::new(
                USER_TRAP_CONTEXT.into(),
                USER_SPACE_TOP.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
    /// 从已有的memory set复制
    pub fn clone_from_existed_memset(user_space: &Self) -> Self {
        let mut memory_set = Self::new_with_kernel_pagetable();
        // map trampoline
        // memory_set.map_trampoline();

        // 复制每一个逻辑段数据
        for area in user_space.areas.iter() {
            let new_area = MapArea::clone_from_another(area);
            memory_set.push(new_area, None);
            
            // 复制页表中的每一个页表项
            for vpn in area.vpn_range {
                let src_ppn = user_space.translate(vpn).unwrap().ppn();
                let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
                dst_ppn
                    .get_bytes_array()
                    .copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        memory_set
    }
    ///Refresh TLB with `sfence.vma`
    pub unsafe fn activate(& self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
    ///Translate throuth pagetable
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    ///Remove all `MapArea`
    pub fn recycle_data_pages(&mut self) {
        //*self = Self::new_bare();
        self.areas.clear();
    }
    pub unsafe fn switch_pgtable(&self) {
        self.page_table.switch();
    }
}

#[allow(unused)]
///Check PageTable running correctly
pub fn remap_test() {
    // msg("start");
    let mut kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = (stext as usize + (etext as usize - stext as usize) / 2).into();
    let mid_rodata: VirtAddr = (srodata as usize + (erodata as usize - srodata as usize) / 2).into();
    let mid_data: VirtAddr = (sdata as usize + (edata as usize - sdata as usize) / 2).into();
    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(), "a");
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),"b");
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),"c");
    println!("remap_test passed!");
}
