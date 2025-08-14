use crate::{
    fs::{
        dirent::build_dirents, ffi::MEMINFO, open, procfs::{_self::_SelfInode, interrupts::InterruptInode, irqtable::{SupervisorExternal, SupervisorTimer, IRQTABLE}, meminfo::MeminfoInode, mounts::MountsInode, sys::SysDirInode}, AbsPath, Dirent, FileClass, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags
    },
    mm::frame_allocator::{FrameAllocator, StackFrameAllocator, FRAME_ALLOCATOR},
    sync::{SpinNoIrqLock, TimeStamp},
    utils::SysResult,
};
use alloc::{boxed::Box, collections::btree_map::BTreeMap, format, string::ToString};
use alloc::{
    string::String,
    sync::Arc,
    vec::{self, Vec},
};
use async_trait::async_trait;
use log::error;
use lwext4_rust::bindings::O_RDONLY;

pub struct ProcFsRootInode {
    metadata: InodeMeta,
    children: BTreeMap<String, Arc<dyn InodeTrait>>,
}

impl ProcFsRootInode {
    /// path为绝对路径，inner为要创建的类型
    pub fn new(path: &str) -> Self {
        let mut children = BTreeMap::new();
        children.insert("meminfo".to_string(), MeminfoInode::new());
        children.insert("_self".to_string(), _SelfInode::new());
        children.insert("mounts".to_string(), MountsInode::new());
        children.insert("interrupts".into(), InterruptInode::new());
        children.insert("sys".into(), SysDirInode::new());
        Self {
            metadata: InodeMeta::new(
                InodeType::Dir,
                0,
                path.into(),
            ),
            children,
        }
    }
}

#[async_trait]
impl InodeTrait for ProcFsRootInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
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
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        // 非常重要
        panic!("ProcFsRootInode does not support read_at");
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
        Err(crate::utils::Errno::EISDIR)
    }
    fn look_up(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        let pattern = AbsPath::new(String::from(path)).get_filename();
        return self.children.get(&pattern).cloned();
    }
    fn fstat(&self) -> Kstat {
        // 也是不严谨实现
        let mut res = Kstat::new();
        res.st_mode = 16877;
        res.st_nlink = 1;
        res
    }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.metadata.timestamp
    }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
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
}
