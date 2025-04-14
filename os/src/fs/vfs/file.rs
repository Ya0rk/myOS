use core::sync::atomic::{AtomicUsize, Ordering};
use crate::{fs::{ffi::RenameFlags, Dirent, Kstat, OpenFlags}, mm::{page::Page, UserBuffer}, net::Socket, utils::SysResult};
use alloc::{string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use alloc::boxed::Box;
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
            inode
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
pub trait FileTrait: Send + Sync {
    async fn read(&self, buf: UserBuffer) -> SysResult<usize>;
    async fn write(&self, buf: UserBuffer) -> SysResult<usize>;
    fn get_inode(&self) -> Arc<dyn InodeTrait>;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn executable(&self) -> bool;
    fn get_name(&self) -> SysResult<String>;
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize>;
    fn fstat(&self, stat: &mut Kstat) -> SysResult;
    fn is_dir(&self) -> bool;
    // TODO: 缓存未命中处理
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>>;

    fn get_socket(self: Arc<Self>) -> Arc<dyn Socket> {
        unimplemented!("not support!");
    }
    fn set_flags(&self, flags: OpenFlags){
        unimplemented!("not support!");
    }
    fn get_flags(&self) -> OpenFlags {
        unimplemented!("not support!");
    }
    /// 从指定偏移量读取数据到用户缓冲区(主要是支持sys_pread64)
    async fn pread(&self, mut buf: UserBuffer, offset: usize, len: usize) -> SysResult<usize>{
        unimplemented!("not support!");
    }
    /// 将数据从指定偏移量写入文件，返回实际写入的字节数(主要是支持sys_pwrite64)
    async fn pwrite(&self, buf: UserBuffer, offset: usize, len: usize) -> SysResult<usize> {
        unimplemented!("not support!");
    }
    fn lseek(&self, _offset: isize, _whence: usize) -> SysResult<usize> {
        unimplemented!("not support!");
    }
    fn read_dentry(&self) -> Option<Vec<Dirent>> {
        unimplemented!("not support!");
    }

    // ppoll处理
    // fn poll(&self, events: PollEvents) -> PollEvents;
}

// pub trait Ioctl: File {
//     /// ioctl处理
//     fn ioctl(&self, cmd: usize, arg: usize) -> isize;
// }
