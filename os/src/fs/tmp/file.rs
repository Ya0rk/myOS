use alloc::{string::String, sync::{Arc, Weak}};
use async_trait::async_trait;
use crate::{
    hal::config::PATH_MAX, 
    fs::{ffi::RenameFlags, FileMeta, FileTrait, InodeTrait, Kstat, OpenFlags, SEEK_CUR, SEEK_END, SEEK_SET}, 
    mm::UserBuffer, utils::{Errno, SysResult}
};
use alloc::boxed::Box;

pub struct TmpFile {
    pub path: String, // 文件的路径
    pub parent: Option<Weak<dyn InodeTrait>>, // 对父目录的弱引用
    pub metadata: FileMeta,
}

impl TmpFile {
    #[allow(unused)]
    pub fn new(
        flags: OpenFlags,
        parent: Option<Weak<dyn InodeTrait>>,
        inode: Arc<dyn InodeTrait>,
        path: String,
    ) -> Self {
        Self {
            path,
            parent,
            metadata: FileMeta::new(flags, inode),
        }
    }

    // 判断是否存在同名文件
    pub fn is_child(&self, path: &str) -> bool {
        self.parent
        .as_ref()
        .expect("no parent, plz check!")
        .upgrade()
        .unwrap()
        .walk(&path)
        .is_none()
    }
}

// 为 OSInode 实现 File Trait
#[async_trait]
impl FileTrait for TmpFile {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.metadata.inode.clone()
    }
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

        let mut new_offset = self.metadata.offset();
        // 这边要使用 iter_mut()，因为要将数据写入
        for slice in buf.buffers.iter_mut() {
            // TODO(YJJ):设置一定的时钟中断次数后yield
            // yield_now();
            let read_size = self.metadata.inode.read_at(new_offset, *slice).await;
            if read_size == 0 {
                break;
            }
            new_offset += read_size;
            total_read_size += read_size;
        }
        self.metadata.set_offset(new_offset);
        Ok(total_read_size)
    }

    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let old_offset = self.metadata.offset();
            let write_size = self.metadata.inode.write_at(old_offset, *slice).await;
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

    fn rename(&mut self, new_path: String, flags: RenameFlags) -> SysResult<usize> {
        if flags.contains(RenameFlags::RENAME_EXCHANGE)
            && (flags.contains(RenameFlags::RENAME_NOREPLACE)
                || flags.contains(RenameFlags::RENAME_WHITEOUT))
        {
            return Err(Errno::EINVAL);
        }

        let newpath_exist = self.is_child(&new_path);
        if newpath_exist && flags.contains(RenameFlags::RENAME_NOREPLACE) {
            return Err(Errno::EEXIST);
        }
        if flags.contains(RenameFlags::RENAME_EXCHANGE) && !newpath_exist {
            return Err(Errno::ENOENT);
        }

        let old_path = self.path.clone();
        if new_path.len() > PATH_MAX || old_path.len() > PATH_MAX {
            return Err(Errno::ENAMETOOLONG);
        }

        let mut ext4file = self.metadata.inode.get_ext4file();
        ext4file.file_rename(&old_path, &new_path).unwrap();
        self.path = new_path;
        
        Ok(0)
    }

    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        let inode = self.metadata.inode.as_ref();
        *stat = inode.fstat();
        Ok(())
    }

    fn is_dir(&self) -> bool {
        self.metadata.inode.is_dir()
    }
}