use alloc::sync::Arc;
use async_trait::async_trait;
use crate::{
    fs::{FileTrait, OpenFlags}, syscall::ShutHow, 
    utils::{Errno, SysResult}
};
use alloc::boxed::Box;
use super::{
    addr::{IpType, Sock, SockAddr}, 
    tcp::TcpSocket, udp::UdpSocket, Port, SockClass, SocketType, AF_INET, AF_INET6
};
use smoltcp::{socket::tcp, wire::IpEndpoint};
pub type TcpState = tcp::State;

pub struct SockMeta {
    pub domain: Sock,
    pub iptype: IpType,
    pub recv_buf_size: usize,
    pub send_buf_size: usize,
    pub port: Option<Port>,
    pub shuthow: Option<ShutHow>,
    pub local_end: Option<IpEndpoint>,
    pub remote_end: Option<IpEndpoint>,
}

impl SockMeta {
    pub fn new(domain: Sock, iptype: IpType, recv_buf_size: usize, send_buf_size: usize) -> Self {
        Self {
            domain,
            iptype,
            recv_buf_size,
            send_buf_size,
            port: None,
            shuthow: None,
            local_end: None,
            remote_end: None,
        }
    }
}

#[allow(unused)]
#[async_trait]
pub trait Socket: FileTrait {

    async fn accept(&self, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)>;

    fn bind(&self, addr: &SockAddr) -> SysResult<()>;

    async fn connect(&self, addr: &SockAddr) -> SysResult<()>;

    fn listen(&self, backlog: usize) -> SysResult<()>;

    async fn send_msg(&self, buf: &[u8], dest_addr: &SockAddr) -> SysResult<usize>;

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
}

impl dyn Socket {
    pub fn new(family: u16, socket_type: SocketType) -> SysResult<SockClass> {
        match family {
            AF_INET =>  Self::new_socket(IpType::Ipv4, socket_type),
            AF_INET6 => Self::new_socket(IpType::Ipv6, socket_type),
            AF_UNIX => todo!(),
            _ => return Err(Errno::EAFNOSUPPORT),
        }
    }

    fn new_socket(ip_type: IpType, socket_type: SocketType) -> SysResult<SockClass> {
        match socket_type {
            ty if ty.contains(SocketType::SOCK_STREAM) => {
                let mut non_block_flags = None;
                if socket_type.contains(SocketType::SOCK_NONBLOCK) {
                    non_block_flags = Some(OpenFlags::O_NONBLOCK);
                }
                Ok(SockClass::Tcp(Arc::new(TcpSocket::new(ip_type, non_block_flags))))
            }
            ty if ty.contains(SocketType::SOCK_DGRAM) => {
                Ok(SockClass::Udp(Arc::new(UdpSocket::new(ip_type))))
            }
            _ => Err(Errno::EINVAL),
        }
    }
}