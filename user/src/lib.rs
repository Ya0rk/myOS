#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
mod ffi;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use buddy_system_allocator::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 32768;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

bitflags! {
    pub struct OpenFlags: u32 {
        const O_RDONLY = 0;
        const O_WRONLY = 1 << 0;
        const O_RDWR = 1 << 1;
        const O_CREATE = 1 << 6;
        const O_TRUNC = 1 << 10;
        const O_DIRECTROY = 1 << 21;
    }
}

pub fn getcwd(buf: &mut [u8], size: usize) -> isize {
    sys_getcwd(buf, size)
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}

pub fn dup2(oldfd: usize, newfd: usize) -> isize {
    sys_dup2(oldfd, newfd)
}

pub fn dup3(oldfd: usize, newfd: usize, flags: u32) -> isize {
    sys_dup3(oldfd, newfd, flags)
}

pub fn mkdir(path: &str, mode: usize) -> isize {
    sys_mkdir(path, mode)
}

pub fn mkdirat(fd: isize, path: &str, mode: usize) -> isize {
    sys_mkdirat(fd, path, mode)
}

pub fn umount2(special: &str, flags: u32) -> isize {
    sys_umount2(special, flags)
}

pub fn umount(special: &str) -> isize {
    sys_umount(special)
}

pub fn mount(source: &str, target: &str, fstype: &str, flags: u32, data: &str) -> isize {
    sys_mount(source, target, fstype, flags, data)
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits())
}

pub fn openat(fd: isize, path: &str, flags: OpenFlags, mode: usize) -> isize {
    sys_openat(fd, path, flags.bits(), mode)
}
pub fn close(fd: usize) -> isize {
    sys_close(fd)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}
