use core::{any::Any, sync::atomic::AtomicUsize};

use crate::{
    fs::{
        ext4::NormalFile, ffi::InodeType, page_cache::PageCache, vfs::alloc_ino, AbsPath, Dentry,
        Dirent, FileClass, FileTrait, Kstat, ModeFlag, OpenFlags, StMode, SEEK_END,
    },
    sync::{once::LateInit, MutexGuard, NoIrqLock, SpinNoIrqLock, TimeStamp},
    utils::{downcast::Downcast, Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use async_trait::async_trait;
use downcast_rs::{impl_downcast, DowncastSync};
use log::warn;
use lwext4_rust::{Ext4File, Ext4InodeType};
use spin::Mutex;

/// inode的基础字段
///
/// timestamp: 每次访问和修改都要更新
pub struct InodeMeta {
    /// 节点的编号
    pub ino: usize,
    /// 文件大小
    pub size: AtomicUsize,
    /// 文件类型
    pub _type: InodeType,
    /// 时间戳
    pub timestamp: SpinNoIrqLock<TimeStamp>,
    pub abspath: String,
    pub i_mode: SpinNoIrqLock<StMode>,
}

impl InodeMeta {
    pub fn new(file_type: InodeType, file_size: usize, path: &str) -> Self {
        Self {
            ino: alloc_ino(),
            size: AtomicUsize::new(file_size),
            _type: file_type,
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
            abspath: String::from(path),
            i_mode: SpinNoIrqLock::new(StMode::new(ModeFlag::empty())),
        }
    }
}

#[async_trait]
pub trait InodeTrait: Any + Send + Sync {
    fn metadata(&self) -> &InodeMeta;
    
    fn is_valid(&self) -> bool {
        true
    }

    fn get_size(&self) -> usize {
        0
    }

    /// 设置大小
    fn set_size(&self, new_size: usize) -> SysResult {
        warn!("[InodeTrait::set_size] not implemented for this inode type");
        Err(Errno::ENOIMPL)
    }

    fn fstat(&self) -> Kstat {
        println!("path = {}, not implemented fstat", self.metadata().abspath);
        todo!()
    }

    /// 这里只是创建一个inode，打开文件还需要使用file结构体包裹inode，然后返回file
    fn do_create(&self, bare_dentry: Arc<Dentry>, _ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        None
    }
    /// 确实应当剥夺walk去创造Inode的权利
    fn look_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        todo!()
    }

    async fn read_at(&self, _off: usize, _buf: &mut [u8]) -> usize {
        println!("read_at: path {}", self.metadata().abspath);
        todo!()
    }

    /// 绕过cache，直接从磁盘读
    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        todo!()
    }

    async fn write_at(&self, _off: usize, _buf: &[u8]) -> usize {
        println!("write_at: path {}", self.metadata().abspath);
        todo!()
    }

    /// 直接写
    async fn write_directly(&self, _offset: usize, _buf: &[u8]) -> usize {
        todo!()
    }

    /// 将文件设置新的size，这里用于将文件size为0
    fn truncate(&self, _size: usize) -> usize {
        todo!()
    }

    /// Synchronizes the file's in-memory state with storage.
    async fn sync(&self) {
        todo!()
    }

    /// unlink 一个路径，将 inode 和这个路径解耦
    /// 注意到，应当传入一个有效的 dentry
    fn unlink(&self, valid_dentry: Arc<Dentry>) -> SysResult<usize> {
        Err(Errno::EINVAL)
    }

    /// link 一个路径，将 inode 和这个路径绑定
    /// 注意到，应当传入一个无效的（也就是没有被使用的）dentry
    fn link(&self, bare_dentry: Arc<Dentry>) -> SysResult<usize> {
        Err(Errno::EINVAL)
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        todo!();
    }

    /// 获取时间戳，用于修改或访问
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        todo!()
    }

    /// get page cache from ext4 file
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        warn!("[InodeTrait::get_page_cache] not implemented for this inode type");
        None
    }

    /// 更改名字
    /// 当前 inode 应当是old_path 所对应的
    fn rename(&self, old_path: Arc<Dentry>, new_path: Arc<Dentry>) -> SysResult<usize> {
        Err(Errno::EACCES)
    }

    /// 获得目录项
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        warn!("[InodeTrait::read_dents] not implemented for this inode type");
        None
    }

    /// io操作, 被sys_ioctl系统调用调用, 默认不支持这个操作
    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        Ok(0)
    }
}

impl dyn InodeTrait {
    pub fn set_timestamps(&self, timestamp: TimeStamp) -> SysResult<usize> {
        self.get_timestamp().lock().set(timestamp);
        Ok(0)
    }
}

impl Downcast for dyn InodeTrait {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}
