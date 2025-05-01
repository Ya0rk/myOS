#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{chdir, execve, fork, wait, yield_};

const TESTCASES: &[&str] = &[
    // "time-test",
    // "test-splice.sh",
    "busybox_testcode.sh",
    // "lua_testcode.sh",
    // "netperf_testcode.sh",
    // "libc-bench",
    // "libctest_testcode.sh",
    // "iozone_testcode.sh",
    // "unixbench_testcode.sh",
    // "cyclictest_testcode.sh",
    // "iperf_testcode.sh",
    // "lmbench_testcode.sh",
];

/// 传入str引用转换为C风格字符串，使其可以被用作系统调用
pub fn conert_str2byte(input: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = input.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn run_cmd(cmd: &str) {
    chdir(&conert_str2byte("musl"));
    if fork() == 0 {
        println!("aaaaaaaaaaaa");
        execve(
            "/musl/busybox\0",
            &[
                "/musl/busybox\0",
                "sh\0",
                "/musl/busybox_testcode.sh\0",
            ],
            &[
                "PATH=/\0",
                "LD_LIBRARY_PATH=/\0",
                "TERM=screen\0",
            ],
        );
    } else {
        println!("hhhhhhhahahah");
        let mut exit_code: i32 = 0;
        let tid = wait(&mut exit_code);
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

    if fork() == 0 {
        for test in TESTCASES {
            run_cmd(test);
        }
    } else {
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