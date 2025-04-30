use core::sync::atomic::{AtomicUsize, Ordering};
use crate::{fs::{ffi::RenameFlags, Dirent, Kstat, OpenFlags}, mm::{page::Page, UserBuffer}, net::Socket, utils::SysResult};
use alloc::{string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use alloc::boxed::Box;
use log::info;
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
    fn get_inode(&self) -> Arc<dyn InodeTrait>;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn executable(&self) -> bool;

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

    /// 设置文件的当前偏移量
    ///
    /// 根据指定的偏移量和起始位置调整文件的当前读写位置。
    /// 并非所有文件类型都支持此操作。
    ///
    /// # 参数
    ///
    /// * `_offset` - 偏移量，可以是正数或负数
    /// * `_whence` - 起始位置，通常为 SEEK_SET(0)、SEEK_CUR(1) 或 SEEK_END(2)
    ///
    /// # 返回
    ///
    /// 设置后的新偏移量位置
    fn get_name(&self) -> SysResult<String>;
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize>;
    fn fstat(&self, stat: &mut Kstat) -> SysResult;
    fn is_dir(&self) -> bool;

    fn read_dents(&self, mut ub: UserBuffer, len: usize) -> usize {
        todo!()
    }

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

    // ppoll处理,代表数据到达，可以读取数据
    async fn pollin(&self) -> bool {
        info!("[pollin] use defaule implement");
        true
    }
    // ppoll处理，代表可以写入数据，如 socket 发送缓冲区有空闲
    async fn pollout(&self) -> bool {
        info!("[pollout] use defaule implement");
        true
    }
}

// pub trait Ioctl: File {
//     /// ioctl处理
//     fn ioctl(&self, cmd: usize, arg: usize) -> isize;
// }
