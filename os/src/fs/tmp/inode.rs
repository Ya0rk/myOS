use core::sync::atomic::Ordering;
use async_trait::async_trait;
use log::info;
use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, SEEK_SET}, file, Ext4File, InodeTypes
};
use crate::{
    fs::{ext4::Ext4Inode, ffi::{as_ext4_de_type, as_inode_type, InodeType}, page_cache::PageCache, stat::as_inode_stat, FileTrait, InodeMeta, InodeTrait, Kstat, INODE_CACHE},
    sync::{new_shared, MutexGuard, NoIrqLock, Shared, TimeStamp},
    utils::{Errno, SysResult}
};

use alloc::{string::String, sync::Arc, vec::Vec};
use alloc::vec;
use alloc::boxed::Box;

pub struct TmpInode {
    pub metadata : InodeMeta,
    pub file     : Shared<Ext4File>,
    pub page_cache: Option<Arc<PageCache>>,
}

unsafe impl Send for TmpInode {}
unsafe impl Sync for TmpInode {}

impl TmpInode {
    /// 创建一个inode，设置pagecache，并将其加入Inodecache
    pub fn new(path: &str, types: InodeTypes, page_cache: Option<Arc<PageCache>>) -> Arc<Self> {
        let file_type = as_inode_type(types.clone());
        let ext4file = new_shared(Ext4File::new(path, types));
        ext4file.lock().file_open(path, O_RDONLY);
        let file_size = ext4file.lock().file_size();
        ext4file.lock().file_close();

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

#[async_trait]
impl InodeTrait for TmpInode {
    /// 获取文件大小
    fn get_size(&self) -> usize {
        let mut lock_file = self.file.lock();
        let binding = lock_file.get_path();
        let path = binding.to_str().unwrap();
        lock_file.file_open(path, O_RDONLY).expect("[TmpInode new]: file open fail!");
        let size = lock_file.file_size() as usize;
        lock_file.file_close().expect("[TmpInode new]: file close fail!");
        size
    }

    fn set_size(&self, new_size: usize) -> SysResult {
        self.metadata.size.store(new_size, Ordering::Relaxed);
        Ok(())
    }

    /// 创建文件或者目录
    fn do_create(&self, path: &str, ty: InodeTypes) -> Option<Arc<dyn InodeTrait>> {
        let page_cache = match ty {
            InodeTypes::EXT4_DE_REG_FILE => Some(PageCache::new_bare()),
            _ => None
        };
        let nf = Ext4Inode::new(path, ty.clone(), page_cache.clone());
        info!("[do_create] path = {}", path);
        
        Some(nf)
    }
    /// 获取文件类型
    fn node_type(&self) -> InodeType {
        as_inode_type(self.file.lock().file_type_get())
    }
    /// 读取文件
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        let file_size = self.get_size();
        if file_size == 0 || offset > file_size{
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
    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0
    }

    /// 写入文件
    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        match &self.page_cache {
            None => {
                info!("llll");
                self.write_directly(offset, buf).await
            }
            Some(cache) => {
                // info!("ssss");
                cache.write(buf, offset).await
            }
        }
    }

    async fn write_directly(&self, _offset: usize, _buf: &[u8]) -> usize {
        0
    }

    /// 截断文件
    fn truncate(&self, size: usize) -> usize {
        self.set_size(size).expect("tmp inode set size fail");
        size
    }
    /// 同步文件
    async fn sync(&self) {
        todo!()
    }
    /// 读取文件所有内容
    async fn read_all(&self) -> SysResult<Vec<u8>> {
        let mut buf = vec![0; self.get_size()];
        self.read_at(0, &mut buf).await;
        Ok(buf)
    }
    /// 在当前路径下查询是否存在这个path的文件
    /// 
    /// 如果存在就创建一个inode
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
        // let (atime, mtime, ctime) = self.metadata.timestamp.lock().get();
        // let file_size = self.size();
        // let st_size = file_size / BLOCK_SIZE;
        // let ino= self.metadata.ino;
        Kstat::new()

        // let mut file = self.file.lock();
        // match file.fstat() {
        //     Ok(stat) => {
        //         let (atime, mtime, ctime) = self.metadata.timestamp.lock().get();
        //         as_inode_stat(stat, atime, mtime, ctime)
        //     }
        //     Err(_) => Kstat::new()
        // }
    }
    /// 删除文件
    fn unlink(&self, _child_name: &str) -> SysResult<usize> {
        todo!()
    }
    fn get_timestamp(&self) -> MutexGuard<'_, TimeStamp, NoIrqLock, > {
        self.metadata.timestamp.lock()
    }
    // fn get_ext4file(&self) -> MutexGuard<'_, Ext4File, NoIrqLock, > {
    //     self.file.lock()
    // }
    fn is_dir(&self) -> bool {
        self.metadata.file_type.is_dir()
    }

    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        self.page_cache.as_ref().cloned()
    }
}

impl Drop for TmpInode {
    fn drop(&mut self) {
        let mut file = self.file.lock();
        // debug!("Drop struct FileWrapper {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
    }
}