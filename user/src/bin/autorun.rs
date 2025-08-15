#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::{chdir, execve, exit, fork, wait, waitpid, yield_};

// 可能会造成动态链接库冲突的测试
const TEST: &[&str] = &[
    // "./basic_testcode.sh\0",
    // "./busybox_testcode.sh\0",
];

const FINAL_STAGE1: &[&str] = &[
    // "./copy-file-range_testcode.sh\0",
    // "./interrupts_testcode.sh\0",
    // "./splice_testcode.sh\0",
];

const MUSL_LTP: &[&str] = &[
    // "/ltp_testcode_musl.sh\0",
];

const GLIBC_LTP: &[&str] = &[
    // "/ltp_testcode_glibc.sh\0"
];

const TESTCASES: &[&str] = &[
    "./libctest_testcode.sh\0",
    // "./lua_testcode.sh\0",
    // "./libcbench_testcode.sh\0",
    "./iozone_testcode.sh\0",
    // "./lmbench_testcode.sh\0",
    // "./iperf_testcode.sh\0",
    // "./netperf_testcode.sh\0",
    // "./unixbench_testcode.sh\0",
    // "./cyclictest_testcode.sh\0",
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
            &[&path, "sh\0", "-c\0", cmd],
            &[
                ["PATH=", "/bin:", pwd.strip_suffix("/").unwrap()]
                    .concat()
                    .as_str(),
                "HOME=/\0",
                ["LD_LIBRARY_PATH=", pwd.strip_suffix("/").unwrap(), "/lib\0"]
                    .concat()
                    .as_str(),
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
    let child_pid = fork();
    if child_pid == 0 {
        let cd = "/glibc/";
        chdir(&conert_str2byte(cd));
        for test in TEST {
            run_cmd(test, cd);
        }
        for test in FINAL_STAGE1 {
            run_cmd(test, cd);
        }
        // 拷贝动态库到指定位置,这里是glibc的动态库
        #[cfg(target_arch = "loongarch64")]
        {
            run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-loongarch-lp64d.so.1 /lib64/ld-linux-loongarch-lp64d.so.1\0", "/glibc/");
            run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-loongarch-lp64d.so.1 /ld-linux-loongarch-lp64d.so.1\0", "/glibc/");
        }
        #[cfg(target_arch = "riscv64")]
        {
            run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64d.so.1\0", "/glibc/");
            run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-riscv64-lp64d.so.1 /ld-linux-riscv64-lp64d.so.1\0", "/glibc/");
            run_cmd("/glibc/busybox cp /glibc/lib/libc.so.6 /lib/libc.so.6\0", "/glibc/");
            
        }

        // 除去busybox和basic
        for test in TESTCASES {
            run_cmd(test, cd);
        }

        run_cmd("/glibc/busybox rm -rf /lib/*\0", "/glibc/"); // 删除glibc的动态库，避免影响musl的basic测试
        run_cmd("/glibc/busybox rm -rf /lib64/*\0", "/glibc/");
        exit(0); 
    } else {
        println!("main parent");
        loop {
            let mut exit_code: i32 = 0;
            let pid = waitpid(child_pid as usize, &mut exit_code, 0);
            if pid == child_pid {
                break;
            }
        }
    }

    let child_pid = fork();
    if child_pid == 0 {
        let cd = "/musl/";
        chdir(&conert_str2byte(cd));
        for test in TEST {
            run_cmd(test, cd);
        }
        for test in FINAL_STAGE1 {
            run_cmd(test, cd);
        }

        // 拷贝动态库到指定位置,这里是musl的动态库
        #[cfg(target_arch = "loongarch64")]
        {
            run_cmd(
                "/musl/busybox cp /musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1\0",
                "/musl/",
            );
        }
        #[cfg(target_arch = "riscv64")]
        {
            run_cmd(
                "/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1\0",
                "/musl/",
            );
            run_cmd(
                "/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64.so.1\0",
                "/musl/",
            );
            run_cmd(
                "/musl/busybox cp /musl/lib/dlopen_dso.so /lib/dlopen_dso.so\0",
                "/musl/",
            );
            run_cmd(
                "/musl/busybox cp /musl/lib/tls_get_new-dtv_dso.so /lib/tls_get_new-dtv_dso.so\0",
                "/musl/",
            );
        }

        run_cmd("/musl/busybox cat /proc/meminfo\0", "/musl/");
        for test in TESTCASES {
            run_cmd(test, cd);
        }
        run_cmd("/musl/busybox cat /proc/meminfo\0", "/musl/");

        run_cmd("/musl/busybox rm -rf /lib/*\0", "/musl/");
        run_cmd("/musl/busybox rm -rf /lib64/*\0", "/musl/");
        
        exit(0);
    } else {
        println!("main parent");
        loop {
            let mut exit_code: i32 = 0;
            let pid = waitpid(child_pid as usize, &mut exit_code, 0);
            if pid == child_pid {
                break;
            }
        }
    }

    // // musl ltp 测例
    // let child_pid = fork();
    // if child_pid == 0 {
    //     musl_ltp();
    //     exit(0);
    // } else {
    //     loop {
    //         let mut exit_code: i32 = 0;
    //         let pid = waitpid(child_pid as usize, &mut exit_code, 0);
    //         if pid == child_pid {
    //             break;
    //         }
    //     }
    // }

    // // glibc ltp 测试
    // let child_pid = fork();
    // if child_pid == 0 {
    //     glibc_ltp();
    //     exit(0);
    // } else {
    //     loop {
    //         let mut exit_code: i32 = 0;
    //         let pid = waitpid(child_pid as usize, &mut exit_code, 0);
    //         if pid == child_pid {
    //             break;
    //         }
    //     }
    // }

    0
}


fn glibc_ltp() {
    let cd = "/glibc/";
    chdir(&conert_str2byte(cd));
    #[cfg(target_arch = "loongarch64")]
    {
        run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-loongarch-lp64d.so.1 /lib64/ld-linux-loongarch-lp64d.so.1\0", "/glibc/");
        run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-loongarch-lp64d.so.1 /ld-linux-loongarch-lp64d.so.1\0", "/glibc/");
    }
    #[cfg(target_arch = "riscv64")]
    {
        run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64d.so.1\0", "/glibc/");
        run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-riscv64-lp64d.so.1 /ld-linux-riscv64-lp64d.so.1\0", "/glibc/");
        run_cmd("/glibc/busybox cp /glibc/lib/libc.so.6 /lib/libc.so.6\0", "/glibc/");
    }

    for test in GLIBC_LTP {
        run_cmd(test, cd);
    }
}

fn musl_ltp() {
    let cd = "/musl/";
    chdir(&conert_str2byte(cd));
    #[cfg(target_arch = "loongarch64")]
    {
        run_cmd("/musl/busybox cp /musl/lib/libc.so /lib64/ld-musl-loongarch-lp64d.so.1\0", "/musl/");
    }
    #[cfg(target_arch = "riscv64")]
    {
        run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64-sf.so.1\0", "/musl/");
        run_cmd("/musl/busybox cp /musl/lib/libc.so /lib/ld-musl-riscv64.so.1\0", "/musl/");
        run_cmd("/musl/busybox cp /musl/lib/dlopen_dso.so /lib/dlopen_dso.so\0", "/musl/");
        run_cmd("/musl/busybox cp /musl/lib/tls_get_new-dtv_dso.so /lib/tls_get_new-dtv_dso.so\0", "/musl/");
    }
    for test in MUSL_LTP {
        run_cmd(test, cd);
    }
}