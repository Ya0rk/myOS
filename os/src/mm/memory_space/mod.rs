pub mod vm_area;
use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use core::{
    cell::SyncUnsafeCell,
    cmp,
    ops::{Range, RangeBounds},
};
use log::info;

// use core::arch::riscv64::sfence_vma_vaddr; 关于core::arch::riscv64::中的内容会在crate::hal::arch中统一引入
use crate::{
    fs::{open, resolve_path, OpenFlags},
    hal::arch::sfence_vma_vaddr,
    task::{aux, current_task},
};
// use riscv::register::scause; 将从riscv库引入scause替换为从hal::arch引入。在hal::arch中会间接引入riscv::register::scause
// use crate::hal::arch::scause;
// use async_utils::block_on;
use crate::{
    fs::{FileClass, FileTrait},
    hal::config::{
        align_down_by_page, is_aligned_to_page, DL_INTERP_OFFSET, MMAP_PRE_ALLOC_PAGES, PAGE_SIZE,
        USER_ELF_PRE_ALLOC_PAGE_CNT, USER_STACK_PRE_ALLOC_SIZE, USER_STACK_SIZE, U_SEG_FILE_BEG,
        U_SEG_FILE_END, U_SEG_HEAP_BEG, U_SEG_HEAP_END, U_SEG_SHARE_BEG, U_SEG_SHARE_END,
        U_SEG_STACK_BEG, U_SEG_STACK_END,
    },
    sync::block_on,
};

// use memory::{pte::PTEFlags, PageTable, PhysAddr, VirtAddr, VirtPageNum};
use self::vm_area::VmArea;
use super::address::{PhysAddr, VirtAddr, VirtPageNum};
use super::page::Page;
use super::page_table::PageTable;
use crate::hal::mem::page_table::PTEFlags;
use crate::utils::container::range_map::RangeMap;
use crate::utils::{Errno, SysResult};
use crate::{
    mm::memory_space::vm_area::{MapPerm, VmAreaType},
    task::{
        aux::{generate_early_auxv, AuxHeader, AT_BASE, AT_NULL, AT_PHDR, AT_RANDOM},
        TaskControlBlock,
    },
};
use xmas_elf::ElfFile;

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
    fn initproc_start();
    fn initproc_end();
}
bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PageFaultAccessType: u8 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
    }
}

impl PageFaultAccessType {
    pub const RO: Self = Self::READ;
    pub const RW: Self = Self::RO.union(Self::WRITE);
    pub const RX: Self = Self::RO.union(Self::EXECUTE);

    // pub fn from_exception(e: scause::Exception) -> Self {
    //     match e {
    //         scause::Exception::InstructionPageFault => Self::RX,
    //         scause::Exception::LoadPageFault => Self::RO,
    //         scause::Exception::StorePageFault => Self::RW,
    //         _ => panic!("unexcepted exception type for PageFaultAccessType"),
    //     }
    // }

    pub fn can_access(self, flag: MapPerm) -> bool {
        if self.contains(Self::WRITE) && !flag.contains(MapPerm::W) {
            return false;
        }
        if self.contains(Self::EXECUTE) && !flag.contains(MapPerm::X) {
            return false;
        }
        true
    }
}

bitflags! {
    // Defined in <bits/mman-linux.h>
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MmapFlags: i32 {
        // Sharing types (must choose one and only one of these).
        /// Share changes.
        const MAP_SHARED = 0x01;
        /// Changes are private.
        const MAP_PRIVATE = 0x02;
        /// Share changes and validate
        const MAP_SHARED_VALIDATE = 0x03;
        const MAP_TYPE_MASK = 0x03;

        // Other flags
        /// Interpret addr exactly.
        const MAP_FIXED = 0x10;
        /// Don't use a file.
        const MAP_ANONYMOUS = 0x20;
        /// Don't check for reservations.
        const MAP_NORESERVE = 0x04000;
    }
}

bitflags! {
    // Defined in <bits/mman-linux.h>
    // NOTE: Zero bit flag is discouraged. See https://docs.rs/bitflags/latest/bitflags/#zero-bit-flags
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MmapProt: i32 {
        /// Page can be read.
        const PROT_READ = 0x1;
        /// Page can be written.
        const PROT_WRITE = 0x2;
        /// Page can be executed.
        const PROT_EXEC = 0x4;
    }
}

impl From<MmapProt> for MapPerm {
    fn from(prot: MmapProt) -> Self {
        let mut ret = Self::U;
        if prot.contains(MmapProt::PROT_READ) {
            ret |= Self::R;
        }
        if prot.contains(MmapProt::PROT_WRITE) {
            ret |= Self::W;
        }
        if prot.contains(MmapProt::PROT_EXEC) {
            ret |= Self::X;
        }
        ret
    }
}

pub struct MemorySpace {
    // NOTE: The reason why `page_table` and `areas` are `SyncUnsafeCell` is because they both
    // represent memory region, it is likely to modify the two both.
    /// Page table of this memory space.
    page_table: SyncUnsafeCell<PageTable>,
    /// Map of `VmArea`s in this memory space.
    /// NOTE: stores range that is lazy allocated
    areas: SyncUnsafeCell<RangeMap<VirtAddr, VmArea>>,
}

