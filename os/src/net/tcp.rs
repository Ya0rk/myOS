use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use async_trait::async_trait;
use lwext4_rust::bindings::BUFSIZ;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp;
use smoltcp::wire::IpEndpoint;
use crate::fs::FileMeta;
use crate::fs::FileTrait;
use crate::fs::RenameFlags;
use crate::mm::UserBuffer;
use crate::net::addr::Sock;
use crate::net::SOCKET_SET;
use crate::utils::SysResult;
use super::addr::DomainType;
use super::Socket;
use super::TcpState;
use super::SockMeta;
use super::BUFF_SIZE;
use alloc::boxed::Box;

/// TCP 是一种面向连接的字节流套接字
pub struct TcpSocket {
    pub handle: SocketHandle,
    pub sockmeta: SockMeta,
    pub local_end: Option<IpEndpoint>,
    pub remote_end: Option<IpEndpoint>,
    pub state: TcpState,
}

impl TcpSocket {
    pub fn new() -> Self {
        let socket = Self::new_sock();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SockMeta::new(
            Sock::Tcp,
            BUFF_SIZE,
            BUFF_SIZE,
        );

        Self {
            handle,
            sockmeta,
            local_end: None,
            remote_end: None,
            state: TcpState::Established,
        }
    }

    fn new_sock() -> tcp::Socket<'static> {
        let recv_buf = tcp::SocketBuffer::new(
            vec![0; BUFF_SIZE]
        );
        let send_buf = tcp::SocketBuffer::new(
            vec![0; BUFF_SIZE]
        );
        tcp::Socket::new(recv_buf, send_buf)
    }
}

#[async_trait]
impl Socket for TcpSocket {
    async fn accept(&self, _addr: Option<&mut DomainType>) -> SysResult<Arc<dyn Socket>> {
        unimplemented!()
    }
    fn bind(&self, _addr: &DomainType) -> SysResult<()> {
        unimplemented!()
    }
    fn connect(&self, _addr: &DomainType) -> SysResult<()> {
        unimplemented!()
    }
    fn listen(&self, _backlog: usize) -> SysResult<()> {
        unimplemented!()
    }
    fn set_recv_buf_size(&self, _size: usize) -> SysResult<()> {
        unimplemented!()
    }
    fn set_send_buf_size(&self, _size: usize) -> SysResult<()> {
        unimplemented!()
    }
}

#[async_trait]
impl FileTrait for TcpSocket {
    fn get_inode(&self) -> Arc<dyn crate::fs::InodeTrait> {
        unimplemented!()
    }
    fn readable(&self) -> bool {
        unimplemented!()
    }
    fn writable(&self) -> bool {
        unimplemented!()
    }
    fn executable(&self) -> bool {
        false
    }
    async fn read(&self, _buf: UserBuffer) -> SysResult<usize> {
        unimplemented!()
    }
    async fn write(&self, _buf: UserBuffer) -> SysResult<usize> {
        unimplemented!()
    }
    fn get_name(&self) -> SysResult<String> {
        unimplemented!()
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        unimplemented!()
    }
    fn fstat(&self, _stat: &mut crate::fs::Kstat) -> SysResult {
        unimplemented!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<crate::mm::page::Page>> {
        unimplemented!()
    }
}