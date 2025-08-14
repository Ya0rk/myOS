use async_trait::async_trait;
use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, sync::Arc, vec::Vec};
#[cfg(any(feature = "board_qemu", feature = "2k1000la"))]
use crate::fs::devfs::char::CharDevInode;
#[cfg(feature = "vf2")]
use crate::fs::devfs::char::CharDevInode;
use crate::{
    fs::{devfs::{dev_loop::DevLoopInode, DevNullInode, DevRandomInode, DevRtcInode, DevTtyInode, DevZeroInode}, dirent::build_dirents, AbsPath, Dirent, InodeMeta, InodeTrait, InodeType, Kstat},
    sync::{Shared, SpinNoIrqLock, TimeStamp},
    utils::{Errno, SysResult},
};

use super::urandom;

pub struct DevFsRootInode {
    metadata: InodeMeta,
    children: BTreeMap<String, Arc<dyn InodeTrait>>,
}

impl DevFsRootInode {
    pub fn new() -> Self {
        let mut children = BTreeMap::new();
        children.insert("null".into(), DevNullInode::new());
        children.insert("rtc".into(), DevRtcInode::new());
        #[cfg(any(feature = "board_qemu", feature = "2k1000la"))]
        children.insert("tty".into(), DevTtyInode::new());
        #[cfg(feature = "vf2")]
        children.insert("tty".into(), CharDevInode::new());
        children.insert("urandom".into(), DevRandomInode::new());
        children.insert("zero".into(), DevZeroInode::new());
        children.insert("loop0".into(), DevLoopInode::new());
        Self {
            metadata: InodeMeta::new(
                InodeType::Dir, 
                0,
                "/dev"
            ),
            children,
        }
    }
}

#[async_trait]
impl InodeTrait for DevFsRootInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    fn get_page_cache(&self) -> Option<alloc::sync::Arc<crate::fs::page_cache::PageCache>> {
        None
    }
    fn get_size(&self) -> usize {
        4096
    }
    fn set_size(&self, new_size: usize) -> crate::utils::SysResult {
        Ok(())
    }
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        0
    }

    async fn read_dirctly(&self, offset: usize, buf: &mut [u8]) -> usize {
        0
    }
    // 疑似被弃用
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
        Err(Errno::EISDIR)
    }

    fn look_up(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        // 暂时不实现
        let binding = AbsPath::new(String::from(path)).get_filename();
        let pattern = binding.as_str();
        match pattern {
            "rtc" | "rtc0" => self.children.get("rtc").cloned(),
            _ => self.children.get(pattern).cloned(),
        }
    }
    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_mode = 16877;
        res.st_nlink = 1;
        res
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        // (path, ino, d_type)
        let mut entries: Vec<(&str, u64, u8)> = alloc::vec![
            (".", 1, 4),
            ("..", 0, 4),
            ("null", 2, 6),
            ("rtc", 3, 0),
            ("tty", 4, 2),
            ("urandom", 5, 8),
            ("zero", 6, 8),
            ("loop0", 7, 8)
        ];
        Some(build_dirents(entries))
    }
}
