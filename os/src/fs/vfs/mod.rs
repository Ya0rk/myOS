mod inode;

use super::{stat::Kstat, InodeType};
use crate::{
    mm::UserBuffer,
    utils::{Errno, SysResult},
};
use alloc::{string::String, sync::Arc, vec::Vec};

pub use inode::*;
///
pub trait SuperBlock: Send + Sync {
    fn root_inode(&self) -> Arc<dyn Inode>;
    fn sync(&self);
    // fn fs_stat(&self) -> Statfs;
    fn ls(&self);
}
/// Virtual File System (VFS) Inode interface.
///
/// This trait defines the standard operations that can be performed on an inode
/// in the virtual file system. An inode represents either a file, directory, or
/// other file system object.
pub trait Inode: Send + Sync {
    /// Returns the size of the file in bytes.
    ///
    /// # Returns
    ///
    /// The size of the file in bytes.
    fn size(&self) -> usize {
        unimplemented!()
    }

    /// Returns the type of the inode (file, directory, etc.).
    ///
    /// # Returns
    ///
    /// An `InodeType` value indicating the type of this inode.
    fn node_type(&self) -> InodeType {
        unimplemented!()
    }

    /// Returns the file status information.
    ///
    /// # Returns
    ///
    /// A `Kstat` structure containing various metadata about the file.
    fn fstat(&self) -> Kstat {
        unimplemented!()
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
    fn create(&self, _path: &str, _ty: InodeType) -> Option<Arc<dyn Inode>> {
        unimplemented!()
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
    fn find_by_path(&self, _path: &str) -> Option<Arc<dyn Inode>> {
        unimplemented!()
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
    fn read_at(&self, _off: usize, _buf: &mut [u8]) -> usize {
        unimplemented!()
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
    fn write_at(&self, _off: usize, _buf: &[u8]) -> usize {
        unimplemented!()
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
        unimplemented!()
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
        unimplemented!()
    }

    /// Synchronizes the file's in-memory state with storage.
    fn sync(&self) {
        unimplemented!()
    }

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
    fn set_timestamps(&self, _atime_sec: Option<u64>, _mtime_sec: Option<u64>) -> SysResult<usize> {
        unimplemented!()
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
        unimplemented!();
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
    fn rename(&self, _file: Arc<dyn Inode>) -> SysResult<usize> {
        unimplemented!()
    }

    /// Reads the entire contents of the file.
    ///
    /// # Returns
    ///
    /// Ok(Vec<u8>) containing the file's contents, or an error code.
    fn read_all(&self) -> Result<Vec<u8>, Errno> {
        unimplemented!();
    }
}

/// 文件接口
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    /// read 指的是从文件中读取数据放到缓冲区中，最多将缓冲区填满，并返回实际读取的字节数
    fn read(&self, buf: UserBuffer) -> usize;
    /// 将缓冲区中的数据写入文件，最多将缓冲区中的数据全部写入，并返回直接写入的字节数
    fn write(&self, buf: UserBuffer) -> usize;
    /// ppoll处理
    // fn poll(&self, events: PollEvents) -> PollEvents;
    /// 设置偏移量,并非所有文件都支持
    fn lseek(&self, _offset: isize, _whence: usize) -> usize {
        unimplemented!("not support!");
    }

    fn get_name(&self) -> String;

    fn fstat(&self, stat: &mut Kstat) -> ();
}

// pub trait Ioctl: File {
//     /// ioctl处理
//     fn ioctl(&self, cmd: usize, arg: usize) -> isize;
// }
