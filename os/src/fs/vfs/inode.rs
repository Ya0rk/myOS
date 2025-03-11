use crate::{
    fs::{SEEK_CUR, SEEK_END, SEEK_SET},
    mm::UserBuffer,
    utils::Errno,
};

use super::{File, Inode};
use alloc::{
    string::String,
    sync::{Arc, Weak},
};
use spin::Mutex;

pub struct OSInode {
    readable: bool, // 该文件是否允许通过 sys_read 进行读
    writable: bool, // 该文件是否允许通过 sys_write 进行写
    pub inode: Arc<dyn Inode>, // 文件的inode，在ext4中是Ext4_inode
    pub parent: Option<Weak<dyn Inode>>, // 父目录的弱引用
    pub path: String, // 文件的路径
    pub(crate) inner: Mutex<OSInodeInner>, // 文件的内部状态
}
pub struct OSInodeInner {
    pub(crate) offset: usize, // 偏移量
}

impl OSInode {
    pub fn new(
        readable: bool,
        writable: bool,
        inode: Arc<dyn Inode>,
        parent: Option<Weak<dyn Inode>>,
        path: String,
    ) -> Self {
        Self {
            readable,
            writable,
            inode,
            parent,
            path,
            inner: Mutex::new(OSInodeInner { offset: 0 }),
        }
    }
}

// 为 OSInode 实现 File Trait
impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut total_read_size = 0usize;

        if self.inode.size() <= inner.offset {
            //读取位置超过文件大小，返回结果为EOF
            return 0;
        }

        // 这边要使用 iter_mut()，因为要将数据写入
        for slice in buf.buffers.iter_mut() {
            let read_size = self.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }

    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = self.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
    fn lseek(&self, offset: isize, whence: usize) -> usize {
        if offset < 0 || whence > 2 {
            return Errno::EINVAL.into();
        }
        let offset: usize = offset as usize;
        let mut inner = self.inner.lock();
        if whence == SEEK_SET {
            inner.offset = offset;
        } else if whence == SEEK_CUR {
            inner.offset += offset;
        } else if whence == SEEK_END {
            inner.offset = self.inode.size() + offset;
        }
        inner.offset
    }
    
    fn get_name(&self) -> String {
        self.path.clone()
    }
}
