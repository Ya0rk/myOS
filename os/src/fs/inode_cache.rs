//! File System Index Management
//! 
//! This module provides functionality for managing the inodes in the file system
//! index. It uses a hash map to store the mapping between file paths and their
//! corresponding inode objects. The index is thread-safe and allows for concurrent
//! read and write operations.

use core::ops::{Deref, DerefMut};
use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use hashbrown::HashMap;
use lazy_static::*;
use spin::RwLock;

use super::InodeTrait;

// A global, thread-safe file system index using a read-write lock.
lazy_static! {
    /// 这个是inode cache，用hashmap结构实现，key是path，value是inode
    pub static ref INODE_CACHE: InodeCache = InodeCache::new();
}

pub struct InodeCache(pub RwLock<HashMap<String, Arc<dyn InodeTrait>>>);

impl InodeCache {
    pub fn new() -> Self {
        Self(
            RwLock::new(HashMap::new())
        )
    }

    pub fn has_inode(&self, path: &str) -> bool {
        self.read().contains_key(path)
    }

    pub fn get(&self, path: &str) -> Option<Arc<dyn InodeTrait>> {
        self.read().get(path).map(|inode| inode.clone())
    }

    pub fn insert(&self, path: &str, inode: Arc<dyn InodeTrait>) {
        self.write().insert(path.to_string(), inode);
    }

    pub fn remove(&self, path: &str) {
        self.write().remove(path);
    }
}

impl Deref for InodeCache {
    type Target = RwLock<HashMap<String, Arc<dyn InodeTrait>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InodeCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}