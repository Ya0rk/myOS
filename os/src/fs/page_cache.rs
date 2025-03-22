use alloc::{collections::btree_map::BTreeMap, sync::{Arc, Weak}, vec::Vec};
use spin::rwlock::RwLock;
use crate::{config::{BLOCK_SIZE, PAGE_SIZE}, mm::{frame_alloc, FrameTracker}, sync::SleepLock, utils::{Errno, SysResult}};
use super::InodeTrait;

/// 使用对齐的地址作为key
pub struct PageCache {
    pages: RwLock<BTreeMap<usize, Arc<Page>>>,
    inode: Option<Weak<dyn InodeTrait>>
}

impl PageCache {
    pub fn new(inode: Arc<dyn InodeTrait>) -> Self {
        Self {
            inode: Some(Arc::downgrade(&inode)),
            pages: RwLock::new(BTreeMap::new()),
        }
    }

    /// 在page cache中寻找目标页
    pub fn get_page(
        &self,
        offset: usize,
    ) -> SysResult<Option<Arc<Page>>> {
        let offset_aligned = offset & !(PAGE_SIZE - 1);
        // 从cache中寻找
        Ok(self.pages.read().get(&offset_aligned).cloned())
    }

    /// 将page插入cache
    pub fn insert_page(
        &self,
        offset: usize
    ) -> SysResult<Option<Arc<Page>>> {
        let offset_aligned = offset & !(PAGE_SIZE - 1);
        let frame = frame_alloc().expect("no more frame!");
        let page = Arc::new(Page::new(
            frame
        ));
        self.pages.write().insert(offset_aligned, page.clone());
        Ok(Some(page))
    }

    /// 清空page cache，需要判断是否dirty
    pub async fn flush(
        &self,
    ) -> SysResult<usize> {
        for (page_addr_aligned, page) in self.pages.read().iter() {
            let inode = self.inode.as_ref().ok_or(Errno::EBADF)?.upgrade().unwrap();
            let mut dirty_blocks = page.dirty_blocks.lock().await;
            while !dirty_blocks.is_empty() {
                let idx = dirty_blocks.pop().unwrap();
                let start_offset = idx * BLOCK_SIZE;
                let start = page_addr_aligned + start_offset;
                let buf = &page.frame.ppn.get_bytes_array()[start_offset..start_offset + BLOCK_SIZE].to_vec();
                inode.clone().write_back(start, BLOCK_SIZE, buf);
            }
        }
        Ok(0)
    }

}


pub struct Page {
    frame: FrameTracker,
    /// 存放dirty block的idx
    dirty_blocks: SleepLock<Vec<usize>>,
}

impl Page {
    fn new(frame: FrameTracker) -> Self {
        Self { 
            frame, 
            dirty_blocks: SleepLock::new(Vec::new()) 
        }
    }

    pub async fn set_dirty(&self, offset: usize) {
        let idx = offset / BLOCK_SIZE;
        let mut dirty_blocks = self.dirty_blocks.lock().await;
        dirty_blocks.push(idx);
    }
}