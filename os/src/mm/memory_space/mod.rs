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

use crate::{
    fs::{open, resolve_path, OpenFlags},
    hal::arch::sfence_vma_vaddr,
    task::{aux, current_task}, utils::elf::check_magic,
};

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
use xmas_elf::{header, ElfFile};

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



    // TODO: what if writeonly or execonly
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
    // TODO: more flags
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

    page_table: SyncUnsafeCell<PageTable>,

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


    // TODO(lsz): do not read whole file, instead, read the header and the sections
    pub async fn new_user_from_elf(
        elf_file: Arc<dyn FileTrait>,
    ) -> SysResult<(Self, usize, usize, Vec<AuxHeader>)> {
        let elf_data = elf_file
            .get_inode()
            .read_all()
            .await?;
        let (mut memory_space, entry_point, auxv) =
            MemorySpace::new_user().parse_and_map_elf_data(&elf_data)?;
        let sp_init = memory_space.alloc_stack(USER_STACK_SIZE).into();
        memory_space.alloc_heap();
        Ok((memory_space, entry_point, sp_init, auxv))
    }
    pub async fn new_user_from_elf_lazily(
        elf_file: Arc<dyn FileTrait>,
    ) -> SysResult<(Self, usize, usize, Vec<AuxHeader>)> {
        let elf_data = elf_file
            .get_inode()
            .read_all()
            .await
            .expect("[new_user_from_elf_lazily] read elf file failed");
        let (mut memory_space, entry_point, auxv) =
            MemorySpace::new_user().parse_and_map_elf(elf_file, &elf_data)?;
        let sp_init = memory_space.alloc_stack_lazily(USER_STACK_SIZE).into();
        memory_space.alloc_heap_lazily();
        Ok((memory_space, entry_point, sp_init, auxv))
    }

    pub fn parse_and_map_elf_data(mut self, elf_data: &[u8]) -> SysResult<(Self, usize, Vec<AuxHeader>)> {

        // NOTE: no more need for check magic
        let elf = xmas_elf::ElfFile::new(elf_data).map_err(|_| Errno::ENOEXEC)?;
        // check_magic(&elf)?;
        let header = elf.header;

        let entry_point = header.pt2.entry_point() as usize;
        let ph_entry_size = header.pt2.ph_entry_size() as usize;
        let ph_count = header.pt2.ph_count() as usize;

        let mut auxv = generate_early_auxv(ph_entry_size, ph_count, entry_point);

        auxv.push(AuxHeader::new(AT_BASE, 0));

        let header_va = self.map_elf_data(&elf, 0.into())?;

        let ph_head_addr = header_va.0 + elf.header.pt2.ph_offset() as usize;
        // log::info!("[from_elf] AT_PHDR  ph_head_addr is {ph_head_addr:x} ");
        auxv.push(AuxHeader::new(AT_PHDR, ph_head_addr));

        Ok((self, entry_point, auxv))
    }

    pub fn map_elf_data(&mut self, elf: &ElfFile, offset: VirtAddr) -> SysResult<VirtAddr> {

        let mut header_va: Option<VirtAddr> = None;

        for ph in elf.program_iter() {

            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }
            let start_va: VirtAddr = (ph.virtual_addr() as usize + offset.0).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize + offset.0).into();
            header_va = header_va.or(Some(start_va));

            let map_perm = ph.flags().into();
            let mut vm_area = VmArea::new(start_va..end_va, map_perm, VmAreaType::Elf, false);

            log::info!(
                "[map_elf_data] ph offset {:#x}, file size {:#x}, mem size {:#x}",
                ph.offset(),
                ph.file_size(),
                ph.mem_size()
            );

            self.push_vma_with_data(
                vm_area,
                start_va.page_offset(),
                &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
            );
        }

        header_va.ok_or(Errno::ENOEXEC)
    }

    pub fn map_elf(
        &mut self,
        elf_file: Arc<dyn FileTrait>,
        elf: &ElfFile,
        offset: VirtAddr,
    ) -> SysResult<VirtAddr> {

        let mut header_va: Option<VirtAddr> = None;

        for ph in elf.program_iter() {

            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }
            let start_va: VirtAddr = (ph.virtual_addr() as usize + offset.0).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize + offset.0).into();

            header_va = header_va.or(Some(start_va));
            let mut map_perm = ph.flags().into();
            let mut vm_area = VmArea::new(start_va..end_va, map_perm, VmAreaType::Elf, false);



            let offset = ph.offset() as usize;
            // full initialization
            if ph.file_size() == ph.mem_size() && is_aligned_to_page(offset) {

                let mut pre_alloc_page_cnt = 0;
                vm_area.range_vpn().try_for_each(|vpn| {
                    block_on(async {
                        elf_file.get_page_at(offset + (vpn - vm_area.start_vpn()) * PAGE_SIZE).await
                    }).map_or(Err(Errno::ENOEXEC), | p | {
                        // make it COW
                        let mut pte_flags: PTEFlags = map_perm.into();
                        pte_flags.set_COW(pte_flags.is_W()).set_W(false).set_D(false);

                        // alloc immediately, not lazy
                        self.page_table_mut().map_leaf(vpn, p.ppn(), pte_flags);
                        vm_area.pages.insert(vpn, p);
                        unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                        Ok(())
                    })
                })?;

                self.push_vma_lazily(vm_area);
            } else {
                self.push_vma_with_data(
                    vm_area,
                    start_va.page_offset(),
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                );
            }
        }

        header_va.ok_or(Errno::ENOEXEC)
    }

    pub fn parse_and_map_elf(
        mut self,
        elf_file: Arc<dyn FileTrait>,
        elf_data: &[u8],
    ) -> SysResult<(Self, usize, Vec<AuxHeader>)> {
        // const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

        // map program headers of elf, with U flag
        
        let elf = xmas_elf::ElfFile::new(elf_data).map_err(|_| Errno::ENOEXEC)?;
        let header = elf.header;
        // assert_eq!(elf_header.pt1.magic, ELF_MAGIC, "invalid elf!");
        let mut entry = header.pt2.entry_point() as usize;
        let ph_entry_size = header.pt2.ph_entry_size() as usize;
        let ph_count = header.pt2.ph_count() as usize;

        let mut auxv = generate_early_auxv(ph_entry_size, ph_count, entry);

        // maybe needed?
        // auxv.push(AuxHeader::new(AT_BASE, 0));

        if let Some(interp_entry) = self.load_dl_interp_if_needed(&elf).unwrap_or(None) {
            auxv.push(AuxHeader::new(AT_BASE, DL_INTERP_OFFSET));
            entry = interp_entry;
        } else {
            auxv.push(AuxHeader::new(AT_BASE, 0));
        }

        let header_va = self.map_elf(elf_file, &elf, 0.into())?;

        let ph_head_addr = header_va.0 + elf.header.pt2.ph_offset() as usize;
        auxv.push(AuxHeader::new(AT_RANDOM, ph_head_addr));
        // log::info!("[parse_and_map_elf] AT_PHDR  ph_head_addr is {ph_head_addr:x}",);
        auxv.push(AuxHeader::new(AT_PHDR, ph_head_addr));

        Ok((self, entry, auxv))
    }

    pub fn load_dl_interp_if_needed(&mut self, elf: &ElfFile) -> SysResult<Option<usize>> {
        let header = elf.header;
        // let ph_count = elf_header.pt2.ph_count();

        let mut is_dl = elf.program_iter().any(
            | ph | ph.get_type().unwrap() == xmas_elf::program::Type::Interp
        );

        if is_dl {
            // adapted from phoenix
            log::error!("[load_dl] encounter a dl elf");
            let section = elf.find_section_by_name(".interp").unwrap();
            let mut interp_path = String::from_utf8(section.raw_data(&elf).to_vec()).unwrap();
            interp_path = interp_path.strip_suffix("\0").unwrap_or(&interp_path).to_string();
            log::error!("[load_dl] interp {}", interp_path);


            let cwd = current_task().unwrap().get_current_path();
            let target_path = resolve_path(cwd, interp_path);
            if let Ok(FileClass::File(interp_file)) = open(target_path, OpenFlags::O_RDONLY) {
                let interp_elf_data = block_on(async { interp_file.get_inode().read_all().await })?;
                let interp_elf = xmas_elf::ElfFile::new(&interp_elf_data).map_err(|_| Errno::ENOEXEC)?;
                self.map_elf(interp_file, &interp_elf, DL_INTERP_OFFSET.into());
                Ok(Some(
                    interp_elf.header.pt2.entry_point() as usize + DL_INTERP_OFFSET,
                ))
            } else {
                Err(Errno::ENOENT)
            }
        } else {
            // no dynamic link
            log::info!("[load_dl] encounter a static elf");
            Ok(None)
        }
    }


    #[allow(unused)]
    pub fn attach_shm(
        &mut self,
        size: usize,
        shmaddr: VirtAddr,
        map_perm: MapPerm,
        pages: &mut Vec<Weak<Page>>,
    ) -> VirtAddr {

        let shared = true;

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
            VmArea::new(range, map_perm, VmAreaType::Shm, shared)
        } else {
            log::info!("[attach_shm] user defined addr");
            let shm_end = shmaddr + size;
            VmArea::new(shmaddr..shm_end, map_perm, VmAreaType::Shm, shared)
        };
        if pages.is_empty() {

            vm_area.range_vpn().for_each( | vpn | {
                let page = Page::new();
                self.page_table_mut().map_leaf(vpn, page.ppn(), map_perm.into());
                pages.push(Arc::downgrade(&page));
                vm_area.pages.insert(vpn, page);    
            });
        } else {

            vm_area.range_vpn()
                .zip(pages.iter())
                .for_each(|(vpn, page)| {
                    let page = page.upgrade().unwrap();
                    self.page_table_mut()
                        .map_leaf(vpn, page.ppn(), map_perm.into());
                    vm_area.pages.insert(vpn, page.clone());
                });
            // int*(id)(int, int)
        }
        self.push_vma_lazily(vm_area);
        return ret_addr;
    }


    pub fn detach_shm(&mut self, shmaddr: VirtAddr) {
        let mut range2remove = None;
        if let Some((range, vm_area)) = self
            .areas()
            .iter()
            .find(|(range, _)| range.start == shmaddr)
        {

            range2remove = Some(range);

            vm_area.range_vpn().map(| vpn | self.page_table_mut().unmap(vpn));
        } else {
            panic!("[detach_shm] this won't happen");
        }

        range2remove.map(| range | self.areas_mut().force_remove_one(range));
    }


    pub fn alloc_stack_lazily(&mut self, size: usize) -> VirtAddr {

        let shared = false;

  
        let range = VirtAddr::from_usize_range(U_SEG_STACK_END - size..U_SEG_STACK_END);

        // align to 16 bytes
        let sp_init = VirtAddr::from(((range.end.to_usize()) - 1) & !0xf);
        // log::info!("[MemorySpace::alloc_stack] stack: {range:x?}, sp_init: {sp_init:x?}");

        let mut vm_area = VmArea::new(range.clone(), MapPerm::URW, VmAreaType::Stack, shared);
        vm_area.map_range(
            self.page_table_mut(),
            range.end - USER_STACK_PRE_ALLOC_SIZE..range.end,
        );
        self.push_vma_lazily(vm_area);
        sp_init
    }
    pub fn alloc_stack(&mut self, size: usize) -> VirtAddr {

        let shared = false;

        let range = VirtAddr::from_usize_range(U_SEG_STACK_END - size..U_SEG_STACK_END);

        // align to 16 bytes
        let sp_init = VirtAddr::from(((range.end.to_usize()) - 1) & !0xf);
        // log::info!("[MemorySpace::alloc_stack] stack: {range:x?}, sp_init: {sp_init:x?}");

        let mut vm_area = VmArea::new(range, MapPerm::URW, VmAreaType::Stack, shared);
        self.push_vma(vm_area);
        sp_init
    }

    /// Alloc heap lazily.
    pub fn alloc_heap_lazily(&mut self) {

        let shared = false;

        const INIT_SIZE: usize = PAGE_SIZE;
        let range = VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_BEG + INIT_SIZE);

        let vm_area = VmArea::new(range, MapPerm::URW, VmAreaType::Heap, shared);
        self.push_vma_lazily(vm_area);
    }
    pub fn alloc_heap(&mut self) {
        
        let shared = false;

        // let heap_range: Range<VirtAddr> =
        //     VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_END);

        const INIT_SIZE: usize = PAGE_SIZE;
        let range = VirtAddr::from_usize_range(U_SEG_HEAP_BEG..U_SEG_HEAP_BEG + INIT_SIZE);

        let vm_area = VmArea::new(range, MapPerm::URW, VmAreaType::Heap, shared);
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
                // let (range_va, _) = self.areas_mut().get_key_value(range.start).unwrap();
                let new_heap_range = range.start..new_brk;
                let heap_vma = self.areas_mut().force_remove_one(new_heap_range.clone());
                let (left, middle, right) = heap_vma.split(new_heap_range);
                let mut range2remove = right.unwrap();
                range2remove.unmap(self.page_table_mut());
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
            // debug_assert_eq!(range, new_area.range_va());
            area.range_vpn().for_each( | vpn | {
                area.pages.get(&vpn).map( | page | {
                    let pte = user_space.page_table_mut().find_pte(vpn).unwrap();
                    let mut pte_flags = pte.flags();
                    let shared = area.shared;
                    let is_writable = pte_flags.is_W();
                    pte_flags.set_COW( !shared && is_writable )
                        .set_W(shared && is_writable)
                        .set_D(shared && is_writable);
                    pte.set_flags(pte_flags);
                    new_area.pages.insert(vpn, page.clone());
                    memory_space.page_table_mut().map_leaf(vpn, page.ppn(), pte_flags);
                });
            });

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


    pub fn push_vma_with_data(&mut self, mut vma: VmArea, offset: usize, data: &[u8]) {
        vma.map(self.page_table_mut());
        vma.fill_zero();
        vma.copy_data_with_offset(self.page_table_mut(), offset, data);
        self.areas_mut().try_insert(vma.range_va(), vma).unwrap();
    }



    pub fn alloc_mmap_anon(
        &mut self,
        addr: VirtAddr,
        length: usize,
        perm: MapPerm,
        flags: MmapFlags,
    ) -> SysResult<VirtAddr> {

        /// TODO: what if shared_validate
        let shared = flags.intersection(MmapFlags::MAP_TYPE_MASK) == MmapFlags::MAP_SHARED;


        let mmap_range = VirtAddr::from_usize_range(U_SEG_FILE_BEG..U_SEG_FILE_END);


        let range = flags.contains(MmapFlags::MAP_FIXED)
        .then(|| addr..addr + length)
        .unwrap_or_else(||{
            self.areas_mut()
                .find_free_range(mmap_range, length)
                .expect("mmap range is full")
        });

        let vma = VmArea::new_mmap(range, perm, flags, None, 0, shared);

        let start_va = vma.start_va();

        // TODO(lsz): cannot support lazy allocation of shared anon map now
        
        if shared {
            self.push_vma(vma);
        }
        else {
            self.push_vma_lazily(vma);
        }

        Ok(start_va)

    }


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

        let shared = flags.intersection(MmapFlags::MAP_TYPE_MASK) == MmapFlags::MAP_SHARED;

        let mmap_range: Range<VirtAddr> =
            VirtAddr::from_usize_range(U_SEG_FILE_BEG..U_SEG_FILE_END);

        let range = flags.contains(MmapFlags::MAP_FIXED)
            .then( || addr..addr + length)
            .unwrap_or_else( || {
                self.areas_mut()
                    .find_free_range(mmap_range, length)
                    .expect("mmap range is full")
            });
        let start = range.start;

        let page_table = self.page_table_mut();
        // let inode = file.inode();
        let mut vma = VmArea::new_mmap(range, perm, flags, Some(file.clone()), offset, shared);
        // let mut range_vpn = vma.range_vpn();
        let length = length.min(MMAP_PRE_ALLOC_PAGES * PAGE_SIZE);
        vma.range_vpn()
            .zip( (offset..offset + length).step_by(PAGE_SIZE) )
            .try_for_each( | (vpn, offset_aligned) | {
                block_on( async {
                    file.get_page_at(offset_aligned).await
                })
                .map_or(Err(Errno::EINVAL), | page | {
                    let pte_flags = flags.contains(MmapFlags::MAP_PRIVATE)
                        .then( || {
                            let mut pte_flags: PTEFlags = perm.into();
                            pte_flags.set_COW(pte_flags.is_W()).set_W(false).set_D(false);
                            pte_flags
                        }).unwrap_or( perm.into());
                    page_table.map_leaf(vpn, page.ppn(), pte_flags);
                    vma.pages.insert(vpn, page);
                    unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                    Ok(())
                })
                
            } )?;
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

        // First find the left most vm_area containing `range.start`.
        if let Some((first_range, first_vma)) = self.areas_mut().get_key_value_mut(range.start) {
            // first_range.
            if first_range.start >= range.start && first_range.end <= range.end {

                let mut vma = self.areas_mut().force_remove_one(first_range);
                vma.unmap(self.page_table_mut());
            } else {
                // do split and unmap
                let split_range = range.start..range.end.min(first_range.end);
                let (_, middle, _) = self.split_area(first_range, split_range);
                if let Some(middle) = middle {
                    let mut vma = self.areas_mut().force_remove_one(middle.range_va());
                    vma.unmap(self.page_table_mut());
                }
            }
        }
        for (r, vma) in self.areas_mut().range_mut(range.clone()) {
            if r.start >= range.start && r.end <= range.end {
                let mut vma = self.areas_mut().force_remove_one(r);
                vma.unmap(self.page_table_mut());
            } else if r.end > range.end {

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

pub fn create_elf_tables(
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


    // 必须手动保证对sp的访问不非法
    // 存放环境与参数的字符串本身
    fn push_str(sp: &mut usize, s: &str) -> usize {
        let len = s.len();
        *sp -= len + 1; // +1 for NUL ('\0')
        unsafe {
            // for (i, c) in s.bytes().enumerate() {
            //     // log::trace!(
            //     //     "push_str: {:x} ({:x}) <- {:?}",
            //     //     *sp + i,
            //     //     i,
            //     //     core::str::from_utf8_unchecked(&[c])
            //     // );
            //     *((*sp as *mut u8).add(i)) = c;
            // }
            s.bytes().enumerate().for_each(|(i, c)| {
                *((*sp as *mut u8).add(i)) = c;
            });
            *(*sp as *mut u8).add(len) = 0u8;
        }
        *sp
    }
    // 存放 auxv
    fn push_aux_elm(sp: &mut usize, elm: &AuxHeader) {
        *sp -= core::mem::size_of::<AuxHeader>();
        unsafe {
            core::ptr::write(*sp as *mut AuxHeader, *elm);
        }
    }
    // 存放 envp 与 argv 指针
    fn push_usize(sp: &mut usize, ptr: usize) {
        *sp -= core::mem::size_of::<usize>();
        unsafe {
            core::ptr::write(*sp as *mut usize, ptr);
        }
    }
    let env_ptrs: Vec<usize> = envp.iter().rev().map(|s| push_str(&mut sp, s)).collect();
    let arg_ptrs: Vec<usize> = args.iter().rev().map(|s| push_str(&mut sp, s)).collect();

    // 随机对齐 (我们取 0 长度的随机对齐), 平台标识符，随机数与对齐
    fn align16(sp: &mut usize) {
        *sp = (*sp - 1) & !0xf;
    }

    let rand_size = 0;
    let platform = "RISC-V64";
    let rand_bytes = "Dlnx w.r.t.Phnx"; // 15 + 1 char for 16bytes

    sp -= rand_size;
    push_str(&mut sp, platform);
    push_str(&mut sp, rand_bytes);
    align16(&mut sp);


    // 注意推栈是 "倒着" 推的，所以先放 null, 再逆着放别的
    push_aux_elm(&mut sp, &AuxHeader::new(AT_NULL, 0));
    for aux in auxv.into_iter().rev() {
        push_aux_elm(&mut sp, &aux);
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
