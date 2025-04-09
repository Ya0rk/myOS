use alloc::{string::String, sync::{Arc, Weak}, vec::Vec};
use async_trait::async_trait;
use log::info;
use sbi_spec::pmu::cache_event::NODE;
use crate::{
    config::PATH_MAX, 
    fs::{ffi::RenameFlags, Dirent, FileMeta, FileTrait, InodeTrait, Kstat, OpenFlags, SEEK_CUR, SEEK_END, SEEK_SET}, 
    mm::{UserBuffer, page::Page}, utils::{Errno, SysResult}
};
use alloc::boxed::Box;

pub struct NormalFile {
    pub path: String, // 文件的路径
    pub parent: Option<Weak<dyn InodeTrait>>, // 对父目录的弱引用
    pub metadata: FileMeta,
}

impl NormalFile {
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
impl FileTrait for NormalFile {
    fn set_flags(&self, flags: OpenFlags) {
        *self.metadata.flags.write() = flags;
    }

    fn get_flags(&self) -> OpenFlags {
        self.metadata.flags.read().clone()
    }

    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.metadata.inode.clone()
    }
    fn readable(&self) -> bool {
        self.metadata.flags.read().readable()
    }

    fn writable(&self) -> bool {
        self.metadata.flags.read().writable()
    }

    fn executable(&self) -> bool {
        let stat = self.metadata.inode.fstat();
        stat.st_mode & 0o111 != 0
    }

    async fn read(&self, mut buf: UserBuffer) -> SysResult<usize> {
        let mut total_read_size = 0usize;
        info!("read file: {}, offset: {}", self.path, self.metadata.offset());

        if self.metadata.inode.size() <= self.metadata.offset() {
            //读取位置超过文件大小，返回结果为EOF
            return Ok(0);
        }

        let mut new_offset = self.metadata.offset();
        for slice in buf.buffers.iter_mut() {
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

    /// 从指定偏移量读取数据到用户缓冲区
    async fn pread(&self, mut buf: UserBuffer, offset: usize, len: usize) -> SysResult<usize> {
        let mut total_read_size = 0usize;
        info!("pread file: {}, offset: {}", self.path, offset);

        if self.metadata.inode.size() <= offset {
            //读取位置超过文件大小，返回结果为EOF
            return Ok(0);
        }

        let mut new_offset = offset;
        for slice in buf.buffers.iter_mut() {
            let read_size = self.metadata.inode.read_at(new_offset, *slice).await;
            if read_size == 0 {
                break;
            }
            new_offset += read_size;
            total_read_size += read_size;
        }
        Ok(total_read_size)
    }

    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        let mut total_write_size = 0usize;
        let file_size = self.metadata.inode.size();
        let offset = self.metadata.offset();
        // TODO(YJJ): 如果不检测 maybe bug???
        if buf.len() > file_size - offset {
            // info!("[write] file size = {}", file_size);
            // info!("[write] buf size = {}, offset = {}", buf.len(), offset);
            // self.metadata.inode.truncate(offset + buf.len());
            self.metadata.inode.set_size(buf.len() + offset).expect("[write_at]: set size fail!");
            // info!("[write] set size = {}", self.metadata.inode.size());
        }
        for slice in buf.buffers.iter() {
            let old_offset = self.metadata.offset();
            let write_size = self.metadata.inode.write_at(old_offset, *slice).await;
            self.metadata.set_offset(old_offset+write_size);
            total_write_size += write_size;
        }
        Ok(total_write_size)
    }

    async fn pwrite(&self, buf: UserBuffer, offset: usize, len: usize) -> SysResult<usize> {
        let mut total_write_size = 0usize;
        let mut offset = offset;
        let file_size = self.metadata.inode.size();
        if offset > file_size - buf.len() {
            self.metadata.inode.set_size(buf.len() + offset).expect("[pwrite]: set size fail!");
        }
        for slice in buf.buffers.iter() {
            let write_size = self.metadata.inode.write_at(offset, *slice).await;
            total_write_size += write_size;
            offset += write_size;
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

    fn read_dentry(&self) -> Option<Vec<Dirent>> {
        if !self.is_dir() {
            return None;
        }

        let ext4_file = self.metadata.inode.get_ext4file();
        let dirs = ext4_file.read_dir_from(0).unwrap();
        let mut dir_entrys = Vec::new();

        for dir in dirs {
            let (d_ino, d_off, d_reclen, d_type, d_name) = (
                dir.d_ino,
                dir.d_off,
                dir.d_reclen,
                dir.d_type,
                dir.d_name
            );

            let entry = Dirent::new(d_name, d_off, d_ino, d_type, d_reclen);
            self.metadata.set_offset(d_off as usize);
            dir_entrys.push(entry);
        }
        Some(dir_entrys)
    }
    
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        self.metadata.inode.get_page_cache().unwrap().get_page(offset).await
    }
}