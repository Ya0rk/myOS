use alloc::{string::String, sync::Arc, vec};
use log::info;
use smoltcp::{iface::SocketHandle, socket::udp::{self, PacketMetadata}, storage::PacketBuffer, wire::IpEndpoint};
use crate::{fs::{FileMeta, RenameFlags}, sync::SpinNoIrqLock, utils::{Errno, SysResult}};
use super::{addr::{Sock, SockAddr}, alloc_port, Port, SockMeta, Socket, BUFF_SIZE, META_SIZE, NET_DEV, PORT_MANAGER, SOCKET_SET};
use alloc::boxed::Box;
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
use async_trait::async_trait;

/// UDP 是一种无连接的报文套接字
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub sockmeta: SockMeta,
    pub inner: SpinNoIrqLock<UdpSockInner>,
}

struct UdpSockInner {
    pub port: Option<Port>,
    pub local_end: Option<IpEndpoint>,
    pub remote_end: Option<IpEndpoint>,
}

impl UdpSocket {
    pub fn new() -> Self {
        let socket = Self::new_socket();
        let handle = SOCKET_SET.lock().add(socket);
        let inner = SpinNoIrqLock::new(UdpSockInner {
            port: None,
            local_end: None,
            remote_end: None,
        });
        // TODO(YJJ): maybe bug
        // NET_DEV.lock().poll();
        Self {
            handle,
            sockmeta: SockMeta::new(
                Sock::Udp,
                BUFF_SIZE,
                BUFF_SIZE,
            ),
            inner
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
    fn bind(&self, addr: &SockAddr) -> SysResult<()> {
        NET_DEV.lock().poll();
        let mut endpoint = IpEndpoint::try_from(addr.clone()).map_err(|_| Errno::EINVAL)?;
        let mut inner = self.inner.lock();
        if inner.port.is_some() {
            return Err(Errno::EADDRINUSE);
        }
        let mut p: Port;
        let mut port_manager = PORT_MANAGER.lock();

        if endpoint.port == 0 {
            p = alloc_port(Sock::Udp)?;
            endpoint.port = p.port;
        } else {
            if port_manager.udp_used_ports.get(endpoint.port as usize).unwrap() {
                return Err(Errno::EADDRINUSE);
            }
            p = Port::new(Sock::Udp, endpoint.port);
            port_manager.udp_used_ports.set(endpoint.port as usize, true);
        }
        drop(port_manager);
        inner.port = Some(p);

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        match socket.bind(endpoint) {
            Ok(_) => {
                info!("[bind] bind to port: {}", endpoint.port);
                inner.local_end = Some(endpoint);
            }
            Err(_) => {
                return Err(Errno::EADDRINUSE);
            }
        }

        Ok(())
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