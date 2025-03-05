mod inode;

use super::{InodeType, Kstat, Statfs};
use crate::{
    mm::UserBuffer,
    syscall::PollEvents,
    utils::{GeneralRet, SysErrNo, SyscallRet},
};
use alloc::{sync::Arc, vec::Vec};

pub use inode::*;
///
pub trait SuperBlock: Send + Sync {
    fn root_inode(&self) -> Arc<dyn Inode>;
    fn sync(&self);
    fn fs_stat(&self) -> Statfs;
    fn ls(&self);
}
/// VfsInode接口
pub trait Inode: Send + Sync {
    ///
    fn size(&self) -> usize {
        unimplemented!()
    }
    ///
    fn node_type(&self) -> InodeType {
        unimplemented!()
    }
    ///
    fn fstat(&self) -> Kstat {
        unimplemented!()
    }
    /// 在当前目录下创建文件或目录
    fn create(&self, path: &str, ty: InodeType) -> Option<Arc<dyn Inode>> {
        unimplemented!()
    }
    /// 在当前目录下查找文件
    fn find_by_path(&self, path: &str) -> Option<Arc<dyn Inode>> {
        unimplemented!()
    }
    ///
    fn read_at(&self, off: usize, buf: &mut [u8]) -> SyscallRet {
        unimplemented!()
    }
    ///
    fn write_at(&self, off: usize, buf: &[u8]) -> SyscallRet {
        unimplemented!()
    }
    /// 读取目录项
    // fn read_dentry(&self, off: usize) -> Option<Dirent>;
    fn read_dentry(&self, off: usize, len: usize) -> Option<(Vec<u8>, isize)> {
        unimplemented!()
    }
    ///
    fn truncate(&self, size: usize) -> GeneralRet {
        unimplemented!()
    }
    ///
    fn sync(&self) {
        unimplemented!()
    }
    ///
    fn set_timestamps(&self, atime_sec: Option<u64>, mtime_sec: Option<u64>) -> GeneralRet {
        unimplemented!()
    }
    // fn link(&self);
    fn unlink(&self, child_name: &str) -> GeneralRet {
        unimplemented!();
    }
    fn rename(&self, file: Arc<dyn Inode>) -> GeneralRet {
        unimplemented!()
    }
    fn read_all(&self) -> Result<Vec<u8>, SysErrNo> {
        unimplemented!();
    }
    // #[cfg(feature = "fat32")]
    // fn ls(&self) -> Vec<String> {
    //     unimplemented!()
    // }
}

/// 文件接口
pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    /// read 指的是从文件中读取数据放到缓冲区中，最多将缓冲区填满，并返回实际读取的字节数
    fn read(&self, buf: UserBuffer) -> SyscallRet;
    /// 将缓冲区中的数据写入文件，最多将缓冲区中的数据全部写入，并返回直接写入的字节数
    fn write(&self, buf: UserBuffer) -> SyscallRet;
    /// ppoll处理
    fn poll(&self, events: PollEvents) -> PollEvents;
    /// 设置偏移量,并非所有文件都支持
    fn lseek(&self, offset: isize, whence: usize) -> SyscallRet {
        unimplemented!("not support!");
    }
}

pub trait Ioctl: File {
    /// ioctl处理
    fn ioctl(&self, cmd: usize, arg: usize) -> isize;
}
