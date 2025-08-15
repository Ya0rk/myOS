use crate::{
    fs::{ffi::RenameFlags, Dirent, FileMeta, FileTrait, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags, S_IFCHR},
    mm::page::Page,
    sync::{SpinNoIrqLock, TimeStamp},
    utils::{Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use async_trait::async_trait;
use log::info;

pub struct DevZeroInode {
    metadata: InodeMeta,
}

impl DevZeroInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::CharDevice,
                0,
                "/dev/zero",
            ),
        })
    }
}

#[async_trait]
impl InodeTrait for DevZeroInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    fn get_page_cache(&self) -> Option<Arc<crate::fs::page_cache::PageCache>> {
        None
    }

    fn get_size(&self) -> usize {
        0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }

    async fn read_at(&self, _offset: usize, buf: &mut [u8]) -> usize {
        buf.fill(0);
        buf.len()
    }

    async fn write_at(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len()
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len()
    }

    fn truncate(&self, _size: usize) -> usize {
        0
    }

    async fn sync(&self) {}

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Ok(Vec::new())
    }

    fn look_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        stat.st_ino = self.metadata.ino as u64;
        stat
    }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }

    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        buf.fill(0);
        buf.len()
    }
}
