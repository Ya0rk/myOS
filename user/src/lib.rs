#![no_std]
#![feature(linkage)]
// #![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;
mod ffi;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use core::{convert::TryInto, ptr::{null}};

use alloc::vec::Vec;
use buddy_system_allocator::LockedHeap;
use syscall::*;

const USER_HEAP_SIZE: usize = 0x32000;

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

/// 地址协议簇类型
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 10;

/// 套接字类型
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32  = 2;

/// 协议类型
pub const IPPROTO_TCP: i32 = 6;
pub const IPPROTO_UDP: i32 = 17;

#[repr(C)]
pub struct SockIpv4 {
    /// 地址协议族(AF_INET)
    pub family: u16,
    /// Ipv4 的端口
    pub port: u16,
    /// Ipv4 的地址
    pub addr: [u8; 4],
    /// 零位，用于后续扩展
    pub zero: [u8; 8],
}

impl SockIpv4 {
    /// 创建一个默认的 SockIpv4，地址为本地回环
    pub fn new_ipv4(port: u16) -> Self {
        Self {
            family: AF_INET,
            port,
            addr: [127, 0, 0, 1], // 默认地址为本地回环
            zero: [0u8; 8],
        }
    }
    /// 创建一个指定地址的 SockIpv4
    pub fn new_ipv4_withaddr(port: u16, addr: [u8; 4]) -> Self {
        Self {
            family: AF_INET,
            port,
            addr,
            zero: [0u8; 8],
        }
    }
}


pub fn socket(domain: usize, ty: usize, protocol: usize) -> isize {
    sys_socket(domain, ty, protocol)
}

pub fn bind(sockfd: usize, addr: usize, addrlen: usize) -> isize {
    sys_bind(sockfd, addr, addrlen)
}

pub fn listen(sockfd: usize, backlog: usize) -> isize {
    sys_listen(sockfd, backlog)
}

pub fn accept(sockfd: usize, addr: usize, addrlen: usize) -> isize {
    sys_accept(sockfd, addr, addrlen)
}

pub fn connect(sockfd: usize, addr: usize, addrlen: usize) -> isize {
    sys_connect(sockfd, addr, addrlen)
}

pub fn sendto(sockfd: usize, buf: &[u8], buflen: usize, flags: u32, dest_addr: *const SockIpv4, addrlen: usize) -> isize {
    sys_sendto(sockfd, buf.as_ptr() as usize, buflen, flags as usize, dest_addr as usize, addrlen)
}

pub fn recvfrom(sockfd: usize, buf: &mut [u8], buflen: usize, flags: u32, src_addr: *mut SockIpv4, addrlen: *const u32) -> isize {
    sys_recvfrom(sockfd, buf.as_mut_ptr() as usize, buflen, flags as usize, src_addr as usize, addrlen as usize)
}

pub fn unlink(dirfd: isize, path: &str, flags: OpenFlags) -> isize {
    sys_unlinkat(dirfd, path, flags.bits())
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

pub fn mkdir(path: &[u8], mode: usize) -> isize {
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

pub fn chdir(path: &[u8]) -> isize {
    sys_chdir(path)
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
pub fn pipe(fd: &mut [u32]) -> isize {
    sys_pipe2(fd, 0)
}
pub fn getdents64(fd: usize, buf: &mut [u8], len: usize) -> isize {
    sys_getdents64(fd, buf, len)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn fstat(fd: usize, kst: &mut [u8]) -> isize {
    sys_fstat(fd, kst)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn times(tms: &mut [u8]) -> isize {
    sys_times(tms)
}
pub fn uname(buf: &mut [u8]) -> isize {
    sys_uname(buf)
}

pub fn get_time() -> isize {
    let mut ts = [0u8; 16];
    sys_gettimeofday(ts.as_mut())
}
pub fn get_mtime() -> isize {
    let mut ts = [0u8; 16];
    sys_gettimeofday(ts.as_mut());
    let (first, second) = ts.split_at(8);
    
    // Convert each 8-byte slice to usize
    let first_usize = usize::from_ne_bytes(first.try_into().unwrap());
    let second_usize = usize::from_ne_bytes(second.try_into().unwrap());
    ((first_usize & 0xffff) * 1000 + second_usize / 1000) as isize
}
pub fn getpid() -> isize {
    sys_getpid()
}

pub fn getppid() -> isize {
    sys_getppid()
}

pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_execve(path, null(), null())
}
pub fn execve(path: &str, argv: &[&str], env: &[&str]) -> isize {
    let mut argv: Vec<usize> = argv.iter().map(|s| (*s).as_ptr() as usize).collect();
    let mut env: Vec<usize> = env.iter().map(|s| (*s).as_ptr() as usize).collect();
    argv.push(0);
    env.push(0);
    sys_execve(path, argv.as_ptr(), env.as_ptr())
}

pub fn wait(exit_code: &mut i32) -> isize {
    sys_wait4(-1, exit_code as *mut _, 0)
}
pub fn waitpid(pid: usize, exit_code: &mut i32, options: usize) -> isize {
    sys_wait4(pid as isize, exit_code as *mut _, options)
}

pub fn sleep(period_ms: usize) {
    let req = [period_ms as u8, 0, 0, 0, 0, 0, 0, 0];
    sys_nanosleep(&req, &[0; 8]);
}

pub fn mmap(addr: *const u8, length: usize, prot: i32, flags: i32, fd: i32, offset: usize) -> *mut u8 {
    sys_mmap(addr, length, prot, flags, fd, offset)
}

pub fn munmap(addr: *const u8, length: usize) -> isize {
    sys_munmap(addr, length)
}

pub fn brk(end_data_segment: *const u8) -> isize {
    sys_brk(end_data_segment)
}

pub fn kill(pid: usize, signal: usize) -> isize {
    sys_kill(pid, signal)
}