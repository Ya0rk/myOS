use core::fmt::{self, Display};
use num_enum::FromPrimitive;
use zerocopy::{Immutable, IntoBytes};

use crate::sync::timer::get_time_s;

#[derive(IntoBytes, Immutable)]
#[allow(unused)]
#[repr(C)]
pub struct Utsname {
    /// 操作系统名称
    pub sysname:    [u8; 65],
    /// 节点名称（主机名称）
    pub nodename:   [u8; 65],
    /// 操作系统的发布版本
    pub release:    [u8; 65],
    /// 操作系统的版本信息
    pub version:    [u8; 65],
    /// 硬件架构标识符
    pub machine:    [u8; 65],
    /// NIS 或 YP 域名
    pub domainname: [u8; 65],
}

impl Utsname {
    pub fn new() -> Self {
        Self {
            sysname: Self::copy_bytes("YooOs"),
            nodename: Self::copy_bytes("Ya0rk"),
            release: Self::copy_bytes("1.1"),
            version: Self::copy_bytes("1.1"),
            machine: Self::copy_bytes("riscv64"),
            domainname: Self::copy_bytes("Ya0rk"),
        }
    }
    fn copy_bytes(s: &str) -> [u8; 65] {
        let mut buf = [0; 65];
        let bytes = s.as_bytes();
        for i in 0..bytes.len() {
            buf[i] = bytes[i];
        }
        buf
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(usize)]
#[allow(unused)]
#[allow(non_camel_case_types)]
pub enum SysCode {
    SYSCALL_GETCWD    = 17,
    SYSCALL_DUP       = 23,
    SYSCALL_DUP3      = 24,
    SYSCALL_FCNTL     = 25,
    SYSCALL_MKDIRAT   = 34,
    SYSCALL_UNLINKAT  = 35,
    SYSCALL_LINKAT    = 37,
    SYSCALL_UMOUNT2   = 39,
    SYSCALL_MOUNT     = 40,
    SYSCALL_FACCESSAT = 48,
    SYSCALL_CHDIR     = 49,
    SYSCALL_OPENAT    = 56,
    SYSCALL_CLOSE     = 57,
    SYSCALL_PIPE2     = 59,
    SYSCALL_GETDENTS64= 61,
    SYSCALL_LSEEK     = 62,  
    SYSCALL_READ      = 63,
    SYSCALL_WRITE     = 64,
    SYSCALL_READV     = 65,
    SYSCALL_WRITEV    = 66,
    SYSCALL_SENDFILE  = 71,
    SYSCALL_FSTAT     = 80,
    SYSCALL_EXIT      = 93,
    SYSCALL_EXIT_GROUP= 94,
    SYSCALL_SET_TID_ADDRESS = 96,
    SYSCALL_NANOSLEEP = 101,
    SYSCALL_CLOCK_SETTIME = 112,
    SYSCALL_CLOCK_GETTIME = 113,
    SYSCALL_YIELD     = 124,
    SYSCALL_SIGRETURN = 139,
    SYSCALL_TIMES     = 153,
    SYSCALL_SETPGID   = 154,
    SYSCALL_SETSID    = 157,
    SYSCALL_UNAME     = 160,
    SYSCALL_GETTIMEOFDAY  = 169,
    SYSCALL_GETPID    = 172,
    SYSCALL_GETPPID   = 173,
    SYSCALL_SYSINFO   = 179,
    SYSCALL_BRK       = 214,
    SYSCALL_MUNMAP    = 215,
    SYSCALL_CLONE     = 220,
    SYSCALL_EXEC      = 221,
    SYSCALL_MMAP      = 222,
    SYSCALL_WAIT4     = 260,
    GETRANDOM         = 278,
    #[num_enum(default)]
    SYSCALL_UNKNOWN,
}

// 实现Display trait，方便打印
impl Display for SysCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_info())
    }
}

