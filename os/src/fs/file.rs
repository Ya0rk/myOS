use alloc::{string::String, sync::{Arc, Weak}};
use async_trait::async_trait;
use crate::{mm::UserBuffer, utils::{Errno, SysResult}};
use super::{FileMeta, FileTrait, InodeTrait, Kstat, OpenFlags, SEEK_CUR, SEEK_END, SEEK_SET};
use alloc::boxed::Box;

pub struct NormalFile {
    pub parent: Option<Weak<dyn InodeTrait>>, // 父目录的弱引用
    pub path: String, // 文件的路径
    pub metadata: FileMeta,
}

impl NormalFile {
    pub fn new(
        flags: OpenFlags,
        inode: Arc<dyn InodeTrait>,
        parent: Option<Weak<dyn InodeTrait>>,
        path: String,
    ) -> Self {
        Self {
            parent,
            path,
            metadata: FileMeta::new(flags, inode),
        }
    }
}

// 为 OSInode 实现 File Trait
#[async_trait]
impl FileTrait for NormalFile {
    fn readable(&self) -> bool {
        self.metadata.flags.read().readable()
    }

    fn writable(&self) -> bool {
        self.metadata.flags.read().writable()
    }

    async fn read(&self, mut buf: UserBuffer) -> SysResult<usize> {
        let mut total_read_size = 0usize;

        if self.metadata.inode.size() <= self.metadata.offset() {
            //读取位置超过文件大小，返回结果为EOF
            return Ok(0);
        }

        // 这边要使用 iter_mut()，因为要将数据写入
        for slice in buf.buffers.iter_mut() {
            let old_offset = self.metadata.offset();
            let read_size = self.metadata.inode.read_at(old_offset, *slice).await;
            if read_size == 0 {
                break;
            }
            self.metadata.set_offset(old_offset+read_size);
            total_read_size += read_size;
        }
        Ok(total_read_size)
    }

    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let old_offset = self.metadata.offset();
            let write_size = self.metadata.inode.write_at(old_offset, *slice).await;
            assert_eq!(write_size, slice.len());
            self.metadata.set_offset(old_offset+write_size);
            total_write_size += write_size;
        }
        Ok(total_write_size)
    }
    fn lseek(&self, offset: isize, whence: usize) -> SysResult<usize> {
        if offset < 0 || whence > 2 {
            return Err(Errno::EINVAL);
        }
        let offset: usize = offset as usize;
        let old_offset = self.metadata.offset();
        let res = match whence {
            SEEK_SET => offset,
            SEEK_CUR => old_offset + offset,
            SEEK_END => offset + self.metadata.inode.size(),
            _ => return Err(Errno::EINVAL)
        };
        self.metadata.set_offset(res);
        Ok(res)
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok(self.path.clone())
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        let inode = self.metadata.inode.as_ref();
        *stat = inode.fstat();
        Ok(())
    }
}