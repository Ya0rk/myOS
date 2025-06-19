#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::ptr::null;

use user_lib::{exec, execve, fork, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    println!("initproc running...");
    if fork() == 0 {
        exec("user_shell\0");
        // execve("musl/busybox\0", &["busybox\0", "sh\0"], &[]);
    } else {
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            }
            println!(
                "[initproc] Released a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }
    0
}