impl MemorySpace {
    /// Create an empty `MemorySpace`
    pub fn new() -> Self {
        Self {
            page_table: SyncUnsafeCell::new(PageTable::new()),
            areas: SyncUnsafeCell::new(RangeMap::new()),
        }
    }

    /// Create a new user memory space that inherits kernel page table.
    pub fn new_user() -> Self {
        Self {
            page_table: SyncUnsafeCell::new(PageTable::new_user()),
            areas: SyncUnsafeCell::new(RangeMap::new()),
        }
    }

    pub fn areas(&self) -> &RangeMap<VirtAddr, VmArea> {
        unsafe { &*self.areas.get() }
    }

    pub fn areas_mut(&self) -> &mut RangeMap<VirtAddr, VmArea> {
        unsafe { &mut *self.areas.get() }
    }

    pub fn page_table(&self) -> &PageTable {
        unsafe { &*self.page_table.get() }
    }

    pub fn page_table_mut(&self) -> &mut PageTable {
        unsafe { &mut *self.page_table.get() }
    }

    pub fn token(&self) -> usize {
        self.page_table().token()
    }

    pub async fn new_user_from_elf(
        elf_file: Arc<dyn FileTrait>,
    ) -> (Self, usize, usize, Vec<AuxHeader>) {
        let elf_data = elf_file
            .get_inode()
            .read_all()
            .await
            .expect("[new_user_from_elf] read elf file failed");
        let (mut memory_space, entry_point, auxv) =
            MemorySpace::new_user().parse_and_map_elf_data(&elf_data);
        let sp_init = memory_space.alloc_stack(USER_STACK_SIZE).into();
        memory_space.alloc_heap();
        (memory_space, entry_point, sp_init, auxv)
    }
    pub async fn new_user_from_elf_lazily(
        elf_file: Arc<dyn FileTrait>,
    ) -> (Self, usize, usize, Vec<AuxHeader>) {
        let elf_data = elf_file
            .get_inode()
            .read_all()
            .await
            .expect("[new_user_from_elf_lazily] read elf file failed");
        let (mut memory_space, entry_point, auxv) =
            MemorySpace::new_user().parse_and_map_elf(elf_file, &elf_data);
        let sp_init = memory_space.alloc_stack_lazily(USER_STACK_SIZE).into();
        memory_space.alloc_heap_lazily();
        (memory_space, entry_point, sp_init, auxv)
    }
    /// Include sections in elf and TrapContext and user stack,
    /// also returns user_sp and entry point.
    // PERF: resolve elf file lazily
    // TODO: dynamic interpreter
    pub fn parse_and_map_elf_data(mut self, elf_data: &[u8]) -> (Self, usize, Vec<AuxHeader>) {
        const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        assert_eq!(elf_header.pt1.magic, ELF_MAGIC, "invalid elf!");
        let entry_point = elf_header.pt2.entry_point() as usize;
        let ph_entry_size = elf_header.pt2.ph_entry_size() as usize;
        let ph_count = elf_header.pt2.ph_count() as usize;

        let mut auxv = generate_early_auxv(ph_entry_size, ph_count, entry_point);

        auxv.push(AuxHeader::new(AT_BASE, 0));

        let (max_end_vpn, header_va) = self.map_elf_data(&elf, 0.into());

        let ph_head_addr = header_va.0 + elf.header.pt2.ph_offset() as usize;
        log::info!("[from_elf] AT_PHDR  ph_head_addr is {ph_head_addr:x} ");
        auxv.push(AuxHeader::new(AT_PHDR, ph_head_addr));

        (self, entry_point, auxv)
    }

