use crate::{
    fs::{
        ffi::{as_ext4_de_type, as_inode_type, InodeType},
        open,
        page_cache::PageCache,
        root_inode,
        stat::as_inode_stat,
        Dentry, Dirent, FileTrait, InodeMeta, InodeTrait, Kstat,
    },
    sync::{new_shared, MutexGuard, NoIrqLock, Shared, SpinNoIrqLock, TimeStamp},
    utils::{Errno, SysResult},
};
use async_trait::async_trait;
use core::{error, sync::atomic::Ordering};
use log::{debug, error, info, warn};
use lwext4_rust::{
    bindings::{
        ext4_inode_stat, EXT4_DE_DIR, EXT4_DE_REG_FILE, O_CREAT, O_RDONLY, O_RDWR, O_TRUNC,
        SEEK_SET,
    },
    file, Ext4File, InodeTypes,
};

use alloc::boxed::Box;
use alloc::vec;
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use super::NormalFile;

pub struct Ext4Inode {
    pub metadata: InodeMeta,
    pub file: Shared<Ext4File>,
    pub page_cache: Option<Arc<PageCache>>,
}

unsafe impl Send for Ext4Inode {}
unsafe impl Sync for Ext4Inode {}

impl Ext4Inode {
    /// 创建一个inode，设置pagecache，并将其加入Inodecache
    pub fn new(
        path: &str,
        types: InodeTypes,
        page_cache: Option<Arc<PageCache>>,
    ) -> Arc<dyn InodeTrait> {
        // warn!("[Ext4Inode::new] path = {} ssssss", path);
        // if INODE_CACHE.has_inode(path) {
        //     return INODE_CACHE.get(path).clone().unwrap();
        // }
        let file_type = as_inode_type(types.clone());
        let ext4file = new_shared(Ext4File::new(path, types.clone()));
        let mut file_size = 0u64;
        if types == InodeTypes::EXT4_DE_DIR || types == InodeTypes::EXT4_INODE_MODE_DIRECTORY {
            // file_size = ext4file.lock().file_size();
            file_size = 0;
        } else {
            ext4file.lock().file_open(path, O_RDONLY);
            file_size = ext4file.lock().file_size();
            ext4file.lock().file_close();
        }

        let inode = Arc::new(Self {
            metadata: InodeMeta::new(file_type, file_size as usize, path),
            file: ext4file,
            page_cache: page_cache.clone(),
        });
        // 修改 inode.page_cache
        if let Some(pg) = &inode.page_cache {
            pg.set_inode(inode.clone());
        }
        // INODE_CACHE.insert(path, inode.clone());
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
    /// 检查inode是否有效
    fn is_valid(&self) -> bool {
        let mut file = self.file.lock();
        let types = file.get_type();
        let c_path = file.get_path();
        let c_path = c_path.to_str().unwrap();
        let res = file.check_inode_exist(c_path, types);
        // info!("[check inode is valid] path: {}, res: {}", c_path, res);
        res
    }

    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        return self.page_cache.as_ref().cloned();
    }

    /// 获取文件大小
    fn get_size(&self) -> usize {
        let size = self.metadata.size.load(Ordering::Relaxed);
        debug!("[get_size] {}", size);
        size
    }

    fn set_size(&self, new_size: usize) -> SysResult {
        self.metadata.size.store(new_size, Ordering::Relaxed);
        // info!("    [set_size] {}", new_size);
        Ok(())
    }

    /// 创建文件或者目录,self是父目录,path是子文件的绝对路径,这里是要创建一个Inode
    fn do_create(&self, bare_dentry: Arc<Dentry>, types: InodeType) -> Option<Arc<dyn InodeTrait>> {
        if bare_dentry.is_valid() {
            return None;
        }
        let path = &bare_dentry.get_abs_path();
        info!("[do_create] start {}", path);
        // let page_cache = match types.into() {
        //     InodeTypes::EXT4_DE_REG_FILE => Some(PageCache::new_bare()),
        //     _ => None
        // };
        let page_cache = Some(PageCache::new_bare());
        // 注意到原来这里是一个虚假的闯将,应当修改为在ext4文件系统中,真实的创建.
        // 这里文件创建的标识 O_RDWR | O_CREAT | O_TRUNC 和原来的fs/mod.rs::open函数中保持一致
        match types.clone().into() {
            InodeTypes::EXT4_DE_DIR => {
                debug!("[do_create] type is dir {}", path);
                let mut file = Ext4File::new(path, types.clone().into());
                if let Ok(_) = file.dir_mk(path) {
                    debug!("[do_create] succeed {}", path);
                } else {
                    debug!("[do_create] failed {}", path);
                }
                file.file_close();
            }
            InodeTypes::EXT4_DE_REG_FILE => {
                debug!("[do_create] type is reg file {}", path);
                let mut file = Ext4File::new(path, types.clone().into());
                if let Ok(_) = file.file_open(path, O_CREAT | O_TRUNC | O_RDWR) {
                    debug!("[do_create] succeed {}", path);
                } else {
                    debug!("[do_create] failed {}", path);
                }
                file.file_close();
            }
            _ => {}
        }

        let nf = Ext4Inode::new(path, types.clone().into(), page_cache.clone());
        bare_dentry.bind(nf.clone());
        if nf.is_valid() {
            info!("[do_create] succe {}", path);
        } else {
            info!("[do_create] faild {}", path);
        }

        Some(nf)
    }
    /// 获取文件类型
    fn node_type(&self) -> InodeType {
        as_inode_type(self.metadata.file_type.into())
    }
    /// 读取文件
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        let file_size = self.get_size();
        if file_size == 0 || offset >= file_size {
            return 0;
        }