impl SysCode {
    pub fn get_info(&self) -> &'static str{
        match self {
            Self::SYSCALL_FCNTL => "fcntl",
            Self::SYSCALL_WRITEV => "writev",
            Self::SYSCALL_READV => "readv",
            Self::SYSCALL_SYSINFO => "sysinfo",
            Self::SYSCALL_SIGRETURN => "sigreturn",
            Self::SYSCALL_SETPGID => "setpgid",
            Self::SYSCALL_SETSID => "setsid",
            Self::SYSCALL_LSEEK => "lseek",
            Self::SYSCALL_FACCESSAT => "faccessat",
            Self::SYSCALL_SENDFILE => "sendfile",
            Self::SYSCALL_CLOCK_SETTIME => "clock_settime",
            Self::SYSCALL_CLOCK_GETTIME => "clock_gettime",
            Self::SYSCALL_EXIT_GROUP => "exit_group",
            Self::SYSCALL_SET_TID_ADDRESS => "set_tid_address",
            Self::SYSCALL_GETCWD => "getcwd",
            Self::SYSCALL_DUP => "dup",
            Self::SYSCALL_DUP3 => "dup3",
            Self::SYSCALL_MKDIRAT => "mkdirat",
            Self::SYSCALL_UNLINKAT => "unlinkat",
            Self::SYSCALL_LINKAT => "linkat",
            Self::SYSCALL_UMOUNT2 => "umount2",
            Self::SYSCALL_MOUNT => "mount",
            Self::SYSCALL_CHDIR => "chdir",
            Self::SYSCALL_OPENAT => "openat",
            Self::SYSCALL_CLOSE => "close",
            Self::SYSCALL_PIPE2 => "pipe2",
            Self::SYSCALL_GETDENTS64 => "getdents64",
            Self::SYSCALL_READ => "read",
            Self::SYSCALL_WRITE => "write",
            Self::SYSCALL_FSTAT => "fstat",
            Self::SYSCALL_EXIT => "exit",
            Self::SYSCALL_NANOSLEEP => "nanosleep",
            Self::SYSCALL_YIELD => "yield",
            Self::SYSCALL_TIMES => "times",
            Self::SYSCALL_UNAME => "uname",
            Self::SYSCALL_GETTIMEOFDAY => "get_timeofday",
            Self::SYSCALL_GETPID => "getpid",
            Self::SYSCALL_GETPPID => "getppid",
            Self::SYSCALL_BRK => "brk",
            Self::SYSCALL_MUNMAP => "munmap",
            Self::SYSCALL_CLONE => "clone",
            Self::SYSCALL_EXEC => "exec",
            Self::SYSCALL_MMAP => "mmap",
            Self::SYSCALL_WAIT4 => "wait4",
            Self::SYSCALL_UNKNOWN => "unknown",
            Self::GETRANDOM => "getrandom",
        }
    }
}

bitflags! {
    #[derive(Debug,Clone,Copy)]
    pub struct CloneFlags: u32 {
        /// 子进程退出时发送 SIGCHLD 信号（传统 fork() 行为）
        const SIGCHLD = (1 << 4) | (1 << 0);
        /// 共享虚拟内存（线程的典型行为）
        const CLONE_VM = 1 << 8;
        /// 共享文件系统信息（根目录/工作目录等）
        const CLONE_FS = 1 << 9;
        /// 共享打开的文件描述符表
        const CLONE_FILES = 1 << 10;
        /// 共享信号处理函数和阻塞信号掩码
        const CLONE_SIGHAND = 1 << 11;
        /// 在父进程中返回子进程的 pidfd（进程文件描述符）
        const CLONE_PIDFD = 1 << 12;
        /// 允许调试器继续跟踪子进程
        const CLONE_PTRACE = 1 << 13;
        /// 父进程阻塞，直到子进程调用 exec() 或退出（类似 vfork()）
        const CLONE_VFORK = 1 << 14;
        /// 子进程与调用者共享父进程（而非成为调用者的子进程）
        const CLONE_PARENT = 1 << 15;
        /// 将子进程放入同一线程组（实现 POSIX 线程）
        const CLONE_THREAD = 1 << 16;
        /// 创建新的挂载命名空间（Mount Namespace）
        const CLONE_NEWNS = 1 << 17;
        /// 共享 System V 信号量的 SEM_UNDO 状态
        const CLONE_SYSVSEM = 1 << 18;
        /// 为子进程设置新的线程本地存储（TLS）
        const CLONE_SETTLS = 1 << 19;
        /// 将子进程的线程ID（TID）写入父进程的指定地址
        const CLONE_PARENT_SETTID = 1 << 20;
        /// 子进程退出时清除其线程ID（用于线程库同步）
        const CLONE_CHILD_CLEARTID = 1 << 21;
        /// （已废弃）早期标记线程为"分离状态"
        const CLONE_DETACHED = 1 << 22;
        /// 禁止调试进程强制启用 CLONE_PTRACE
        const CLONE_UNTRACED = 1 << 23;
        /// 将子进程的线程ID写入子进程的指定地址
        const CLONE_CHILD_SETTID = 1 << 24;
        /// 创建新的 Cgroup 命名空间
        const CLONE_NEWCGROUP = 1 << 25;
        /// 创建新的 UTS 命名空间（隔离主机名和域名）
        const CLONE_NEWUTS = 1 << 26;
        /// 创建新的 IPC 命名空间（隔离 System V IPC/POSIX 消息队列）
        const CLONE_NEWIPC = 1 << 27;
        /// 创建新的用户命名空间（隔离用户/组 ID）
        const CLONE_NEWUSER = 1 << 28;
        /// 创建新的 PID 命名空间（隔离进程 ID）
        const CLONE_NEWPID = 1 << 29;
        /// 创建新的网络命名空间（隔离网络设备、端口等）
        const CLONE_NEWNET = 1 << 30;
        /// 共享 I/O 上下文（优化块设备 I/O 调度）
        const CLONE_IO = 1 << 31;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// Defined in <bits/waitflags.h>.
    pub struct WaitOptions: i32 {
        /// 如果pid所指向的子进程状态未改变，则立即返回0，不会阻塞
        const WNOHANG = 1;
        /// 除了返回子进程的信息外，还要返回因信号而停止的子进程信息
        const WUNTRACED = 1 << 1;
        /// 返回那些因收到SIGCONT信号而恢复执行并且已经停止的子进程信息
        const WCONTINUED = 1 << 3;
    }
}

/// 允许删除目录（通常与unlinkat等系统调用一起使用）
pub const AT_REMOVEDIR: u32 = 0x200;

/// 跟随符号链接（即操作符号链接指向的目标文件）
pub const AT_SYMLINK_FOLLOW: u32 = 0x400;

// 禁止自动挂载文件系统（当使用 *at 系列函数时，不自动挂载路径中的挂载点）
pub const AT_NO_AUTOMOUNT: u32 = 0x800;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FaccessatFlags: u32 {
        /// 不跟随符号链接（即操作符号链接本身而非其指向的目标）
        const AT_SYMLINK_NOFOLLOW = 0x100;
        const AT_EACCESS = 0x200;
        const AT_EMPTY_PATH = 0x1000;
    }
    pub struct FaccessatMode: u32 {
        /// 检查文件是否存在
        const F_OK = 0;
        /// 检查文件是否可读
        const R_OK = 4;
        /// 检查文件是否可写
        const W_OK = 2;
        /// 检查文件是否可执行
        const X_OK = 1;
    }
}

