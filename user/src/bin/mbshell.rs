#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{chdir, execve, exit, fork, getpid, mkdir, wait, waitpid, yield_};

/// 传入str引用转换为C风格字符串，使其可以被用作系统调用
pub fn conert_str2byte(input: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = input.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

// fn run_cmd(cmd: &str) {
//     // println!("task run cmd: {}", cmd);
//     let cd = "/musl/";
//     chdir(&conert_str2byte(cd));
//     if fork() == 0 {
//         // println!("task run cmd child: {}, pid: {}", cmd, getpid());
//         execve(
//             "/musl/busybox\0",
//             &["/musl/busybox\0", "sh\0", "-c\0", cmd],
//             ENV,
//         );
//         exit(0);
//     } else {
//         // println!("task run cmd parent: {}", cmd);
//         let mut exit_code: i32 = 0;
//         let tid = wait(&mut exit_code);
//         // println!("return tid:{}", tid);
//         if tid == -1 {
//             yield_();
//         }
//     }
// }

// #[no_mangle]
// fn main() -> i32 {
//     // println!("finish install");
//     // // 拷贝动态库到指定位置,这里是musl的动态库
//     #[cfg(target_arch = "loongarch64")]
//     {
//         run_cmd("/musl/busybox cp /musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1\0");
//     }
//     #[cfg(target_arch = "riscv64")]
//     {
//         run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1\0");
//         run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64.so.1\0");
//         run_cmd("/musl/busybox cp /musl/lib/dlopen_dso.so /lib/dlopen_dso.so\0");
//         run_cmd("/musl/busybox cp /musl/lib/tls_get_new-dtv_dso.so /lib/tls_get_new-dtv_dso.so\0");
//     }
//     let child_pid = fork();
//     if child_pid == 0 {
//         run_cmd("/musl/busybox --install /bin\0");
//         // run_cmd("/musl/busybox cp /musl/cc1 /bin/cc1\0");
//         // run_cmd("/musl/busybox cp /musl/as /bin/as\0");
//         // run_cmd("/musl/busybox cp /musl/ld /bin/ld\0");

//         run_cmd("/bin/sh\0");
//         // execve("/musl/busybox\0", &["/musl/busybox\0", "sh\0"], ENV);
//         exit(0);
//     } else {
//         // println!("main parent");
//         loop {
//             // println!("back is in front");
//             let mut exit_code: i32 = 0;
//             let pid = waitpid(child_pid as usize, &mut exit_code, 0);
//             if pid == child_pid {
//                 println!("main find child");
//                 break;
//             }
//         }
//     }
//     println!("main exit");
//     0
// }

// const ENV: &[&str] = &[
//     "PATH=/bin:/musl/basic:/musl/ltp:/musl:/:/gcc/usr/bin\0",
//     "HOME=/\0",
//     // "CFLAGS=-I/gcc/usr/include -I/gcc/usr/include/riscv64-alpine-linux-musl\0",  // 新增头文件路径
//     // "LDFLAGS=-L/gcc/usr/lib -L/gcc/usr/lib/gcc/riscv64-alpine-linux-musl/14.2.0\0",       // 新增库路径
//     "LD_LIBRARY_PATH=/:/lib:/gcc/lib:/gcc/usr/lib:/gcc/usr/libexec/gcc/riscv64-alpine-linux-musl/14.2.0:/gcc/usr/lib\0",
//     "TERM=xterm\0",
//     "LTPROOT=Ya0rk\0",
//     "GIT_PAGER=cat",
// ];

fn run_cmd(cmd: &str) {
    // println!("task run cmd: {}", cmd);
    let cd = "/";
    // chdir(&conert_str2byte(cd));
    if fork() == 0 {
        // println!("task run cmd child: {}, pid: {}", cmd, getpid());
        execve(
            "/bin/busybox\0",
            // &["/bin/sh\0"],
            &["/bin/busybox\0", "sh\0"],
            ENV,
        );
        exit(0);
    } else {
        // println!("task run cmd parent: {}", cmd);
        let mut exit_code: i32 = 0;
        let tid = wait(&mut exit_code);
        // println!("return tid:{}", tid);
        if tid == -1 {
            yield_();
        }
    }
}
#[no_mangle]
fn main() -> i32 {
    println!("start main");
    let child_pid = fork();
    if child_pid == 0 {
        // run_cmd("/bin/busybox --install /bin\0");
        run_cmd("/bin/busybox cp /usr/libexec/gcc/riscv64-alpine-linux-musl/14.2.0/cc1 /bin/cc1\0");

        run_cmd("/bin/sh\0");
        // execve("/musl/busybox\0", &["/musl/busybox\0", "sh\0"], ENV);
        exit(0);
    } else {
        // println!("main parent");
        loop {
            // println!("back is in front");
            let mut exit_code: i32 = 0;
            let pid = waitpid(child_pid as usize, &mut exit_code, 0);
            if pid == child_pid {
                println!("main find child");
                break;
            }
        }
    }
    println!("main exit");
    0
}

const ENV: &[&str] = &[
    "PATH=/bin:/usr/bin:/usr/local/bin:\0",
    "HOME=/\0",
    "LD_LIBRARY_PATH=/usr/lib:/:/lib:/usr/bin/lib:\0",
    "TERM=xterm\0",
    "LTPROOT=Ya0rk\0",
    "GIT_PAGER=cat\0",
];