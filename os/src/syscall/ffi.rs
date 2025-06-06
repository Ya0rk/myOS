use core::fmt::{self, Display};
use num_enum::{FromPrimitive, TryFromPrimitive};
use zerocopy::{Immutable, IntoBytes};

use crate::{hal::config::{BLOCK_SIZE, PATH_MAX}, sync::{timer::get_time_s, TimeVal}};

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
            release: Self::copy_bytes("6.1"),
            version: Self::copy_bytes("6.1"),
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
    SYSCALL_IOCTL     = 29,
    SYSCALL_MKDIRAT   = 34,
    SYSCALL_UNLINKAT  = 35,
    SYSCALL_LINKAT    = 37,
    SYSCALL_UMOUNT2   = 39,
    SYSCALL_MOUNT     = 40,
    SYSCALL_STATFS    = 43,
    SYSCALL_FTRUNCATE64 = 46,
    SYSCALL_FALLOCAT  = 47,
    SYSCALL_FACCESSAT = 48,
    SYSCALL_CHDIR     = 49,
    SYSCALL_FCHMODAT  = 53,
    SYSCALL_FCHOWNAT  = 54,
    SYSCALL_OPENAT    = 56,
    SYSCALL_CLOSE     = 57,
    SYSCALL_PIPE2     = 59,
    SYSCALL_GETDENTS64= 61,
    SYSCALL_LSEEK     = 62,  
    SYSCALL_READ      = 63,
    SYSCALL_WRITE     = 64,
    SYSCALL_READV     = 65,
    SYSCALL_WRITEV    = 66,
    SYSCALL_PREAD64   = 67,
    SYSCALL_PWRITE64  = 68,
    SYSCALL_SENDFILE  = 71,
    SYSCALL_PSELECT   = 72,
    SYSCALL_PPOLL     = 73,
    SYSCALL_READLINKAT = 78,
    SYSCALL_FSTATAT   = 79,
    SYSCALL_FSTAT     = 80,
    SYSCALL_SYNC      = 81,
    SYSCALL_FSYNC     = 82,
    SYSCALL_UTIMENSAT = 88,
    SYSCALL_EXIT      = 93,
    SYSCALL_EXIT_GROUP= 94,
    SYSCALL_SET_TID_ADDRESS = 96,
    SYSCALL_FUTEX     = 98,
    SYSCALL_SET_ROBUST_LIST = 99,
    SYSCALL_GET_ROBUST_LIST = 100,
    SYSCALL_NANOSLEEP = 101,
    SYSCALL_GETITIMER  = 102,
    SYSCALL_SETITIMER  = 103,
    SYSCALL_CLOCK_SETTIME = 112,
    SYSCALL_CLOCK_GETTIME = 113,
    SYSCALL_CLOCK_NANOSLEEP = 115,
    SYSCALL_SYSLOG    = 116,
    SYSCALL_SCHED_SETAFFINITY = 122,
    SYSCALL_SCHED_GETAFFINITY = 123,
    SYSCALL_YIELD     = 124,
    SYSCALL_KILL      = 129,
    SYSCALL_TKILL     = 130,
    SYSCALL_TGKILL    = 131,
    SYSCALL_SIGACTION = 134,
    SYSCALL_SIGPROCMASK = 135,
    SYSCALL_SIGTIMEDWAIT = 137,
    SYSCALL_SIGRETURN = 139,
    SYSCALL_TIMES     = 153,
    SYSCALL_SETPGID   = 154,
    SYSCALL_GETPGID   = 155,
    SYSCALL_SETSID    = 157,
    SYSCALL_UNAME     = 160,
    SYSCALL_GETRUSAGE = 165,
    SYSCALL_UMASK     = 166,
    SYSCALL_GETTIMEOFDAY  = 169,
    SYSCALL_GETPID    = 172,
    SYSCALL_GETPPID   = 173,
    SYSCALL_GETUID    = 174,
    SYSCALL_GETEUID   = 175,
    SYSCALL_GETGID    = 176,
    SYSCALL_GETEGID   = 177,
    SYSCALL_GETTID    = 178,
    SYSCALL_SYSINFO   = 179,
    SYSCALL_SHMGET    = 194,
    SYSCALL_SHMCTL    = 195,
    SYSCALL_SHMAT     = 196,
    SYSCALL_SHMDT     = 197,
    SYSCALL_SOCKET    = 198,
    SYSCALL_SOCKETPAIR= 199,
    SYSCALL_BIND      = 200,
    SYSCALL_LISTEN    = 201,
    SYSCALL_ACCEPT    = 202,
    SYSCALL_CONNECT   = 203,
    SYSCALL_GETSOCKNAME = 204,
    SYSCALL_GETPEERNAME = 205,
    SYSCALL_SENDTO    = 206,
    SYSCALL_RECVFROM  = 207,
    SYSCALL_SETSOCKOPT= 208,
    SYSCALL_GETSOCKOPT= 209,
    SYSCALL_SHUTDOWN  = 210,
    SYSCALL_BRK       = 214,
    SYSCALL_MUNMAP    = 215,
    SYSCALL_MREMAP    = 216,
    SYSCALL_CLONE     = 220,
    SYSCALL_EXECVE    = 221,
    SYSCALL_MMAP      = 222,
    SYSCALL_MPROTECT  = 226,
    SYSCALL_MSYNC     = 227,
    SYSCALL_MADVISE   = 233,
    SYSCALL_GET_MEMPOLICY = 236,
    SYSCALL_ACCEPT4   = 242,
    SYSCALL_WAIT4     = 260,
    SYSCALL_RENAMEAT2 = 276,
    SYSCALL_PRLIMIT64 = 261,
    GETRANDOM         = 278,
    MEMEBARRIER       = 283,
    SYS_STATX         = 291,
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
            Self::SYSCALL_UMASK => "umask",
            Self::SYSCALL_FSYNC => "fsync",
            Self::SYSCALL_GET_MEMPOLICY => "get_mempolicy",
            Self::SYSCALL_PSELECT => "pselect",
            Self::SYSCALL_FALLOCAT => "fallocate",
            Self::SYSCALL_MSYNC => "msync",
            Self::SYSCALL_FCHOWNAT => "fchownat",
            Self::SYSCALL_GETGID => "getgid",
            Self::SYSCALL_SCHED_GETAFFINITY => "sched_getaffinity",
            Self::SYSCALL_SCHED_SETAFFINITY => "sched_setaffinity",
            Self::SYSCALL_GETITIMER => "getitimer",
            Self::SYSCALL_SETITIMER => "settimer",
            Self::MEMEBARRIER => "membarrier",
            Self::SYSCALL_GETRUSAGE => "getrusage",
            Self::SYSCALL_MREMAP => "mremap",
            Self::SYSCALL_MADVISE => "madvise",
            Self::SYSCALL_STATFS => "statfs",
            Self::SYSCALL_TKILL => "tkill",
            Self::SYSCALL_SIGTIMEDWAIT => "sigtimedwait",
            Self::SYSCALL_RENAMEAT2 => "renameat2",
            Self::SYSCALL_READLINKAT => "readlinkat",
            Self::SYSCALL_CLOCK_NANOSLEEP => "clock_nanosleep",
            Self::SYSCALL_GETPGID => "getpgid",
            Self::SYSCALL_TGKILL => "tgkill",
            Self::SYSCALL_READLINKAT => "readlinkat",
            Self::SYSCALL_PRLIMIT64 => "prlimit64",
            Self::SYSCALL_GET_ROBUST_LIST => "get_robust_list",
            Self::SYSCALL_SET_ROBUST_LIST => "set_robust_list",
            Self::SYSCALL_FUTEX => "futex",
            Self::SYSCALL_UTIMENSAT => "utimensat",
            Self::SYSCALL_KILL => "kill",
            Self::SYSCALL_SYSLOG => "syslog",
            Self::SYSCALL_IOCTL => "ioctl",
            Self::SYSCALL_PPOLL => "ppoll",
            Self::SYSCALL_SYNC => "sync",
            Self::SYSCALL_GETEGID => "getegid",
            Self::SYSCALL_GETEUID => "geteuid",
            Self::SYSCALL_GETTID => "gettid",
            Self::SYSCALL_SIGACTION => "sigaction",
            Self::SYSCALL_SIGPROCMASK => "sigprocmask",
            Self::SYSCALL_GETUID => "getuid",
            Self::SYSCALL_SETSOCKOPT => "setsockopt",
            Self::SYSCALL_GETSOCKOPT => "getsockopt",
            Self::SYSCALL_SOCKETPAIR => "socketpair",
            Self::SYSCALL_RECVFROM => "recvfrom",
            Self::SYSCALL_SENDTO => "sendto",
            Self::SYSCALL_GETPEERNAME => "getpeername",
            Self::SYSCALL_GETSOCKNAME => "getsockname",
            Self::SYSCALL_ACCEPT4 => "accept4",
            Self::SYSCALL_ACCEPT => "accept",
            Self::SYSCALL_CONNECT => "connect",
            Self::SYSCALL_SHUTDOWN => "shutdown",
            Self::SYSCALL_LISTEN => "listen",
            Self::SYSCALL_BIND => "bind",
            Self::SYSCALL_SOCKET => "socket",
            Self::SYSCALL_FSTATAT => "fstatat",
            Self::SYSCALL_PREAD64 => "pread64",
            Self::SYSCALL_PWRITE64 => "pwrite64",
            Self::SYSCALL_FCHMODAT => "fchmodat",
            Self::SYSCALL_FTRUNCATE64 => "ftruncate64",
            Self::SYSCALL_FCNTL => "fcntl",
            Self::SYSCALL_WRITEV => "writev",
            Self::SYSCALL_READV => "readv",
            Self::SYSCALL_SYSINFO => "sysinfo",
            Self::SYSCALL_SIGRETURN => "sigreturn",
            Self::SYSCALL_SETPGID => "setpgid",
            Self::SYSCALL_GETPGID => "getpgid",
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
            Self::SYSCALL_EXECVE => "execve",
            Self::SYSCALL_MMAP => "mmap",
            Self::SYSCALL_MPROTECT => "mprotect",
            Self::SYSCALL_WAIT4 => "wait4",
            Self::SYSCALL_UNKNOWN => "unknown",
            Self::GETRANDOM => "getrandom",
            Self::SYS_STATX => "statx",
            Self::SYSCALL_SHMGET => "shmget",
            Self::SYSCALL_SHMAT => "shmat",
            Self::SYSCALL_SHMDT => "shmdt",
            Self::SYSCALL_SHMCTL => "shmctl",
        }
    }
}

