#![no_std]
#![no_main]

use user_lib::brk;

#[macro_use]
extern crate user_lib;

// use core::ptr::null;

// use user_lib::{exec, execve, fork, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    println!("initproc running...");
    brk(0x0 as *const u8);
    brk(0x4010_0000 as *const u8);
    brk(0x1234_5678 as *const u8);
    loop {}
    0
}
