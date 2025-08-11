use crate::{
    fs::{
        dirent::build_dirents, ffi::MEMINFO, open, procfs::irqtable::{SupervisorExternal, SupervisorTimer, IRQTABLE}, AbsPath, Dirent, FileClass, InodeTrait, InodeType, Kstat, OpenFlags
    },
    mm::frame_allocator::{FrameAllocator, StackFrameAllocator, FRAME_ALLOCATOR},
    sync::{SpinNoIrqLock, TimeStamp},
    utils::SysResult,
};
use alloc::{boxed::Box, format};
use alloc::{
    string::String,
    sync::Arc,
    vec::{self, Vec},
};
use async_trait::async_trait;
use log::error;
use lwext4_rust::bindings::O_RDONLY;

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
pub enum ProcFsInodeInner {
    /// 根目录
    root,
    /// 当前进程的内容, 应当是一个文件夹
    _self,
    /// 当前执行的文件
    exe,
    /// 内存使用信息
    meminfo,
    /// 记录当前系统挂载的所有文件系统信息(busybox的df测例)
    mounts,
    /// 记录中断次数
    interrupts,
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
    /// path为绝对路径，inner为要创建的类型
    pub fn new(path: &str, inner: ProcFsInodeInner) -> Self {
        Self {
            inner,
            path: path.into(),
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
        }
    }
}

#[async_trait]
impl InodeTrait for ProcFsInode {
    fn get_page_cache(&self) -> Option<alloc::sync::Arc<crate::fs::page_cache::PageCache>> {
        // 这里不需要page_cache
        None
    }
    fn get_size(&self) -> usize {
        4000
    }
    fn set_size(&self, new_size: usize) -> crate::utils::SysResult {
        // 疑似被弃用
        Ok(())
    }
    fn node_type(&self) -> crate::fs::InodeType {
        match self.inner {
            ProcFsInodeInner::root => crate::fs::InodeType::Dir,
            ProcFsInodeInner::_self => crate::fs::InodeType::Dir,
            ProcFsInodeInner::exe => crate::fs::InodeType::File,
            ProcFsInodeInner::meminfo => crate::fs::InodeType::File,
            ProcFsInodeInner::mounts => crate::fs::InodeType::File,
            ProcFsInodeInner::interrupts => crate::fs::InodeType::File,
        }
    }
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        // 非常重要
        match self.inner {
            ProcFsInodeInner::exe => {
                if let Ok(FileClass::File(exe)) = open("/bin/sh".into(), OpenFlags::O_RDONLY) {
                    exe.metadata.inode.read_at(offset, &mut buf).await
                } else {
                    // error!("open /bin/sh failed");
                    0
                }
            }
            ProcFsInodeInner::meminfo => {
                // 这里不能read_at
                let (mem_total, mem_free, mem_available) = {
                    let frame_allocator = FRAME_ALLOCATOR.lock();
                    (
                        frame_allocator.frame_total() * 4,
                        frame_allocator.frame_free() * 4,
                        frame_allocator.frame_free() * 4,
                    )
                };

                // TODO: 要补充
                let meminfo = format!(
                    r"MemTotal:     {mem_total:>10} kB
MemFree:      {mem_free:>10} kB
MemAvailable: {mem_available:>10} kB
",
                    mem_total = mem_total,
                    mem_free = mem_free,
                    mem_available = mem_available
                );

                let meminfo = Vec::from(meminfo);
                let len = meminfo.len();
                if offset < len {
                    let read_len = core::cmp::min(len - offset, buf.len());
                    buf[..read_len].copy_from_slice(&meminfo[offset..offset + read_len]);
                    read_len
                } else {
                    0
                }
            }
            ProcFsInodeInner::interrupts => {
                let irqinfo = IRQTABLE.lock().tostring();
                let irqinfo = Vec::from(irqinfo);
                let len = irqinfo.len();
                if offset < len {
                    let read_len = core::cmp::min(len - offset, buf.len());
                    buf[..read_len].copy_from_slice(&irqinfo[offset..offset + read_len]);
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
                // 随便返回一个, 在tcb里面没找到当前进程的可执行文件的路径
                if let Ok(FileClass::File(exe)) = open("/bin/sh".into(), OpenFlags::O_RDONLY) {
                    exe.metadata.inode.read_all().await
                } else {
                    Err(crate::utils::Errno::EACCES)
                }
            }
            ProcFsInodeInner::meminfo => {
                // 也是随便返回

                let (mem_total, mem_free, mem_available) = {
                    let frame_allocator = FRAME_ALLOCATOR.lock();
                    (
                        frame_allocator.frame_total() * 4,
                        frame_allocator.frame_free() * 4,
                        frame_allocator.frame_free() * 4,
                    )
                };

                let meminfo = format!(
                    r"MemTotal:     {mem_total:>10} kB
MemFree:      {mem_free:>10} kB
MemAvailable: {mem_available:>10} kB
",
                    mem_total = mem_total,
                    mem_free = mem_free,
                    mem_available = mem_available
                );
                let mut buf = Vec::from(meminfo);
                Ok(buf)
            }
            ProcFsInodeInner::interrupts => {
                Ok(Vec::from(IRQTABLE.lock().tostring()))
            }
            _ => {
                // error!("[read_all] is a directory");
                Err(crate::utils::Errno::EISDIR)
            }
        }
    }
    fn look_up(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let pattern = AbsPath::new(String::from(path)).get_filename();
        match self.inner {
            ProcFsInodeInner::root => {
                if pattern == "self" {
                    Some(Arc::new(ProcFsInode::new(path, ProcFsInodeInner::_self)))
                } else if pattern == "meminfo" {
                    Some(Arc::new(ProcFsInode::new(path, ProcFsInodeInner::meminfo)))
                } else if pattern == "mounts" {
                    Some(Arc::new(ProcFsInode::new(path, ProcFsInodeInner::mounts)))
                } else if pattern == "interrupts" {
                    Some(Arc::new(ProcFsInode::new(path, ProcFsInodeInner::interrupts)))
                } else {
                    None
                }
            }
            ProcFsInodeInner::_self => {
                if pattern == "exe" {
                    Some(Arc::new(ProcFsInode::new(path, ProcFsInodeInner::exe)))
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
                if let Ok(FileClass::File(exe)) = open("/bin/sh".into(), OpenFlags::O_RDONLY) {
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
            ProcFsInodeInner::interrupts => {
                res.st_mode = InodeType::File as u32;
                res.st_nlink = 1;
                res.st_size = IRQTABLE.lock().tostring().len() as i64;
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
    // fn rename(&self, old_path: &String, new_path: &String) {
    //     // 不可以改名字
    // }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        match self.inner {
            ProcFsInodeInner::root => {
                // (path, ino, d_type)
                let mut entries: Vec<(&str, u64, u8)> = alloc::vec![
                    (".", 1, 4),
                    ("..", 0, 4),
                    ("self", 2, 4),
                    ("meminfo", 3, 8),
                    ("mounts", 4, 8),
                    ("interrupts", 5, 8),
                ];

                Some(build_dirents(entries))
            }
            ProcFsInodeInner::_self => {
                let mut entries = alloc::vec![(".", 2, 4), ("..", 1, 4), ("exe", 4, 8),];
                Some(build_dirents(entries))
            }
            _ => None,
        }
    }
}
