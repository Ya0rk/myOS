#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{ fstat, openat, write, OpenFlags};

pub struct Kstat {
    pub st_dev: u32,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u32,
    pub __pad: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub __pad2: i32,
    pub st_blocks: u64,
    pub st_atime_sec: i64,
    pub st_atime_nsec: i64,
    pub st_mtime_sec: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime_sec: i64,
    pub st_ctime_nsec: i64,
    pub __unused: [u32; 2],
}

#[no_mangle]
pub fn main() -> i32 {
    let f = openat(-100, "/AAA_test_dir\0", OpenFlags::O_CREATE | OpenFlags::O_DIRECTROY, 0);
    if f < 0 {
        print!("error {} openat dir fail\n", f);
    }
    let f = openat(-100, "/BBB_test_file\0", OpenFlags::O_CREATE | OpenFlags::O_RDWR, 0);
    let buf = b"hello world";
    write(f as usize, buf);
    let mut kst = [0u8; size_of::<Kstat>()];
    fstat(f as usize, &mut kst);
    // 将字节数组解释为 Kstat 结构体
    let kstat = unsafe { &*(kst.as_ptr() as *const Kstat) };
    // 打印 st_size
    println!("File size: {}", kstat.st_size);
    if f < 0 {
        print!("error {} openat file fail\n", f);
    }
    0
}