use core::sync::atomic::Ordering;
use async_trait::async_trait;
use log::info;
use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, SEEK_SET}, file, Ext4File, InodeTypes
};
use riscv::register::mstatus::set_fs;
use crate::{
    fs::{ffi::{as_ext4_de_type, as_inode_type, InodeType}, open, page_cache::PageCache, root_inode, stat::as_inode_stat, FileTrait, InodeMeta, InodeTrait, Kstat, INODE_CACHE},
    sync::{new_shared, MutexGuard, NoIrqLock, Shared, TimeStamp},
    utils::{Errno, SysResult}
};

use alloc::{string::{String, ToString}, sync::Arc, vec::Vec};
use alloc::vec;
use alloc::boxed::Box;

use super::NormalFile;

pub struct Ext4Inode {
    pub metadata : InodeMeta,
    pub file     : Shared<Ext4File>,
    pub page_cache: Option<Arc<PageCache>>,
}

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    /// 创建一个inode，设置pagecache，并将其加入Inodecache
    pub fn new(path: &str, types: InodeTypes, page_cache: Option<Arc<PageCache>>) -> Arc<Self> {
        info!("path = {} ssssss", path);
        let file_type = as_inode_type(types.clone());
        let ext4file = new_shared(Ext4File::new(path, types.clone()));
        let mut file_size = 0u64;
        if types != InodeTypes::EXT4_DE_DIR {
            info!("path = {}, types = {}", path, true);
            ext4file.lock().file_open(path, O_RDONLY);
            file_size = ext4file.lock().file_size();
            info!("path = {}, size = {} aaaaaaaaaaa",path, file_size);
            ext4file.lock().file_close();
        }

        let inode = Arc::new(Self {
            metadata: InodeMeta::new(file_type, file_size as usize),
            file    : ext4file,
            page_cache: page_cache.clone()
        });
        // 修改 inode.page_cache
        if let Some(pg) = &inode.page_cache {
            pg.set_inode(inode.clone());
        }
        INODE_CACHE.insert(path, inode.clone());
        inode
    }
}

impl Drop for Ext4Inode {
    fn drop(&mut self) {
        self.sync();
        let mut file = self.file.lock();
        file.file_close().expect("failed to close fd");
    }
}

