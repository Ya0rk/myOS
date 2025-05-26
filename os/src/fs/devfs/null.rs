use crate::{fs::{ffi::RenameFlags, FileTrait, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags, S_IFCHR}, mm::{page::Page, UserBuffer}, utils::{Errno, SysResult}};
use alloc::{string::{String, ToString}, sync::Arc, vec::Vec};
use async_trait::async_trait;
use lwext4_rust::InodeTypes;
use spin::Mutex;
use crate::{
    fs::{ext4::NormalFile, page_cache::PageCache, Dirent, FileClass, SEEK_END},
    sync::{once::LateInit, MutexGuard, NoIrqLock, SpinNoIrqLock, TimeStamp},
};
use alloc::boxed::Box;
use log::info;

pub struct DevNull {
    inode: Arc<DevNullInode>,
}

impl DevNull {
    pub fn new() -> Self {
        Self {
            inode: Arc::new(DevNullInode::new()),
        }
    }
}

#[async_trait]
impl FileTrait for DevNull {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    async fn read(&self, mut _user_buf: &mut [u8]) -> SysResult<usize> {
        Ok(0)
    }
    /// 填满0
    async fn pread(&self, mut user_buf: &mut [u8], offset: usize, len: usize) -> SysResult<usize> {
        info!("[pread] from nullfs, fill 0");
        user_buf.fill(0);
        Ok(len)
    }
    async fn write(&self, user_buf: & [u8]) -> SysResult<usize> {
        Ok(user_buf.len())
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/null".to_string())
    }

    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }

    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
}

struct DevNullInode {
    pub metadata: InodeMeta,
}

unsafe impl Send for DevNullInode {}
unsafe impl Sync for DevNullInode {}

impl DevNullInode {
    pub fn new() -> Self {
        Self {
            metadata: InodeMeta::new(InodeType::CharDevice, 0),
        }
    }
}

#[async_trait]
impl InodeTrait for DevNullInode {
    fn get_size(&self) -> usize {
        0 // /dev/null 的大小始终为 0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(()) // /dev/null 不支持设置大小，直接返回成功
    }

    fn node_type(&self) -> InodeType {
        InodeType::File // /dev/null 是一个文件类型
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = S_IFCHR; // 字符设备
        stat
    }

    fn do_create(&self, _path: &str, _ty: InodeType) -> Option<Arc<dyn InodeTrait>> {
        None // /dev/null 不支持创建子文件或目录
    }

    fn walk(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None // /dev/null 不支持路径解析
    }

    async fn read_at(&self, _off: usize, _buf: &mut [u8]) -> usize {
        0 // /dev/null 的读取始终返回 0 字节
    }

    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0 // /dev/null 的直接读取也返回 0 字节
    }

    async fn write_at(&self, _off: usize, buf: &[u8]) -> usize {
        buf.len() // /dev/null 的写入始终成功，返回写入的字节数
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len() // /dev/null 的直接写入也返回写入的字节数
    }

    fn truncate(&self, _size: usize) -> usize {
        0 // /dev/null 的大小始终为 0
    }

    async fn sync(&self) {
        // /dev/null 不需要同步操作
    }

    fn unlink(&self, _child_name: &str) -> SysResult<usize> {
        Err(Errno::EINVAL) // /dev/null 不支持删除操作
    }

    fn link(&self, _new_path: &str) -> SysResult<usize> {
        Err(Errno::EINVAL) // /dev/null 不支持链接操作
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Ok(Vec::new()) // /dev/null 的读取始终返回空内容
    }

    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.metadata.timestamp// 返回一个空的时间戳
    }

    fn is_dir(&self) -> bool {
        false // /dev/null 不是目录
    }

    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None // /dev/null 不支持页面缓存
    }

    // fn rename(&self, _old_path: &String, _new_path: &String) {
    //     // /dev/null 不支持重命名操作
    // }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None // /dev/null 不支持目录项读取
    }
}