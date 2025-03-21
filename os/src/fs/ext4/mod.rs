mod inode;
pub mod super_block;

pub use inode::*;
pub use super_block::*;

use alloc::sync::Arc;
use lazy_static::*;

use crate::drivers::{BlockDeviceImpl, Disk};

use super::{InodeTrait, SuperBlockTrait};

lazy_static! {
    static ref SUPER_BLOCK: Arc<dyn SuperBlockTrait> = {
        Arc::new(Ext4SuperBlock::new(
            Disk::new(BlockDeviceImpl::new_device()),
        ))
    };
}

pub fn root_inode() -> Arc<dyn InodeTrait> {
    SUPER_BLOCK.root_inode()
}

#[allow(unused)]
pub fn sync() {
    SUPER_BLOCK.sync()
}

// pub fn fs_stat() -> Statfs {
//     SUPER_BLOCK.fs_stat()
// }

pub fn ls() {
    SUPER_BLOCK.ls()
}