#[async_trait]
impl InodeTrait for Ext4Inode {
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        return self.page_cache.as_ref().cloned();
    }

    /// 获取文件大小
    fn get_size(&self) -> usize {
        let size = self.metadata.size.load(Ordering::Relaxed);
        size
    }

    fn set_size(&self, new_size: usize) -> SysResult {
        self.metadata.size.store(new_size, Ordering::Relaxed);
        Ok(())
    }

    /// 创建文件或者目录,self是父目录,path是子文件的绝对路径,这里是要创建一个Inode
    fn do_create(&self, path: &str, types: InodeTypes) -> Option<Arc<dyn InodeTrait>> {
        let page_cache = match types {
            InodeTypes::EXT4_DE_REG_FILE => Some(PageCache::new_bare()),
            _ => None
        };
        let nf = Ext4Inode::new(path, types.clone(), page_cache.clone());
        // nf.file.lock().file_open(path, O_RDWR).expect("[do_create] create file failed!");
        info!("[do_create] path = {}", path);
        
        Some(nf)
    }
    /// 获取文件类型
    fn node_type(&self) -> InodeType {
        as_inode_type(self.file.lock().file_type_get())
    }
    /// 读取文件 TODO(YJJ):这里可能有问题
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        let file_size = self.get_size();
        if file_size == 0 || offset >= file_size{
            info!("aaabusfdlkj");
            return 0;
        }

        // 缩减buf长度，不需要那么长
        if buf.len() > file_size - offset {
            buf = &mut buf[..file_size-offset];
        }

        match &self.page_cache {
            // 没有cache就直接读磁盘
            None => {
                self.read_dirctly(offset, buf).await
            }
            // 有cache就从cache中找
            Some(cache) => {
                cache.read(buf, offset).await
            }
        }
    }

    /// 直接读取
    async fn read_dirctly(&self, offset: usize, buf: &mut [u8]) -> usize {
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| Errno::EIO).unwrap();
        file.file_seek(offset as i64, SEEK_SET)
            .map_err(|_| Errno::EIO).unwrap();
        let r = file.file_read(buf);
        file.file_close().expect("[read_dirctly]: file close fail!");
        r.map_err(|_| Errno::EIO).unwrap()
    }

    /// 写入文件
    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        match &self.page_cache {
            None => {
                self.write_directly(offset, buf).await
            }
            Some(cache) => {
                cache.write(buf, offset).await
            }
        }
    }

    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        let file_size = self.get_size();
        if file_size < offset + buf.len() {
            self.set_size(buf.len() + offset).expect("[write_directly]: set size fail!");
        }
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR).map_err(|_| Errno::EIO).unwrap();
        file.file_seek(offset as i64, SEEK_SET)
            .map_err(|_| Errno::EIO).unwrap();
        let r = file.file_write(buf);
        file.file_close().expect("[write_directly]: file close fail!");
        r.map_err(|_| Errno::EIO).unwrap()
    }

    /// 改变文件size
    fn truncate(&self, size: usize) -> usize {
        let mut file = self.file.lock();

        let r = file.file_truncate(size as u64);
        self.set_size(size).expect("[truncate]: set size fail!");

        // file.file_close();
        r.map_or_else(|_| Errno::EIO.into(), |_| 0)
    }
    /// 同步文件
    async fn sync(&self) {
        info!("[ext4Inode sync] do sync with pagecache");
        if let Some(cache) = &self.page_cache {
            cache.flush().await;
        }
    }
    /// 读取文件所有内容
    async fn read_all(&self) -> SysResult<Vec<u8>> {
        info!("[read_all] read all file, size = {}", self.get_size());
        let mut buf = vec![0; self.get_size()];
        self.read_at(0, &mut buf).await;
        Ok(buf)
    }
    /// 判断在当前路径下是否有这个path的文件
    /// 
    /// self是parent，如果没找到就返回false，代表在lwext4中没有这个inode
    fn walk(&self, path: &str) -> bool {
        let mut file = self.file.lock();
        if file.check_inode_exist(path, InodeTypes::EXT4_DE_DIR) {
            true
        } else if file.check_inode_exist(path, InodeTypes::EXT4_DE_REG_FILE) {
            true
        } else {
            false
        }
    }
    /// 获取文件状态
    fn fstat(&self) -> Kstat {
        let size = match self.metadata.size.load(Ordering::Relaxed) {
            0 => self.get_size(),
            size => size
        };
        // info!("[Ext4Inode] fstat size = {}", size);
        let mut file = self.file.lock();
        // let size = self.size();
        match file.fstat() {
            Ok(stat) => {
                let (atime, mtime, ctime) = self.metadata.timestamp.lock().get();
                as_inode_stat(stat, atime, mtime, ctime, size)
            }
            Err(_) => Kstat::new()
        }
    }
    /// 删除文件
    fn unlink(&self, child_abs_path: &str) -> SysResult<usize> {
        // mayby bug? 这个用的parent cnt
        let mut lock_file = self.file.lock();
        INODE_CACHE.remove(child_abs_path);
        match lock_file.links_cnt().unwrap() {
            cnt if cnt <= 1 => {
                lock_file.file_remove(child_abs_path);
            }
            _ => { return Ok(0); }
        }
        Ok(0)
    }

    fn link(&self, new_path: &str) -> SysResult<usize> {
        let mut file = self.file.lock();
        file.link(new_path);
        Ok(0)
    }

    fn get_timestamp(&self) -> MutexGuard<'_, TimeStamp, NoIrqLock, > {
        self.metadata.timestamp.lock()
    }
    fn get_ext4file(&self) -> MutexGuard<'_, Ext4File, NoIrqLock, > {
        self.file.lock()
    }
    fn is_dir(&self) -> bool {
        self.metadata.file_type.is_dir()
    }
}