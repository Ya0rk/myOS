pub mod file;
mod inode;
pub mod super_block;

pub use file::*;
pub use inode::*;
pub use super_block::*;

use super::{InodeTrait, Kstat, SuperBlockTrait};
use crate::{drivers::Disk, syscall::StatFs};
use alloc::sync::Arc;
use lazy_static::*;

lazy_static! {
    pub static ref SUPER_BLOCK: Arc<dyn SuperBlockTrait> = {
        let block_device = crate::drivers::get_block_device().unwrap();
        Arc::new(Ext4SuperBlock::new(Disk::new(block_device)))
    };
}

pub fn root_inode() -> Arc<dyn InodeTrait> {
    log::info!("want to get root_inode");
    SUPER_BLOCK.root_inode()
}

#[allow(unused)]
pub fn sync() {
    SUPER_BLOCK.sync()
}

pub fn fs_stat() -> StatFs {
    SUPER_BLOCK.fs_stat()
}

pub fn ls() {
    SUPER_BLOCK.ls()
}
