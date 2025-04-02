#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)] // for mod up's SyncUnsafeCell
// #![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![allow(unused)]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
pub mod mm;
pub mod fs;
pub mod task;
pub mod trap;
pub mod sync;
pub mod utils;
pub mod syscall;
pub mod drivers;
pub mod arch;
pub mod signal;


use core::{arch::global_asm, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};
use sync::{block_on, timer};
use task::{executor, get_current_hart_id, spawn_kernel_task};

#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("entry.asm"));

#[cfg(target_arch = "loongarch64")]
global_asm!(include_str!("entry_la.asm"));

#[macro_use]
extern crate lazy_static;

static FIRST_HART: AtomicBool = AtomicBool::new(true);
static INIT_FINISHED: AtomicBool = AtomicBool::new(false);
static START_HART_ID: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub fn rust_main(hart_id: usize) -> ! {

    println!("hello world!");

    if FIRST_HART
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok() 
    {
        utils::clear_bss();
        utils::logo();

        mm::init(true);
        
        #[cfg(feature = "test")]
        {
            mm::remap_test();
            fs::path_test();
        }

        utils::logger_init();

        // TODO:后期可以丰富打印的初始化信息
        println!(
            "[kernel] ---------- hart {} is starting... ----------",
            hart_id
        );
        trap::init();
        task::init_processors();
        println!("a");
        block_on(fs::init());
        println!("b");
        spawn_kernel_task(async move {
            task::add_initproc()
        });
        INIT_FINISHED.store(true, Ordering::SeqCst);
        START_HART_ID.store(hart_id, Ordering::SeqCst);
        #[cfg(feature = "mul_hart")]
        utils::boot_all_harts(hart_id);
    } else {

        trap::init();
        mm::init(false);        
    }
    
    unsafe { sync::enable_timer_interrupt() };
    timer::set_next_trigger();

    // 列出目前的应用
    let finish = AtomicBool::new(false);
    if get_current_hart_id() == START_HART_ID.load(Ordering::SeqCst) {
        finish.store(fs::list_apps(), Ordering::SeqCst);
    }
    while !finish.load(Ordering::SeqCst) {}
    executor::run();
    panic!("Unreachable in rust_main!");
}