    pub fn map_elf_data(&mut self, elf: &ElfFile, offset: VirtAddr) -> (VirtPageNum, VirtAddr) {
        let elf_header = elf.header;
        let ph_count = elf_header.pt2.ph_count();

        let mut max_end_vpn = offset.floor();
        let mut header_va = 0;
        let mut has_found_header_va = false;
        log::info!(
            "[map_elf_data]: entry point {:#x}",
            elf.header.pt2.entry_point()
        );

        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }
            let start_va: VirtAddr = (ph.virtual_addr() as usize + offset.0).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize + offset.0).into();
            if !has_found_header_va {
                header_va = start_va.0;
                has_found_header_va = true;
            }
            let mut map_perm = MapPerm::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                map_perm |= MapPerm::R;
            }
            if ph_flags.is_write() {
                map_perm |= MapPerm::W;
            }
            if ph_flags.is_execute() {
                map_perm |= MapPerm::X;
            }
            let mut vm_area = VmArea::new(start_va..end_va, map_perm, VmAreaType::Elf);

            // log::debug!("[map_elf] [{start_va:#x}, {end_va:#x}], map_perm: {map_perm:?} start...",);

            max_end_vpn = vm_area.end_vpn();

            let map_offset = start_va - start_va.align_down();

            log::info!(
                "[map_elf_data] ph offset {:#x}, file size {:#x}, mem size {:#x}",
                ph.offset(),
                ph.file_size(),
                ph.mem_size()
            );

            self.push_vma_with_data(
                vm_area,
                map_offset,
                &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
            );
        }

        (max_end_vpn, header_va.into())
    }

    /// Map the sections in the elf.
    ///
    /// Return the max end vpn and the first section's va.
    pub fn map_elf(
        &mut self,
        elf_file: Arc<dyn FileTrait>,
        elf: &ElfFile,
        offset: VirtAddr,
    ) -> (VirtPageNum, VirtAddr) {
        let elf_header = elf.header;
        let ph_count = elf_header.pt2.ph_count();

        let mut max_end_vpn = offset.floor();
        let mut header_va = 0;
        let mut has_found_header_va = false;
        // log::info!("[map_elf]: entry point {:#x}", elf.header.pt2.entry_point());

        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }
            let start_va: VirtAddr = (ph.virtual_addr() as usize + offset.0).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize + offset.0).into();
            if !has_found_header_va {
                header_va = start_va.0;
                has_found_header_va = true;
            }
            let mut map_perm = MapPerm::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() {
                map_perm |= MapPerm::R;
            }
            if ph_flags.is_write() {
                map_perm |= MapPerm::W;
            }
            if ph_flags.is_execute() {
                map_perm |= MapPerm::X;
            }
            let mut vm_area = VmArea::new(start_va..end_va, map_perm, VmAreaType::Elf);

            max_end_vpn = vm_area.end_vpn();

            let map_offset = start_va - start_va.align_down();

            // log::info!(
            //     "[map_elf] ph offset {:#x}, file size {:#x}, mem size {:#x}, flags{}",
            //     ph.offset(),
            //     ph.file_size(),
            //     ph.mem_size(),
            //     ph_flags
            // );

            if ph.file_size() == ph.mem_size() && is_aligned_to_page(ph.offset() as usize) {
                // assert!(!map_perm.contains(MapPerm::W));
                // NOTE: only add cow flag in elf page newly mapped.
                // FIXME: mprotect is not checked yet
                // WARN: the underlying elf file page cache may be edited, may cause unknown
                // behavior
                let mut pre_alloc_page_cnt = 0;
                for vpn in vm_area.range_vpn() {
                    let start_offset = ph.offset() as usize;
                    let offset = start_offset + (vpn - vm_area.start_vpn()) * PAGE_SIZE;
                    let offset_aligned = align_down_by_page(offset);
                    // if let Ok(page) = block_on(async { elf_file.get_page_at(offset_aligned).await })
                    if let Some(page) =
                        block_on(async { elf_file.get_page_at(offset_aligned).await })
                    //
                    {
                        if pre_alloc_page_cnt < USER_ELF_PRE_ALLOC_PAGE_CNT {
                            let new_page = Page::new();
                            info!("pre alloc");
                            // WARN: area outer than region may should be set to zero
                            new_page.copy_from_slice(page.get_bytes_array());
                            self.page_table_mut()
                                .map_leaf(vpn, new_page.ppn(), map_perm.into());
                            vm_area.pages.insert(vpn, new_page);
                            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                        } else {
                            let (pte_flags, ppn) = {
                                let mut new_flags: PTEFlags = map_perm.into();
                                new_flags |= PTEFlags::COW;
                                new_flags.remove(PTEFlags::W);
                                (new_flags, page.ppn())
                            };
                            // info!("[map_elf] lazy alloc: vpn: {:#x} offset: {:#x} ppn: {:#x}", vpn.0, offset_aligned, ppn.0 );
                            self.page_table_mut().map_leaf(vpn, ppn, pte_flags);
                            vm_area.pages.insert(vpn, page);
                            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                        }
                        pre_alloc_page_cnt += 1;
                    } else {
                        info!("break");
                        break;
                    }
                }
                self.push_vma_lazily(vm_area);
            } else {
                self.push_vma_with_data(
                    vm_area,
                    map_offset,
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
            }
        }

        (max_end_vpn, header_va.into())
    }

    pub fn parse_and_map_elf(
        mut self,
        elf_file: Arc<dyn FileTrait>,
        elf_data: &[u8],
    ) -> (Self, usize, Vec<AuxHeader>) {
        const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap_or_else(|err| {
            println!("[parse_and_map_elf] file is {:?}", elf_file.get_name());
            panic!("parse elf failed: {err}");
        });
        let elf_header = elf.header;
        assert_eq!(elf_header.pt1.magic, ELF_MAGIC, "invalid elf!");
        let mut entry = elf_header.pt2.entry_point() as usize;
        let ph_entry_size = elf_header.pt2.ph_entry_size() as usize;
        let ph_count = elf_header.pt2.ph_count() as usize;

        let mut auxv = generate_early_auxv(ph_entry_size, ph_count, entry);

        // maybe needed?
        // auxv.push(AuxHeader::new(AT_BASE, 0));

        if let Some(interp_entry) = self.load_dl_interp_if_needed(&elf).unwrap_or(None) {
            auxv.push(AuxHeader::new(AT_BASE, DL_INTERP_OFFSET));
            entry = interp_entry;
        } else {
            auxv.push(AuxHeader::new(AT_BASE, 0));
        }

        let (_max_end_vpn, header_va) = self.map_elf(elf_file, &elf, 0.into());

        let ph_head_addr = header_va.0 + elf.header.pt2.ph_offset() as usize;
        auxv.push(AuxHeader::new(AT_RANDOM, ph_head_addr));
        // log::info!("[parse_and_map_elf] AT_PHDR  ph_head_addr is {ph_head_addr:x}",);
        auxv.push(AuxHeader::new(AT_PHDR, ph_head_addr));

        (self, entry, auxv)
    }

    pub fn load_dl_interp_if_needed(&mut self, elf: &ElfFile) -> SysResult<Option<usize>> {
        let elf_header = elf.header;
        let ph_count = elf_header.pt2.ph_count();

        let mut is_dl = false;
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Interp {
                is_dl = true;
                break;
            }
        }

        if is_dl {
            // adapted from phoenix
            log::info!("[load_dl] encounter a dl elf");
            let section = elf.find_section_by_name(".interp").unwrap();
            let mut interp = String::from_utf8(section.raw_data(&elf).to_vec()).unwrap();
            interp = interp.strip_suffix("\0").unwrap_or(&interp).to_string();
            log::info!("[load_dl] interp {}", interp);

            // let mut interps: Vec<String> = vec![interp.clone()];

            // log::info!("interp {}", interp);

            // let mut interp_dentry: SysResult<Arc<dyn Dentry>> = Err(Errno::ENOENT);
            // for interp in interps.into_iter() {
            //     if let Ok(dentry) = current_task_ref().resolve_path(&interp) {
            //         interp_dentry = Ok(dentry);
            //         break;
            //     }
            // }
            // let interp_dentry: Arc<dyn Dentry> = interp_dentry.unwrap();
            // let interp_file = interp_dentry.open().ok().unwrap();
            let cwd = current_task().unwrap().get_current_path();
            let target_path = resolve_path(cwd, interp);
            if let Ok(FileClass::File(interp_file)) = open(target_path, OpenFlags::O_RDONLY) {
                let interp_elf_data = block_on(async { interp_file.get_inode().read_all().await })?;
                let interp_elf = xmas_elf::ElfFile::new(&interp_elf_data).unwrap();
                self.map_elf(interp_file, &interp_elf, DL_INTERP_OFFSET.into());
                Ok(Some(
                    interp_elf.header.pt2.entry_point() as usize + DL_INTERP_OFFSET,
                ))
            } else {
                Err(Errno::ENOENT)
            }
        } else {
            // no dynamic link
            log::debug!("[load_dl] encounter a static elf");
            Ok(None)
        }
    }

    /// Attach given `pages` to the MemorySpace. If pages is not given, it will
    /// create pages according to the `size` and map them to the MemorySpace.
    /// if `shmaddr` is set to `0`, it will chooses a suitable page-aligned
    /// address to attach.
    ///
    /// `size` and `shmaddr` need to be page-aligned
    #[allow(unused)]
    pub fn attach_shm(
        &mut self,
        size: usize,
        shmaddr: VirtAddr,
        map_perm: MapPerm,
        pages: &mut Vec<Weak<Page>>,
    ) -> VirtAddr {
        info!(
            "[attach_shm] shmaddr: {:#x}, size: {:#x}, map_perm: {:?}, pages: {:?}",
            shmaddr.0, size, map_perm, pages
        );
        let mut ret_addr = shmaddr;
        let mut vm_area = if shmaddr == 0.into() {
            let shared_range: Range<VirtAddr> =
                VirtAddr::from_usize_range(U_SEG_SHARE_BEG..U_SEG_SHARE_END);
            let range: Range<VirtAddr> = self
                .areas()
                .find_free_range(shared_range, size)
                .expect("no free shared area");
            ret_addr = range.start;
            VmArea::new(range, map_perm, VmAreaType::Shm)
        } else {
            log::info!("[attach_shm] user defined addr");
            let shm_end = shmaddr + size;
            VmArea::new(shmaddr..shm_end, map_perm, VmAreaType::Shm)
        };
        if pages.is_empty() {
            for vpn in vm_area.range_vpn() {
                let page = Page::new();
                self.page_table_mut()
                    .map_leaf(vpn, page.ppn(), map_perm.into());
                pages.push(Arc::downgrade(&page));
                vm_area.pages.insert(vpn, page);
            }
        } else {
            debug_assert!(pages.len() == vm_area.range_vpn().end - vm_area.range_vpn().start);
            let mut pages = pages.iter();
            for vpn in vm_area.range_vpn() {
                let page = pages.next().unwrap().upgrade().unwrap();
                self.page_table_mut()
                    .map_leaf(vpn, page.ppn(), map_perm.into());
                vm_area.pages.insert(vpn, page.clone());
            }
        }
        self.push_vma_lazily(vm_area);
        return ret_addr;
    }

    /// `shmaddr` must be the return value of shmget (i.e. `shmaddr` is page
    /// aligned and in the beginning of the vm_area with type Shm). The
    /// check should be done at the caller who call `detach_shm`
    pub fn detach_shm(&mut self, shmaddr: VirtAddr) {
        let mut range_to_remove = None;
        if let Some((range, vm_area)) = self
            .areas()
            .iter()
            .find(|(range, _)| range.start == shmaddr)
        {
            if vm_area.vma_type != VmAreaType::Shm {
                panic!("[detach_shm] 'vm_area.vma_type != VmAreaType::Shm' this won't happen");
            }
            log::info!("[detach_shm] try to remove {:?}", range);
            range_to_remove = Some(range);
            for vpn in vm_area.range_vpn() {
                self.page_table_mut().unmap(vpn);
            }
        } else {
            panic!("[detach_shm] this won't happen");
        }
        if let Some(range) = range_to_remove {
            self.areas_mut().force_remove_one(range);
        } else {
            panic!("[detach_shm] range_to_remove is None! This should never happen");
        }
    }

    /// Alloc stack and map it in the page table.
    ///
    /// Return the address of the stack top, which is aligned to 16 bytes.
    ///
    /// The stack has a range of [sp - size, sp].
    pub fn alloc_stack_lazily(&mut self, size: usize) -> VirtAddr {
        let stack_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_STACK_BEG..U_SEG_STACK_END);

        let range = self
            .areas()
            .find_free_range(stack_range, size)
            .expect("too many stack!");

        // align to 16 bytes
        let sp_init = VirtAddr::from(((range.end.to_usize()) - 1) & !0xf);
        // log::info!("[MemorySpace::alloc_stack] stack: {range:x?}, sp_init: {sp_init:x?}");

        let mut vm_area = VmArea::new(range.clone(), MapPerm::URW, VmAreaType::Stack);
        vm_area.map_range(
            self.page_table_mut(),
            range.end - USER_STACK_PRE_ALLOC_SIZE..range.end,
        );
        self.push_vma_lazily(vm_area);
        sp_init
    }
    pub fn alloc_stack(&mut self, size: usize) -> VirtAddr {
        let stack_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_STACK_BEG..U_SEG_STACK_END);

        let range = self
            .areas()
            .find_free_range(stack_range, size)
            .expect("too many stack!");

        // align to 16 bytes
        let sp_init = VirtAddr::from(((range.end.to_usize()) - 1) & !0xf);
        // log::info!("[MemorySpace::alloc_stack] stack: {range:x?}, sp_init: {sp_init:x?}");

        let mut vm_area = VmArea::new(range, MapPerm::URW, VmAreaType::Stack);
        self.push_vma(vm_area);
        sp_init
    }

    /// Alloc heap lazily.
    pub fn alloc_heap_lazily(&mut self) {
        let heap_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_END);

        const INIT_SIZE: usize = PAGE_SIZE;
        let range = VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_BEG + INIT_SIZE);

        let vm_area = VmArea::new(range, MapPerm::URW, VmAreaType::Heap);
        self.push_vma_lazily(vm_area);
    }
    pub fn alloc_heap(&mut self) {
        let heap_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_END);

        const INIT_SIZE: usize = PAGE_SIZE;
        let range = VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_BEG + INIT_SIZE);

        let vm_area = VmArea::new(range, MapPerm::URW, VmAreaType::Heap);
        self.push_vma(vm_area);
    }

    pub fn get_heap_break(&self) -> VirtAddr {
        // HACK: directly get U_SEG_HEAP_BEG instead？
        let (range, _) = self
            .areas()
            .iter()
            .find(|(_, vma)| vma.vma_type == VmAreaType::Heap)
            .unwrap();
        range.end
    }

    /// NOTE: The actual Linux system call returns the new program break on
    /// success. On failure, the system call returns the current break.
    pub fn reset_heap_break(&mut self, new_brk: VirtAddr) -> VirtAddr {
        let (range, _vma) = self
            .areas_mut()
            .iter_mut()
            .find(|(_, vma)| vma.vma_type == VmAreaType::Heap)
            .unwrap();
        // log::info!("[MemorySpace::reset_heap_break] heap range: {range:?}, new_brk: {new_brk:?}");
        let result = if new_brk > range.end {
            let ret = self.areas_mut().extend_back(range.start..new_brk);
            if ret.is_ok() {
                let (range_va, vm_area) = self.areas_mut().get_key_value_mut(range.start).unwrap();
                vm_area.set_range_va(range_va);
            }
            ret
        } else if new_brk < range.end {
            let ret = self.areas_mut().reduce_back(range.start, new_brk);
            if ret.is_ok() {
                let (range_va, _) = self.areas_mut().get_key_value(range.start).unwrap();
                let vma = self.areas_mut().force_remove_one(range_va.clone());
                let (left, middle, right) = vma.split(range_va);
                debug_assert!(left.is_none());
                debug_assert!(middle.is_some());
                debug_assert!(right.is_some());
                let mut right_vma = right.unwrap();
                right_vma.unmap(self.page_table_mut());
                self.push_vma_lazily(middle.unwrap());
            }
            ret
        } else {
            Ok(())
        };
        match result {
            Ok(_) => new_brk,
            Err(_) => range.end,
        }
    }

    /// Clone a same `MemorySpace` lazily.
    pub fn from_user_lazily(user_space: &mut Self) -> Self {
        info!("[from_user_lazily] enter during process fork");
        let mut memory_space = Self::new_user();
        for (range, area) in user_space.areas().iter() {
            log::info!("[MemorySpace::from_user_lazily] cloning {area:?}");
            let mut new_area = area.clone();
            debug_assert_eq!(range, new_area.range_va());
            for vpn in area.range_vpn() {
                if let Some(page) = area.pages.get(&vpn) {
                    let pte = user_space.page_table_mut().find_pte(vpn).unwrap();
                    let (pte_flags, ppn) = match area.vma_type {
                        VmAreaType::Shm => {
                            // If shared memory,
                            // then we don't need to modify the pte flags,
                            // i.e. no copy-on-write.
                            // log::info!("[from_user_lazily] clone Shared Memory");
                            new_area.pages.insert(vpn, page.clone());
                            (pte.flags(), page.ppn())
                        }

                        VmAreaType::Mmap => {
                            if area.mmap_flags.contains(MmapFlags::MAP_SHARED) {
                                new_area.pages.insert(vpn, page.clone());
                                (pte.flags(), page.ppn())
                            }
                            else {
                                // info!("[from_user_lazily] make pte {:#x} COW, at va {:#x}", pte.bits, vpn.0 << 12);
                                let mut new_flags = pte.flags() | PTEFlags::COW;
                                new_flags.remove(PTEFlags::W);
                                new_flags.remove(PTEFlags::D);
                                pte.set_flags(new_flags);
                                (new_flags, page.ppn())
                            }
                        }
                        _ => {
                            // copy on write
                            // TODO: MmapFlags::MAP_SHARED
                            // info!("[from_user_lazily] make pte {:#x} COW, at va {:#x}", pte.bits, vpn.0 << 12);
                            let mut new_flags = pte.flags() | PTEFlags::COW;
                            new_flags.remove(PTEFlags::W);
                            new_flags.remove(PTEFlags::D);
                            pte.set_flags(new_flags);
                            (new_flags, page.ppn())
                        }
                    };
                    memory_space.page_table_mut().map_leaf(vpn, ppn, pte_flags);
                } else {
                    // lazy allocated area
                }
            }
            memory_space.push_vma_lazily(new_area);
        }
        memory_space
    }

    /// Push `VmArea` into `MemorySpace` and map it in page table.
    pub fn push_vma(&mut self, mut vma: VmArea) {
        vma.map(self.page_table_mut());
        self.areas_mut().try_insert(vma.range_va(), vma).unwrap();
    }

    /// Push `VmArea` into `MemorySpace` without mapping it in page table.
    pub fn push_vma_lazily(&mut self, vma: VmArea) {
        self.areas_mut().try_insert(vma.range_va(), vma).unwrap();
    }

    /// Push `VmArea` into `MemorySpace` and map it in page table, also copy
    /// `data` at `offset` of `vma`.
    /// TODO: too slow, considering to abandon it
    pub fn push_vma_with_data(&mut self, mut vma: VmArea, offset: usize, data: &[u8]) {
        vma.map(self.page_table_mut());
        vma.fill_zero();
        vma.copy_data_with_offset(self.page_table_mut(), offset, data);
        self.areas_mut().try_insert(vma.range_va(), vma).unwrap();
    }

    pub fn alloc_mmap_shared_anonymous(
        &mut self,
        addr: VirtAddr,
        length: usize,
        perm: MapPerm,
        flags: MmapFlags,
    ) -> SysResult<VirtAddr> {
        let shared_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_SHARE_BEG..U_SEG_SHARE_END);
        let range = if flags.contains(MmapFlags::MAP_FIXED) {
            addr..addr + length
        } else {
            self.areas_mut()
                .find_free_range(shared_range, length)
                .expect("shared range is full")
        };
        let start = range.start;
        let vma = VmArea::new(range, perm, VmAreaType::Shm);
        self.push_vma(vma);
        Ok(start)
    }

    pub fn alloc_mmap_anonymous(
        &mut self,
        addr: VirtAddr,
        length: usize,
        perm: MapPerm,
        flags: MmapFlags,
    ) -> SysResult<VirtAddr> {
        let mmap_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_FILE_BEG..U_SEG_FILE_END);
        let range = if flags.contains(MmapFlags::MAP_FIXED) {
            addr..addr + length
        } else {
            self.areas_mut()
                .find_free_range(mmap_range, length)
                .expect("mmap range is full")
        };
        let start = range.start;
        let vma = VmArea::new_mmap(range, perm, flags, None, 0);
        self.push_vma_lazily(vma);
        Ok(start)
    }

    // NOTE: can not alloc all pages from `PageCache`, otherwise lmbench
    // lat_pagefault will test page fault time as zero.
    pub fn alloc_mmap_area_lazily(
        &mut self,
        addr: VirtAddr,
        length: usize,
        perm: MapPerm,
        flags: MmapFlags,
        file: Arc<dyn FileTrait>,
        offset: usize,
    ) -> SysResult<VirtAddr> {
        debug_assert!(is_aligned_to_page(offset));

        let mmap_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_FILE_BEG..U_SEG_FILE_END);

        let range = if flags.contains(MmapFlags::MAP_FIXED) {
            addr..addr + length
        } else {
            self.areas_mut()
                .find_free_range(mmap_range, length)
                .expect("mmap range is full")
        };
        let start = range.start;

        let page_table = self.page_table_mut();
        // let inode = file.inode();
        let mut vma = VmArea::new_mmap(range, perm, flags, Some(file.clone()), offset);
        let mut range_vpn = vma.range_vpn();
        let length = cmp::min(length, MMAP_PRE_ALLOC_PAGES * PAGE_SIZE);
        for offset_aligned in (offset..offset + length).step_by(PAGE_SIZE) {
            if let Some(page) = block_on(async { file.get_page_at(offset_aligned).await }) {
                let vpn = range_vpn.next().unwrap();
                if flags.contains(MmapFlags::MAP_PRIVATE) {
                    let (pte_flags, ppn) = {
                        let mut new_flags: PTEFlags = perm.into();
                        new_flags |= PTEFlags::COW;
                        new_flags.remove(PTEFlags::W);
                        (new_flags, page.ppn())
                    };
                    page_table.map_leaf(vpn, ppn, pte_flags);
                    vma.pages.insert(vpn, page);
                    unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                } else {
                    page_table.map_leaf(vpn, page.ppn(), perm.into());
                    vma.pages.insert(vpn, page);
                    unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                }
            } else {
                break;
            }
        }
        self.push_vma_lazily(vma);
        Ok(start)
    }

    fn split_area(
        &self,
        old_range: Range<VirtAddr>,
        split_range: Range<VirtAddr>,
    ) -> (
        Option<&mut VmArea>,
        Option<&mut VmArea>,
        Option<&mut VmArea>,
    ) {
        let area = self.areas_mut().force_remove_one(old_range);
        let (left, middle, right) = area.split(split_range);
        let left_ret = left.map(|left| self.areas_mut().try_insert(left.range_va(), left).unwrap());
        let right_ret = right.map(|right| {
            self.areas_mut()
                .try_insert(right.range_va(), right)
                .unwrap()
        });
        let middle_ret = middle.map(|middle| {
            self.areas_mut()
                .try_insert(middle.range_va(), middle)
                .unwrap()
        });
        (left_ret, middle_ret, right_ret)
    }

    pub fn unmap(&mut self, range: Range<VirtAddr>) -> SysResult<()> {
        debug_assert!(range.start.aligned());
        debug_assert!(range.end.aligned());
        log::debug!("[MemorySpace::unmap] remove area {:?}", range.clone());

        // First find the left most vm_area containing `range.start`.
        if let Some((first_range, first_vma)) = self.areas_mut().get_key_value_mut(range.start) {
            if first_range.start >= range.start && first_range.end <= range.end {
                log::debug!(
                    "[MemorySpace::unmap] remove left most area {:?}",
                    first_range.clone()
                );
                let mut vma = self.areas_mut().force_remove_one(first_range);
                vma.unmap(self.page_table_mut());
            } else {
                // do split and unmap
                let split_range = range.start..cmp::min(range.end, first_range.end);
                log::debug!("[MemorySpace::unmap] split and remove left most vma {first_vma:?} in range {split_range:?}");
                let (_, middle, _) = self.split_area(first_range, split_range);
                if let Some(middle) = middle {
                    let mut vma = self.areas_mut().force_remove_one(middle.range_va());
                    vma.unmap(self.page_table_mut());
                }
            }
        }
        for (r, vma) in self.areas_mut().range_mut(range.clone()) {
            if r.start >= range.start && r.end <= range.end {
                log::debug!("[MemorySpace::unmap] remove area {:?}", r);
                let mut vma = self.areas_mut().force_remove_one(r);
                vma.unmap(self.page_table_mut());
            } else if r.end > range.end {
                // do split and unmap
                log::debug!(
                    "[MemorySpace::unmap] split and remove vma {vma:?} in range {:?}",
                    r.start..range.end
                );
                let (_, middle, _) = self.split_area(r.clone(), r.start..range.end);
                if let Some(middle) = middle {
                    let mut vma = self.areas_mut().force_remove_one(middle.range_va());
                    vma.unmap(self.page_table_mut());
                }
            }
        }
        Ok(())
    }

    pub fn mprotect(&mut self, range: Range<VirtAddr>, perm: MapPerm) -> SysResult<()> {
        debug_assert!(range.start.aligned() && range.end.aligned());
        let (old_range, area) = self
            .areas_mut()
            .get_key_value_mut(range.start)
            .ok_or(Errno::ENOMEM)?;
        if range == old_range {
            area.set_perm_and_flush(self.page_table_mut(), perm);
        } else {
            debug_assert!(old_range.end >= range.end);
            // do split and remap
            let (_, middle, _) = self.split_area(old_range, range);
            if let Some(middle) = middle {
                middle.set_perm_and_flush(self.page_table_mut(), perm);
            }
        }
        Ok(())
    }

    pub fn handle_page_fault(
        &mut self,
        va: VirtAddr,
        access_type: PageFaultAccessType,
    ) -> SysResult<()> {
        log::trace!("[MemorySpace::handle_page_fault] {va:?}");
        let vm_area = self.areas_mut().get_mut(va.align_down()).ok_or_else(|| {
            log::error!("[handle_page_fault] no area containing {va:?}");
            Errno::EFAULT
        })?;
        vm_area.handle_page_fault(self.page_table_mut(), va.floor(), access_type)?;
        Ok(())
    }

    pub unsafe fn switch_page_table(&self) {
        self.page_table().enable();
    }

    pub fn recycle_data_pages(&mut self) {
        self.areas.get_mut().remove_all();
    }
}

