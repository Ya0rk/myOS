//! File System Index Management
//! 
//! This module provides functionality for managing the inodes in the file system
//! index. It uses a hash map to store the mapping between file paths and their
//! corresponding inode objects. The index is thread-safe and allows for concurrent
//! read and write operations.

use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use hashbrown::HashMap;
use lazy_static::*;
use spin::RwLock;

use super::Inode;

// A global, thread-safe file system index using a read-write lock.
lazy_static! {
    pub static ref FSIDX: RwLock<HashMap<String, Arc<dyn Inode>>> = RwLock::new(HashMap::new());
}

/// Checks if an inode exists for the given path.
///
/// # Arguments
///
/// * `path` - The path to check for an existing inode.
///
/// # Returns
///
/// `true` if the inode exists, `false` otherwise.
pub fn has_inode(path: &str) -> bool {
    FSIDX.read().contains_key(path)
}

/// Finds the inode associated with the given path.
///
/// # Arguments
///
/// * `path` - The path to look up the inode.
///
/// # Returns
///
/// An `Option` containing the inode if it exists, or `None` if it does not.
pub fn find_inode_idx(path: &str) -> Option<Arc<dyn Inode>> {
    FSIDX.read().get(path).map(|vfile| Arc::clone(vfile))
}

/// Inserts a new inode into the file system index.
///
/// # Arguments
///
/// * `path` - The path associated with the inode.
/// * `vfile` - The inode to insert, wrapped in an `Arc`.
pub fn insert_inode_idx(path: &str, vfile: Arc<dyn Inode>) {
    FSIDX.write().insert(path.to_string(), vfile);
}

/// Removes an inode from the file system index.
///
/// # Arguments
///
/// * `path` - The path associated with the inode to remove.
pub fn remove_inode_idx(path: &str) {
    FSIDX.write().remove(path);
}

/// Prints the current keys in the file system index to the console.
pub fn print_inner() {
    println!("{:#?}", FSIDX.read().keys());
}
