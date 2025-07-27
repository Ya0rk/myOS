///! This module contains utility functions that are used throughout the OS.
mod random;
// pub mod boot;
pub mod errtype;
pub mod logger;
pub mod elf;
// pub mod flags;
pub mod cache;
pub mod container;
pub mod downcast;
pub mod raw_ptr;
pub mod user_backtrace;

use crate::{
    hal::arch::{fp_read, ra_read},
    mm::VirtAddr,
};
use core::arch::asm;
use log::warn;

pub use errtype::{Errno, SysResult};
pub use logger::logger_init;
pub use random::{LcgRng, RNG};
// pub use boot::{boot_all_harts, jump_helper, clear_bss, logo};

pub fn backtrace() {
    extern "C" {
        fn stext();
        fn etext();
    }
    unsafe {
        let mut current_pc: usize = ra_read();
        let mut current_fp: usize = fp_read();
        // asm!("mv {}, s0", out(reg) current_fp);
        // asm!("mv {}, ra", out(reg) current_pc);

        while current_pc >= stext as usize && current_pc <= etext as usize && current_fp != 0 {
            warn!("[stack_backtrace] {:#x}", current_pc);
            current_fp = *(current_fp as *const usize).offset(-2);
            current_pc = *(current_fp as *const usize).offset(-1);
        }
    }
}
