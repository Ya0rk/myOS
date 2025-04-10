use super::{AF_INET6, AF_UNIX};


/// 协议簇类型
pub enum DomainType {
    Unspec = 0,
    Unix = 1,
    Inet4 = 2,
    Inet6 = 10,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ipv4 {
    /// 地址协议族(AF_INET)
    pub family: u16,
    /// Ipv4 的端口
    pub port: u16,
    /// Ipv4 的地址
    pub addr: [u8; 4],
    /// 零位，用于后续扩展
    pub zero: [u8; 8],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Ipv6 {
    /// 地址协议族(AF_INET6)
    pub family: u16,
    /// Ipv6 的端口
    pub port: u16,
    /// Ipv6 的流信息
    pub flowinfo: u32,
    /// Ipv6 的地址
    pub addr: [u8; 16],
    /// IPv6 的范围ID
    pub scope_id: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockUnix {
    /// 地址协议族(AF_UNIX)
    pub family: u16,
    /// 文件系统路径 (以null结尾)
    pub path: [u8; 108],
}

impl From<u16> for DomainType {
    fn from(value: u16) -> Self {
        match value {
            AF_UNIX => DomainType::Unix,
            AF_INET => DomainType::Inet4,
            AF_INET6 => DomainType::Inet6,
            _ => DomainType::Unspec
        }
    }
}