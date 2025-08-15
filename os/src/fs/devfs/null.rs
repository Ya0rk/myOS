use crate::{
    fs::{ext4::NormalFile, page_cache::PageCache, Dirent, FileClass, FileMeta, SEEK_END},
    sync::{once::LateInit, MutexGuard, NoIrqLock, SpinNoIrqLock, TimeStamp},
};
use crate::{
    fs::{
        ffi::RenameFlags, FileTrait, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags, S_IFCHR,
    },
    mm::page::Page,
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
use lwext4_rust::Ext4InodeType;
use spin::Mutex;

pub struct DevNullInode {
    pub metadata: InodeMeta,
}

unsafe impl Send for DevNullInode {}
unsafe impl Sync for DevNullInode {}

impl DevNullInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self {
            metadata: InodeMeta::new(InodeType::CharDevice, 0, "/dev/null"),
        })
    }
}

#[async_trait]
impl InodeTrait for DevNullInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    fn get_size(&self) -> usize {
        0 // /dev/null 的大小始终为 0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(()) // /dev/null 不支持设置大小，直接返回成功
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = S_IFCHR; // 字符设备
        stat.st_ino = self.metadata.ino as u64;
        stat
    }

    fn look_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None // /dev/null 不支持路径解析
    }

    async fn read_at(&self, _off: usize, _buf: &mut [u8]) -> usize {
        0 // /dev/null 的读取始终返回 0 字节
    }

    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0 // /dev/null 的直接读取也返回 0 字节
    }

    async fn write_at(&self, _off: usize, buf: &[u8]) -> usize {
        buf.len() // /dev/null 的写入始终成功，返回写入的字节数
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len() // /dev/null 的直接写入也返回写入的字节数
    }

    fn truncate(&self, _size: usize) -> usize {
        0 // /dev/null 的大小始终为 0
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Ok(Vec::new()) // /dev/null 的读取始终返回空内容
    }

    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.metadata.timestamp // 返回一个空的时间戳
    }

    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None // /dev/null 不支持页面缓存
    }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None // /dev/null 不支持目录项读取
    }
}
