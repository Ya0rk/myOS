use alloc::{string::String, sync::Arc, vec::{self, Vec}};
use async_trait::async_trait;
use alloc::boxed::Box;
use log::error;
use lwext4_rust::bindings::O_RDONLY;
use riscv::register::hcounteren::read;
use crate::{fs::{dirent::build_dirents, ffi::MEMINFO, open_file, Dirent, FileClass, InodeTrait, InodeType, Kstat, OpenFlags, Path}, sync::{SpinNoIrqLock, TimeStamp}, utils::SysResult};

/// ProcFsInodeInner 是一个枚举类型, 代表proc文件系统中的inode的类型
/// 
/// 它有四种类型:
/// 
/// - root: 代表proc文件系统的根目录
/// 
/// - _self: 代表当前进程的内容, 应当是一个文件夹
/// 
/// - exe: 代表当前执行的文件
/// 
/// - meminfo: 代表内存使用信息
enum ProcFsInodeInner {
    /// 根目录
    root,
    /// 当前进程的内容, 应当是一个文件夹
    _self,
    /// 当前执行的文件 
    exe,
    /// 内存使用信息
    meminfo,
}

/// ProcFsInode is a struct that represents an inode in the proc filesystem.
/// 
/// ProcFsInode 是一个表示proc文件系统中的inode的结构体
/// 
/// inner: 代表类型, 有root, _self, exe, meminfo四种类型
/// 
/// ptah: 代表路径, 例如"/proc/self", "/proc/meminfo"等
/// 
/// 讲道理是要为ProcFsInodeInner中的所有类型都实现一个ProcFsInode的
/// 
/// 但是就这几个就用模式匹配了
/// 
/// 也可以用继承的方式
/// 
pub struct ProcFsInode {
    inner: ProcFsInodeInner,
    path: String,
    timestamp: SpinNoIrqLock<TimeStamp>,
}

impl ProcFsInode {
    pub fn new_root(path: &str) -> Self {
        Self { 
            inner: ProcFsInodeInner::root,
            path: String::from(path),
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
        }
    }
    pub fn new_self(path: &str) -> Self {
        Self { 
            inner: ProcFsInodeInner::_self,
            path: String::from(path),
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
        }
    }
    pub fn new_exe(path: &str) -> Self {
        Self { 
            inner: ProcFsInodeInner::exe,
            path: String::from(path),
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
        }
    }
    pub fn new_meminfo(path: &str) -> Self {
        Self { 
            inner: ProcFsInodeInner::meminfo,
            path: String::from(path),
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
        }
    }
}

