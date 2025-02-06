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


use core::{arch::global_asm, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};
use mm::KERNEL_SPACE;
use task::get_current_hart_id;
use utils::boot;

global_asm!(include_str!("entry.asm"));

static FIRST_HART: AtomicBool = AtomicBool::new(true);
static INIT_FINISHED: AtomicBool = AtomicBool::new(false);
static START_HART_ID: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main(hart_id: usize) -> ! {
    if FIRST_HART
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok() 
    {
        boot::clear_bss();
        boot::logo();

        mm::init();
        mm::remap_test();
        logger::init();
        println!(
            "[kernel] ---------- hart {} is starting... ----------",
            hart_id
        );
        trap::init();
        task::init_processors();
        task::add_initproc();
        INIT_FINISHED.store(true, Ordering::SeqCst);
        START_HART_ID.store(hart_id, Ordering::SeqCst);
        boot::boot_all_harts(hart_id);
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
    } else {
        // barrier
        while !INIT_FINISHED.load(Ordering::SeqCst) {}

        
        trap::init();
        KERNEL_SPACE.lock().activate();
        trap::enable_timer_interrupt();
        timer::set_next_trigger();

        // loop {}
    }
    if get_current_hart_id() == START_HART_ID.load(Ordering::SeqCst) {
        fs::list_apps();
    }
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
