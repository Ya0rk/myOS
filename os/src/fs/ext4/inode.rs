use async_trait::async_trait;
use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, SEEK_SET},
    Ext4File, InodeTypes,
};
use spin::Mutex;
use crate::{
    fs::{ffi::{as_ext4_de_type, as_inode_type, InodeType}, 
    page_cache::PageCache, stat::as_inode_stat, InodeMeta, InodeTrait, Kstat, INODE_CACHE},
    sync::{new_shared, MutexGuard, NoIrqLock, Shared, TimeStamp},
    utils::{Errno, SysResult},
};

use alloc::{sync::Arc, vec::Vec};
use alloc::vec;
use alloc::boxed::Box;

pub struct Ext4Inode {
    pub metadata: InodeMeta,
    pub file    : Shared<Ext4File>,
    /// 页面缓存
    pub page_cache: Mutex<Option<Arc<PageCache>>>,
}

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    /// 创建一个inode，设置pagecache，并将其加入Inodecache
    pub fn new(path: &str, types: InodeTypes) -> Arc<Self> {
        let inode = Arc::new(Self {
            metadata: InodeMeta::new(),
            file    : new_shared(Ext4File::new(path, types)),
            page_cache: Mutex::new(None)
        });
        inode.set_page_cache();
        INODE_CACHE.insert(path, inode.clone());
        inode
    }

    pub fn set_page_cache(self: &Arc<Self>) {
        let mut cache = self.page_cache.lock();
        if cache.is_none() {
            let page_cache = PageCache::new(self.clone());
            *cache = Some(Arc::new(page_cache));
        }
    }
}

#[async_trait]
impl InodeTrait for Ext4Inode {
    /// 获取文件大小
    fn size(&self) -> usize {
        let mut file = self.file.lock();
        let types = as_inode_type(file.file_type_get());
        if types == InodeType::File {
            let path = file.get_path();
            let path = path.to_str().unwrap();
            let _ = file.file_open(path, O_RDONLY);
            let fsize = file.file_size();
            let _ = file.file_close();
            fsize as usize
        } else {
            0
        }
    }

    /// 创建文件或者目录
    fn do_create(&self, path: &str, ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        let types = as_ext4_de_type(ty);
        let mut file = self.file.lock();
        let nf = Ext4Inode::new(path, types.clone());

        if !file.check_inode_exist(path, types.clone()) {
            drop(file);
            let mut ext4file = nf.file.lock();
            if types == InodeTypes::EXT4_DE_DIR {
                if ext4file.dir_mk(path).is_err() {
                    return None;
                }
            } else {
                ext4file.file_open(path, O_RDWR | O_CREAT | O_TRUNC).expect("create file failed!");
                ext4file.file_close();
            }
        }
        Some(nf)
    }
    /// 获取文件类型
    fn node_type(&self) -> InodeType {
        as_inode_type(self.file.lock().file_type_get())
    }
    /// 读取文件
    async fn read_at(&self, off: usize, buf: &mut [u8]) -> usize {
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| Errno::EIO).unwrap();
        file.file_seek(off as i64, SEEK_SET)
            .map_err(|_| Errno::EIO).unwrap();
        let r = file.file_read(buf);
        file.file_close();
        r.map_err(|_| Errno::EIO).unwrap()
    }
    /// 写入文件
    async fn write_at(&self, off: usize, buf: &[u8]) -> usize {
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR).map_err(|_| Errno::EIO).unwrap();
        file.file_seek(off as i64, SEEK_SET)
            .map_err(|_| Errno::EIO).unwrap();
        let r = file.file_write(buf);
        file.file_close();
        r.map_err(|_| Errno::EIO).unwrap()
    }
    /// 截断文件
    fn truncate(&self, size: usize) -> usize {
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR | O_CREAT | O_TRUNC)
            .map_err(|_| Errno::EIO).unwrap();

        let r = file.file_truncate(size as u64);

        file.file_close();
        r.map_or_else(|_| Errno::EIO.into(), |_| 0)
    }
    /// 同步文件
    fn sync(&self) {
        todo!()
    }
    /// 读取文件所有内容
    fn read_all(&self) -> Result<Vec<u8>, Errno> {
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| Errno::EIO)?;
        let mut buf: Vec<u8> = vec![0; file.file_size() as usize];
        file.file_seek(0, SEEK_SET).map_err(|_| Errno::EIO)?;
        let r = file.file_read(buf.as_mut_slice());
        file.file_close();
        r.map_or_else(|_| Err(Errno::EIO), |_| Ok(buf))
    }
    /// 在当前路径下查询是否存在这个path的文件
    /// 
    /// 如果存在就创建一个inode
    fn walk(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let mut file = self.file.lock();
        if file.check_inode_exist(path, InodeTypes::EXT4_DE_DIR) {
            // debug!("lookup new DIR FileWrapper");
            Some(Ext4Inode::new(path, InodeTypes::EXT4_DE_DIR))
        } else if file.check_inode_exist(path, InodeTypes::EXT4_DE_REG_FILE) {
            // debug!("lookup new FILE FileWrapper");
            Some(Ext4Inode::new(path, InodeTypes::EXT4_DE_REG_FILE))
        } else {
            None
        }
    }
    /// 获取文件状态
    fn fstat(&self) -> Kstat {
        let mut file = self.file.lock();
        match file.fstat() {
            Ok(stat) => {
                let (atime, mtime, ctime) = self.metadata.timestamp.lock().get();
                as_inode_stat(stat, atime, mtime, ctime)
            }
            Err(_) => Kstat::new()
        }
    }
    /// 读取目录项
    fn read_dentry(&self, _off: usize, _len: usize) -> Option<(Vec<u8>, isize)> {
        todo!()
    }
    /// 删除文件
    fn unlink(&self, _child_name: &str) -> SysResult<usize> {
        todo!()
    }
    fn get_timestamp(&self) -> MutexGuard<'_, TimeStamp, NoIrqLock, > {
        self.metadata.timestamp.lock()
    }
    fn get_ext4file(&self) -> MutexGuard<'_, Ext4File, NoIrqLock, > {
        self.file.lock()
    }
}

impl Drop for Ext4Inode {
    fn drop(&mut self) {
        let mut file = self.file.lock();
        // debug!("Drop struct FileWrapper {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
    }
}