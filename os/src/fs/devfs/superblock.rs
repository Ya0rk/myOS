use alloc::sync::Arc;
use log::info;
use crate::fs::{devfs::root::DevFsRootInode, InodeTrait, SuperBlockTrait};


lazy_static! {
    /// devfs的超级块
    pub static ref DEVFS_SUPER_BLOCK: Arc<DevFsSuperBlock> = Arc::new(DevFsSuperBlock::new());
}

pub struct DevFsSuperBlock {
    root: Arc<DevFsRootInode>,
}

impl DevFsSuperBlock {
    pub fn new() -> Self {
        info!("init devfs superblock");
        let root = Arc::new(DevFsRootInode::new());
        Self { root }
    }
}

impl SuperBlockTrait for DevFsSuperBlock {
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
        info!("devfs does not need to sync");
    }
}