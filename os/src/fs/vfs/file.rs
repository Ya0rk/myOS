use crate::{
    fs::{ffi::RenameFlags, Dirent, Kstat, OpenFlags},
    mm::{page::Page, UserBuffer},
    net::Socket,
    utils::{Errno, SysResult},
};
use alloc::{boxed::Box, string::ToString};
use alloc::{string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast, DowncastSync};
use core::{
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
    any::Any
};
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
    fn get_inode(&self) -> Arc<dyn InodeTrait>;

    // ======== 权限相关 ========
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn executable(&self) -> bool;

    // ======== 文件类型相关 ========
    /// 这里是临时机制，主要在splice中使用，后序可以通过inodetype实现判断
    fn is_pipe(&self) -> bool {
        false
    }
    /// 临时机制, 为了在ioctl中判断
    fn is_deivce(&self) -> bool {
        false
    }
    fn is_dir(&self) -> bool {
        false
    }

    /// 从文件中读取数据到用户缓冲区
    ///
    /// 尝试从文件中读取数据并填充到提供的缓冲区中，最多将缓冲区填满。
    ///
    /// # 参数
    ///
    /// * `buf` - 用户空间缓冲区，用于存储读取的数据
    ///
    /// # 返回
    ///
    /// 实际读取的字节数
    async fn read(&self, buf: &mut [u8]) -> SysResult<usize>;

    /// 从指定偏移量读取数据到用户缓冲区
    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> SysResult<usize> {
        let inode = self.get_inode();
        if offset > inode.get_size() {
            return Ok(0);
        }
        Ok(inode.read_at(offset, buf).await)
    }

    /// 将用户缓冲区中的数据写入文件
    ///
    /// 尝试将提供的缓冲区中的数据写入文件，最多写入缓冲区中的所有数据。
    ///
    /// # 参数
    ///
    /// * `buf` - 包含要写入数据的用户空间缓冲区
    ///
    /// # 返回
    ///
    /// 实际写入的字节数
    async fn write(&self, buf: &[u8]) -> SysResult<usize>;

    /// 将数据从指定偏移量写入文件，返回实际写入的字节数
    async fn write_at(&self, offset: usize, buf: &[u8]) -> SysResult<usize> {
        let inode = self.get_inode();
        if offset > inode.get_size() {
            let newsize = offset + buf.len();
            inode.truncate(newsize);
        }
        Ok(inode.write_at(offset, buf).await)
    }

    /// ppoll处理
    // fn poll(&self, events: PollEvents) -> PollEvents;

    /// 获取文件路径，这里是绝对路径
    fn get_name(&self) -> SysResult<String> {
        warn!("[FileTrait::get_name] not implemented for this file type");
        Ok("Normal".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        warn!("[FileTrait::rename] not implemented for this file type");
        Err(Errno::ENOIMPL)
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        warn!("[FileTrait::fstat] not implemented for this file type");
        Err(Errno::ENOIMPL)
    }

    fn read_dents(&self, mut ub: usize, len: usize) -> usize {
        unimplemented!("File Trait read_dents");
    }

    // TODO: 缓存未命中处理
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        warn!("[FileTrait::get_page_at] not implemented for this file type");
        None
    }

    fn get_socket(&self) -> SysResult<Arc<dyn Socket>> {
        Err(Errno::ENOTSOCK)
    }
    fn set_flags(&self, flags: OpenFlags) {
        warn!("[FileTrait::set_flags] not implemented for this file type");
    }
    fn get_flags(&self) -> OpenFlags {
        // unimplemented!("not support!");
        info!("[filetrait::get_flags] default");
        OpenFlags::O_RDWR
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
        info!("{}", self.get_name().unwrap());
        // unimplemented!("not support!");
        Ok(0)
    }

    // ppoll处理,代表数据到达，可以读取数据
    fn pollin(&self, waker: Waker) -> SysResult<bool> {
        // info!("[Filetrait::pollin] file: {}", self.get_name().unwrap());
        // info!("[pollin] use defaule implement");
        // println!("default implement");
        Ok(true)
    }
    // ppoll处理，代表可以写入数据，如 socket 发送缓冲区有空闲
    fn pollout(&self, waker: Waker) -> SysResult<bool> {
        info!("[pollout] use defaule implement");
        Ok(true)
    }
}

impl_downcast!(sync FileTrait);
