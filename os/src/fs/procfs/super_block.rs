use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};

use alloc::sync::Arc;
use log::info;

use crate::{
    fs::{
        procfs::inode::{ProcFsInode, ProcFsInodeInner},
        InodeTrait, SuperBlockTrait,
    },
    sync::Shared,
};

pub struct ProcFsSuperBlock {
    root: Arc<ProcFsInode>,
    alloc: AtomicU64,
    change_pluge: AtomicBool,
}

impl ProcFsSuperBlock {
    pub fn new(path: &str) -> Self {
        info!("init procfs superblock");
        let root = Arc::new(ProcFsInode::new(path, ProcFsInodeInner::root));
        Self {
            root,
            alloc: AtomicU64::new(1),
            change_pluge: AtomicBool::new(false),
        }
    }
    // 就不做错误处理了
    pub fn alloc_ino(&self) -> u64 {
        let res = self.alloc.load(core::sync::atomic::Ordering::Relaxed);
        self.alloc
            .store(res + 1, core::sync::atomic::Ordering::Relaxed);
        res
    }
    pub fn plug_in(&self) {
        self.change_pluge
            .store(true, core::sync::atomic::Ordering::Relaxed);
    }
    pub fn plug_un(&self) {
        self.change_pluge
            .store(false, core::sync::atomic::Ordering::Relaxed);
    }
    pub fn changeable(&self) -> bool {
        self.change_pluge
            .load(core::sync::atomic::Ordering::Relaxed)
    }
}

impl SuperBlockTrait for ProcFsSuperBlock {
    fn root_inode(&self) -> Arc<dyn InodeTrait> {
        self.root.clone()
    }
    fn fs_stat(&self) -> crate::syscall::StatFs {
        crate::syscall::StatFs::new()
    }
    fn ls(&self) {
        self.root.read_dents().unwrap().iter().for_each(|x| {
            println!("{}", x);
        });
    }
    fn sync(&self) {
        // procfs does not need to sync
        info!("procfs does not need to sync");
    }
}
