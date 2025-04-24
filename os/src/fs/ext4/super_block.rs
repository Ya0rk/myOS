#![allow(non_snake_case)]
use log::info;
use lwext4_rust::{Ext4BlockWrapper, InodeTypes};

use crate::{
    drivers::{BlockDriver, Disk},
    fs::{page_cache::PageCache, InodeTrait, Kstat, SuperBlockTrait},
};

use alloc::sync::Arc;

use super::Ext4Inode;

unsafe impl<T: BlockDriver> Send for Ext4SuperBlock<T> {}
unsafe impl<T: BlockDriver> Sync for Ext4SuperBlock<T> {}

pub struct Ext4SuperBlock<T: BlockDriver> {
    inner: Ext4BlockWrapper<Disk<T>>,
    root: Arc<Ext4Inode>,
}

impl<T: BlockDriver> Ext4SuperBlock<T> {
    pub fn new(disk: Disk<T>) -> Self {
        println!("init ext4 device superblock");
        let inner =
            Ext4BlockWrapper::<Disk<T>>::new(disk).expect("failed to initialize EXT4 filesystem");
        let page_cache = Some(PageCache::new_bare());
        let root = Ext4Inode::new("/", InodeTypes::EXT4_DE_DIR, page_cache.clone());
        Self { inner, root }
    }
}

impl<T: BlockDriver> SuperBlockTrait for Ext4SuperBlock<T> {
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

