mod fs;
mod process;
mod sys_num;

use fs::*;
use process::*;
use sys_num::SysCode;

use crate::utils::errtype::SysResult;

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> SysResult<usize> {
    let syscode = SysCode::from(syscall_id);
    match syscode {
        SysCode::SYSCALL_GETCWD => sys_getcwd(args[0] as *mut u8, args[1]),
        SysCode::SYSCALL_DUP => sys_dup(args[0]),
        SysCode::SYSCALL_OPENAT => sys_openat(args[0] as isize, args[1] as *const u8, args[2] as u32, args[3] as usize),
        SysCode::SYSCALL_CLOSE => sys_close(args[0]),
        SysCode::SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SysCode::SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SysCode::SYSCALL_EXIT => sys_exit(args[0] as i32),
        SysCode::SYSCALL_YIELD => sys_yield(),
        SysCode::SYSCALL_GET_TIME => sys_get_time(),
        SysCode::SYSCALL_GETPID => sys_getpid(),
        SysCode::SYSCALL_FORK => sys_fork(),
        SysCode::SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SysCode::SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
