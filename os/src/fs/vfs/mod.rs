mod dentry;
mod file;
mod inode;
mod super_block;

pub use dentry::*;
pub use file::*;
pub use inode::*;
pub use super_block::*;

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

lazy_static! {
    static ref INO_NUMBER: AtomicUsize = AtomicUsize::new(0);
}

fn alloc_ino() -> usize {
    INO_NUMBER.fetch_add(1, Relaxed)
}
