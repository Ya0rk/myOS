use core::arch::asm;
use crate::ffi::*;

const AT_FDCWD: isize = -100;

#[cfg(target_arch="riscv64")]
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


#[cfg(target_arch="loongarch64")]
fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "syscall 0",
            inlateout("$a0") args[0] => ret,
            in("$a1") args[1],
            in("$a2") args[2],
            in("$a3") args[3],
            in("$a4") args[4],
            in("$a5") args[5],
            in("$a7") id
        );
    }
    ret
}
pub fn sys_socket(domain: usize, ty: usize, protocol: usize) -> isize {
    syscall(SYSCALL_SOCKET, [domain, ty, protocol, 0, 0, 0])
}

pub fn sys_bind(sockfd: usize, addr: usize, addrlen: usize) -> isize {
    syscall(SYSCALL_BIND, [sockfd, addr, addrlen, 0, 0, 0])
}

pub fn sys_listen(sockfd: usize, backlog: usize) -> isize {
    syscall(SYSCALL_LISTEN, [sockfd, backlog, 0, 0, 0, 0])
}

pub fn sys_connect(sockfd: usize, addr: usize, addrlen: usize) -> isize {
    syscall(SYSCALL_CONNECT, [sockfd, addr, addrlen, 0, 0, 0])
}

pub fn sys_accept(sockfd: usize, addr: usize, addrlen: usize) -> isize {
    syscall(SYSCALL_ACCEPT, [sockfd, addr, addrlen, 0, 0, 0])
}

pub fn sys_sendto(sockfd: usize, msg: usize, msglen: usize, flags: usize, dest_addr: usize, addrlen: usize) -> isize {
    syscall(SYSCALL_SENDTO, [sockfd, msg, msglen, flags, dest_addr, addrlen])
}

pub fn sys_recvfrom(sockfd: usize, buf: usize, buflen: usize, flags: usize, src_addr: usize, addrlen: usize) -> isize {
    syscall(SYSCALL_RECVFROM, [sockfd, buf, buflen, flags, src_addr, addrlen])
}

pub fn sys_unlinkat(dirfd: isize, path: &str, flags: u32) -> isize {
    syscall(SYSCALL_UNLINKAT,[dirfd as usize, path.as_ptr() as usize, flags as usize, 0, 0, 0])
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

pub fn sys_mkdir(path: &[u8], mode: usize) -> isize {
    syscall(SYSCALL_MKDIRAT, [AT_FDCWD as usize, path.as_ptr() as usize, mode as usize, 0, 0, 0])
}

pub fn sys_mkdirat(fd: isize, path: &str, mode: usize) -> isize {
    syscall(SYSCALL_MKDIRAT, [fd as usize, path.as_ptr() as usize, mode as usize, 0, 0, 0])
}

pub fn sys_umount2(special: &str, flags: u32) -> isize {
    syscall(SYSCALL_UMOUNT2, [special.as_ptr() as usize, flags as usize, 0, 0, 0, 0])
}

pub fn sys_umount(special: &str) -> isize {
    syscall(SYSCALL_UMOUNT2, [special.as_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_mount(source: &str, target: &str, fstype: &str, flags: u32, data: &str) -> isize {
    syscall(SYSCALL_MOUNT, [source.as_ptr() as usize, target.as_ptr() as usize, fstype.as_ptr() as usize, flags as usize, data.as_ptr() as usize, 0])
}

pub fn sys_chdir(path: &[u8]) -> isize {
    syscall(SYSCALL_CHDIR, [path.as_ptr() as usize, 0, 0, 0, 0, 0])
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

pub fn sys_pipe2(fd: &mut [u32], flags: u32) -> isize {
    syscall(SYSCALL_PIPE2, [fd.as_mut_ptr() as usize, flags as usize, 0, 0, 0, 0])
}

pub fn sys_getdents64(fd: usize, buf: &mut [u8], len: usize) -> isize {
    syscall(SYSCALL_GETDENTS64, [fd, buf.as_ptr() as usize, len, 0, 0, 0])
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

pub fn sys_fstat(fd: usize, kst: &mut [u8]) -> isize {
    syscall(SYSCALL_FSTAT, [fd, kst.as_ptr() as usize, 0, 0, 0, 0])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0, 0, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_nanosleep(req: &[u8], rem: &[u8]) -> isize {
    syscall(SYSCALL_NANOSLEEP, [req.as_ptr() as usize, rem.as_ptr() as usize, 0, 0, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0, 0, 0, 0])
}

pub fn sys_times(tms: &mut [u8]) -> isize {
    syscall(SYSCALL_TIMES, [tms.as_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_uname(buf: &mut [u8]) -> isize {
    syscall(SYSCALL_UNAME, [buf.as_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_gettimeofday(ts: &mut [u8]) -> isize {
    syscall(SYSCALL_GETTIMEOFDAY, [ts.as_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0, 0, 0, 0])
}

pub fn sys_getppid() -> isize {
    syscall(SYSCALL_GETPPID, [0, 0, 0, 0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0, 0, 0, 0])
}

pub fn sys_execve(path: &str, argv: *const usize, env: *const usize) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, argv as usize, env as usize, 0, 0, 0])
}

pub fn sys_wait4(pid: isize, exit_code: *mut i32, options: usize) -> isize {
    syscall(SYSCALL_WAIT4, [pid as usize, exit_code as usize, options as usize, 0, 0, 0])
}

pub fn sys_mmap(addr: *const u8, length: usize, prot: i32, flags: i32, fd: i32, offset: usize) -> *mut u8 {
    syscall(SYSCALL_MMAP, [addr as usize, length, prot as usize, flags as usize, fd as usize, offset]) as *mut u8
}

pub fn sys_munmap(addr: *const u8, length: usize) -> isize {
    syscall(SYSCALL_MUNMAP, [addr as usize, length, 0, 0, 0, 0])
}

pub fn sys_brk(end_data_segment: *const u8) -> isize {
    syscall(SYSCALL_BRK, [end_data_segment as usize, 0, 0, 0, 0, 0])
}

pub fn sys_kill(pid: usize, signal: usize) -> isize {
    syscall(SYSCALL_KILL, [pid, signal, 0, 0, 0, 0])
}