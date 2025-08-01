mod ffi;
mod fs;
mod io;
mod io_async;
mod mm;
mod net;
mod process;
mod sync;

use crate::sync::TimeSpec;
use crate::utils::SysResult;
pub use ffi::CloneFlags;
pub use ffi::CpuSet;
pub use ffi::RLimit64;
pub use ffi::SchedParam;
pub use ffi::ShutHow;
pub use ffi::StatFs;
pub use ffi::SysCode;
use fs::*;
use io::*;
use log::info;
use mm::{sys_brk, sys_mmap, sys_munmap};
use mm::{sys_membarrier, sys_mprotect, sys_mremap, sys_shmat, sys_shmctl, sys_shmdt, sys_shmget};
use net::*;
use process::*;
use sync::*;

/// handle syscall exception with `syscall_id` and other arguments
pub async fn syscall(syscall_id: usize, args: [usize; 6]) -> SysResult<usize> {
    let syscode = SysCode::from(syscall_id);
    // info!("syscode = {}", syscode);
    match syscode {
        SysCode::SYSCALL_SHUTDOWN => sys_shutdown(args[0] as usize, args[1] as u8),
        SysCode::SYSCALL_SETHOSTNAME => sys_sethostname(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_SETDOMINNAME => sys_setdominname(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_SPLICE => {
            sys_splice(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
                args[4] as usize,
                args[5] as u32,
            )
            .await
        }
        SysCode::SYSCALL_SCHED_GETPARAM => sys_sched_getparam(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_SCHED_SETPARAM => sys_sched_setparam(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_SETRESUID => sys_setresuid(),
        SysCode::SYSCALL_SETUID => sys_setuid(),
        SysCode::SYSCALL_FCHDIR => sys_fchdir(args[0] as usize),
        SysCode::SYSCALL_SETGID => sys_setgid(args[0] as usize),
        SysCode::SYSCALL_SIGSUSPEND => sys_sigsuspend(args[0] as usize).await,
        SysCode::SYSCALL_UMASK => sys_umask(),
        SysCode::SYSCALL_FSYNC => sys_fsync(),
        SysCode::SYSCALL_PSELECT => {
            sys_pselect(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
                args[4] as usize,
                args[5] as usize,
            )
            .await
        }
        SysCode::SYSCALL_GET_MEMPOLICY => sys_get_mempolicy(),
        SysCode::SYSCALL_GETITIMER => sys_getitimer(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_SETITIMER => {
            sys_setitimer(args[0] as usize, args[1] as usize, args[2] as usize).await
        }
        SysCode::SYSCALL_FALLOCAT => sys_fallocate(),
        SysCode::SYSCALL_MSYNC => sys_msync(),
        SysCode::SYSCALL_FCHOWNAT => sys_fchownat(),
        SysCode::SYSCALL_GETGID => sys_getgid(),
        SysCode::SYSCALL_SCHED_GETAFFINITY => {
            sys_sched_getaffinity(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_SCHED_SETAFFINITY => {
            sys_sched_setaffinity(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::MEMEBARRIER => sys_membarrier(),
        SysCode::SYSCALL_GETRUSAGE => sys_getrusage(args[0] as isize, args[1] as usize),
        SysCode::SYSCALL_SETSOCKOPT => sys_setsockopt(
            args[0] as usize,
            args[1] as usize,
            args[2] as usize,
            args[3] as usize,
            args[4] as usize,
        ),
        SysCode::SYSCALL_CONNECT => {
            sys_connect(args[0] as usize, args[1] as usize, args[2] as usize).await
        }
        SysCode::SYSCALL_MREMAP => sys_mremap(),
        SysCode::SYSCALL_MADVISE => sys_madvise(),
        SysCode::SYSCALL_STATFS => sys_statfs(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_TKILL => sys_tkill(args[0] as usize, args[1] as i32),
        SysCode::SYSCALL_SIGTIMEDWAIT => {
            sys_sigtimedwait(args[0] as usize, args[1] as usize, args[2] as usize).await
        }
        SysCode::SYSCALL_CLOCK_NANOSLEEP => {
            sys_clock_nanosleep(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
            )
            .await
        }
        SysCode::SYSCALL_RENAMEAT2 => sys_renameat2(
            args[0] as isize,
            args[1] as usize,
            args[2] as isize,
            args[3] as usize,
            args[4] as u32,
        ),
        SysCode::SYSCALL_READLINKAT => sys_readlinkat(
            args[0] as isize,
            args[1] as usize,
            args[2] as usize,
            args[3] as usize,
        ),
        SysCode::SYSCALL_GETPGID => sys_getpgid(args[0] as usize),
        SysCode::SYSCALL_TGKILL => sys_tgkill(args[0] as usize, args[1] as usize, args[2] as i32),
        SysCode::SYSCALL_READLINKAT => sys_readlinkat(
            args[0] as isize,
            args[1] as usize,
            args[2] as usize,
            args[3] as usize,
        ),
        SysCode::SYSCALL_PRLIMIT64 => sys_prlimit64(
            args[0] as usize,
            args[1] as i32,
            args[2] as usize,
            args[3] as usize,
        ),
        SysCode::SYSCALL_GET_ROBUST_LIST => {
            sys_get_robust_list(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_SET_ROBUST_LIST => sys_set_robust_list(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_FUTEX => {
            sys_futex(
                args[0] as usize,
                args[1] as i32,
                args[2] as u32,
                args[3] as usize,
                args[4] as usize,
                args[5] as u32,
            )
            .await
        }
        SysCode::SYSCALL_UTIMENSAT => sys_utimensat(
            args[0] as isize,
            args[1] as usize,
            args[2] as *const [TimeSpec; 2],
            args[3] as i32,
        ),
        SysCode::SYSCALL_KILL => sys_kill(args[0] as isize, args[1] as usize),
        SysCode::SYSCALL_SYSLOG => sys_log(args[0] as i32, args[1] as usize, args[2] as usize),
        SysCode::SYSCALL_IOCTL => sys_ioctl(args[0] as usize, args[1] as usize, args[2] as usize),
        SysCode::SYSCALL_PPOLL => {
            sys_ppoll(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
            )
            .await
        }
        SysCode::SYSCALL_SYNC => sys_sync(),
        SysCode::SYSCALL_GETEGID => sys_getegid(),
        SysCode::SYSCALL_GETEUID => sys_geteuid(),
        SysCode::SYSCALL_GETTID => sys_gettid(),
        SysCode::SYSCALL_FCNTL => sys_fcntl(args[0] as usize, args[1] as u32, args[2] as usize),
        SysCode::SYSCALL_SIGACTION => {
            sys_sigaction(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_SIGPROCMASK => sys_sigprocmask(
            args[0] as usize,
            args[1] as usize,
            args[2] as usize,
            args[3] as usize,
        ),
        SysCode::SYSCALL_GETUID => sys_getuid(),
        SysCode::SYSCALL_GETCWD => sys_getcwd(args[0] as usize, args[1]),
        SysCode::SYSCALL_DUP => sys_dup(args[0]),
        SysCode::SYSCALL_DUP3 => sys_dup3(args[0], args[1], args[2] as u32),
        SysCode::SYSCALL_MKDIRAT => {
            sys_mkdirat(args[0] as isize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_UNLINKAT => {
            sys_unlinkat(args[0] as isize, args[1] as usize, args[2] as u32)
        }
        SysCode::SYSCALL_LINKAT => sys_linkat(
            args[0] as isize,
            args[1] as usize,
            args[2] as isize,
            args[3] as usize,
            args[4] as u32,
        ),
        SysCode::SYSCALL_UMOUNT2 => sys_umount2(args[0] as usize, args[1] as u32),
        SysCode::SYSCALL_MOUNT => sys_mount(
            args[0] as usize,
            args[1] as usize,
            args[2] as usize,
            args[3] as u32,
            args[4] as usize,
        ),
        SysCode::SYSCALL_CHDIR => sys_chdir(args[0] as usize),
        SysCode::SYSCALL_OPENAT => sys_openat(
            args[0] as isize,
            args[1] as usize,
            args[2] as u32,
            args[3] as usize,
        ),
        SysCode::SYSCALL_CLOSE => sys_close(args[0]),
        SysCode::SYSCALL_PIPE2 => sys_pipe2(args[0] as *mut u32, args[1] as i32),
        SysCode::SYSCALL_GETDENTS64 => {
            sys_getdents64(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_READ => sys_read(args[0], args[1] as usize, args[2]).await,
        SysCode::SYSCALL_WRITE => sys_write(args[0], args[1] as usize, args[2]).await,
        SysCode::SYS_STATX => sys_statx(
            args[0] as i32,
            args[1] as usize,
            args[2] as u32,
            args[3] as u32,
            args[4] as usize,
        ),
        SysCode::SYSCALL_FSTAT => sys_fstat(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_EXIT => sys_exit(args[0] as i32),
        SysCode::SYSCALL_NANOSLEEP => sys_nanosleep(args[0] as usize, args[1] as usize).await,
        SysCode::SYSCALL_YIELD => sys_yield().await,
        SysCode::SYSCALL_TIMES => sys_times(args[0] as usize),
        SysCode::SYSCALL_UNAME => sys_uname(args[0] as usize),
        SysCode::SYSCALL_GETTIMEOFDAY => sys_gettimeofday(args[0] as usize, args[1] as *mut u8),
        SysCode::SYSCALL_GETPID => sys_getpid(),
        SysCode::SYSCALL_GETPPID => sys_getppid(),
        SysCode::SYSCALL_BRK => sys_brk(args[0] as *const u8),
        SysCode::SYSCALL_MUNMAP => sys_munmap(args[0] as *const u8, args[1]),
        SysCode::SYSCALL_CLONE3 => sys_clone3(args[0] as usize, args[1] as usize),
        #[cfg(target_arch = "riscv64")]
        SysCode::SYSCALL_CLONE => sys_clone(args[0], args[1], args[2], args[3], args[4]),
        #[cfg(target_arch = "loongarch64")]
        SysCode::SYSCALL_CLONE => sys_clone(args[0], args[1], args[2], args[4], args[3]),
        SysCode::SYSCALL_EXECVE => sys_execve(args[0], args[1], args[2]).await,
        SysCode::SYSCALL_MMAP => sys_mmap(
            args[0] as *const u8,
            args[1],
            args[2] as i32,
            args[3] as i32,
            args[4],
            args[5],
        ),
        SysCode::SYSCALL_WAIT4 => {
            sys_wait4(
                args[0] as isize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
            )
            .await
        }
        SysCode::GETRANDOM => {
            sys_getrandom(args[0] as *const u8, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_SET_TID_ADDRESS => sys_set_tid_address(args[0] as usize),
        SysCode::SYSCALL_EXIT_GROUP => sys_exit_group(args[0] as i32),
        SysCode::SYSCALL_CLOCK_GETTIME => sys_clock_gettime(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_CLOCK_SETTIME => sys_clock_settime(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_SENDFILE => {
            sys_sendfile(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
            )
            .await
        }
        SysCode::SYSCALL_FACCESSAT => sys_faccessat(
            args[0] as isize,
            args[1] as usize,
            args[2] as u32,
            args[3] as u32,
        ),
        SysCode::SYSCALL_LSEEK => sys_lseek(args[0] as usize, args[1] as isize, args[2] as usize),
        SysCode::SYSCALL_SETSID => sys_setsid(),
        SysCode::SYSCALL_SETPGID => sys_setpgid(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_GETPGID => sys_getpgid(args[0] as usize),
        SysCode::SYSCALL_SIGRETURN => sys_sigreturn(),
        SysCode::SYSCALL_SYSINFO => sys_sysinfo(args[0] as usize),
        SysCode::SYSCALL_READV => {
            sys_readv(args[0] as usize, args[1] as usize, args[2] as usize).await
        }
        SysCode::SYSCALL_FTRUNCATE64 => sys_ftruncate64(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_FCHMODAT => sys_fchmodat(),
        SysCode::SYSCALL_PREAD64 => {
            sys_pread64(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
            )
            .await
        }
        SysCode::SYSCALL_PWRITE64 => {
            sys_pwrite64(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as usize,
            )
            .await
        }
        SysCode::SYSCALL_FSTATAT => sys_fstatat(
            args[0] as isize,
            args[1] as usize,
            args[2] as usize,
            args[3] as u32,
        ),
        SysCode::SYSCALL_SOCKET => sys_socket(args[0] as usize, args[1] as usize, args[2] as usize),
        SysCode::SYSCALL_BIND => sys_bind(args[0] as usize, args[1] as usize, args[2] as usize),
        SysCode::SYSCALL_LISTEN => sys_listen(args[0] as usize, args[1] as usize),
        SysCode::SYSCALL_ACCEPT4 => {
            sys_accept4(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as u32,
            )
            .await
        }
        SysCode::SYSCALL_ACCEPT => {
            sys_accept(args[0] as usize, args[1] as usize, args[2] as usize).await
        }
        SysCode::SYSCALL_GETSOCKNAME => {
            sys_getsockname(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_GETPEERNAME => {
            sys_getpeername(args[0] as usize, args[1] as usize, args[2] as usize)
        }
        SysCode::SYSCALL_SENDTO => {
            sys_sendto(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as u32,
                args[4] as usize,
                args[5] as usize,
            )
            .await
        }
        // TODO: 还需修改udp的recvfrom
        SysCode::SYSCALL_RECVFROM => {
            sys_recvfrom(
                args[0] as usize,
                args[1] as usize,
                args[2] as usize,
                args[3] as u32,
                args[4] as usize,
                args[5] as usize,
            )
            .await
        }
        SysCode::SYSCALL_SOCKETPAIR => sys_socketpair(
            args[0] as usize,
            args[1] as usize,
            args[2] as usize,
            args[3] as usize,
        ),
        SysCode::SYSCALL_GETSOCKOPT => sys_getsockopt(
            args[0] as usize,
            args[1] as usize,
            args[2] as usize,
            args[3] as usize,
            args[4] as usize,
        ),
        SysCode::SYSCALL_WRITEV => {
            sys_writev(args[0] as usize, args[1] as usize, args[2] as usize).await
        }
        SysCode::SYSCALL_MPROTECT => {
            sys_mprotect(args[0] as *const u8, args[1] as usize, args[2] as i32)
        }
        SysCode::SYSCALL_SHMGET => sys_shmget(args[0] as isize, args[1] as usize, args[2] as i32),
        SysCode::SYSCALL_SHMAT => sys_shmat(args[0] as isize, args[1] as *const u8, args[2] as i32),
        SysCode::SYSCALL_SHMDT => sys_shmdt(args[0] as *const u8),
        SysCode::SYSCALL_SHMCTL => {
            sys_shmctl(args[0] as isize, args[1] as isize, args[2] as *const u8)
        }
        SysCode::SYSCALL_COPY_FILE_RANGE => {
            sys_copy_file_range(
                args[0] as u32,
                args[1] as usize,
                args[2] as u32,
                args[3] as usize,
                args[4] as usize,
                args[5] as usize,
            )
            .await
        }

        _ => {
            log::error!("Unsupported syscall_id: {}", syscall_id);
            panic!("unimpl syscall: No.{}", syscall_id);
            Ok(0)
        }
    }
}
