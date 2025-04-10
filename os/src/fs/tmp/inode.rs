use core::sync::atomic::Ordering;
use async_trait::async_trait;
use log::info;
use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, SEEK_SET},
    Ext4File, InodeTypes
};
use crate::{
    fs::{ffi::{as_ext4_de_type, as_inode_type, InodeType},
    page_cache::PageCache, stat::as_inode_stat, InodeMeta, InodeTrait, Kstat, INODE_CACHE},
    sync::{new_shared, MutexGuard, NoIrqLock, Shared, TimeStamp},
    utils::{Errno, SysResult}
};

use alloc::{sync::Arc, vec::Vec};
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

        let inode = Arc::new(Self {
            metadata: InodeMeta::new(file_type),
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
    fn size(&self) -> usize {
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
    fn do_create(&self, path: &str, ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        let types = as_ext4_de_type(ty);
        let mut file = self.file.lock();
        let page_cache = match ty {
            InodeType::File => Some(PageCache::new_bare()),
            _ => None
        };
        let nf = TmpInode::new(path, types.clone(), page_cache.clone());

        if !file.check_inode_exist(path, types.clone()) {
            drop(file);
            let mut ext4file = nf.file.lock();
            if types == InodeTypes::EXT4_DE_DIR {
                if ext4file.dir_mk(path).is_err() {
                    return None;
                }
            } else {
                ext4file.file_open(path, O_RDWR | O_CREAT | O_TRUNC).expect("create file failed!");
                ext4file.file_close().expect("[do_creat]: file clone fail!");
            }
        }
        Some(nf)
    }
    /// 获取文件类型
    fn node_type(&self) -> InodeType {
        as_inode_type(self.file.lock().file_type_get())
    }
    /// 读取文件 TODO(YJJ):这里可能有问题
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        let file_size = self.size();
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
        let mut buf = vec![0; self.size()];
        self.read_at(0, &mut buf).await;
        Ok(buf)
    }
    /// 在当前路径下查询是否存在这个path的文件
    /// 
    /// 如果存在就创建一个inode
    fn walk(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let mut file = self.file.lock();
        if file.check_inode_exist(path, InodeTypes::EXT4_DE_DIR) {
            // debug!("lookup new DIR FileWrapper");
            let page_cache = None;
            Some(TmpInode::new(path, InodeTypes::EXT4_DE_DIR, page_cache.clone()))
        } else if file.check_inode_exist(path, InodeTypes::EXT4_DE_REG_FILE) {
            // debug!("lookup new FILE FileWrapper");
            let page_cache = Some(PageCache::new_bare());
            Some(TmpInode::new(path, InodeTypes::EXT4_DE_REG_FILE, page_cache.clone()))
        } else {
            None
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