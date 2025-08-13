use alloc::sync::Arc;
use log::info;

use crate::fs::{
    procfs::root::ProcFsRootInode,
    InodeTrait, SuperBlockTrait,
};

lazy_static! {
    /// procfs的超级块
    pub static ref PROCFS_SUPER_BLOCK: Arc<ProcFsSuperBlock> = Arc::new(ProcFsSuperBlock::new("/proc"));
}

pub struct ProcFsSuperBlock {
    root: Arc<ProcFsRootInode>,
}

impl ProcFsSuperBlock {
    pub fn new(path: &str) -> Self {
        info!("init procfs superblock");
        let root = Arc::new(ProcFsRootInode::new(path));
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
