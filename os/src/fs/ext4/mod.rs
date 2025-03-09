mod inode;
mod super_block;

pub use inode::*;
pub use super_block::*;

use alloc::sync::Arc;
use lazy_static::*;

use crate::{
    drivers::{BlockDeviceImpl, Disk},
    fs::SuperBlock,
};

use super::Inode;

lazy_static! {
    static ref SUPER_BLOCK: Arc<dyn SuperBlock> = {
        Arc::new(Ext4SuperBlock::new(
            Disk::new(BlockDeviceImpl::new_device()),
        ))
    };
}

pub fn root_inode() -> Arc<dyn Inode> {
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