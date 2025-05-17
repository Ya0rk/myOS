mod inode;
pub mod file;
pub mod super_block;

pub use file::*;
pub use inode::*;
pub use super_block::*;

use alloc::sync::Arc;
use lazy_static::*;
use crate::drivers::{Disk};
use super::{InodeTrait, Kstat, SuperBlockTrait};

lazy_static! {
    pub static ref SUPER_BLOCK: Arc<dyn SuperBlockTrait> = {
        let block_device = crate::drivers::get_block_device().unwrap(); 
        Arc::new(Ext4SuperBlock::new(
            Disk::new(block_device),
        ))
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

pub fn fs_stat() -> Kstat {
    SUPER_BLOCK.fs_stat()
}

pub fn ls() {
    SUPER_BLOCK.ls()
}