use core::{cell::{SyncUnsafeCell, UnsafeCell}, hash::Hash};

use crate::{
    fs::{FileClass, PageCache},
    hal::{config::{BLOCK_SIZE, PAGE_SIZE}, mem::page_table::PageTableEntry},
    mm::{frame_allocator::{frame_alloc, FrameTracker}, memory_space::vm_area::VmArea, VirtAddr}, sync::SpinNoIrqLock, utils::container::BitSet8,
};
use alloc::sync::{Arc, Weak};
use hashbrown::HashSet;
use log::info;
// use crate::fs::File;
use crate::mm::address::PhysPageNum;
use crate::sync::mutex::SleepLock;


pub struct Page {
    pub frame: FrameTracker,
    /// 存放dirty block的idx
    pub page_type: PageType,
    // pub reverse_set: SpinNoIrqLock<HashMap<ReverseMapping>>,
    // pub file_cache: Option<Weak<PageCache>>,
    // pub offset: usize,
}

pub enum PageType {
    Anon,
    File(DirtySet),
}


pub struct ReverseMapping {
    pub vma: Weak<VmArea>,
    pub va: VirtAddr,
    pub pte: &'static mut PageTableEntry,
}

// pub enum PageStatus {
//     Cached(FrameTracker),
//     Uncached,
//     Swapped,
// }

impl Page {
    pub fn new_file() -> Arc<Self> {
        Arc::new(Self {
            frame: frame_alloc().expect("frame alloc failed"),
            page_type: PageType::File(SleepLock::new(BitSet8::new())),
        })
    }

    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            frame: frame_alloc().expect("frame alloc failed"),
            page_type: PageType::Anon,
        })
    }

    pub async fn set_dirty(&self, offset: usize) {
        match &self.page_type {
            PageType::Anon => {
                panic!("Cannot set dirty block for an anonymous map!");
            }
            PageType::File(dirty_set) => {
                dirty_set.set_block(offset);
            }
        }
    }

    pub async fn blocks_clear(&self) {
        match &self.page_type {
            PageType::File(dirty_set) => {
                dirty_set.lock().await.clear();
            }
            _ => {
                panic!("Cannot get dirty blocks for an anonymous map!");
            }
        }
    }

    pub fn dirty_set(&self) -> Option<&DirtySet> {
        match &self.page_type {
            PageType::File(dirty_set) => Some(&dirty_set),
            _ => None,
        }
    }

    pub fn ppn(&self) -> PhysPageNum {
        self.frame.ppn
    }

    pub fn copy_from_slice(&self, slice: &[u8]) {
        let len = slice.len().min(PAGE_SIZE);
        self.ppn().get_bytes_array()[0..len].copy_from_slice(slice);
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        self.ppn().get_bytes_array()
    }

    pub fn copy_from_page(&self, page: &Self) {
        self.copy_from_slice(page.get_bytes_array());
    }

    pub fn fill_zero(&self) {
        self.get_bytes_array().fill(0);
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        // info!("Page Dropped, ppn:{:#x}", self.frame.ppn.0);
    }
}

// 改为bitset
pub type DirtySet = SleepLock<BitSet8>;

impl DirtySet {
    pub async fn set_block(&self, offset: usize) {
        let idx = offset / BLOCK_SIZE;
        let mut dirty_blocks = self.lock().await;
        dirty_blocks.insert(idx);
    }

    pub async fn get_blocks(&self) -> BitSet8 {
        self.lock().await.clone()
    }
}
