use alloc::{string::String, sync::Arc, vec};
use smoltcp::{iface::SocketHandle, socket::udp::{self, PacketMetadata}, storage::PacketBuffer};
use crate::{fs::{FileMeta, RenameFlags}, utils::SysResult};
use super::{addr::{SockAddr, Sock}, SockMeta, Socket, BUFF_SIZE, META_SIZE, SOCKET_SET};
use alloc::boxed::Box;
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
use async_trait::async_trait;

/// UDP 是一种无连接的报文套接字
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub sockmeta: SockMeta,
}

impl UdpSocket {
    pub fn new() -> Self {
        let socket = Self::new_socket();
        let handle = SOCKET_SET.lock().add(socket);
        
        Self {
            handle,
            sockmeta: SockMeta::new(
                Sock::Udp,
                BUFF_SIZE,
                BUFF_SIZE,
            ),
        }
    }

    fn new_socket() -> udp::Socket<'static> {
        let recv_buf = PacketBuffer::new(
            vec![PacketMetadata::EMPTY; META_SIZE],
            vec![0; BUFF_SIZE], 
        );
        let send_buf = PacketBuffer::new(
            vec![PacketMetadata::EMPTY; META_SIZE],
            vec![0; BUFF_SIZE], 
        );
        udp::Socket::new(
            recv_buf,
            send_buf,
        )
    }
}

#[async_trait]
impl Socket for UdpSocket {
    async fn accept(&self, _addr: Option<&mut SockAddr>) -> SysResult<Arc<dyn Socket>> {
        unimplemented!()
    }
    fn bind(&self, _addr: &SockAddr) -> SysResult<()> {
        unimplemented!()
    }
    fn connect(&self, _addr: &SockAddr) -> SysResult<()> {
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
impl FileTrait for UdpSocket {
    fn get_socket(self: Arc<Self>) -> Arc<dyn Socket> {
        self
    }
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