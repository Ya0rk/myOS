#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{chdir, execve, fork, getpid, mkdir, wait, yield_};

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
        execve("/musl/busybox\0", &["/busybox\0", "sh\0", "-c\0", cmd], ENV);
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
    run_cmd("/musl/busybox --install /bin\0");
    println!("finish install");
    // // 拷贝动态库到指定位置,这里是musl的动态库
    // #[cfg(target_arch = "loongarch64")]
    // {
    //     run_cmd("/musl/busybox cp /musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1\0");
    // }
    // #[cfg(target_arch = "riscv64")]
    // {
    //     run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1\0");
    //     run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64.so.1\0");
    //     run_cmd("/musl/busybox cp /musl/lib/dlopen_dso.so /lib/dlopen_dso.so\0");
    //     run_cmd("/musl/busybox cp /musl/lib/tls_get_new-dtv_dso.so /lib/tls_get_new-dtv_dso.so\0");
    // }
    // #[cfg(target_arch = "loongarch64")]
    // {
    //     run_cmd("/musl/busybox cp /musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1\0");
    // }
    // #[cfg(target_arch = "riscv64")]
    // {
    //     run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1\0");
    //     run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64.so.1\0");
    //     run_cmd("/musl/busybox cp /musl/lib/dlopen_dso.so /lib/dlopen_dso.so\0");
    //     run_cmd("/musl/busybox cp /musl/lib/tls_get_new-dtv_dso.so /lib/tls_get_new-dtv_dso.so\0");
    // }
    if fork() == 0 {
        println!("main run sh");
        run_cmd("/bin/sh\0");
    } else {
        println!("main parent");
        loop {
            println!("back is in front");
            let mut exit_code: i32 = 0;
            let pid = wait(&mut exit_code);
            if pid < 0 {
                break;
            }
        }
    }
    0
}

const ENV: &[&str] = &[
    "PATH=/bin:/musl/basic:/musl/ltp:/musl:/\0",
    "HOME=/\0",
    "LD_LIBRARY_PATH=/lib\0",
    "TERM=xterm\0",
    "LTPROOT=Ya0rk\0",
    "GIT_PAGER=cat",
];