bitflags! {
    #[derive(Debug,Clone,Copy)]
    pub struct CloneFlags: u32 {
        /// 子进程退出时发送 SIGCHLD 信号（传统 fork() 行为）
        const SIGCHLD = (1 << 4) | (1 << 0);
        /// 共享虚拟内存（线程的典型行为）
        const CLONE_VM = 0x0000100;
        /// 共享文件系统信息（根目录/工作目录等）
        const CLONE_FS = 0x0000200;
        /// 共享打开的文件描述符表
        const CLONE_FILES = 0x0000400;
        /// 共享信号处理函数和阻塞信号掩码
        const CLONE_SIGHAND = 0x00000800;
        /// 在父进程中返回子进程的 pidfd（进程文件描述符）
        const CLONE_PIDFD = 0x00001000;
        /// 允许调试器继续跟踪子进程
        const CLONE_PTRACE = 1 << 13;
        /// 父进程阻塞，直到子进程调用 exec() 或退出（类似 vfork()）
        const CLONE_VFORK = 1 << 14;
        /// 子进程与调用者共享父进程（而非成为调用者的子进程）
        const CLONE_PARENT = 0x00008000;
        /// 将子进程放入同一线程组（实现 POSIX 线程）
        const CLONE_THREAD = 0x00010000;
        /// 创建新的挂载命名空间（Mount Namespace）
        const CLONE_NEWNS = 1 << 17;
        /// 共享 System V 信号量的 SEM_UNDO 状态
        const CLONE_SYSVSEM = 0x00040000;
        /// 为子进程设置新的线程本地存储（TLS）
        const CLONE_SETTLS = 0x00080000;
        /// 将子进程的线程ID（TID）写入父进程的指定地址
        const CLONE_PARENT_SETTID = 0x00100000;
        /// 子进程退出时清除其线程ID（用于线程库同步）
        const CLONE_CHILD_CLEARTID = 0x00200000;
        /// （已废弃）早期标记线程为"分离状态"
        const CLONE_DETACHED = 0x00400000;
        /// 禁止调试进程强制启用 CLONE_PTRACE
        const CLONE_UNTRACED = 0x00800000;
        /// 将子进程的线程ID写入子进程的指定地址
        const CLONE_CHILD_SETTID = 0x01000000;
        /// 创建新的 Cgroup 命名空间
        const CLONE_NEWCGROUP = 0x02000000;
        /// 创建新的 UTS 命名空间（隔离主机名和域名）
        const CLONE_NEWUTS = 0x04000000;
        /// 创建新的 IPC 命名空间（隔离 System V IPC/POSIX 消息队列）
        const CLONE_NEWIPC = 0x08000000;
        /// 创建新的用户命名空间（隔离用户/组 ID）
        const CLONE_NEWUSER = 0x10000000;
        /// 创建新的 PID 命名空间（隔离进程 ID）
        const CLONE_NEWPID = 0x20000000;
        /// 创建新的网络命名空间（隔离网络设备、端口等）
        const CLONE_NEWNET = 0x40000000;
        /// 共享 I/O 上下文（优化块设备 I/O 调度）
        const CLONE_IO = 0x80000000;
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


bitflags! {
    #[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
    pub struct FcntlFlags: u32 {
        /// 复制文件描述符，跟dup()函数功能一样
        const F_DUPFD = 0;
        /// 设置close-on-exec标志
        const F_DUPFD_CLOEXEC = 1030;
        /// 获取文件描述符标志
        const F_GETFD = 1;
        /// 设置文件描述符标志
        const F_SETFD = 2;
        /// 获取文件状态
        const F_GETFL = 3;
        /// 设置文件状态
        const F_SETFL = 4;
    }

    #[derive(PartialEq, Eq, Debug)]
    pub struct FcntlArgFlags: u32 {
        const FD_CLOEXEC = 1;
        const AT_EMPTY_PATH = 1 << 0;
        const AT_SYMLINK_NOFOLLOW = 1 << 8;
        const AT_EACCESS = 1 << 9;
        const AT_NO_AUTOMOUNT = 1 << 11;
        const AT_DUMMY = 1 << 12;
    }

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    #[repr(C)]
    pub struct ShutHow: u8 {
        /// 关闭接收端 (SHUT_RD)
        /// - 后续不能再接收数据
        const SHUT_RD = 0;

        /// 关闭发送端 (SHUT_WR)
        /// - 后续不能再发送数据
        /// - 会发送FIN包给对端
        const SHUT_WR = 1;

        /// 同时关闭收发端 (SHUT_RDWR)
        /// - 完全关闭连接
        /// - 相当于先SHUT_RD再SHUT_WR
        const SHUT_RDWR = 2;
    }
}

pub const SOL_SOCKET: u8 = 1;
pub const SOL_TCP: u8 = 6;

/// 如果协议是TCP，并且当前的套接字状态不是侦听(listen)或关闭(close)，
/// 那么，当option_value不是零时，启用TCP保活定时 器，否则关闭保活定时器。
pub const SO_KEEPALIVE: u32 = 9;// 设置是否保持连接
pub const SO_SNDBUF: u32 = 7;   // 设置发送缓冲区大小
pub const SO_RCVBUF: u32 = 8;   // 设置接收缓冲区大小
pub const MAXSEGMENT: u32 = 2;  // 限制TCP 最大段大小 MSS
pub const CONGESTION: u32 = 13; // 拥塞控制算法
pub const NODELAY: u32 = 1;     // 关闭Nagle算法

/// 主要用于ppoll系统调用
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PollFd {
    pub fd: i32,      // 要监听的文件描述符
    pub events:  PollEvents, // 你关心的事件（输入参数）
    pub revents: PollEvents, // 实际发生的事件（输出参数）
}

impl PollFd {
    pub fn new(fd: i32, events: PollEvents) -> Self {
        Self {
            fd,
            events,
            revents: PollEvents::empty()
        }
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct PollEvents: i16 {
        /// 普通数据可读（例如 TCP 接收缓冲区有数据）
        /// - 对应 `POLLIN`，表示文件描述符有数据可读取且不会阻塞
        const POLLIN = 0x001;

        /// 紧急/带外数据可读（如 TCP 紧急指针指向的数据）
        /// - 对应 `POLLPRI`，用于高优先级数据（如 TCP 带外数据）
        const POLLPRI = 0x002;

        /// 普通数据可写（例如发送缓冲区未满）
        /// - 对应 `POLLOUT`，表示可以写入数据且不会阻塞
        const POLLOUT = 0x004;

        /// 错误条件（隐式事件，无需手动设置）
        /// - 对应 `POLLERR`，当文件描述符发生错误时由内核自动设置
        /// - 例如：套接字连接意外断开、管道写入端关闭后读取等
        const POLLERR = 0x008;

        /// 连接挂断（隐式事件，无需手动设置）
        /// - 对应 `POLLHUP`，表示连接已被对端关闭
        /// - 例如：TCP 对端调用 `close()` 或管道所有写入端关闭
        const POLLHUP = 0x010;

        /// 无效文件描述符（隐式事件，无需手动设置）
        /// - 对应 `POLLNVAL`，当 `fd` 未打开或非法时由内核设置
        /// - 通常表示程序逻辑错误（如重复关闭文件描述符）
        const POLLNVAL = 0x020;

        /// 普通数据可读（XPG4.2 标准定义）
        /// - 对应 `POLLRDNORM`，与 `POLLIN` 行为等价
        /// - 用于兼容性，表示普通优先级数据可读
        const POLLRDNORM = 0x040;

        /// 优先级数据可读（XPG4.2 标准定义）
        /// - 对应 `POLLRDBAND`，用于非紧急的带外数据
        /// - 例如：SCTP 协议中的多流数据
        const POLLRDBAND = 0x080;

        /// 普通数据可写（XPG4.2 标准定义）
        /// - 对应 `POLLWRNORM`，与 `POLLOUT` 行为等价
        /// - 用于兼容性，表示普通优先级数据可写
        const POLLWRNORM = 0x100;

        /// 优先级数据可写（XPG4.2 标准定义）
        /// - 对应 `POLLWRBAND`，表示可写入非紧急的带外数据
        /// - 实际使用较少，常见于特定协议扩展
        const POLLWRBAND = 0x200;
    }
}

pub const LOGINFO: &str = r"YooOs version 0.01-riscv64";

#[repr(i32)]
#[derive(Clone)]
pub enum SyslogCmd {
    LOG_CLOSE = 0,
    LOG_OPEN = 1,
    LOG_READ = 2,
    LOG_READ_ALL = 3,
    LOG_READ_CLEAR = 4,
    LOG_CLEAR = 5,
    LOG_CONSOLE_OFF = 6,
    LOG_CONSOLE_ON = 7,
    LOG_CONSOLE_LEVEL = 8,
    LOG_SIZE_UNREAD = 9,
    LOG_SIZE_BUFFER = 10,
}

impl From<i32> for SyslogCmd {
    fn from(value: i32) -> Self {
        match value {
            0 => SyslogCmd::LOG_CLOSE,
            1 => SyslogCmd::LOG_OPEN,
            2 => SyslogCmd::LOG_READ,
            3 => SyslogCmd::LOG_READ_ALL,
            4 => SyslogCmd::LOG_READ_CLEAR,
            5 => SyslogCmd::LOG_CLEAR,
            6 => SyslogCmd::LOG_CONSOLE_OFF,
            7 => SyslogCmd::LOG_CONSOLE_ON,
            8 => SyslogCmd::LOG_CONSOLE_LEVEL,
            9 => SyslogCmd::LOG_SIZE_UNREAD,
            10 => SyslogCmd::LOG_SIZE_BUFFER,
            _ => panic!("Invalid value for SyslogCmd"),
        }
    }
}

/// 资源限制结构体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RLimit64 {
    pub rlim_cur: usize, // 实际生效的限制，进程可以自由降低或者提高，但是不能超过硬限制
    pub rlim_max: usize, // 硬限制，只有root用户可以修改
}

impl RLimit64 {
    pub fn new_bare() -> Self {
        Self {
            rlim_cur: 0,
            rlim_max: 0
        }
    }
    pub fn new(rlim_cur: usize, rlim_max: usize) -> Self {
        Self {
            rlim_cur,
            rlim_max
        }
    }
}

#[repr(i32)]
#[derive(TryFromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RlimResource {
    /// 最大 CPU 时间（秒），超时触发 `SIGXCPU`
    Cpu = 0,
    /// 最大文件大小（字节），超限触发 `SIGXFSZ`
    Fsize = 1,
    /// 数据段（堆/初始化数据）大小（字节）
    Data = 2,
    /// 进程栈的最大大小（字节）
    Stack = 3,
    /// 核心转储（core dump）文件大小（字节）
    Core = 4,
    /// 驻留集大小（Resident Set Size，已废弃或未严格实现）
    Rss = 5,
    /// 用户可创建的最大进程数
    Nproc = 6,
    /// 文件描述符（File Descriptor）数量上限
    Nofile = 7,
    /// 可锁定在内存中的地址空间大小（字节）
    Memlock = 8,
    /// 进程虚拟内存（地址空间）总大小（字节）
    AddressSpace = 9,
    /// 文件锁数量（Linux 2.4 后废弃）
    Locks = 10,
    /// 待处理（Pending）信号队列的最大信号数
    Sigpending = 11,
    /// POSIX 消息队列的最大字节数
    Msgqueue = 12,
    /// 进程 `nice` 值的可调整上限
    Nice = 13,
    /// 实时优先级（`sched_rt_priority`）的上限
    Rtprio = 14,
    /// 实时任务在不阻塞下的最大 CPU 时间（微秒）
    Rttime = 15,
    // 注：`RLIMIT_NLIMITS`（16）是总数，非实际资源类型
}


#[derive(Default, Debug, Clone, Copy, IntoBytes, Immutable)]
#[repr(C)]
pub struct StatFs {
    /// 是个 magic number，每个知名的 fs 都各有定义，但显然我们没有
    pub f_type: i64,
    /// 最优传输块大小
    pub f_bsize: i64,
    /// 总的块数
    pub f_blocks: u64,
    /// 还剩多少块未分配
    pub f_bfree: u64,
    /// 对用户来说，还有多少块可用
    pub f_bavail: u64,
    /// 总的 inode 数
    pub f_files: u64,
    /// 空闲的 inode 数
    pub f_ffree: u64,
    /// 文件系统编号，但实际上对于不同的OS差异很大，所以不会特地去用
    pub f_fsid: [i32; 2],
    /// 文件名长度限制，这个OS默认FAT已经使用了加长命名
    pub f_namelen: isize,
    /// 片大小
    pub f_frsize: isize,
    /// 一些选项，但其实也没用到
    pub f_flags: isize,
    /// 空余 padding
    pub f_spare: [isize; 4],
}

impl StatFs {
    pub fn new() -> Self {
        Self {
            f_type: 1,
            f_bsize: BLOCK_SIZE as i64,
            f_blocks: 1 << 20,
            f_bfree: 1 << 18,
            f_bavail: 1 << 16,
            f_files: 1 << 10,
            f_ffree: 100,
            f_fsid: [0; 2],
            f_namelen: PATH_MAX as isize,
            f_frsize: 4096,
            f_flags: 1 as isize,
            f_spare: [0; 4],
        }
    }

    pub fn to_u8(&self) -> [u8; core::mem::size_of::<Self>()] {
        let mut buf = [0; core::mem::size_of::<Self>()];
        let bytes = self.as_bytes();
        for i in 0..core::mem::size_of::<Self>() {
            buf[i] = bytes[i];
        }
        buf
    }
}


pub const RUSAGE_SELF:     isize = 0;
pub const RUSAGE_CHILDREN: isize = -1;
pub const RUSAGE_THREAD:   isize = 1;


#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Rusage {
    pub utime: TimeVal,   // 用户态 CPU 时间（进程在用户模式下的总执行时间）
    pub stime: TimeVal,   // 内核态 CPU 时间（进程在内核模式下的总执行时间）
    pub maxrss: usize,    // 最大驻留集大小（进程使用的物理内存峰值，单位通常为 KB）
    pub ixrss: usize,     // 共享内存大小（历史字段，现代系统通常不再使用）
    pub idrss: usize,     // 非共享数据内存大小（历史字段，现代系统通常不再使用）
    pub isrss: usize,     // 非共享栈内存大小（历史字段，现代系统通常不再使用）
    pub minflt: usize,    // 软缺页次数（无需磁盘 I/O 的页故障，如写时复制）
    pub majflt: usize,    // 硬缺页次数（需磁盘 I/O 的页故障，如页面未加载）
    pub nswap: usize,     // 交换次数（进程被换出物理内存的次数）
    pub inblock: usize,   // 块输入操作次数（从存储设备读取数据的次数）
    pub oublock: usize,   // 块输出操作次数（向存储设备写入数据的次数）
    pub msgsnd: usize,    // 发送的消息数（IPC 相关，现代系统通常不再使用）
    pub msgrcv: usize,    // 接收的消息数（IPC 相关，现代系统通常不再使用）
    pub nsignals: usize,  // 接收的信号数量（历史字段，现代系统通常不再使用）
    pub nvcsw: usize,     // 自愿上下文切换次数（进程主动释放 CPU，如等待 I/O）
    pub nivcsw: usize,    // 非自愿上下文切换次数（进程被强制切换，如时间片耗尽）
}

