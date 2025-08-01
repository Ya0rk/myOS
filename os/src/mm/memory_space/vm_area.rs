use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::fmt::Display;
use core::hash::{Hash, Hasher};
use core::ops::{Range, RangeBounds};
use log::{error, info};

// use core::arch::riscv64::sfence_vma_vaddr; //这个core::arch::riscv64包会在hal::arch中统一引入
use crate::hal::arch::sfence_vma_vaddr;
// use async_utils::block_on;
use crate::hal::config::{align_down_by_page, PAGE_SIZE};
use crate::hal::mem::page_table::{PTEFlags, PageTableEntry};
// use memory::{pte::PTEFlags, VirtAddr, VirtPageNum};
use crate::mm::page_table::PageTable;
// use PTEFlags
use super::{MmapFlags, PageFaultAccessType};
use crate::fs::{FileClass, FileTrait};
use crate::mm::address::{VirtAddr, VirtPageNum};
use crate::mm::page::Page;
use crate::sync::block_on;
use crate::task::current_task;
use crate::utils::{backtrace, Errno, SysResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmAreaType {
    // For user.
    /// Segments from user elf file, e.g. text, rodata, data, bss
    Elf,
    /// User Stack
    Stack,
    /// User Heap
    Heap,
    /// Mmap
    Mmap,
    /// Shared memory
    Shm,
    /// Kernel pagetable segments
    Kernel,
}

bitflags! {
    /// Map permission corresponding to that in pte: `R W X U`
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MapPerm: u16 {
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// Excutable
        const X = 1 << 3;
        /// Accessible in U mode
        const U = 1 << 4;

        const RW = Self::R.bits() | Self::W.bits();
        const RX = Self::R.bits() | Self::X.bits();
        const WX = Self::W.bits() | Self::X.bits();
        const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits();

        const UR = Self::U.bits() | Self::R.bits();
        const UW = Self::U.bits() | Self::W.bits();
        const UX = Self::U.bits() | Self::X.bits();
        const URW = Self::U.bits() | Self::RW.bits();
        const URX = Self::U.bits() | Self::RX.bits();
        const UWX = Self::U.bits() | Self::WX.bits();
        const URWX = Self::U.bits() | Self::RWX.bits();
    }
}

impl From<PTEFlags> for MapPerm {
    fn from(flags: PTEFlags) -> Self {
        let mut ret = Self::from_bits(0).unwrap();
        if flags.is_U() {
            ret |= MapPerm::U
        }
        if flags.is_R() {
            ret |= MapPerm::R;
        }
        if flags.is_W() {
            ret |= MapPerm::W;
        }
        if flags.is_X() {
            ret |= MapPerm::X;
        }
        ret
    }
}

impl From<xmas_elf::program::Flags> for MapPerm {
    fn from(flags: xmas_elf::program::Flags) -> Self {
        let mut ret = Self::U;
        if flags.is_read() {
            ret |= MapPerm::R;
        }
        if flags.is_write() {
            ret |= MapPerm::W;
        }
        if flags.is_execute() {
            ret |= MapPerm::X;
        }
        ret
    }

}

/// A contiguous virtual memory area.
/// ADDITION: only in user space
#[derive(Clone)]
pub struct VmArea {
    /// Aligned `VirtAddr` range for the `VmArea`.
    range_va: Range<VirtAddr>,
    /// Hold pages with RAII.
    pub pages: BTreeMap<VirtPageNum, Arc<Page>>,
    /// Map permission of this area.
    pub map_perm: MapPerm,
    /// Type of this area.
    pub vma_type: VmAreaType,

    // For mmap.
    /// Mmap flags.
    pub mmap_flags: MmapFlags,
    /// The underlying file being mapped.
    pub backed_file: Option<Arc<dyn FileTrait>>,
    /// Start offset in the file.
    pub offset: usize,

    pub shared: bool,
}

impl core::fmt::Debug for VmArea {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmArea")
            .field("range_va", &self.range_va)
            .field("map_perm", &self.map_perm)
            .field("vma_type", &self.vma_type)
            .finish()
    }
}