const _F_SIZE: usize = 20 - 2 * size_of::<u64>() - size_of::<u32>();

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Sysinfo {
    /// 系统启动以来的秒数
    pub uptime: i64,
    /// 1分钟、5分钟和15分钟的系统平均负载
    pub loads: [u64; 3],
    /// 总可用主内存大小（单位：mem_unit字节）
    pub totalram: u64,
    /// 可用内存大小（单位：mem_unit字节）
    pub freeram: u64,
    /// 共享内存大小（单位：mem_unit字节）
    pub sharedram: u64,
    /// 缓冲区使用的内存（单位：mem_unit字节）
    pub bufferram: u64,
    /// 总交换空间大小（单位：mem_unit字节）
    pub totalswap: u64,
    /// 剩余可用交换空间（单位：mem_unit字节）
    pub freeswap: u64,
    /// 当前进程数量
    pub procs: u16,
    /// 为m68k架构显式填充的字段
    pub pad: u16,
    /// 总高端内存大小（单位：mem_unit字节）
    pub totalhigh: u64,
    /// 可用高端内存大小（单位：mem_unit字节）
    pub freehigh: u64,
    /// 内存单位大小（字节）
    pub mem_uint: u32,
    /// 填充字段：libc5曾使用此字段...
    pub _f: [u8; _F_SIZE],
}

impl Sysinfo {
    pub fn new(proc_num: u16) -> Self {
        Self {
            uptime: get_time_s() as i64,
            loads: [0; 3],
            totalram: 0,
            freeram: 0,
            sharedram: 0,
            bufferram: 0,
            totalswap: 0,
            freeswap: 0,
            procs: proc_num,
            pad: 0,
            totalhigh: 0,
            freehigh: 0,
            mem_uint: 0,
            _f: [0; _F_SIZE],
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IoVec {
    /// start address of the buffer
    pub iov_base: usize,
    /// length of the buffer
    pub iov_len: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
#[allow(non_camel_case_types)]
#[repr(isize)]
pub enum FcntlOp {
    F_DUPFD = 0,
    F_DUPFD_CLOEXEC = 1030,
    F_GETFD = 1,
    F_SETFD = 2,
    F_GETFL = 3,
    F_SETFL = 4,
    #[default]
    F_UNIMPL,
}