pub fn init_stack(
    sp_init: VirtAddr,
    args: Vec<String>,
    envp: Vec<String>,
    auxv: Vec<AuxHeader>,
) -> (usize, usize, usize, usize) {
    // spec says:
    //      In the standard RISC-V calling convention, the stack grows downward
    //      and the stack pointer is always kept 16-byte aligned.

    // 参考：https://www.cnblogs.com/likaiming/p/11193697.html
    // 初始化之后的栈应该长这样子：
    // content                         size(bytes) + comment
    // -----------------------------------------------------------------------------
    //
    // [argc = number of args]         8
    // [argv[0](pointer)]              8
    // [argv[1](pointer)]              8
    // [argv[...](pointer)]            8 * x
    // [argv[n-1](pointer)]            8
    // [argv[n](pointer)]              8 (=NULL)
    //
    // [envp[0](pointer)]              8
    // [envp[1](pointer)]              8
    // [envp[..](pointer)]             8 * x
    // [envp[term](pointer)]           8 (=NULL)
    //
    // [auxv[0](Elf64_auxv_t)]         16
    // [auxv[1](Elf64_auxv_t)]         16
    // [auxv[..](Elf64_auxv_t)]        16 * x
    // [auxv[term](Elf64_auxv_t)]      16 (=NULL)
    //
    // [padding]                       >= 0
    // [rand bytes]                    16
    // [String identifying platform]   >= 0
    // [padding for align]             >= 0 (sp - (get_random_int() % 8192)) &
    // (~0xf)
    //
    // [argument ASCIIZ strings]       >= 0
    // [environment ASCIIZ str]        >= 0
    // --------------------------------------------------------------------------------
    // 在构建栈的时候，我们从底向上塞各个东西

    info!("[init_stack] in with sp:{:#x}", sp_init.0);
    let mut sp = sp_init.to_usize();
    debug_assert!(sp & 0xf == 0);

    // 存放环境与参数的字符串本身
    fn push_str(sp: &mut usize, s: &str) -> usize {
        let len = s.len();
        *sp -= len + 1; // +1 for NUL ('\0')
        unsafe {
            for (i, c) in s.bytes().enumerate() {
                log::trace!(
                    "push_str: {:x} ({:x}) <- {:?}",
                    *sp + i,
                    i,
                    core::str::from_utf8_unchecked(&[c])
                );
                *((*sp as *mut u8).add(i)) = c;
            }
            *(*sp as *mut u8).add(len) = 0u8;
        }
        *sp
    }

    let env_ptrs: Vec<usize> = envp.iter().rev().map(|s| push_str(&mut sp, s)).collect();
    let arg_ptrs: Vec<usize> = args.iter().rev().map(|s| push_str(&mut sp, s)).collect();

    // 随机对齐 (我们取 0 长度的随机对齐), 平台标识符，随机数与对齐
    fn align16(sp: &mut usize) {
        *sp = (*sp - 1) & !0xf;
    }

    let rand_size = 0;
    let platform = "RISC-V64";
    let rand_bytes = "Meow~ O4 here;D"; // 15 + 1 char for 16bytes

    sp -= rand_size;
    push_str(&mut sp, platform);
    push_str(&mut sp, rand_bytes);
    align16(&mut sp);

    // 存放 auxv
    fn push_aux_elm(sp: &mut usize, elm: &AuxHeader) {
        *sp -= core::mem::size_of::<AuxHeader>();
        unsafe {
            core::ptr::write(*sp as *mut AuxHeader, *elm);
        }
    }
    // 注意推栈是 "倒着" 推的，所以先放 null, 再逆着放别的
    push_aux_elm(&mut sp, &AuxHeader::new(AT_NULL, 0));
    for aux in auxv.into_iter().rev() {
        push_aux_elm(&mut sp, &aux);
    }

    // 存放 envp 与 argv 指针
    fn push_usize(sp: &mut usize, ptr: usize) {
        *sp -= core::mem::size_of::<usize>();
        log::debug!("addr: 0x{:x}, content: {:x}", *sp, ptr);
        unsafe {
            core::ptr::write(*sp as *mut usize, ptr);
        }
    }

    push_usize(&mut sp, 0);
    env_ptrs.iter().for_each(|ptr| push_usize(&mut sp, *ptr));
    let env_ptr_ptr = sp;

    push_usize(&mut sp, 0);
    arg_ptrs.iter().for_each(|ptr| push_usize(&mut sp, *ptr));
    let arg_ptr_ptr = sp;

    // 存放 argc
    let argc = args.len();
    push_usize(&mut sp, argc);

    // info!("[init_stack] out");
    // 返回值
    (sp, argc, arg_ptr_ptr, env_ptr_ptr)
}

pub fn test_la_memory_space() {
    info!("[test_la_memory_space] in");
    let mut memory_space = MemorySpace::new_user();
    let sp = memory_space.alloc_stack(USER_STACK_SIZE);
    unsafe {
        memory_space.switch_page_table();
    }
    let mut x: usize;
    unsafe {
        x = *((sp.0 - 0x1000) as *const usize);
    }
    info!("[test_la_memory_space] read x: {x:#x}");
}
