use core::fmt::{self, Display};
use num_enum::FromPrimitive;

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[repr(usize)]
#[allow(unused)]
#[allow(non_camel_case_types)]
pub enum SysCode {
    SYSCALL_GETCWD    = 17,
    SYSCALL_DUP       = 23,
    SYSCALL_DUP3      = 24,
    SYSCALL_OPENAT    = 56,
    SYSCALL_CLOSE     = 57,
    SYSCALL_READ      = 63,
    SYSCALL_WRITE     = 64,
    SYSCALL_EXIT      = 93,
    SYSCALL_YIELD     = 124,
    SYSCALL_GET_TIME  = 169,
    SYSCALL_GETPID    = 172,
    SYSCALL_FORK      = 220,
    SYSCALL_EXEC      = 221,
    SYSCALL_WAITPID   = 260,
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
            Self::SYSCALL_OPENAT => "openat",
            Self::SYSCALL_CLOSE => "close",
            Self::SYSCALL_READ => "read",
            Self::SYSCALL_WRITE => "write",
            Self::SYSCALL_EXIT => "exit",
            Self::SYSCALL_YIELD => "yield",
            Self::SYSCALL_GET_TIME => "get_time",
            Self::SYSCALL_GETPID => "getpid",
            Self::SYSCALL_FORK => "fork",
            Self::SYSCALL_EXEC => "exec",
            Self::SYSCALL_WAITPID => "waitpid",
            Self::SYSCALL_UNKNOWN => "unknown",
        }
    }
}