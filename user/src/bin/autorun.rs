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

const TESTCASES: &[&str] = &[
    "/ltp_testcode_ours.sh\0",
    // "./time-test\0",
    // "./test-splice.sh\0",
    // "./libctest_testcode.sh\0",
    // "./lua_testcode.sh\0",
    // "./netperf_testcode.sh\0",
    // "./libcbench_testcode.sh\0",
    // "./iozone_testcode.sh\0",
    // "./unixbench_testcode.sh\0",
    // "./cyclictest_testcode.sh\0",
    // "./iperf_testcode.sh\0",
    // "./lmbench_testcode.sh\0",
    // "./run-static.sh\0",
];

const TESTCASES_LTP: &[&str] = &[
    // 已过滤掉所有未选中的测试用例
    // 以下按原始顺序列出所有选中的测试路径
    // "./ltp/testcases/bin/abs01\0",
    // "./ltp/testcases/bin/accept01\0",
    // "./ltp/testcases/bin/accept02\0",
    // "./ltp/testcases/bin/accept03\0",
    // "./ltp/testcases/bin/accept04\0",
    // "./ltp/testcases/bin/accept4_01\0",
    // "./ltp/testcases/bin/asapi_01\0",
    // "./ltp/testcases/bin/alarm02\0",
    // "./ltp/testcases/bin/alarm03\0",
    // "./ltp/testcases/bin/alarm05\0",
    // "./ltp/testcases/bin/alarm06\0",
    // "./ltp/testcases/bin/alarm07\0",
    // "./ltp/testcases/bin/atof01\0",
    // "./ltp/testcases/bin/bind01\0",
    // "./ltp/testcases/bin/bind02\0",
    // "./ltp/testcases/bin/brk01\0",
    // "./ltp/testcases/bin/capget01\0",
    // "./ltp/testcases/bin/chmod01\0",
    // "./ltp/testcases/bin/chmod03\0",
    // "./ltp/testcases/bin/chmod05\0",
    // "./ltp/testcases/bin/chown01\0",
    // "./ltp/testcases/bin/chown02\0",
    // "./ltp/testcases/bin/chown03\0",
    // "./ltp/testcases/bin/chown05\0",
    // "./ltp/testcases/bin/chroot03\0",
    // "./ltp/testcases/bin/clock_getres01\0",
    // "./ltp/testcases/bin/clock_nanosleep04\0",
    // "./ltp/testcases/bin/close_range02\0",
    // "./ltp/testcases/bin/close01\0",
    // "./ltp/testcases/bin/clone01\0",
    // "./ltp/testcases/bin/clone02\0",
    // "./ltp/testcases/bin/clone03\0",
    // "./ltp/testcases/bin/clone06\0",
    // "./ltp/testcases/bin/clone07\0",
    // "./ltp/testcases/bin/clone302\0",
    // "./ltp/testcases/bin/connect01\0",
    // "./ltp/testcases/bin/dup01\0",
    // "./ltp/testcases/bin/dup02\0",
    // "./ltp/testcases/bin/exit01\0",
    // "./ltp/testcases/bin/exit02\0",
    // "./ltp/testcases/bin/fchdir01\0",
    // "./ltp/testcases/bin/fchdir02\0",
    // "./ltp/testcases/bin/fcntl01\0",
    // "./ltp/testcases/bin/fcntl02\0",
    // "./ltp/testcases/bin/getgid01\0",
    // "./ltp/testcases/bin/getcwd01\0",
    // "./ltp/testcases/bin/getcwd02\0",
    // "./ltp/testcases/bin/getpeername01\0",
    // "./ltp/testcases/bin/getpgid01\0",
    // "./ltp/testcases/bin/getpid02\0",
    // "./ltp/testcases/bin/getsockname01\0",
    // "./ltp/testcases/bin/getsockopt01\0",
    // "./ltp/testcases/bin/getitimer01\0",
    // "./ltp/testcases/bin/getitimer02\0",
    // "./ltp/testcases/bin/lseek01\0",
    // "./ltp/testcases/bin/mkdir02\0",
    // "./ltp/testcases/bin/mkdir03\0",
    // "./ltp/testcases/bin/mkdirat01\0",
    // "./ltp/testcases/bin/mkdirat02\0",
    // "./ltp/testcases/bin/nextafter01\0",
    // "./ltp/testcases/bin/open01\0",
    // "./ltp/testcases/bin/open02\0",
    // "./ltp/testcases/bin/openat01\0",
    // "./ltp/testcases/bin/pipe01\0",
    // "./ltp/testcases/bin/rt_sigaction01\0",
    // "./ltp/testcases/bin/sched_setparam01\0",
    // "./ltp/testcases/bin/sched_setparam02\0",
    // "./ltp/testcases/bin/setpgid01\0",
    // "./ltp/testcases/bin/setpgid02\0",
    // "./ltp/testcases/bin/setpgrp01\0",
    // "./ltp/testcases/bin/setregid01\0",
    // "./ltp/testcases/bin/setregid04\0",
    // "./ltp/testcases/bin/setreuid01\0",
    // "./ltp/testcases/bin/setrlimit01\0",
    // "./ltp/testcases/bin/setrlimit02\0",
    // "./ltp/testcases/bin/setrlimit03\0",
    // "./ltp/testcases/bin/setrlimit04\0",
    // "./ltp/testcases/bin/setsid01\0",
    // "./ltp/testcases/bin/setsockopt01\0",
    // "./ltp/testcases/bin/setsockopt03\0",
    // "./ltp/testcases/bin/setsockopt04\0",
    // "./ltp/testcases/bin/settimeofday02\0",
    // "./ltp/testcases/bin/setuid01\0",
    // "./ltp/testcases/bin/setxattr02\0",
    // "./ltp/testcases/bin/sigaction01\0",
    // "./ltp/testcases/bin/sigaction02\0",
    // "./ltp/testcases/bin/sigaltstack01\0",
    // "./ltp/testcases/bin/sigaltstack02\0",
    // "./ltp/testcases/bin/signal01\0",
    // "./ltp/testcases/bin/signal02\0",
    // "./ltp/testcases/bin/signal03\0",
    // "./ltp/testcases/bin/signal04\0",
    // "./ltp/testcases/bin/signal05\0",
    // "./ltp/testcases/bin/sigwait01\0",
    // "./ltp/testcases/bin/socket02\0",
    // "./ltp/testcases/bin/socketpair01\0",
    // "./ltp/testcases/bin/socketpair02\0",
    // "./ltp/testcases/bin/splice01\0",
    // "./ltp/testcases/bin/splice03\0",
    // "./ltp/testcases/bin/splice07\0",
    // "./ltp/testcases/bin/stack_space\0",
    // "./ltp/testcases/bin/stat01\0",
    // "./ltp/testcases/bin/stat01_64\0",
    // "./ltp/testcases/bin/stat02\0",
    // "./ltp/testcases/bin/stat02_64\0",
    // "./ltp/testcases/bin/stream01\0",
    // "./ltp/testcases/bin/stream02\0",
    // "./ltp/testcases/bin/stream03\0",
    // "./ltp/testcases/bin/stream04\0",
    // "./ltp/testcases/bin/stream05\0",
    // "./ltp/testcases/bin/string01\0",
    // "./ltp/testcases/bin/sync_file_range01\0",
    // "./ltp/testcases/bin/syscall01\0",
    // "./ltp/testcases/bin/sysconf01\0",
    // "./ltp/testcases/bin/sysinfo01\0",
    // "./ltp/testcases/bin/sysinfo02\0",
    // "./ltp/testcases/bin/waitpid01\0",
    // "./ltp/testcases/bin/write01\0",
    // "./ltp/testcases/bin/write02\0",
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
                ["PATH=", "/bin:", pwd.strip_suffix("/").unwrap(), ].concat().as_str(),
                "HOME=/\0",
                ["LD_LIBRARY_PATH=", pwd.strip_suffix("/").unwrap(), "/lib\0"].concat().as_str(),
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
    // run_cmd("/glibc/busybox --install /bin\0");
    // let child_pid = fork();
    // if child_pid == 0 {
    //     let cd = "/glibc/";
    //     chdir(&conert_str2byte(cd));
    //     for test in TEST {
    //         run_cmd(test, cd);
    //     }
    //     // 拷贝动态库到指定位置,这里是glibc的动态库
    //     #[cfg(target_arch = "loongarch64")]
    //     {
    //         run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-loongarch-lp64d.so.1 /lib64/ld-linux-loongarch-lp64d.so.1\0", "/glibc/");
    //         run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-loongarch-lp64d.so.1 /ld-linux-loongarch-lp64d.so.1\0", "/glibc/");
    //     }
    //     #[cfg(target_arch = "riscv64")]
    //     {
    //         run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-riscv64-lp64d.so.1 /lib/ld-linux-riscv64-lp64d.so.1\0", "/glibc/");
    //         run_cmd("/glibc/busybox cp /glibc/lib/ld-linux-riscv64-lp64d.so.1 /ld-linux-riscv64-lp64d.so.1\0", "/glibc/");
    //         run_cmd("/glibc/busybox cp /glibc/lib/libc.so.6 /lib/libc.so.6\0", "/glibc/");
            
    //     }

    //     // 除去busybox和basic
    //     for test in TESTCASES {
    //         run_cmd(test, cd);
    //     }

    //     // // ltp测试
    //     // run_cmd("/glibc/busybox echo '#### OS COMP TEST GROUP START ltp-musl ####'", "/glibc/");
    //     // for test in TESTCASES_LTP {
    //     //     run_cmd(test, cd);
    //     // }
    //     // run_cmd("echo '#### OS COMP TEST GROUP END ltp-musl ####'", "/glibc/");
        
    //     run_cmd("/glibc/busybox rm -rf /lib/*\0", "/glibc/"); // 删除glibc的动态库，避免影响musl的basic测试
    //     exit(0); 
    // } else {
    //     println!("main parent");
    //     loop {
    //         let mut exit_code: i32 = 0;
    //         let pid = waitpid(child_pid as usize, &mut exit_code, 0);
    //         // let pid = wait(&mut exit_code);
    //         if pid == child_pid {
    //             break;
    //         }
    //     }
    // }

    let child_pid = fork();
    if child_pid == 0 {
        let cd = "/musl/";
        chdir(&conert_str2byte(cd));
        for test in TEST {
            run_cmd(test, cd);
        }
        
        // 拷贝动态库到指定位置,这里是musl的动态库
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

        for test in TESTCASES {
            run_cmd(test, cd);
            run_cmd("/musl/busybox cat /proc/meminfo\0", "/musl/");
        }

        // musl的ltp测试
        // run_cmd("/musl/busybox mkdir /testcase", "/musl/");
        // run_cmd("/musl/busybox cp /musl/ltp/testcases/bin/abs01 /testcase/abs01\0", "/musl/");
        // println!("cp abs01 done");
        // run_cmd("/musl/busybox cp /musl/ltp/testcases/bin/accept01 /testcase/accept01\0", "/musl/");
        // println!("cp accept01 done");
        // run_cmd("/musl/busybox sed 's|target_dir=\"ltp/testcases/bin\"|target_dir=\"/testcase\"|' testcode.sh\0", "/musl/");
        // println!("sed done");
        // for test in TESTCASES_LTP {
        //     run_cmd(test, cd);
        // }

        // run_cmd("/musl/busybox rm -rf /lib/*\0", "/musl/");
        // run_cmd("/musl/busybox --install /bin\0", "/musl/");
        
        exit(0);
    } else {
        println!("main parent");
        loop {
            let mut exit_code: i32 = 0;
            let pid = waitpid(child_pid as usize, &mut exit_code, 0);
            // let pid = wait(&mut exit_code);
            if pid == child_pid {
                break;
            }
        }
    } 

    0
}