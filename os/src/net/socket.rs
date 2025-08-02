use core::task::Waker;
use super::{
    addr::{IpType, Sock, SockAddr},
    tcp::TcpSocket,
    udp::UdpSocket,
    unix::UnixSocket,
    SockClass, SocketType, AF_INET, AF_INET6,
};
use crate::{
    fs::{FileMeta, FileTrait, OpenFlags},
    syscall::ShutHow,
    utils::{Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use log::info;
use smoltcp::{socket::tcp, wire::{IpEndpoint, IpListenEndpoint}};
pub type TcpState = tcp::State;

pub struct SockMeta {
    pub domain: Sock,
    pub iptype: IpType,
    pub recv_buf_size: usize,
    pub send_buf_size: usize,
    pub flags: OpenFlags,
    pub port: Option<u16>,
    pub shuthow: Option<ShutHow>,
    pub local_end: Option<IpListenEndpoint>,
    pub remote_end: Option<IpEndpoint>,
}

impl SockMeta {
    pub fn new(domain: Sock, iptype: IpType, recv_buf_size: usize, send_buf_size: usize, flags: OpenFlags) -> Self {
        Self {
            domain,
            iptype,
            recv_buf_size,
            send_buf_size,
            flags,
            port: None,
            shuthow: None,
            local_end: None,
            remote_end: None,
        }
    }
}

#[allow(unused)]
#[async_trait]
pub trait Socket: Send + Sync {
    async fn accept(&self, sockfd: usize, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)>;

    fn bind(&self, sockfd:usize, addr: &SockAddr) -> SysResult<()>;

    async fn connect(&self, sockfd: usize, addr: &SockAddr) -> SysResult<()>;

    fn listen(&self, backlog: usize) -> SysResult<()>;

    async fn send_msg(&self, buf: &[u8], dest_addr: Option<SockAddr>) -> SysResult<usize>;

    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)>;

    fn set_recv_buf_size(&self, size: u32) -> SysResult<()>;

    fn set_send_buf_size(&self, size: u32) -> SysResult<()>;

    fn get_recv_buf_size(&self) -> SysResult<usize>;

    fn get_send_buf_size(&self) -> SysResult<usize>;

    fn shutdown(&self, how: ShutHow) -> SysResult<()>;

    fn get_sockname(&self) -> SysResult<SockAddr>;

    fn get_peername(&self) -> SysResult<SockAddr>;

    fn set_keep_alive(&self, action: u32) -> SysResult<()>;

    fn enable_nagle(&self, action: u32) -> SysResult<()>;

    fn get_socktype(&self) -> SysResult<Sock>;

    async fn pollin(&self) -> SysResult<bool>;

    async fn pollout(&self) -> SysResult<bool>;

    fn get_flags(&self) -> SysResult<OpenFlags>;
}

impl dyn Socket {
    pub fn new(family: u16, socket_type: SocketType) -> SysResult<SockClass> {
        match family {
            AF_INET => Self::new_socket(IpType::Ipv4, socket_type),
            AF_INET6 => Self::new_socket(IpType::Ipv6, socket_type),
            AF_UNIX => return Err(Errno::EAFNOSUPPORT),
            _ => return Err(Errno::EAFNOSUPPORT),
        }
    }

    fn new_socket(ip_type: IpType, socket_type: SocketType) -> SysResult<SockClass> {
        match socket_type {
            ty if ty.contains(SocketType::SOCK_STREAM) => {
                let mut flags = OpenFlags::empty();
                if socket_type.contains(SocketType::SOCK_NONBLOCK) {
                    flags.insert(OpenFlags::O_NONBLOCK);
                }
                if socket_type.contains(SocketType::SOCK_CLOEXEC) {
                    flags.insert(OpenFlags::O_CLOEXEC);
                }
                info!("[new_socket] new Tcp socket, iptype = {:?}", ip_type);
                Ok(SockClass::Tcp(Arc::new(TcpSocket::new(ip_type, flags))))
            }
            ty if ty.contains(SocketType::SOCK_DGRAM) => {
                info!("[new_socket] new Udp socket, iptype = {:?}", ip_type);
                Ok(SockClass::Udp(Arc::new(UdpSocket::new(ip_type))))
            }
            _ => Err(Errno::EINVAL),
        }
    }
}
