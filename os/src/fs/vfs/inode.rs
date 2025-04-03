use core::sync::atomic::AtomicUsize;
use crate::{
    fs::{ext4::NormalFile, ffi::InodeType, page_cache::PageCache, FileClass, FileTrait, Kstat, OpenFlags, SEEK_END},
    sync::{once::LateInit, MutexGuard, NoIrqLock, SpinNoIrqLock, TimeStamp},
    utils::SysResult
};
use alloc::{
    string::String, sync::{Arc, Weak},
    vec::Vec
};
use alloc::boxed::Box;
use async_trait::async_trait;
use lwext4_rust::Ext4File;
use spin::Mutex;
use super::alloc_ino;


/// inode的基础字段
/// 
/// timestamp: 每次访问和修改都要更新
pub struct InodeMeta {
    /// 节点的编号
    pub ino: usize,
    /// 文件大小
    pub size: AtomicUsize,
    /// 文件类型
    pub file_type: InodeType,
    /// 时间戳
    pub timestamp: SpinNoIrqLock<TimeStamp>,
}

impl InodeMeta {
    pub fn new(file_type: InodeType) -> Self {
        Self {
            ino:  alloc_ino(), 
            size: AtomicUsize::new(0), 
            file_type,
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
        }
    }
}

/// Virtual File System (VFS) Inode interface.
///
/// This trait defines the standard operations that can be performed on an inode
/// in the virtual file system. An inode represents either a file, directory, or
/// other file system object.
#[async_trait]
pub trait InodeTrait: Send + Sync {
    /// Returns the size of the file in bytes.
    ///
    /// # Returns
    ///
    /// The size of the file in bytes.
    fn size(&self) -> usize {
        todo!()
    }

    /// 设置大小
    fn set_size(&self, new_size: usize) -> SysResult;

    /// Returns the type of the inode (file, directory, etc.).
    ///
    /// # Returns
    ///
    /// An `InodeType` value indicating the type of this inode.
    fn node_type(&self) -> InodeType {
        todo!()
    }

    /// Returns the file status information.
    ///
    /// # Returns
    ///
    /// A `Kstat` structure containing various metadata about the file.
    fn fstat(&self) -> Kstat {
        todo!()
    }

    /// Creates a new file or directory in the current directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The name of the new file or directory
    /// * `ty` - The type of inode to create
    ///
    /// # Returns
    ///
    /// Some(Arc<dyn Inode>) if creation succeeds, None otherwise.
    fn do_create(&self, _path: &str, _ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        todo!()
    }

    /// Finds an inode by its path relative to this inode.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to search for
    ///
    /// # Returns
    ///
    /// Some(Arc<dyn Inode>) if found, None otherwise.
    fn walk(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        todo!()
    }

    /// Reads data from the file at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `off` - The offset from which to start reading
    /// * `buf` - The buffer to read into
    ///
    /// # Returns
    ///
    /// The number of bytes actually read.
    async fn read_at(&self, _off: usize, _buf: &mut [u8]) -> usize {
        todo!()
    }

    /// 绕过cache，直接从磁盘读
    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize;

    /// Writes data to the file at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `off` - The offset at which to start writing
    /// * `buf` - The buffer containing the data to write
    ///
    /// # Returns
    ///
    /// The number of bytes actually written.
    async fn write_at(&self, _off: usize, _buf: &[u8]) -> usize {
        todo!()
    }

    /// 直接写
    async fn write_directly(&self, _offset: usize, _buf: &[u8]) -> usize;    

    /// 将文件设置新的size，这里用于将文件size为0
    ///
    /// # Arguments
    ///
    /// * `size` - The new size for the file
    ///
    /// # Returns
    ///
    /// The actual new size of the file.
    fn truncate(&self, _size: usize) -> usize {
        todo!()
    }

    /// Synchronizes the file's in-memory state with storage.
    fn sync(&self) {
        todo!()
    }

    /// Removes a child entry from this directory.
    ///
    /// # Arguments
    ///
    /// * `child_name` - The name of the child to unlink
    ///
    /// # Returns
    ///
    /// Ok(0) on success, or an error code.
    fn unlink(&self, _child_name: &str) -> SysResult<usize> {
        todo!();
    }

    /// Reads the entire contents of the file.
    ///
    /// # Returns
    ///
    /// Ok(Vec<u8>) containing the file's contents, or an error code.
    async fn read_all(&self) -> SysResult<Vec<u8>> {
        todo!();
    }

    /// 将数据写回
    /// 
    /// offset：数据开始的地址
    /// 
    /// len : 长度
    /// 
    /// buf: 数据存在的位置
    fn write_back(self: Arc<Self>, _offset: usize, _len: usize, _buf: &[u8]) -> SysResult {
        todo!();
    }

    /// 获取时间戳，用于修改或访问
    fn get_timestamp(&self) -> MutexGuard<'_, TimeStamp, NoIrqLock, >;

    /// 获取lwext4的ext4file
    fn get_ext4file(&self) -> MutexGuard<'_, Ext4File, NoIrqLock, >;

    fn is_dir(&self) -> bool;

    /// get page cache from ext4 file
    fn get_page_cache(&self) -> Option<Arc<PageCache>>;
}

impl dyn InodeTrait {
    pub fn set_timestamps(&self, timestamp: TimeStamp) -> SysResult<usize> {
        let mut mytime = self.get_timestamp();
        mytime.set(timestamp);
        Ok(0)
    }

    /// 打开一个inode，创建一个管理该inode的对应的file返回
    pub fn do_open(self: Arc<Self>, parent: Option<Weak<dyn InodeTrait>>, flags: OpenFlags, path: String) -> Option<FileClass> {
        let new_file = NormalFile::new(
            flags, 
            parent,
            self,
            path
        );
        // 将指针移到文件末尾
        if flags.contains(OpenFlags::O_APPEND) {
            new_file.lseek(0, SEEK_END).unwrap();
        }
        // 截断文件长度为0
        if flags.contains(OpenFlags::O_TRUNC) {
            new_file.metadata.inode.truncate(0);
        }
        
        Some(FileClass::File(Arc::new(new_file)))
    }
}