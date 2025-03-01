mod inode;
mod stdio;
mod dirent;
mod mount;
mod stat;
mod pipe;
mod ffi;

/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
    
    fn get_fstat(&self, kstat: &mut Kstat);

    fn get_dirent(&self, dirent: &mut Dirent) -> isize;

    fn get_name(&self) -> String;

    fn set_offset(&self, offset: usize);
}

use crate::mm::UserBuffer;
use alloc::string::String;
pub use inode::{list_apps, open, OSInode, chdir, open_file};
pub use dirent::Dirent;
pub use mount::MNT_TABLE;
pub use pipe::Pipe;
pub use stat::Kstat;
pub use stdio::{Stdin, Stdout};
pub use ffi::*;
