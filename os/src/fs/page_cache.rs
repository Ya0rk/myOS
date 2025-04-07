use core::cmp::min;

use alloc::{collections::btree_map::BTreeMap, sync::{Arc, Weak}};
use hashbrown::HashSet;
use log::info;
use spin::RwLock;
use crate::{config::{BLOCK_SIZE, PAGE_SIZE}, mm::{frame_alloc, FrameTracker}, sync::{yield_now, SleepLock}, task::get_current_cpu, utils::{Errno, SysResult}};
use super::InodeTrait;
use crate::mm::page::*;

/// 使用对齐的地址作为key
pub struct PageCache {
    pub pages: RwLock<BTreeMap<usize, Arc<Page>>>,
    inode: RwLock<Option<Weak<dyn InodeTrait>>>
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
            inode: RwLock::new(None)
        })
    }

    pub fn set_inode(&self, inode: Arc<dyn InodeTrait>) {
        self.inode.write().replace(Arc::downgrade(&inode));
    }

    /// 在page cache中寻找目标页
    pub fn get_page(
        &self,
        offset: usize,
    ) -> Option<Arc<Page>> {
        let offset_aligned = offset & !(PAGE_SIZE - 1);
        // 从cache中寻找
        self.pages.read().get(&offset_aligned).cloned()
    }

    /// 将page插入cache
    pub fn insert_page(
        &self,
        offset: usize
    ) -> Arc<Page> {
        let offset_aligned = offset & !(PAGE_SIZE - 1);
        // let frame = frame_alloc().expect("no more frame!");
        let page = Page::new_file();
        self.pages.write().insert(offset_aligned, page.clone());
        // info!("insert page: {:#x}", self.pages.read().get(&offset_aligned).unwrap().frame.ppn.0);
        page
    }

    /// 清空page cache，需要判断是否dirty
    pub async fn flush(
        &self,
    ) -> SysResult<usize> {
        for (page_addr_aligned, page) in self.pages.read().iter() {
            let inode = self.inode.read().as_ref().ok_or(Errno::EBADF)?.upgrade().unwrap();
            let mut dirty_blocks = page.dirty_set().unwrap().lock().await;
            if !dirty_blocks.is_empty() {
                for idx in dirty_blocks.iter() {
                    let start_offset = idx * BLOCK_SIZE;
                    let start = page_addr_aligned + start_offset;
                    let buf = &page.frame.ppn.get_bytes_array()[start_offset..start_offset + BLOCK_SIZE].to_vec();
                    inode.clone().write_directly(start, buf).await;
                }
                dirty_blocks.clear();
            }
        }
        Ok(0)
    }

    /// 利用cache中的page进行read
    pub async fn read(
        &self,
        buf: &mut [u8],
        offset: usize
    ) -> usize{
        let ppn_start = offset / PAGE_SIZE;
        let ppn_end = (offset + buf.len()).div_ceil(PAGE_SIZE);
        let mut page_offset = offset % PAGE_SIZE;
        let mut buf_cur = 0;
        

        for ppn in ppn_start..ppn_end {
            let page = match self.get_page(ppn * PAGE_SIZE){
                Some(page) => {
                    // info!("[PageCache] read from cache!");
                    page
                }, // cache中找到了page
                None => {
                    // info!("[PageCache] read from device! now page cnt: {}", self.pages.read().len());
                    // cache中没有找到就新建cache page
                    // 补commit：修复page_cache
                    // - let temp_page = self.insert_page(offset);
                    // + let temp_page = self.insert_page(ppn * PAGE_SIZE);
                    info!("[PageCache] read from device! now page cnt: {}", self.pages.read().len());
                    let temp_page = self.insert_page(ppn * PAGE_SIZE);
                    let array = temp_page.frame.ppn.get_bytes_array();
                    self.inode
                        .read()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .read_dirctly(ppn*PAGE_SIZE, array)
                        .await;
                    temp_page
                }
            };

            let page_buf = page.frame.ppn.get_bytes_array();
            let len = min(buf.len() - buf_cur, PAGE_SIZE - page_offset);
            buf[buf_cur..buf_cur + len].copy_from_slice(&page_buf[page_offset..page_offset + len]);
            buf_cur += len;
            // TODO: maybe bug?
            page_offset = 0;
            // 这里需要yield一下，防止cpu占用过高
            if get_current_cpu().timer_irq_cnt() >= 2 {
                yield_now().await;
            }
        }
        buf_cur
    }

    pub async fn write(
        &self,
        buf: &[u8],
        offset: usize
    ) -> usize {
        let ppn_start = offset / PAGE_SIZE;
        let ppn_end = (offset + buf.len()).div_ceil(PAGE_SIZE);
        let mut page_offset = offset % PAGE_SIZE;
        let mut buf_cur = 0;

        for ppn in ppn_start..ppn_end {
            let page = match self.get_page(ppn * PAGE_SIZE){
                Some(page) => page, // cache中找到了page
                None => {
                    // cache中没有找到就新建cache page
                    // info!("[PageCache] write from device! now page cnt: {}", self.pages.read().len());
                    self.insert_page(offset)
                }
            };
            // page.set_dirty(page_offset).await;

            let page_buf = page.frame.ppn.get_bytes_array();
            let len = min(buf.len() - buf_cur, PAGE_SIZE - page_offset);
            for off in (page_offset..page_offset+len).step_by(BLOCK_SIZE) {
                page.set_dirty(off).await;
            }
            page_buf[page_offset..page_offset + len].copy_from_slice(&buf[buf_cur..buf_cur + len]);
            buf_cur += len;
            // TODO: maybe bug?
            page_offset = 0;
            // 这里需要yield一下，防止cpu占用过高
            if get_current_cpu().timer_irq_cnt() >= 2 {
                info!("[PageCache] yield now!");
                yield_now().await;
            }
        }

        buf_cur
    }

}


// pub struct Page {
//     pub frame: FrameTracker,
//     /// 存放dirty block的idx
//     pub page_type: PageType,
// }

// pub enum PageType {
//     Anon,
//     File(DirtySet),
// }

// impl Page {
//     fn new_file() -> Self {
//         Self { 
//             frame: frame_alloc().expect("frame alloc failed"), 
//             page_type: PageType::File(DirtySet::new()),
//         }
//     }

//     fn new() -> Self {
//         Self {
//             frame: frame_alloc().expect("frame alloc failed"),
//             page_type: PageType::Anon,
//         }
//     }

//     pub async fn set_dirty(&self, offset: usize) {
//         // self.set_block(offset);
//         match self.page_type {
//             PageType::Anon => {
//                 panic!("Cannot set dirty block for an anonymous map!");
//             }
//             PageType::File(dirty_set) => {
//                 dirty_set.set_block(offset);
//             }
//         }
//     }

// }

// pub struct DirtySet(SleepLock<HashSet<usize>>);

// impl DirtySet {

//     pub fn new() -> Self {
//         SleepLock::new(HashSet::new())
//     }
//     pub async fn set_block(&self, offset: usize) {
//         let idx = offset / BLOCK_SIZE;
//         let mut dirty_blocks = self.0.lock().await;
//         dirty_blocks.insert(idx);
//     }

//     pub async fn get_blocks(&self) -> HashSet<usize> {
//         self.0.lock().await
//     }
    
// }