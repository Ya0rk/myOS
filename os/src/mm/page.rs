use crate::{
    fs::FileClass,
    hal::config::{BLOCK_SIZE, PAGE_SIZE},
    mm::frame_allocator::{frame_alloc, FrameTracker},
};
use alloc::sync::Arc;
use hashbrown::HashSet;
use log::info;
// use crate::fs::File;
use crate::mm::address::PhysPageNum;
use crate::sync::mutex::SleepLock;
pub struct Page {
    pub frame: FrameTracker,
    /// 存放dirty block的idx
    pub page_type: PageType,
}

pub enum PageType {
    Anon,
    File(DirtySet),
}

impl Page {
    pub fn new_file() -> Arc<Self> {
        Arc::new(Self {
            frame: frame_alloc().expect("frame alloc failed"),
            page_type: PageType::File(SleepLock::new(HashSet::new())),
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

pub type DirtySet = SleepLock<HashSet<usize>>;

impl DirtySet {
    pub async fn set_block(&self, offset: usize) {
        let idx = offset / BLOCK_SIZE;
        let mut dirty_blocks = self.lock().await;
        dirty_blocks.insert(idx);
    }

    pub async fn get_blocks(&self) -> HashSet<usize> {
        self.lock().await.clone()
    }
}