#[async_trait]
impl InodeTrait for ProcFsInode {
    fn get_page_cache(&self) -> Option<alloc::sync::Arc<crate::fs::page_cache::PageCache> > {
        // 这里不需要page_cache
        None
    }
    fn get_size(&self) -> usize {
        4000
    }
    fn set_size(&self,new_size:usize) -> crate::utils::SysResult {
        // 疑似被弃用
        Ok(())
    }
    fn do_create(&self,_path: &str,_ty:crate::fs::InodeType) -> Option<alloc::sync::Arc<dyn InodeTrait> > {
        // 这里不需要创建
        // 应当返回SysResult会更好,因为这个文件系统下就是不给创建文件
        None
    }
    fn node_type(&self) -> crate::fs::InodeType {
        match self.inner {
            ProcFsInodeInner::root => crate::fs::InodeType::Dir,
            ProcFsInodeInner::_self => crate::fs::InodeType::Dir,
            ProcFsInodeInner::exe => crate::fs::InodeType::File,
            ProcFsInodeInner::meminfo => crate::fs::InodeType::File,
        }
    }
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        // 非常重要
        match self.inner {
            ProcFsInodeInner::exe => {
                if let Ok(FileClass::File(exe)) = open_file("/bin/sh", OpenFlags::O_RDONLY) {
                    exe.metadata.inode.read_at(offset, &mut buf).await
                } else {
                    // error!("open /bin/sh failed");
                    0
                }
            }
            ProcFsInodeInner::meminfo => {
                // 这里不能read_at
                let mut meminfo = Vec::from(MEMINFO);
                let len = meminfo.len();
                if offset < len {
                    let read_len = core::cmp::min(len - offset, buf.len());
                    buf[..read_len].copy_from_slice(&meminfo[offset..offset + read_len]);
                    read_len
                } else {
                    0
                }
            }
            _ => {
                // error!("[read_at] is a directory");
                0
            }
        }
    }
    async fn read_dirctly(&self, offset: usize, buf: &mut [u8]) -> usize {
        // 疑似被弃用
        0
    }
    
    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        // 非常重要
        // 这里不能write_at
        0
    }
    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        // 这里不能write_directly
        0
    }
    fn truncate(&self, size: usize) -> usize {
        // 这里不能truncate
        0
    }
    async fn sync(&self) {
        // 这里不需要sync
    }
    async fn read_all(&self) -> SysResult<Vec<u8>> {
        // Ok(alloc::vec![])
        match self.inner {
            ProcFsInodeInner::exe => {
                // 瞎**返回一个, 在tcb里面没找到当前进程的可执行文件的路径
                if let Ok(FileClass::File(exe)) = open_file("/bin/sh", OpenFlags::O_RDONLY) {
                    exe.metadata.inode.read_all().await
                } else {
                    Err(crate::utils::Errno::EACCES)
                }
            }
            ProcFsInodeInner::meminfo => {
                // 也是瞎**返回
                let mut buf = Vec::from(MEMINFO);
                Ok(buf)
            }
            _ => {
                // error!("[read_all] is a directory");
                Err(crate::utils::Errno::EISDIR)
            }
        }
    }
    fn walk(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let pattern = Path::string2path(String::from(path)).get_filename();
        match self.inner {
            ProcFsInodeInner::root => {
                if pattern == "self" {
                    Some(Arc::new(ProcFsInode::new_self(path)))
                } else if pattern == "meminfo" {
                    Some(Arc::new(ProcFsInode::new_meminfo(path)))
                } else {
                    None
                }
            }
            ProcFsInodeInner::_self => {
                if pattern == "exe" {
                    Some(Arc::new(ProcFsInode::new_exe(path)))
                } else {
                    None
                }
            }
            _ => None,
            
        }
    }
    fn fstat(&self) -> Kstat {
        // 也是不严谨实现
        let mut res = Kstat::new();
        match self.inner {
            ProcFsInodeInner::exe => {
                if let Ok(FileClass::File(exe)) = open_file("/bin/sh", OpenFlags::O_RDONLY) {
                    exe.metadata.inode.fstat()
                } else {
                    // error!("open /bin/sh failed");
                    res.st_mode = InodeType::File as u32;
                    res
                }
            }
            ProcFsInodeInner::meminfo => {
                res.st_mode = InodeType::File as u32;
                res.st_nlink = 1;
                res.st_size = MEMINFO.len() as i64;
                res
            }
            _ => {
                // error!("[fstat] is a directory");
                res.st_mode = 16877;
                res.st_nlink = 1;
                res
            }
        }
    }
    fn unlink(&self, child_abs_path: &str) -> SysResult<usize> {
        // 这里不需要unlink
        Ok(0)
    }
    fn link(&self, new_path: &str) -> SysResult<usize> {
        // 这里不需要link
        Err(crate::utils::Errno::EACCES)
    }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.timestamp
    }
    fn is_dir(&self) -> bool {
        match self.inner {
            ProcFsInodeInner::root => true,
            ProcFsInodeInner::_self => true,
            _ => false,
        }
    }
    fn rename(&self, old_path: &String, new_path: &String) {
        // 不可以改名字
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        match self.inner {
            ProcFsInodeInner::root => {
                // (path, ino, d_type)
                let mut entries: Vec<(&str, u64, u8)> = alloc::vec![
                    (".", 1, 4),
                    ("..", 0, 4),
                    ("self", 2, 4),
                    ("meminfo", 3, 8),
                ];

                Some(build_dirents(entries))
            }
            ProcFsInodeInner::_self => {
                let mut entries = alloc::vec![
                    (".", 2, 4),
                    ("..", 1, 4),
                    ("exe", 4, 8),
                ];
                Some(build_dirents(entries))
            }
            _ => None,
        }
    }
}