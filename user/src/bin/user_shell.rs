#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::{string::String, vec::Vec};
use user_lib::console::getchar;
use user_lib::{chdir, exec, exit, fork, getcwd, getpid, mkdir, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    chdir(&conert_str2byte("musl/basic"));
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    match parts[0] {
                        "cd" => {
                            if parts.len() > 1 {
                                println!("chdir to {}", parts[1]);
                                let byte_slice: &[u8] = &conert_str2byte(parts[1]);
                                chdir(byte_slice);
                            } else {
                                println!("cd command error! no enough arguments");
                            }
                            line.clear();
                        }
                        "mkdir" => {
                            if parts.len() > 1 {
                                println!("mkdir {}", parts[1]);
                                let byte_slice: &[u8] = &conert_str2byte(parts[1]);
                                mkdir(byte_slice, 0);
                            } else {
                                println!("cd command error! no enough arguments");
                            }
                            line.clear();
                        }
                        _ => {
                            line.push('\0');
                            let pid = fork();
                            let p_pid = &pid as *const isize;
                            println!("address of pid is {:#x}", p_pid as usize);
                            println!("pid after fork is {}", pid);
                            if pid == 0 {
                                // child process
                                println!("[basic] child get pid = {}", pid);
                                exec(line.as_str());
                                exit(0);
                            } else {
                                println!("[basic] parent get pid = {}", pid);
                                let mut exit_code: i32 = 0;
                                println!("pid before wait4 is {}", pid);
                                let exit_pid = waitpid(pid as usize, &mut exit_code, 0);
                                println!("pid after wait4 is {}", pid);
                                assert_eq!(pid, exit_pid);
                                println!("Shell: Process {} exited with code {}", pid, exit_code);
                            }
                            line.clear();
                        }
                    }
                }
                let mut buffer: [u8; 256] = [0; 256];
                let result = getcwd(&mut buffer, 256);
                if result >= 0 {
                    let cwd = String::from_utf8_lossy(&buffer[..result as usize]);
                    print!("{} >> ", cwd);
                } else {
                    print!("get cwd error >>");
                }
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}

/// 传入str引用转换为C风格字符串，使其可以被用作系统调用
pub fn conert_str2byte(input: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = input.as_bytes().to_vec();
    bytes.push(0);
    bytes
}