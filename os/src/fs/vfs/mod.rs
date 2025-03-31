mod inode;
mod super_block;
mod file;

pub use inode::*;
pub use super_block::*;
pub use file::*;

use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

lazy_static! {
    static ref INO_NUMBER: AtomicUsize = AtomicUsize::new(0);
}

fn alloc_ino() -> usize {
    INO_NUMBER.fetch_add(1, Relaxed)
}