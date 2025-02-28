use core::arch::asm;
use crate::ffi::*;

const AT_FDCWD: isize = -100;

fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x13") args[3],
            in("x14") args[4],
            in("x15") args[5],
            in("x17") id
        );
    }
    ret
}

pub fn sys_getcwd(buf: &mut [u8], size: usize) -> isize {
    syscall(SYSCALL_GETCWD, [buf.as_mut_ptr() as usize, size as usize, 0, 0, 0, 0])
}

pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0, 0, 0, 0])
}

pub fn sys_dup2(oldfd: usize, newfd: usize) -> isize {
    syscall(SYSCALL_DUP3, [oldfd, newfd, 0, 0, 0, 0])
}

pub fn sys_dup3(oldfd: usize, newfd: usize, flags: u32) -> isize {
    syscall(SYSCALL_DUP3, [oldfd, newfd, flags as usize, 0, 0, 0])
}

pub fn sys_mkdir(path: &str, mode: usize) -> isize {
    syscall(SYSCALL_MKDIRAT, [AT_FDCWD as usize, path.as_ptr() as usize, mode as usize, 0, 0, 0])
}

pub fn sys_mkdirat(fd: isize, path: &str, mode: usize) -> isize {
    syscall(SYSCALL_MKDIRAT, [fd as usize, path.as_ptr() as usize, mode as usize, 0, 0, 0])
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPENAT, [AT_FDCWD as usize, path.as_ptr() as usize, flags as usize, 0, 0, 0])
}

pub fn sys_openat(fd: isize, path: &str, flags: u32, mode: usize) -> isize {
    syscall(SYSCALL_OPENAT, [fd as usize, path.as_ptr() as usize, flags as usize, mode as usize, 0, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0, 0, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len(), 0, 0, 0],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(), 0, 0, 0])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0, 0, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0, 0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0, 0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0, 0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0, 0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0, 0, 0, 0])
}
