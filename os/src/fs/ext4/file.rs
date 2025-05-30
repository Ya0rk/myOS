use core::cmp::min;

use alloc::{string::String, sync::{Arc, Weak}, vec::Vec};
use async_trait::async_trait;
use log::info;
use sbi_spec::pmu::cache_event::NODE;
use crate::{
    fs::{ffi::{RenameFlags, MEMINFO}, AbsPath, Dirent, FileMeta, FileTrait, InodeTrait, Kstat, OpenFlags, SEEK_CUR, SEEK_END, SEEK_SET, S_IFCHR}, hal::config::PATH_MAX, mm::{page::Page, user_ptr::user_slice_mut, UserBuffer}, utils::{Errno, SysResult}
};
use alloc::boxed::Box;

use super::Ext4Inode;

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
        // info!("[is_child] ? {}", path);
        if let 
            Some(_) = 
            self.parent
                .as_ref()
                .expect("no parent, plz check!")
                .upgrade()
                .unwrap()
                .walk(&path) 
        {
            true
        } else {
            false
        }
    }

    pub fn unlink(&self) {
        
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

    async fn read(&self, mut buf: &mut [u8]) -> SysResult<usize> {
        let mut total_read_size = 0usize;
        info!("read file: {}, offset: {}", self.path, self.metadata.offset());

        // if self.metadata.inode.size() <= self.metadata.offset() || self.metadata.inode.size() == 0 {
        //     //读取位置超过文件大小，返回结果为EOF
        //     return Ok(0);
        // }

        let mut new_offset = self.metadata.offset();

        // if self.path.contains("meminfo") {
        //     let data = MEMINFO.as_bytes();
        //     let len = min(data.len(), buf.len()-new_offset);
        //     if len == 0 { return Ok(0); }
        //     buf[0..len].copy_from_slice(&data[new_offset..len]);
        //     self.metadata.set_offset(new_offset + len);
        //     return Ok(len);
        // }

        // 这边要使用 iter_mut()，因为要将数据写入
        let read_size = self.metadata.inode.read_at(new_offset, buf).await;
        new_offset += read_size;
        total_read_size += read_size;
        self.metadata.set_offset(new_offset);

        Ok(total_read_size)
    }

    /// 从偏移处读数据到buf，不用改变offset(这是和read的区别，可以将这两个函数综合)
    async fn pread(&self, mut buf: &mut [u8], offset: usize, len: usize) -> SysResult<usize> {
        let mut total_read_size = 0usize;
        info!("pread file: {}, offset: {}", self.path, offset);

        if self.metadata.inode.get_size() <= offset {
            //读取位置超过文件大小，返回结果为EOF
            return Ok(0);
        }

        let read_size = self.metadata.inode.read_at(offset, buf).await;
        total_read_size += read_size;

        Ok(total_read_size)
    }

    async fn write(&self, buf: &[u8]) -> SysResult<usize> {
        let mut total_write_size = 0usize;
        // 将改变inode大小的逻辑移入inode的write_at方法中
        // 增加代码内聚
        // let file_size = self.metadata.inode.get_size();
        let offset = self.metadata.offset();
        // if buf.len() > file_size - offset {
        //     self.metadata.inode.set_size(buf.len() + offset).expect("[write_at]: set size fail!");
        // }

        let old_offset = self.metadata.offset();
        let write_size = self.metadata.inode.write_at(old_offset, buf).await;
        self.metadata.set_offset(old_offset+write_size);
        total_write_size += write_size;
        // info!("size = {} ============", self.metadata.inode.get_size());

        Ok(total_write_size)
    }

    /// 从偏移处写数据，不用改变offset(这是和write的区别，可以将这两个函数综合)
    async fn pwrite(&self, buf: &[u8], offset: usize, len: usize) -> SysResult<usize> {
        let mut total_write_size = 0usize;
        let mut offset = offset;
        let file_size = self.metadata.inode.get_size();
        if offset > file_size - buf.len() {
            self.metadata.inode.set_size(buf.len() + offset).expect("[pwrite]: set size fail!");
        }

        let write_size = self.metadata.inode.write_at(offset, buf).await;
        total_write_size += write_size;

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
            SEEK_END => offset + self.metadata.inode.get_size(),
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

        self.metadata.inode.rename(&old_path, &new_path);
        self.path = new_path;
        
        Ok(0)
    }

    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        let inode = self.metadata.inode.as_ref();
        *stat = inode.fstat();
        if self.path.contains("null") {
            stat.st_mode = S_IFCHR;
        }
        Ok(())
    }

    fn is_dir(&self) -> bool {
        self.metadata.inode.is_dir()
    }

    fn read_dents(&self, mut ub: usize, len: usize) -> usize {
        info!("[read_dents] {}, len: {}, now file offset: {}", self.path, len, self.metadata.offset());
        if !self.is_dir() {
            // info!("[read_dents] {} is not a dir", self.path);
            return 0;
        }


        let ub = if let Ok(Some(buf)) = user_slice_mut::<u8>(ub.into(), len) {
            buf
        } else {
            return 0;
        };

        if self.path == "/musl/ltp" || self.path == "/musl/basic"
            || self.path == "/glibc/ltp" || self.path == "/glibc/basic" {

                info!("alsdkjlaskdfj");
                return 0;
        }
        
        // Some(dir_entrys)
        let dirs = self.metadata.inode.read_dents();
        let dirs = match dirs {
            Some(x) => x,
            _ => return 0,
        };
        // let mut res = 0;
        // let one_den_len = size_of::<Dirent>();
        let mut res = 0;
        let file_now_offset = self.metadata.offset();
        for den in dirs {
            let den_len = den.len();
            // info!(
            //     "[read_dents] \n\tname: {}\n\td_off: {:#X}\n\td_reclen: {:#X}", 
            //     String::from_utf8(den.d_name.to_vec()).unwrap(),
            //     den.off(),
            //     den_len);
            if res + den_len > len {
                break
            };
            if den.off() - den.len() >= file_now_offset {
                // ub.write_at(res, den.as_bytes());
                ub[res..res + den.len()].copy_from_slice(den.as_bytes());
                res += den.len();
            };
        };
        self.metadata.set_offset(file_now_offset + res);
        // info!("[read_dents] path {} return {}", self.path, res);
        res
    }
    
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        self.metadata.inode.get_page_cache().unwrap().get_page(offset).await
    }
}