#![allow(non_snake_case)]
use lwext4_rust::{Ext4BlockWrapper, InodeTypes};

use crate::{
    drivers::Disk,
    fs::{InodeTrait, SuperBlockTrait},
};

use alloc::sync::Arc;

use super::Ext4Inode;

pub struct Ext4SuperBlock {
    inner: Ext4BlockWrapper<Disk>,
    root: Arc<dyn InodeTrait>,
}

unsafe impl Send for Ext4SuperBlock {}
unsafe impl Sync for Ext4SuperBlock {}

impl SuperBlockTrait for Ext4SuperBlock {
    fn root_inode(&self) -> Arc<dyn InodeTrait> {
        self.root.clone()
    }
    // fn fs_stat(&self) -> crate::fs::Statfs {
    //     todo!()
    // }
    fn sync(&self) {
        todo!()
    }
    fn ls(&self) {
        self.inner
            .lwext4_dir_ls()
            .into_iter()
            .for_each(|s| println!("{}", s));
    }
}

impl Ext4SuperBlock {
    pub fn new(disk: Disk) -> Self {
        let inner =
            Ext4BlockWrapper::<Disk>::new(disk).expect("failed to initialize EXT4 filesystem");
        let root = Arc::new(Ext4Inode::new("/", InodeTypes::EXT4_DE_DIR));
        Self { inner, root }
    }
}