#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{chdir, execve, fork, getpid, wait, yield_};


/// 传入str引用转换为C风格字符串，使其可以被用作系统调用
pub fn conert_str2byte(input: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = input.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn run_cmd(cmd: &str) {
    println!("task run cmd: {}", cmd);
    let cd = "/musl/";
    chdir(&conert_str2byte(cd));
    if fork() == 0 {
        println!("task run cmd child: {}, pid: {}", cmd, getpid());
        execve(
            "/musl/busybox\0",
            &[
                "/musl/busybox\0",
                "sh\0",
                "-c\0",
                cmd,
            ],
            &[
                "PATH=/bin:/\0",
                "HOME=/\0",
                "LD_LIBRARY_PATH=/\0",
                "TERM=screen\0",
            ],
        );
    } else {
        println!("task run cmd parent: {}", cmd);
        let mut exit_code: i32 = 0;
        let tid = wait(&mut exit_code);
        println!("return tid:{}", tid);
        if tid == -1 {
            yield_();
        }
    }
}

#[no_mangle]
fn main() -> i32 {
    // run_cmd("busybox touch sort.src");
    // run_cmd("busybox ln -s /lib/dlopen_dso.so dlopen_dso.so");
    // run_cmd(
    //     "busybox ln -s /lib/glibc/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64d.so.1 ",
    // );
    run_cmd("/musl/busybox --install /bin\0");
    if fork() == 0 {
        println!("main run sh");
        run_cmd("/bin/sh\0");
    } else {
        println!("main parent");
        loop {
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid < 0 {
                break;
            }
        }
    }
    0
}