impl Drop for VmArea {
    fn drop(&mut self) {
        // log::debug!("[VmArea::drop] drop {self:?}",);
    }
}

impl VmArea {
    /// Construct a new vma.
    pub fn new(range_va: Range<VirtAddr>, map_perm: MapPerm, vma_type: VmAreaType, shared: bool) -> Self {
        let range_va = range_va.start.floor().into()..range_va.end.ceil().into();
        let new = Self {
            range_va,
            pages: BTreeMap::new(),
            vma_type,
            map_perm,
            backed_file: None,
            mmap_flags: MmapFlags::default(),
            offset: 0,
            shared,
        };
        new
    }

    pub fn new_mmap(
        range_va: Range<VirtAddr>,
        map_perm: MapPerm,
        mmap_flags: MmapFlags,
        file: Option<Arc<dyn FileTrait>>,
        offset: usize,
        shared: bool,
    ) -> Self {
        let range_va = range_va.start.floor().into()..range_va.end.ceil().into();
        let new = Self {
            range_va,
            pages: BTreeMap::new(),
            vma_type: VmAreaType::Mmap,
            map_perm,
            backed_file: file,
            mmap_flags,
            offset,
            shared,
        };
        new
    }

    pub fn from_another(another: &Self) -> Self {
        Self {
            range_va: another.range_va(),
            pages: BTreeMap::new(),
            vma_type: another.vma_type,
            map_perm: another.map_perm,
            backed_file: another.backed_file.clone(),
            mmap_flags: another.mmap_flags,
            offset: another.offset,
            shared: another.shared,
        }
    }

    pub fn start_va(&self) -> VirtAddr {
        self.range_va().start
    }

    pub fn end_va(&self) -> VirtAddr {
        self.range_va().end
    }

    pub fn start_vpn(&self) -> VirtPageNum {
        self.start_va().floor()
    }

    pub fn end_vpn(&self) -> VirtPageNum {
        self.end_va().ceil()
    }

    pub fn range_va(&self) -> Range<VirtAddr> {
        self.range_va.clone()
    }

    pub fn range_vpn(&self) -> Range<VirtPageNum> {
        self.start_vpn()..self.end_vpn()
    }

    pub fn set_range_va(&mut self, range_va: Range<VirtAddr>) {
        self.range_va = range_va
    }

    pub fn perm(&self) -> MapPerm {
        self.map_perm
    }

    pub fn set_perm(&mut self, perm: MapPerm) {
        self.map_perm = perm;
    }

    pub fn get_page(&self, vpn: VirtPageNum) -> &Arc<Page> {
        self.pages.get(&vpn).expect("no page found for vpn")
    }

    pub fn fill_zero(&self) {
        for page in self.pages.values() {
            page.fill_zero()
        }
    }

    // [LA_MMU] only user page table
    pub fn set_perm_and_flush(&mut self, page_table: &mut PageTable, perm: MapPerm) {
        self.set_perm(perm);
        let pte_flags = perm.into();
        // NOTE: should flush pages that already been allocated, page fault handler will
        // handle the permission of those unallocated pages
        for &vpn in self.pages.keys() {
            let pte = page_table.find_pte(vpn).unwrap();
            log::trace!(
                "[origin pte:{:?}, new_flag:{:?}]",
                pte.flags(),
                pte.flags().union(pte_flags)
            );
            pte.set_flags(pte.flags().union(pte_flags));
            sfence_vma_vaddr(vpn.to_vaddr().into());
        }
    }

    pub fn flush(&mut self) {
        let range_vpn = self.range_vpn();
        for vpn in range_vpn {
            sfence_vma_vaddr(vpn.to_vaddr().into());
        }
    }

