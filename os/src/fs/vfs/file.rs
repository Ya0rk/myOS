use crate::{
    fs::{ffi::RenameFlags, Dirent, Kstat, OpenFlags},
    mm::page::Page,
    net::Socket,
    utils::{Errno, SysResult},
};
use alloc::{boxed::Box, string::ToString};
use alloc::{string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use lwext4_rust::bindings::EXT4_SUPERBLOCK_FLAGS_TEST_FILESYS;
use core::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
};
use downcast_rs::{impl_downcast, Downcast, DowncastSync};
use log::{info, warn};
use spin::RwLock;

use super::InodeTrait;

pub struct FileMeta {
    /// 文件偏移量
    pub offset: AtomicUsize,
    /// 打开文件时的权限
    pub flags: RwLock<OpenFlags>,
    /// 维护文件的inode，这里是Ext4Inode
    pub inode: Arc<dyn InodeTrait>,
}

impl FileMeta {
    pub fn new(flags: OpenFlags, inode: Arc<dyn InodeTrait>) -> Self {
        Self {
            offset: AtomicUsize::new(0),
            flags: RwLock::new(flags),
            inode,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    pub fn set_offset(&self, new_offset: usize) {
        self.offset.store(new_offset, Ordering::Relaxed);
    }
}

/// 文件接口
///
/// 该 trait 定义了文件操作的基本接口，所有文件类型都需要实现这个 trait。
/// 它提供了读取、写入、查询状态等基本文件操作。
#[async_trait]
pub trait FileTrait: Any + Send + Sync + DowncastSync {
    /// 获取metadata，进而得到flags和inode
    fn metadata(&self) -> &FileMeta;

    // ======== 权限相关 ========
    /// 设置权限
    fn set_flags(&self, flags: OpenFlags) {
        warn!("[FileTrait::set_flags] not implemented for this file type");
    }

    /// 从文件中读取数据到用户缓冲区
    async fn read(&self, buf: &mut [u8]) -> SysResult<usize>;

    /// 从指定偏移量读取数据到用户缓冲区
    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> SysResult<usize> {
        let inode = self.metadata().inode.clone();
        if offset > inode.get_size() {
            return Ok(0);
        }
        Ok(inode.read_at(offset, buf).await)
    }

    async fn write(&self, buf: &[u8]) -> SysResult<usize>;

    /// 将数据从指定偏移量写入文件，返回实际写入的字节数
    async fn write_at(&self, offset: usize, buf: &[u8]) -> SysResult<usize> {
        let inode = self.metadata().inode.clone();
        if offset > inode.get_size() {
            let newsize = offset + buf.len();
            inode.truncate(newsize);
        }
        Ok(inode.write_at(offset, buf).await)
    }

    /// 获取文件路径，这里是绝对路径
    fn abspath(&self) -> String {
        warn!("[FileTrait::get_name] not implemented for this file type");
        "Normal".to_string()
    }

    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        warn!("[FileTrait::fstat] not implemented for this file type");
        Err(Errno::ENOIMPL)
    }

    fn read_dents(&self, mut ub: usize, len: usize) -> SysResult<usize> {
        Err(Errno::ENOTDIR)
    }

    // TODO: 缓存未命中处理
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        warn!("[FileTrait::get_page_at] not implemented for this file type");
        None
    }

    fn get_socket(&self) -> SysResult<Arc<dyn Socket>> {
        Err(Errno::ENOTSOCK)
    }

    /// 从指定偏移量读取数据到用户缓冲区(主要是支持sys_pread64)
    async fn pread(&self, mut buf: &mut [u8], offset: usize, len: usize) -> SysResult<usize> {
        // unimplemented!("not support!");
        Ok(0)
    }
    /// 将数据从指定偏移量写入文件，返回实际写入的字节数(主要是支持sys_pwrite64)
    async fn pwrite(&self, buf: &[u8], offset: usize, len: usize) -> SysResult<usize> {
        // unimplemented!("not support!");
        Ok(0)
    }
    fn lseek(&self, _offset: isize, _whence: usize) -> SysResult<usize> {
        info!("{}", self.abspath());
        // unimplemented!("not support!");
        Ok(0)
    }

    // ppoll处理,代表数据到达，可以读取数据
    async fn pollin(&self) -> SysResult<bool> {
        // info!("[Filetrait::pollin] file: {}", self.get_name().unwrap());
        // info!("[pollin] use defaule implement");
        // println!("default implement");
        Ok(true)
    }
    // ppoll处理，代表可以写入数据，如 socket 发送缓冲区有空闲
    async fn pollout(&self) -> SysResult<bool> {
        info!("[pollout] use defaule implement");
        Ok(true)
    }
}

impl_downcast!(sync FileTrait);
