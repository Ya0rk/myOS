use super::{AF_INET, AF_INET6, AF_UNIX};
use crate::utils::{Errno, SysResult};
use core::{intrinsics::unlikely, ptr::copy_nonoverlapping};
use log::{info, trace};
use smoltcp::wire::{IpAddress, IpEndpoint};

/// 协议簇类型
#[derive(Debug, Clone, Copy)]
pub enum SockAddr {
    Unspec,
    Unix(SockUnix),
    Inet4(SockIpv4),
    Inet6(SockIpv6),
}

impl SockAddr {
    pub fn write2user(&self, buf: &mut [u8], len: usize) -> SysResult<()> {
        let len = unsafe { *(len as *const u32) } as usize;
        trace!("[write2user] len = {}", len);
        match self {
            SockAddr::Inet4(addr) => {
                if len < core::mem::size_of::<SockIpv4>() {
                    return Err(Errno::EINVAL);
                }
                // 安全地拷贝 Ipv4 结构体到 buf
                unsafe {
                    copy_nonoverlapping(
                        addr as *const SockIpv4 as *const u8,
                        buf.as_mut_ptr(),
                        core::mem::size_of::<SockIpv4>(),
                    );
                }
                return Ok(());
            }
            SockAddr::Inet6(addr) => {
                if len < core::mem::size_of::<SockIpv6>() {
                    return Err(Errno::EINVAL);
                }
                // 安全地拷贝 Ipv6 结构体到 buf
                unsafe {
                    copy_nonoverlapping(
                        addr as *const SockIpv6 as *const u8,
                        buf.as_mut_ptr(),
                        core::mem::size_of::<SockIpv6>(),
                    );
                }
                return Ok(());
            }
            _ => return Err(Errno::EAFNOSUPPORT),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Sock {
    Tcp,
    Udp,
    Unix,
    Unspec,
}

#[derive(Clone, Copy)]
pub enum IpType {
    Ipv4,
    Ipv6,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SockIpv4 {
    /// 地址协议族(AF_INET)
    pub family: u16,
    /// Ipv4 的端口
    pub port: u16,
    /// Ipv4 的地址
    pub addr: [u8; 4],
    /// 零位，用于后续扩展
    pub zero: [u8; 8],
}

impl SockIpv4 {
    pub fn new(port: u16, addr: [u8; 4]) -> Self {
        Self {
            family: AF_INET,
            port,
            addr,
            zero: [0u8; 8],
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SockIpv6 {
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

impl SockIpv6 {
    pub fn new(port: u16, addr: [u8; 16]) -> Self {
        Self {
            family: AF_INET6,
            port,
            flowinfo: 0,
            addr,
            scope_id: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SockUnix {
    /// 地址协议族(AF_UNIX)
    pub family: u16,
    /// 文件系统路径 (以null结尾)
    pub path: [u8; 108],
}

impl SockAddr {
    pub fn from(addr: usize, addrlen: usize) -> Self {
        if unlikely(addr == 0 || addrlen < core::mem::size_of::<u16>()) {
            info!(
                "[sockaddr from] transfer error, addr: {}, addrlen: {}",
                addr, addrlen
            );
            return SockAddr::Unspec;
        }

        let family = unsafe { *(addr as *const u16) };
        match family {
            AF_UNIX => Self::parse_unix(addr, addrlen),
            AF_INET => Self::parse_ipv4(addr, addrlen),
            AF_INET6 => Self::parse_ipv6(addr, addrlen),
            _ => SockAddr::Unspec,
        }
    }

    fn parse_unix(addr: usize, addrlen: usize) -> Self {
        if unlikely(addrlen < core::mem::size_of::<SockUnix>()) {
            info!("[sockaddr from] UNIX socket address too short");
            return SockAddr::Unspec;
        }
        let addr = unsafe { *(addr as *const SockUnix) };
        unsafe { SockAddr::Unix(addr) }
    }

    fn parse_ipv4(addr: usize, addrlen: usize) -> Self {
        if unlikely(addrlen < core::mem::size_of::<SockIpv4>()) {
            info!("[sockaddr from] IPv4 socket address too short");
            return SockAddr::Unspec;
        }
        let addr = unsafe { *(addr as *const SockIpv4) };
        unsafe { SockAddr::Inet4(addr) }
    }

    fn parse_ipv6(addr: usize, addrlen: usize) -> Self {
        if unlikely(addrlen < core::mem::size_of::<SockIpv6>()) {
            info!("[sockaddr from] IPv6 socket address too short");
            return SockAddr::Unspec;
        }
        let addr = unsafe { *(addr as *const SockIpv6) };
        unsafe { SockAddr::Inet6(addr) }
    }
}

impl TryFrom<SockAddr> for IpEndpoint {
    type Error = Errno;

    fn try_from(value: SockAddr) -> Result<Self, Self::Error> {
        match value {
            SockAddr::Inet4(addr) => {
                // 构造大端序的ipv4地址
                let ip = core::net::Ipv4Addr::new(
                    addr.addr[0],
                    addr.addr[1],
                    addr.addr[2],
                    addr.addr[3],
                );
                let port = addr.port;
                Ok(IpEndpoint::new(ip.into(), port))
            }
            SockAddr::Inet6(addr) => {
                // 构造大端序的ipv6地址
                let ip = core::net::Ipv6Addr::new(
                    u16::from_be_bytes([addr.addr[0], addr.addr[1]]),
                    u16::from_be_bytes([addr.addr[2], addr.addr[3]]),
                    u16::from_be_bytes([addr.addr[4], addr.addr[5]]),
                    u16::from_be_bytes([addr.addr[6], addr.addr[7]]),
                    u16::from_be_bytes([addr.addr[8], addr.addr[9]]),
                    u16::from_be_bytes([addr.addr[10], addr.addr[11]]),
                    u16::from_be_bytes([addr.addr[12], addr.addr[13]]),
                    u16::from_be_bytes([addr.addr[14], addr.addr[15]]),
                );
                let port = addr.port;
                Ok(IpEndpoint::new(ip.into(), port))
            }
            SockAddr::Unix(addr) => Err(Errno::EAFNOSUPPORT),
            _ => return Err(Errno::EINVAL),
        }
    }
}

/// 分配地址，这里只涉及到了本地地址
pub fn do_addr127(endpoint: &mut IpEndpoint) {
    if endpoint.addr.is_unspecified() {
        match endpoint.addr {
            IpAddress::Ipv4(_) => {
                info!("[do_addr127] ipv4 -> 127");
                endpoint.addr = IpAddress::v4(127, 0, 0, 1);
            }
            IpAddress::Ipv6(_) => {
                info!("[do_addr127] ipv6 -> 127");
                endpoint.addr = IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1);
            }
        }
    }
}