        // 缩减buf长度，不需要那么长
        if buf.len() > file_size - offset {
            buf = &mut buf[..file_size - offset];
        }

        match &self.page_cache {
            // 没有cache就直接读磁盘
            None => self.read_dirctly(offset, buf).await,
            // 有cache就从cache中找
            Some(cache) => cache.read(buf, offset).await,
        }
    }

    /// 直接读取
    async fn read_dirctly(&self, offset: usize, buf: &mut [u8]) -> usize {
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        if file.file_open(path, O_RDONLY).is_err() {
            return 0;
        }
        file.file_seek(offset as i64, SEEK_SET)
            .map_err(|_| Errno::EIO)
            .unwrap();
        let r = file.file_read(buf);
        file.file_close()
            .expect("    [read_dirctly]: file close fail!");
        r.map_err(|_| Errno::EIO).unwrap()
    }

    /// 写入文件
    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let write_size = match &self.page_cache {
            None => {
                // info!("    [write_at] no cache");
                self.write_directly(offset, buf).await
            }
            Some(cache) => {
                // info!("    [write_at] has cache");
                cache.write(buf, offset).await
            }
        };
        // 增加代码内聚
        if self.get_size() < offset + write_size {
            self.set_size(offset + write_size);
        }
        // info!("    [write_at] return {}", write_size);
        write_size
    }

    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        let file_size = self.get_size();
        if file_size < offset + buf.len() {
            self.set_size(buf.len() + offset)
                .expect("[write_directly]: set size fail!");
        }
        let mut file = self.file.lock();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        file.file_open(path, O_RDWR)
            .map_err(|_| Errno::EIO)
            .unwrap();
        file.file_seek(offset as i64, SEEK_SET)
            .map_err(|_| Errno::EIO)
            .unwrap();
        let r = file.file_write(buf);
        file.file_close()
            .expect("[write_directly]: file close fail!");
        r.map_err(|_| Errno::EIO).unwrap()
    }

    /// 改变文件size
    fn truncate(&self, size: usize) -> usize {
        let mut file = self.file.lock();

        // let r = file.file_truncate(size as u64);  // 暂时注释
        self.set_size(size).expect("[truncate]: set size fail!");

        // file.file_close();
        // r.map_or_else(|_| Errno::EIO.into(), |_| 0) //暂时注释
        0
    }
    /// 同步文件
    async fn sync(&self) {
        debug!("[ext4Inode sync] do sync with pagecache");
        if let Some(cache) = &self.page_cache {
            cache.flush().await;
        }
    }
    /// 读取文件所有内容
    async fn read_all(&self) -> SysResult<Vec<u8>> {
        debug!("[read_all] read all file, size = {}", self.get_size());
        let mut buf = vec![0; self.get_size()];
        // info!("got enough buf");
        self.read_at(0, &mut buf).await;
        Ok(buf)
    }

    /// 恢复原来的作用!!!
    ///
    /// 在当前文件夹下查找该路径的的文件
    ///
    /// 返回一个InodeTrait
    ///
    /// 应当剥夺walk创造inode的权力todo
    fn look_up(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let mut file = self.file.lock();
        if file.check_inode_exist(path, InodeTypes::EXT4_DE_DIR) {
            // let page_cache = None;
            let page_cache = Some(PageCache::new_bare());
            Some(Ext4Inode::new(
                path,
                InodeTypes::EXT4_DE_DIR,
                page_cache.clone(),
            ))
        } else if file.check_inode_exist(path, InodeTypes::EXT4_DE_REG_FILE) {
            let page_cache = Some(PageCache::new_bare());
            Some(Ext4Inode::new(
                path,
                InodeTypes::EXT4_DE_REG_FILE,
                page_cache.clone(),
            ))
        } else {
            None
        }
    }
    /// 获取文件状态
    fn fstat(&self) -> Kstat {
        let size = match self.metadata.size.load(Ordering::Relaxed) {
            0 => self.get_size(),
            size => size,
        };
        debug!("[Ext4Inode] fstat size = {}", size);
        let mut file = self.file.lock();
        // let size = self.size();
        match file.fstat() {
            Ok(mut stat) => {
                let (atime, mtime, ctime) = self.metadata.timestamp.lock().get();
                stat.st_mode += 0o1000;
                as_inode_stat(stat, atime, mtime, ctime, size)
            }
            Err(_) => {
                let mut stat = ext4_inode_stat::default();
                let (atime, mtime, ctime) = self.metadata.timestamp.lock().get();
                stat.st_mode += 0o1000;
                as_inode_stat(stat, atime, mtime, ctime, size)
            }
        }
    }
    /// 删除文件
    fn unlink(&self, valid_dentry: Arc<Dentry>) -> SysResult<usize> {
        // mayby bug? 这个用的parent cnt
        let mut lock_file = self.file.lock();
        info!("[unlink] {}", lock_file.file_path.to_str().unwrap());
        // 获得要去 unlink 的路径
        let child_abs_path = &valid_dentry.get_abs_path();
        let res = if self.metadata.file_type.is_dir() {
            // info!("[unlink] unlink dir {}", child_abs_path);
            debug_point!("");
            lock_file.dir_rm(child_abs_path)
        } else {
            debug_point!("");
            lock_file.file_remove(child_abs_path)
        };
        match res {
            Ok(_) => {
                info!("[unlink] unlink success {}", child_abs_path);
                valid_dentry.release_self();
                Ok(0)
            }
            Err(e) => {
                warn!("[unlink] unlink failed {}, error: {:?}", child_abs_path, e);
                Err(Errno::EIO)
            }
        }
    }

    fn link(&self, bare_dentry: Arc<Dentry>) -> SysResult<usize> {
        let types = {
            self.node_type().into()
        };
        let mut file = self.file.lock();
        if bare_dentry.is_valid() {
            return Err(Errno::EEXIST);
        }
        let new_path = &bare_dentry.get_abs_path();
        info!(
            "    [ext4_link] {} to {}",
            file.file_path.to_str().unwrap(),
            new_path
        );
        
        match file.link(new_path) {
            Ok(_) => {
                debug_point!("[ext4_link]");
                let inode =
                    Ext4Inode::new(&new_path, types, self.get_page_cache());
                debug_point!("[ext4_link]");
                bare_dentry.bind(inode);
                debug_point!("[ext4_link]");
                Ok(0)
            }
            Err(_) => Err(Errno::EIO),
        }
    }

    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.metadata.timestamp
    }
    fn is_dir(&self) -> bool {
        self.metadata.file_type.is_dir()
    }

    fn rename(&self, old_dentry: Arc<Dentry>, new_dentry: Arc<Dentry>) -> SysResult<usize> {
        // 注意到这里并没有，check old_dentry 是否是 self， 其实 self 这个参数是没有用的
        let old_inode = if let Some(inode) = old_dentry.get_inode() {
            inode
        } else {
            return Err(Errno::ENOENT);
        };
        let mut ext4file = self.file.lock();
        if new_dentry.is_valid() {
            return Err(Errno::EEXIST);
        };
        let new_path = new_dentry.get_abs_path();
        let old_path = old_dentry.get_abs_path();
        match ext4file.file_rename(&old_path, &new_path) {
            Ok(_) => {
                let new_inode = Ext4Inode::new(
                    &new_path,
                    old_inode.node_type().into(),
                    old_inode.get_page_cache(),
                );
                new_dentry.bind(new_inode);
                old_dentry.release_self();
                Ok(0)
            }
            Err(_) => Err(Errno::EIO),
        }
    }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        let ext4_file = self.file.lock();
        let dirs = ext4_file.read_dir_from(0).unwrap();
        let mut dir_entrys = Vec::new();

        for dir in dirs {
            let (d_ino, d_off, d_reclen, d_type, d_name) =
                (dir.d_ino, dir.d_off, dir.d_reclen, dir.d_type, dir.d_name);

            let entry = Dirent::new(d_name, d_off, d_ino, d_type, d_reclen);
            dir_entrys.push(entry);
        }
        Some(dir_entrys)
    }
}
