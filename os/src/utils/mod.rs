///! This module contains utility functions that are used throughout the OS.
mod boot;
mod logger;
mod errtype;
mod random;

use core::arch::asm;
use log::warn;
use crate::mm::VirtAddr;

pub use errtype::{Errno, SysResult};
pub use logger::logger_init;
pub use random::{LcgRng, RNG};
pub use boot::{boot_all_harts, jump_helper, clear_bss, logo};

pub fn backtrace() {
    unsafe {
        let mut fp: usize;
        asm!("mv {}, fp", out(reg) fp);
        let mut start: VirtAddr = VirtAddr::from(fp).floor().into();
        let mut end: VirtAddr = VirtAddr::from(fp).ceil().into();
        let mut fp_addr = VirtAddr::from(fp);
        while start <= fp_addr && fp_addr < end {
            let ptr = fp as *const usize;
            warn!("[stack_backtrace] {:#x},", ptr.offset(-8).read());
            fp = ptr.offset(-16).read();
            start = VirtAddr::from(fp).floor().into();
            end = VirtAddr::from(fp).ceil().into();
            fp_addr = VirtAddr::from(fp);
        }
    }
}