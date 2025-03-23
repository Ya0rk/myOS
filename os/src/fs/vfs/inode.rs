use crate::{
    fs::{InodeType, Kstat}, sync::{MutexGuard, NoIrqLock, SpinNoIrqLock, TimeStamp}, utils::{Errno, SysResult}
};
use alloc::{
    sync::Arc, vec::Vec,
};
use alloc::boxed::Box;
use async_trait::async_trait;
use super::alloc_ino;


/// inode的基础字段
/// 
/// timestamp: 每次访问和修改都要更新
pub struct InodeMeta {
    /// 节点的编号
    pub ino: usize,
    /// 文件大小
    pub size: usize,
    /// 时间戳
    pub timestamp: SpinNoIrqLock<TimeStamp>,
}

impl InodeMeta {
    pub fn new() -> Self {
        Self {
            ino: alloc_ino(), 
            size: 0, 
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
    fn create(&self, _path: &str, _ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
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
    fn find_by_path(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
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

    /// Reads directory entries.
    ///
    /// # Arguments
    ///
    /// * `off` - The offset from which to start reading entries
    /// * `len` - Maximum number of bytes to read
    ///
    /// # Returns
    ///
    /// Some((Vec<u8>, isize)) containing the directory entries and the next offset,
    /// or None if no more entries.
    fn read_dentry(&self, _off: usize, _len: usize) -> Option<(Vec<u8>, isize)> {
        todo!()
    }

    /// Truncates or extends the file to the specified size.
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

    /// Renames a file to a new location.
    ///
    /// # Arguments
    ///
    /// * `file` - The inode to be renamed
    ///
    /// # Returns
    ///
    /// Ok(0) on success, or an error code.
    fn rename(&self, _file: Arc<dyn InodeTrait>) -> SysResult<usize> {
        todo!()
    }

    /// Reads the entire contents of the file.
    ///
    /// # Returns
    ///
    /// Ok(Vec<u8>) containing the file's contents, or an error code.
    fn read_all(&self) -> Result<Vec<u8>, Errno> {
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
}

impl dyn InodeTrait {
    /// Sets the access and modification times of the file.
    ///
    /// # Arguments
    ///
    /// * `atime_sec` - Optional new access time in seconds
    /// * `mtime_sec` - Optional new modification time in seconds
    ///
    /// # Returns
    ///
    /// Ok(0) on success, or an error code.
    fn set_timestamps(&self, timestamp: TimeStamp) -> SysResult<usize> {
        let mut mytime = self.get_timestamp();
        mytime.set(timestamp);
        Ok(0)
    }
}