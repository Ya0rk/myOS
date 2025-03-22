use alloc::{string::String, sync::{Arc, Weak}};
use async_trait::async_trait;
use spin::Mutex;
use crate::{mm::UserBuffer, utils::{Errno, SysResult}};
use super::{FileTrait, InodeTrait, Kstat, SEEK_CUR, SEEK_END, SEEK_SET};
use alloc::boxed::Box;

pub struct NormalFile {
    readable: bool, // 该文件是否允许通过 sys_read 进行读
    writable: bool, // 该文件是否允许通过 sys_write 进行写
    pub inode: Arc<dyn InodeTrait>, // 文件的inode，在ext4中是Ext4_inode
    pub parent: Option<Weak<dyn InodeTrait>>, // 父目录的弱引用
    pub path: String, // 文件的路径
    pub inner: Mutex<NormalFileInner>, // 文件的内部状态
}
pub struct NormalFileInner {
    pub(crate) offset: usize, // 偏移量
}

impl NormalFile {
    pub fn new(
        readable: bool,
        writable: bool,
        inode: Arc<dyn InodeTrait>,
        parent: Option<Weak<dyn InodeTrait>>,
        path: String,
    ) -> Self {
        Self {
            readable,
            writable,
            inode,
            parent,
            path,
            inner: Mutex::new(NormalFileInner { offset: 0 }),
        }
    }
}

// 为 OSInode 实现 File Trait
#[async_trait]
impl FileTrait for NormalFile {
    fn readable(&self) -> SysResult<bool> {
        Ok(self.readable)
    }

    fn writable(&self) -> SysResult<bool> {
        Ok(self.writable)
    }

    async fn read(&self, mut buf: UserBuffer) -> SysResult<usize> {
        let mut inner = self.inner.lock();
        let mut total_read_size = 0usize;

        if self.inode.size() <= inner.offset {
            //读取位置超过文件大小，返回结果为EOF
            return Ok(0);
        }

        // 这边要使用 iter_mut()，因为要将数据写入
        for slice in buf.buffers.iter_mut() {
            let read_size = self.inode.read_at(inner.offset, *slice).await;
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        Ok(total_read_size)
    }

    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        let mut inner = self.inner.lock();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = self.inode.write_at(inner.offset, *slice).await;
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        Ok(total_write_size)
    }
    fn lseek(&self, offset: isize, whence: usize) -> SysResult<usize> {
        if offset < 0 || whence > 2 {
            return Err(Errno::EINVAL);
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
        Ok(inner.offset)
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok(self.path.clone())
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        let inode = self.inode.as_ref();
        *stat = inode.fstat();
        Ok(())
    }
}