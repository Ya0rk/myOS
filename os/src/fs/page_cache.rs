use core::cmp::min;

use super::InodeTrait;
use crate::hal::config::align_down_by_page;
use crate::mm::page::*;
use crate::{
    hal::config::{BLOCK_SIZE, PAGE_SIZE},
    mm::{frame_alloc, FrameTracker},
    sync::{yield_now, SleepLock},
    task::get_current_cpu,
    utils::{Errno, SysResult},
};
use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
};
use hashbrown::HashSet;
use log::info;
use spin::RwLock;
use virtio_drivers::device::console::Size;

/// 使用对齐的地址作为key
pub struct PageCache {
    pub pages: RwLock<BTreeMap<usize, Arc<Page>>>,
    inode: RwLock<Option<Weak<dyn InodeTrait>>>,
}

impl PageCache {
    pub fn new(inode: Arc<dyn InodeTrait>) -> Self {
        Self {
            inode: RwLock::new(Some(Arc::downgrade(&inode))),
            pages: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn new_bare() -> Arc<Self> {
        Arc::new(Self {
            pages: RwLock::new(BTreeMap::new()),
            inode: RwLock::new(None),
        })
    }

    pub fn set_inode(&self, inode: Arc<dyn InodeTrait>) {
        self.inode.write().replace(Arc::downgrade(&inode));
    }

    /// 在page cache中寻找目标页
    pub async fn get_page(&self, offset: usize) -> Option<Arc<Page>> {
        let offset_aligned = offset & !(PAGE_SIZE - 1);
        // 从cache中寻找
        let page = self.pages.read().get(&offset_aligned).cloned();
        match page {
            Some(_) => page,
            None => {
                let new_page = self.insert_page(offset_aligned);
                let buf = new_page.frame.ppn.get_bytes_array();
                let len = self.inode
                    .read()
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .read_dirctly(offset_aligned, buf)
                    .await;
                if len < PAGE_SIZE {
                    buf[len..].fill(0);
                }
                // info!("[get_page] read page {:?}", buf);
                Some(new_page)
            }
        }
    }

    /// 将page插入cache
    pub fn insert_page(&self, offset: usize) -> Arc<Page> {
        let offset_aligned = offset & !(PAGE_SIZE - 1);
        let page = Page::new_file();
        self.pages.write().insert(offset_aligned, page.clone());
        page
    }

    /// 清空page cache，需要判断是否dirty
    pub async fn flush(&self) -> SysResult<usize> {
        for (page_addr_aligned, page) in self.pages.read().iter() {
            let inode = self
                .inode
                .read()
                .as_ref()
                .ok_or(Errno::EBADF)?
                .upgrade()
                .unwrap();
            let mut dirty_blocks = page.dirty_set().unwrap().lock().await;
            if !dirty_blocks.is_empty() {
                for idx in dirty_blocks.iter() {
                    let start_offset = idx * BLOCK_SIZE;
                    let start = page_addr_aligned + start_offset;
                    let buf = &page.frame.ppn.get_bytes_array()
                        [start_offset..start_offset + BLOCK_SIZE]
                        .to_vec();
                    inode.clone().write_directly(start, buf).await;
                }
                dirty_blocks.clear();
            }
        }
        Ok(0)
    }

    /// 利用cache中的page进行read
    pub async fn read(&self, buf: &mut [u8], offset: usize) -> usize {
        let ppn_start = offset / PAGE_SIZE;
        let ppn_end = (offset + buf.len()).div_ceil(PAGE_SIZE);
        let mut page_offset = offset % PAGE_SIZE;
        let mut buf_cur = 0;

        for ppn in ppn_start..ppn_end {
            let page = self.get_page(ppn * PAGE_SIZE).await.unwrap();

            let page_buf = page.frame.ppn.get_bytes_array();
            let len = min(buf.len() - buf_cur, PAGE_SIZE - page_offset);
            buf[buf_cur..buf_cur + len].copy_from_slice(&page_buf[page_offset..page_offset + len]);
            buf_cur += len;
            page_offset = 0;
            if get_current_cpu().timer_irq_cnt() >= 2 {
                yield_now().await;
            }
        }
        buf_cur
    }

    pub async fn write(&self, buf: &[u8], offset: usize) -> usize {
        let ppn_start = offset / PAGE_SIZE;
        let ppn_end = (offset + buf.len()).div_ceil(PAGE_SIZE);
        let mut page_offset = offset % PAGE_SIZE;
        let mut buf_cur = 0;

        for ppn in ppn_start..ppn_end {
            let page = self.get_page(ppn * PAGE_SIZE).await.unwrap();

            let page_buf = page.frame.ppn.get_bytes_array();
            let len = min(buf.len() - buf_cur, PAGE_SIZE - page_offset);
            for off in (page_offset..page_offset + len).step_by(BLOCK_SIZE) {
                page.set_dirty(off).await;
            }
            page_buf[page_offset..page_offset + len].copy_from_slice(&buf[buf_cur..buf_cur + len]);
            buf_cur += len;
            page_offset = 0;
            // 这里需要yield一下，防止cpu占用过高
            if get_current_cpu().timer_irq_cnt() >= 2 {
                info!("[PageCache] yield now!");
                yield_now().await;
            }
        }

        buf_cur
    }
    
    pub fn truncate(&self, new_size: usize) {
        let old_size = self.inode
            .read()
            .as_ref()
            .unwrap()
            .upgrade()
            .unwrap()
            .get_size();
        if new_size < old_size {
            let split_page_offset = align_down_by_page(new_size);
            let split_page = self.pages.read().get(&split_page_offset).cloned();
            if let Some(page) = split_page {
                page.ppn().get_bytes_array()[(new_size - split_page_offset)..].fill(0);
            }
            for page_offset in (split_page_offset + PAGE_SIZE..old_size).step_by(PAGE_SIZE) {
                self.pages.write().remove(&page_offset);
            }
        }
    }
}
