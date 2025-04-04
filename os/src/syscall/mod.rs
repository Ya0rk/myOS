mod fs;
mod process;
mod ffi;

use fs::*;
use process::*;
use ffi::SysCode;
pub use ffi::CloneFlags;

use crate::utils::SysResult;

/// handle syscall exception with `syscall_id` and other arguments
pub async fn syscall(syscall_id: usize, args: [usize; 6]) -> SysResult<usize> {
    let syscode = SysCode::from(syscall_id);
    match syscode {
        SysCode::SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        SysCode::SYSCALL_DUP => sys_dup(args[0]),
        SysCode::SYSCALL_DUP3 => sys_dup3(args[0], args[1], args[2] as u32),
        SysCode::SYSCALL_MKDIRAT => sys_mkdirat(args[0] as isize, args[1] as *const u8, args[2] as usize),
        SysCode::SYSCALL_UNLINKAT => sys_unlinkat(args[0] as isize, args[1] as *const u8, args[2] as u32),
        SysCode::SYSCALL_LINKAT => sys_linkat(args[0] as isize, args[1] as *const u8, args[2] as isize, args[3] as *const u8, args[4] as u32),
        SysCode::SYSCALL_UMOUNT2 => sys_umount2(args[0] as *const u8, args[1] as u32),
        SysCode::SYSCALL_MOUNT => sys_mount(args[0] as *const u8, args[1] as *const u8, args[2] as *const u8, args[3] as u32, args[4] as *const u8),
        SysCode::SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SysCode::SYSCALL_OPENAT => sys_openat(args[0] as isize, args[1] as *const u8, args[2] as u32, args[3] as usize),
        SysCode::SYSCALL_CLOSE => sys_close(args[0]),
        SysCode::SYSCALL_PIPE2 => sys_pipe2(args[0] as *mut u32, args[1] as i32),
        SysCode::SYSCALL_GETDENTS64 => sys_getdents64(args[0] as usize, args[1] as *const u8, args[2] as usize),
        SysCode::SYSCALL_READ => sys_read(args[0], args[1] as usize, args[2]).await,
        SysCode::SYSCALL_WRITE => sys_write(args[0], args[1] as usize, args[2]).await,
        SysCode::SYSCALL_FSTAT => sys_fstat(args[0] as usize, args[1] as *const u8),
        SysCode::SYSCALL_EXIT => sys_exit(args[0] as i32),
        SysCode::SYSCALL_NANOSLEEP => sys_nanosleep(args[0] as usize, args[1] as usize).await,
        SysCode::SYSCALL_YIELD => sys_yield().await,
        SysCode::SYSCALL_TIMES => sys_times(args[0] as *const u8),
        SysCode::SYSCALL_UNAME => sys_uname(args[0] as *mut u8),
        SysCode::SYSCALL_GETTIMEOFDAY => sys_gettimeofday(args[0] as *mut u8, args[1] as *mut u8),
        SysCode::SYSCALL_GETPID => sys_getpid(),
        SysCode::SYSCALL_GETPPID => sys_getppid(),
        SysCode::SYSCALL_CLONE => sys_clone(args[0], args[1], args[2], args[3], args[4]),
        SysCode::SYSCALL_EXEC => sys_exec(args[0] as usize).await,
        SysCode::SYSCALL_WAIT4 => sys_wait4(args[0] as isize, args[1] as usize, args[2] as usize, args[3] as usize).await,
        SysCode::GETRANDOM => sys_getrandom(args[0] as *const u8, args[1] as usize, args[2] as usize),
        SysCode::SYSCALL_SET_TID_ADDRESS => sys_set_tid_address(args[0] as usize),
        SysCode::SYSCALL_EXIT_GROUP => sys_exit_group(args[0] as i32),
        SysCode::SYSCALL_CLOCK_GETTIME => sys_clock_gettime(args[0] as usize, args[1] as *mut u8),
        SysCode::SYSCALL_CLOCK_SETTIME => sys_clock_settime(args[0] as usize, args[1] as *const u8),
        SysCode::SYSCALL_SENDFILE => sys_sendfile(args[0] as usize, args[1] as usize, args[2] as usize, args[3] as usize).await,
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
