use alloc::sync::Arc;
use log::info;

use crate::fs::{
    procfs::inode::{ProcFsInode, ProcFsInodeInner},
    InodeTrait, SuperBlockTrait,
};

pub struct ProcFsSuperBlock {
    root: Arc<ProcFsInode>,
}

impl ProcFsSuperBlock {
    pub fn new(path: &str) -> Self {
        info!("init procfs superblock");
        let root = Arc::new(ProcFsInode::new(path, ProcFsInodeInner::root));
        Self { root }
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
