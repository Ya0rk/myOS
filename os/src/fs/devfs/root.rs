use async_trait::async_trait;

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::{fs::{dirent::build_dirents, Dirent, InodeTrait, Kstat}, sync::{Shared, SpinNoIrqLock, TimeStamp}, utils::{Errno, SysResult}};

use super::urandom;


struct DevFsInode {
    timestamp: SpinNoIrqLock<TimeStamp>
}

impl DevFsInode {
    pub fn new() -> Self {
        Self{
            timestamp: SpinNoIrqLock::new(TimeStamp::new())
        }
    }
}

#[async_trait]
impl InodeTrait for DevFsInode {
    fn get_page_cache(&self) -> Option<alloc::sync::Arc<crate::fs::page_cache::PageCache> > {
        None
    }
    fn get_size(&self) -> usize {
        4096
    }
    fn set_size(&self,new_size:usize) -> crate::utils::SysResult {
        Ok(())
    }
    fn do_create(&self,_path: &str,_ty:crate::fs::InodeType) -> Option<alloc::sync::Arc<dyn InodeTrait> > {
        None
    }
    fn node_type(&self) -> crate::fs::InodeType {
        crate::fs::InodeType::Dir
    }
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        0
    }

    async fn read_dirctly(&self, offset: usize, buf: &mut [u8]) -> usize {
        0
    }
        // 疑似被弃用
    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        // 非常重要
        // 这里不能write_at
        0
    }
    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        // 这里不能write_directly
        0
    }
    fn truncate(&self, size: usize) -> usize {
        // 这里不能truncate
        0
    }
    async fn sync(&self) {
        // 这里不需要sync
    }
    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Err(Errno::EISDIR)
    }
    
    fn walk(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        // 暂时不实现
        None
    }
    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_mode = 16877;
        res.st_nlink = 1;
        res
    }
    fn unlink(&self, child_abs_path: &str) -> SysResult<usize> {
        // 这里不需要unlink
        Ok(0)
    }
    fn link(&self, new_path: &str) -> SysResult<usize> {
        // 这里不需要link
        Err(crate::utils::Errno::EACCES)
    }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.timestamp
    }
    fn is_dir(&self) -> bool {
        true
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        // (path, ino, d_type)
        let mut entries: Vec<(&str, u64, u8)> = alloc::vec![
            (".",       1, 4),
            ("..",      0, 4),
            ("null",    2, 6),
            ("rtc",     3, 0),
            ("tty",     4, 2),
            ("urandom", 5, 8),
            ("zero",    6, 8),
        ];
        Some(build_dirents(entries))
    }
}