    /// Map `VmArea` into page table.
    ///
    /// Will alloc new pages for `VmArea` according to `VmAreaType`.
    pub fn map(&mut self, page_table: &mut PageTable) {
        let pte_flags = self.map_perm.into();

        for vpn in self.range_vpn() {
            let page = Page::new();
            // page.clear();
            page_table.map_leaf(vpn, page.ppn(), pte_flags);
            self.pages.insert(vpn, page);
        }
    }

    pub fn map_range(&mut self, page_table: &mut PageTable, range: Range<VirtAddr>) {
        let range_vpn = range.start.into()..range.end.into();
        assert!(self.start_vpn() <= range_vpn.start && self.end_vpn() >= range_vpn.end);
        let pte_flags: PTEFlags = self.map_perm.into();
        for vpn in range_vpn {
            let page = Page::new();
            page_table.map_leaf(vpn, page.ppn(), pte_flags);
            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
            self.pages.insert(vpn, page);
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        let vpns: Vec<_> = self.pages.keys().cloned().collect();
        for vpn in vpns {
            page_table.unmap(vpn);
            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
            self.pages.remove(&vpn);
        }
    }

    // to refactor
    pub fn copy_data_with_offset(
        &mut self,
        page_table: &mut PageTable,
        offset: usize,
        data: &[u8],
    ) {
        let mut offset = offset;
        let mut slice_off: usize = 0;
        let mut vpn = self.start_vpn();
        let slice_len = data.len();
        while slice_off < slice_len {
            let src = &data[slice_off..slice_len.min(slice_off + PAGE_SIZE - offset)];
            page_table
                .find_pte(vpn)
                .unwrap()
                .ppn()
                .get_bytes_array_from_range(offset..offset + src.len())
                .copy_from_slice(src);
            slice_off += PAGE_SIZE - offset;
            offset = 0;
            vpn += 1;
        }
    }

    pub fn split(self, split_range: Range<VirtAddr>) -> (Option<Self>, Option<Self>, Option<Self>) {
        let (mut left, mut middle, mut right) = (None, None, None);
        let (left_range, middle_range, right_range) = (
            self.start_va()..split_range.start,
            split_range.clone(),
            split_range.end..self.end_va(),
        );
        if !left_range.is_empty() {
            let mut left_vma = VmArea::from_another(&self);
            left_vma.set_range_va(left_range);
            left_vma.pages.extend(
                self.pages
                    .range(left_vma.range_vpn())
                    .into_iter()
                    .map(|(&k, v)| (k, v.clone())),
            );
            left_vma.offset += left_vma.start_va() - self.start_va();
            left = Some(left_vma)
        }
        if !middle_range.is_empty() {
            let mut middle_vma = VmArea::from_another(&self);
            middle_vma.set_range_va(middle_range);
            middle_vma.pages.extend(
                self.pages
                    .range(middle_vma.range_vpn())
                    .into_iter()
                    .map(|(&k, v)| (k, v.clone())),
            );
            middle_vma.offset += middle_vma.start_va() - self.start_va();
            middle = Some(middle_vma)
        }
        if !right_range.is_empty() {
            let mut right_vma = VmArea::from_another(&self);
            right_vma.set_range_va(right_range);
            right_vma.pages.extend(
                self.pages
                    .range(right_vma.range_vpn())
                    .into_iter()
                    .map(|(&k, v)| (k, v.clone())),
            );
            right_vma.offset += right_vma.start_va() - self.start_va();
            right = Some(right_vma)
        }
        (left, middle, right)
    }

    pub fn handle_page_fault(
        &mut self,
        page_table: &mut PageTable,
        vpn: VirtPageNum,
        access_type: PageFaultAccessType,
    ) -> SysResult<()> {
        info!(
            "[VmArea::handle_page_fault] {self:?}, {vpn:?} at page table {:?}",
            page_table.root_ppn
        );

        if !access_type.can_access(self.perm()) {
            backtrace();
            error!(
                "[VmArea::handle_page_fault] permission not allowed, perm:{:?}",
                self.perm()
            );
            return Err(Errno::EFAULT);
        }

        let page: Arc<Page>;
        let pte = page_table.find_pte(vpn);
        if let Some(pte) = pte {
            let mut pte_flags = pte.flags();
            let old_page = self.get_page(vpn);
            let cnt = Arc::strong_count(old_page);
            info!("[handle_page_fault] page cnt:{}", cnt);
            if cnt > 1 {
                page = Page::new();
                page.copy_from_slice(old_page.get_bytes_array());

                pte_flags.set_COW(false)
                    .set_W(true)
                    .set_D(true);
                page_table.map_leaf_force(vpn, page.ppn(), pte_flags);
                self.pages.insert(vpn, page);
                sfence_vma_vaddr(vpn.to_vaddr().into());
            } else {
                pte_flags.set_COW(false)
                         .set_W(true)
                         .set_D(true);
                pte.set_flags(pte_flags);
                sfence_vma_vaddr(vpn.to_vaddr().into());
            }
        } else {
            match self.vma_type {
                VmAreaType::Heap | VmAreaType::Stack => {
                    // lazy allcation for heap
                    page = Page::new();
                    page.fill_zero();
                    page_table.map_leaf(vpn, page.ppn(), self.map_perm.into());
                    self.pages.insert(vpn, page);
                    unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                }
                VmAreaType::Mmap => {
                    if !self.mmap_flags.contains(MmapFlags::MAP_ANONYMOUS) {
                        // file mapping
                        let file = self.backed_file.as_ref().unwrap();
                        let offset = self.offset + (vpn - self.start_vpn()) * PAGE_SIZE;
                        let offset_aligned = align_down_by_page(offset);
                        if self.mmap_flags.contains(MmapFlags::MAP_SHARED) {
                            let page =
                                block_on(async { file.get_page_at(offset_aligned).await }).unwrap();
                            page_table.map_leaf(vpn, page.ppn(), self.map_perm.into());
                            self.pages.insert(vpn, page);
                            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                        } else {
                            let page =
                                block_on(async { file.get_page_at(offset_aligned).await }).unwrap();
                            if access_type.contains(PageFaultAccessType::WRITE) {
                                let new_page = Page::new();
                                new_page.copy_from_slice(page.get_bytes_array());
                                page_table.map_leaf(vpn, new_page.ppn(), self.map_perm.into());
                                self.pages.insert(vpn, new_page);
                            } else {
                                let (pte_flags, ppn) = {
                                    let mut new_flags: PTEFlags = self.map_perm.into();
                                    // new_flags |= PTEFlags::COW;
                                    // new_flags.remove(PTEFlags::W);
                                    new_flags.set_COW(new_flags.is_W())
                                             .set_W(false)
                                             .set_D(false);
                                    (new_flags, page.ppn())
                                };
                                page_table.map_leaf(vpn, ppn, pte_flags);
                                self.pages.insert(vpn, page);
                            }
                            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                        }
                    } else if self.mmap_flags.contains(MmapFlags::MAP_PRIVATE) {
                        if self.mmap_flags.contains(MmapFlags::MAP_SHARED) {
                            todo!()
                        } else {
                            // private anonymous area
                            page = Page::new();
                            page.fill_zero();
                            page_table.map_leaf(vpn, page.ppn(), self.map_perm.into());
                            self.pages.insert(vpn, page);
                            unsafe { sfence_vma_vaddr(vpn.to_vaddr().into()) };
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}


// #[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
// struct VmAreaPtr(*mut VmArea);


// impl VmAreaPtr {
//     pub fn new(vma: *mut VmArea) -> Self {
//         Self(vma)
//     }
    
//     pub fn get(&self) -> &VmArea {
//         unsafe { &*self.0 }
//     }

//     pub fn get_mut(&self) -> &mut VmArea {
//         unsafe { &mut *self.0 }
//     }

//     pub fn from_ref(vma: &VmArea) -> Self {
//         unsafe { Self(vma as *const VmArea as *mut VmArea) }
//     }

// }
