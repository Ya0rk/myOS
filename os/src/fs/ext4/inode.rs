use async_trait::async_trait;
use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, SEEK_SET},
    Ext4File, InodeTypes,
};
use spin::Mutex;
use crate::{
    fs::{page_cache::PageCache, stat::as_inode_stat, InodeMeta, InodeTrait, InodeType, Kstat},
    sync::{MutexGuard, NoIrqLock, SyncUnsafeCell, TimeStamp},
    utils::{Errno, SysResult},
};

use alloc::{sync::Arc, vec::Vec};
use alloc::vec;
use alloc::boxed::Box;

pub struct Ext4Inode {
    pub metadata: InodeMeta,
    pub file    : Arc<SyncUnsafeCell<Ext4File>>,
    /// 页面缓存
    pub page_cache: Mutex<Option<Arc<PageCache>>>,
}

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    pub fn new(path: &str, types: InodeTypes) -> Arc<Self> {
        let inode = Arc::new(Self {
            metadata: InodeMeta::new(),
            file    : Arc::new(SyncUnsafeCell::new(Ext4File::new(path, types))),
            page_cache: Mutex::new(None)
        });
        inode.set_page_cache();
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
        let file = self.file.get_unchecked_mut();
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

    /// 创建文件
    fn create(&self, path: &str, ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        let types = as_ext4_de_type(ty);
        let file = self.file.get_unchecked_mut();
        let nf = Ext4Inode::new(path, types.clone());

        if !file.check_inode_exist(path, types.clone()) {
            let nfile = nf.file.get_unchecked_mut();
            if types == InodeTypes::EXT4_DE_DIR {
                if nfile.dir_mk(path).is_err() {
                    return None;
                }
            } else if nfile.file_open(path, O_RDWR | O_CREAT | O_TRUNC).is_err() {
                return None;
            } else {
                let _ = nfile.file_close();
            }
        }
        Some(nf)
    }
    /// 获取文件类型
    fn node_type(&self) -> InodeType {
        as_inode_type(self.file.get_unchecked_mut().file_type_get())
    }
    /// 读取文件
    async fn read_at(&self, off: usize, buf: &mut [u8]) -> usize {
        let file = self.file.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| Errno::EIO).unwrap();
        file.file_seek(off as i64, SEEK_SET)
            .map_err(|_| Errno::EIO).unwrap();
        let r = file.file_read(buf);
        let _ = file.file_close();
        r.map_err(|_| Errno::EIO).unwrap()
    }
    /// 写入文件
    async fn write_at(&self, off: usize, buf: &[u8]) -> usize {
        let file = self.file.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR).map_err(|_| Errno::EIO).unwrap();
        file.file_seek(off as i64, SEEK_SET)
            .map_err(|_| Errno::EIO).unwrap();
        let r = file.file_write(buf);
        let _ = file.file_close();
        r.map_err(|_| Errno::EIO).unwrap()
    }
    /// 截断文件
    fn truncate(&self, size: usize) -> usize {
        let file = self.file.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR | O_CREAT | O_TRUNC)
            .map_err(|_| Errno::EIO).unwrap();

        let t = file.file_truncate(size as u64);

        let _ = file.file_close();
        if let Err(_) = t {
            Errno::EIO.into()
        } else {
            0
        }
    }
    /// 重命名文件
    fn rename(&self, _file: Arc<dyn InodeTrait>) -> SysResult<usize> {
        todo!()
    }
    /// 同步文件
    fn sync(&self) {
        todo!()
    }
    /// 读取文件所有内容
    fn read_all(&self) -> Result<Vec<u8>, Errno> {
        let file = self.file.get_unchecked_mut();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDONLY).map_err(|_| Errno::EIO)?;
        let mut buf: Vec<u8> = vec![0; file.file_size() as usize];
        file.file_seek(0, SEEK_SET).map_err(|_| Errno::EIO)?;
        let r = file.file_read(buf.as_mut_slice());
        let _ = file.file_close();
        if let Err(_) = r {
            Err(Errno::EIO)
        } else {
            Ok(buf)
        }
    }
    /// 在当前路径下查询是否存在这个path的文件
    fn find_by_path(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let file = self.file.get_unchecked_mut();
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
        let file = self.file.get_unchecked_mut();
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
}

impl Drop for Ext4Inode {
    fn drop(&mut self) {
        let file = self.file.get_unchecked_mut();
        // debug!("Drop struct FileWrapper {:?}", file.get_path());
        file.file_close().expect("failed to close fd");
    }
}

fn as_ext4_de_type(types: InodeType) -> InodeTypes {
    match types {
        InodeType::BlockDevice => InodeTypes::EXT4_DE_BLKDEV,
        InodeType::CharDevice => InodeTypes::EXT4_DE_CHRDEV,
        InodeType::Dir => InodeTypes::EXT4_DE_DIR,
        InodeType::Fifo => InodeTypes::EXT4_DE_FIFO,
        InodeType::File => InodeTypes::EXT4_DE_REG_FILE,
        InodeType::Socket => InodeTypes::EXT4_DE_SOCK,
        InodeType::SymLink => InodeTypes::EXT4_DE_SYMLINK,
        InodeType::Unknown => InodeTypes::EXT4_DE_UNKNOWN,
    }
}

fn as_inode_type(types: InodeTypes) -> InodeType {
    match types {
        InodeTypes::EXT4_INODE_MODE_FIFO => InodeType::Fifo,
        InodeTypes::EXT4_INODE_MODE_CHARDEV => InodeType::CharDevice,
        InodeTypes::EXT4_INODE_MODE_DIRECTORY => InodeType::Dir,
        InodeTypes::EXT4_INODE_MODE_BLOCKDEV => InodeType::BlockDevice,
        InodeTypes::EXT4_INODE_MODE_FILE => InodeType::File,
        InodeTypes::EXT4_INODE_MODE_SOFTLINK => InodeType::SymLink,
        InodeTypes::EXT4_INODE_MODE_SOCKET => InodeType::Socket,
        _ => {
            // warn!("unknown file type: {:?}", vtype);
            unreachable!()
        }
    }
}