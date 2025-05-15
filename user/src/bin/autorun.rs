#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{chdir, execve, fork, wait, yield_};

const TESTCASES: &[&str] = &[
    // "./time-test\0",
    // "./test-splice.sh\0",
    // "./busybox_testcode.sh\0",
    // "./lua_testcode.sh\0",
    // "./netperf_testcode.sh\0",
    // "./libcbench_testcode.sh\0",
    // "./libctest_testcode.sh\0",
    // "./iozone_testcode.sh\0",
    "./unixbench_testcode.sh\0",
    // "./cyclictest_testcode.sh\0",
    // "./iperf_testcode.sh\0",
    // "./lmbench_testcode.sh\0",
];

/// 传入str引用转换为C风格字符串，使其可以被用作系统调用
pub fn conert_str2byte(input: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = input.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn run_cmd(cmd: &str, pwd: &str) {
    if fork() == 0 {
        let path = [pwd, "busybox\0"].concat();
        execve(
            &path,
            &[ &path, "sh\0", "-c\0", cmd],
            &[
                "PATH=/:/bin\0",
                "HOME=/\0",
                "LD_LIBRARY_PATH=/\0",
                "TERM=screen\0",
            ],
        );
    } else {
        let mut exit_code: i32 = 0;
        let tid = wait(&mut exit_code);
        if tid == -1 {
            yield_();
        }
    }
}

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        let cd = "/musl/";
        chdir(&conert_str2byte(cd));
        for test in TESTCASES {
            run_cmd(test, cd);
        }
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