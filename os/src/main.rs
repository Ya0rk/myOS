#![allow(warnings)]
// #![deny(warnings)]
#![no_std]
#![no_main]
#![feature(cfg_match)]
#![feature(stmt_expr_attributes)]
#![feature(sync_unsafe_cell)] // for mod up's SyncUnsafeCell
// #![feature(panic_info_message)]
// #![feature(riscv_ext_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![feature(step_trait)]
#![feature(const_ops)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(map_try_insert)]
#![feature(naked_functions)]
#![feature(let_chains)]

#![allow(unused)]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
// TODO: 实际上src/config不能直接遗弃
// mod config;
mod lang_items;
pub mod mm;
pub mod fs;
pub mod task;
pub mod sync;
pub mod utils;
pub mod syscall;
pub mod drivers;
pub mod net;
// pub mod arch;
pub mod signal;
pub mod hal;


use core::{arch::global_asm, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};
use alloc::vec::{self, Vec};
#[cfg(target_arch = "loongarch64")]
use hal::mem::{mmu_init, tlb::{self, tlb_fill}, tlb_init};
#[cfg(target_arch = "riscv64")]
use hal::mem::{mmu_init, tlb_init};
use log::{error, info};
use mm::memory_space::test_la_memory_space;
use sync::{block_on, time_init, timer};
use task::{executor, get_current_hart_id, spawn_kernel_task};


#[macro_use]
extern crate lazy_static;

static FIRST_HART: AtomicBool = AtomicBool::new(true);
static INIT_FINISHED: AtomicBool = AtomicBool::new(false);
static START_HART_ID: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub fn rust_main(hart_id: usize, dt_root: usize) -> ! {
    // 启动顺序：
    // clear_bss 
    // logo 
    // logger_init
    // mm::init
    // trap_init 
    // init_processors
    // probe
    // fs::init
    // 进行测试
    // 载入用户进程
    // 设置时钟中断
    // 开始调度执行
    #[cfg(target_arch = "loongarch64")]
    {
        mmu_init();
        tlb_init(tlb_fill as usize);
    }
    println!("hello world!");
    println!("hart id is {:#X}, dt_root is {:#x}", hart_id, dt_root);

    if FIRST_HART
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok() 
    {
        hal::entry::boot::clear_bss();
        hal::entry::boot::logo();
        utils::logger_init();
        println!("start init mm");
        mm::init(true);
        println!("finished mm::init");
        // utils::logger_init();
        sync::time_init();

        // info!("[rust_main] alloc a vec size is 1387650");
        // let mut buffer: Vec<u8> = alloc::vec![1; 1387650];
        // println!("{:?}", buffer[256]);
        // info!("[rust_main] alloc mem is enough");

        // TODO:后期可以丰富打印的初始化信息
        println!(
            "[kernel] ---------- hart {} is starting... ----------",
            hart_id
        );
        START_HART_ID.store(hart_id, Ordering::SeqCst);
        hal::trap::init();


        
        crate::drivers::init();

        fs::init();
        // 此时完成初始化工作，准备载入进程开始执行

        

        // 测试代码应当放在这里
        #[cfg(feature = "test")]
        {
            // mm::remap_test();
            // info!("start path test");
            // // fs::path_test();
            // info!(" start dentry test");
            // fs::vfs::dentry_test();
            // test_la_memory_space();

            // unsafe  {
            //     let p = 0x9000_0000_0020_1000 as (*const usize);
            //     let ins = *p;
            //     error!("[TEST_ADDR] {:#x}", ins);
            // }
        }


        task::init_processors();
        spawn_kernel_task(async move {
            task::add_initproc().await
        });
        
        INIT_FINISHED.store(true, Ordering::SeqCst);
        #[cfg(feature = "mul_hart")]
        hal::entry::boot::boot_all_harts(hart_id);
    } else {

        hal::trap::init();
        mm::init(false);        
    }
    
    unsafe { sync::enable_timer_interrupt() };
    timer::set_next_trigger();
    info!("[a] src/main.rs:145");
    // 列出目前的应用
    // let finish = AtomicBool::new(false);
    // if get_current_hart_id() == START_HART_ID.load(Ordering::SeqCst) {
    //     finish.store(fs::list_apps(), Ordering::SeqCst);
    // }
    // while !finish.load(Ordering::SeqCst) {}
    // let finish = AtomicBool::new(false);
    // if get_current_hart_id() == START_HART_ID.load(Ordering::SeqCst) {
    //     finish.store(fs::list_apps(), Ordering::SeqCst);
    // }
    // while !finish.load(Ordering::SeqCst) {}
    executor::run();
    panic!("Unreachable in rust_main!");
}
