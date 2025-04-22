#![allow(non_snake_case)]
use log::info;
use lwext4_rust::{Ext4BlockWrapper, InodeTypes};

use crate::{
    drivers::Disk,
    fs::{page_cache::PageCache, InodeTrait, Kstat, SuperBlockTrait},
};

use alloc::sync::Arc;

use super::Ext4Inode;

unsafe impl Send for Ext4SuperBlock {}
unsafe impl Sync for Ext4SuperBlock {}

pub struct Ext4SuperBlock {
    inner: Ext4BlockWrapper<Disk>,
    root: Arc<Ext4Inode>,
}

impl Ext4SuperBlock {
    pub fn new(disk: Disk) -> Self {
        println!("init ext4 device superblock");
        let inner =
            Ext4BlockWrapper::<Disk>::new(disk).expect("failed to initialize EXT4 filesystem");
        let page_cache = Some(PageCache::new_bare());
        let root = Ext4Inode::new("/", InodeTypes::EXT4_DE_DIR, page_cache.clone());
        Self { inner, root }
    }
}

impl SuperBlockTrait for Ext4SuperBlock {
    fn root_inode(&self) -> Arc<dyn InodeTrait> {
        self.root.clone()
    }
    fn fs_stat(&self) -> Kstat {
        Kstat::new()
        // let mut file = self.root.file.lock();
        // match file.fstat() {
        //     Ok(stat) => {
        //         let (atime, mtime, ctime) = self.root.metadata.timestamp.lock().get();
        //         as_inode_stat(stat, atime, mtime, ctime)
        //     }
        //     Err(_) => Kstat::new()
        // }
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

