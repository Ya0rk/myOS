use alloc::sync::Arc;
use async_trait::async_trait;
use crate::{fs::FileTrait, utils::{Errno, SysResult}};
use alloc::boxed::Box;
use super::{addr::{DomainType, Sock}, tcp::TcpSocket, udp::UdpSocket, SockClass, SocketType};
use smoltcp::socket::tcp;
pub type TcpState = tcp::State;

pub struct SockMeta {
    pub domain: Sock,
    pub recv_buf_size: usize,
    pub send_buf_size: usize,
}

impl SockMeta {
    pub fn new(domain: Sock, recv_buf_size: usize, send_buf_size: usize) -> Self {
        Self {
            domain,
            recv_buf_size,
            send_buf_size,
        }
    }
}

#[allow(unused)]
#[async_trait]
pub trait Socket: FileTrait {

    async fn accept(&self, addr: Option<&mut DomainType>) -> SysResult<Arc<dyn Socket>>;

    fn bind(&self, addr: &DomainType) -> SysResult<()>;

    fn connect(&self, addr: &DomainType) -> SysResult<()>;

    fn listen(&self, backlog: usize) -> SysResult<()>;

    fn set_recv_buf_size(&self, size: usize) -> SysResult<()>;

    fn set_send_buf_size(&self, size: usize) -> SysResult<()>;
}

impl dyn Socket {
    pub fn new(family: DomainType, type_: SocketType) -> SysResult<SockClass> {
        match family {
            DomainType::Inet4 | DomainType::Inet6 => {
                if type_.contains(SocketType::SOCK_STREAM) {
                    // 创建 TCP 套接字
                    let sockclass = SockClass::Tcp(Arc::new(TcpSocket::new()));
                    Ok(sockclass)
                } else if type_.contains(SocketType::SOCK_DGRAM) {
                    // 创建 UDP 套接字
                    let sockclass = SockClass::Udp(Arc::new(UdpSocket::new()));
                    Ok(sockclass)
                } else {
                    return Err(Errno::EINVAL);
                }
            }
            DomainType::Unix => {
                // 创建 Unix 套接字
                todo!()
            }
            DomainType::Unspec => {
                return Err(Errno::EAFNOSUPPORT);
            }
        }
    }
}