impl Rusage {
    pub fn new(utime: TimeVal, stime: TimeVal) -> Self {
        let mut ret = Rusage::default();
        ret.utime = utime;
        ret.stime = stime;
        ret
    }
}


#[derive(Clone, Copy)]
#[repr(C)]
pub struct CpuSet {
    pub set: [u64; 16], // 128位的位图，表示CPU集
}

impl Default for CpuSet {
    fn default() -> Self {
        Self {
            set: [0u64; 16], // 初始化为全1，表示包含任何CPU
        }
    }
}

pub const CPUSET_LEN: usize = size_of::<CpuSet>();



/*
/* 
 * Control commands used with semctl, msgctl and shmctl 
 * see also specific commands in sem.h, msg.h and shm.h
 */
#define IPC_RMID 0     /* remove resource */
#define IPC_SET  1     /* set ipc_perm options */
#define IPC_STAT 2     /* get ipc_perm options */
#define IPC_INFO 3     /* see ipcs */



/* super user shmctl commands */
#define SHM_LOCK 	11
#define SHM_UNLOCK 	12

/* ipcs ctl commands */
#define SHM_STAT	13
#define SHM_INFO	14
#define SHM_STAT_ANY    15

*/

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(isize)]
#[allow(unused)]
#[allow(non_camel_case_types)]
pub enum ShmOp {
    IPC_RMID = 0,
    IPC_SET = 1,
    IPC_STAT = 2,
    IPC_INFO = 3,
    SHM_LOCK = 11,
    SHM_UNLOCK = 12,
    SHM_STAT = 13,
    SHM_INFO = 14,
    SHM_STAT_ANY = 15,
    #[num_enum(default)]
    INVALID = -1,
}