// #![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod sbi;
mod timer;
pub mod mm;
pub mod fs;
pub mod task;
pub mod trap;
pub mod sync;
pub mod utils;
pub mod logger;
pub mod syscall;
pub mod drivers;


use core::arch::global_asm;
use utils::boot;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main() -> ! {
    boot::clear_bss();
    println!("[kernel] Hello, world!");
    logger::init();
    mm::init();
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    boot::logo();
    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
