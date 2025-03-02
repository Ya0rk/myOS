use core::{fmt::{self, Display}, mem::size_of};
use num_enum::FromPrimitive;

#[allow(unused)]
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
            release: Self::copy_bytes("0.1"),
            version: Self::copy_bytes("0.1"),
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
    pub fn as_bytes(&self) -> &[u8] {
        let size = size_of::<Self>();
        unsafe {core::slice::from_raw_parts(self as *const Self as usize as *const u8, size)}
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
    SYSCALL_MKDIRAT   = 34,
    // SYSCALL_UNLINKAT  = 35,
    // SYSCALL_LINKAT    = 37,
    SYSCALL_UMOUNT2   = 39,
    SYSCALL_MOUNT     = 40,
    SYSCALL_OPENAT    = 56,
    SYSCALL_CLOSE     = 57,
    SYSCALL_PIPE2     = 59,
    SYSCALL_READ      = 63,
    SYSCALL_WRITE     = 64,
    SYSCALL_EXIT      = 93,
    SYSCALL_NANOSLEEP = 101,
    SYSCALL_YIELD     = 124,
    SYSCALL_TIMES     = 153,
    SYSCALL_UNAME     = 160,
    SYSCALL_GETTIMEOFDAY  = 169,
    SYSCALL_GETPID    = 172,
    SYSCALL_GETPPID   = 173,
    SYSCALL_CLONE     = 220,
    SYSCALL_EXEC      = 221,
    SYSCALL_WAIT4     = 260,
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
            Self::SYSCALL_GETCWD => "getcwd",
            Self::SYSCALL_DUP => "dup",
            Self::SYSCALL_DUP3 => "dup3",
            Self::SYSCALL_MKDIRAT => "mkdirat",
            
            Self::SYSCALL_UMOUNT2 => "umount2",
            Self::SYSCALL_MOUNT => "mount",

            Self::SYSCALL_OPENAT => "openat",
            Self::SYSCALL_CLOSE => "close",
            Self::SYSCALL_PIPE2 => "pipe2",
            Self::SYSCALL_READ => "read",
            Self::SYSCALL_WRITE => "write",
            Self::SYSCALL_EXIT => "exit",
            Self::SYSCALL_NANOSLEEP => "nanosleep",
            Self::SYSCALL_YIELD => "yield",
            Self::SYSCALL_TIMES => "times",
            Self::SYSCALL_UNAME => "uname",
            Self::SYSCALL_GETTIMEOFDAY => "get_timeofday",
            Self::SYSCALL_GETPID => "getpid",
            Self::SYSCALL_GETPPID => "getppid",
            Self::SYSCALL_CLONE => "clone",
            Self::SYSCALL_EXEC => "exec",
            Self::SYSCALL_WAIT4 => "wait4",
            Self::SYSCALL_UNKNOWN => "unknown",
        }
    }
}