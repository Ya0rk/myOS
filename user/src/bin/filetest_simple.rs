#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{close, openat, read, write, OpenFlags};

#[no_mangle]
pub fn main() -> i32 {
    let test_str = "Hello, world!";
    let filea = "filea\0";
    let fd = openat(-100, filea, OpenFlags::O_CREATE | OpenFlags::O_WRONLY, 0);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, test_str.as_bytes());
    close(fd);

    let fd = openat(-100, filea, OpenFlags::O_RDONLY, 0);
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);

    assert_eq!(test_str, core::str::from_utf8(&buffer[..read_len]).unwrap(),);
    println!("file_test passed!");
    0
}
