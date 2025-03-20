#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{ openat, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let f = openat(-100, "/AAA_test_dir\0", OpenFlags::O_CREATE | OpenFlags::O_DIRECTROY, 0);
    if f < 0 {
        print!("error {} openat dir fail\n", f);
    }
    let f = openat(-100, "/BBB_test_file\0", OpenFlags::O_CREATE | OpenFlags::O_WRONLY, 0);
    if f < 0 {
        print!("error {} openat file fail\n", f);
    }
    0
}