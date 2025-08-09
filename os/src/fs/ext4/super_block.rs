#![allow(non_snake_case)]
use log::info;
use lwext4_rust::{Ext4BlockWrapper, Ext4InodeType};

use crate::{
    drivers::Disk,
    fs::{page_cache::PageCache, Ext4Inode, Kstat, SuperBlockTrait},
    syscall::StatFs,
};

use alloc::sync::Arc;

use super::InodeTrait;

unsafe impl Send for Ext4SuperBlock {}
unsafe impl Sync for Ext4SuperBlock {}

pub struct Ext4SuperBlock {
    inner: Ext4BlockWrapper<Disk>,
    root: Arc<dyn InodeTrait>,
}

impl Ext4SuperBlock {
    pub fn new(disk: Disk) -> Self {
        println!("init ext4 device superblock");
        let inner =
            Ext4BlockWrapper::<Disk>::new(disk).expect("failed to initialize EXT4 filesystem");
        let page_cache = Some(PageCache::new_bare());
        let root = Ext4Inode::new("/", Ext4InodeType::EXT4_DE_DIR, page_cache.clone());
        Self { inner, root }
    }
}

impl SuperBlockTrait for Ext4SuperBlock {
    fn root_inode(&self) -> Arc<dyn InodeTrait> {
        self.root.clone()
    }
    fn fs_stat(&self) -> StatFs {
        StatFs::new()
    }
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
