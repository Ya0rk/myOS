use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use async_trait::async_trait;
use log::info;
use lwext4_rust::bindings::BUFSIZ;
use smoltcp::iface::SocketHandle;
use smoltcp::socket;
use smoltcp::socket::tcp;
use smoltcp::wire::IpEndpoint;
use spin::Spin;
use crate::fs::FileMeta;
use crate::fs::FileTrait;
use crate::fs::RenameFlags;
use crate::mm::UserBuffer;
use crate::net::addr::Sock;
use crate::net::alloc_port;
use crate::net::PORT_MANAGER;
use crate::net::SOCKET_SET;
use crate::sync::SpinNoIrqLock;
use crate::utils::Errno;
use crate::utils::SysResult;
use super::addr::SockAddr;
use super::Port;
use super::Socket;
use super::TcpState;
use super::SockMeta;
use super::BUFF_SIZE;
use super::NET_DEV;
use alloc::boxed::Box;

/// TCP 是一种面向连接的字节流套接字
pub struct TcpSocket {
    pub handle: SocketHandle,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub state: SpinNoIrqLock<TcpState>,
}

impl TcpSocket {
    pub fn new() -> Self {
        let socket = Self::new_sock();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(
            SockMeta::new(
                Sock::Tcp,
                BUFF_SIZE,
                BUFF_SIZE,
        ));
        // TODO(YJJ): maybe bug
        // NET_DEV.lock().poll();
        Self {
            handle,
            sockmeta,
            state: SpinNoIrqLock::new(TcpState::Established),
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
    fn bind(&self, addr: &SockAddr) -> SysResult<()> {
        let mut endpoint = IpEndpoint::try_from(addr.clone()).map_err(|_| Errno::EINVAL)?;
        let mut sockmeta = self.sockmeta.lock();
        if sockmeta.local_end.is_some() {
            info!("[bind] The socket is already bound to an address.");
            return Err(Errno::EINVAL);
        }
        let mut p: Port;
        let mut port_manager = PORT_MANAGER.lock();

        if endpoint.port == 0 {
            p = alloc_port(Sock::Tcp)?;
            endpoint.port = p.port;
        } else {
            if port_manager.tcp_used_ports.get(endpoint.port as usize).unwrap() {
                info!("[bind] The port {} is already in use.", endpoint.port);
                return Err(Errno::EADDRINUSE);
            }
            p = Port::new(Sock::Tcp, endpoint.port);
            port_manager.tcp_used_ports.set(endpoint.port as usize, true);
        }
        
        drop(port_manager);
        sockmeta.local_end = Some(endpoint);
        sockmeta.port = Some(p);
        drop(sockmeta);

        info!("[bind] bind to port: {}", endpoint.port);
        Ok(())
    }
    fn listen(&self, backlog: usize) -> SysResult<()> {
        info!("[tcp listen] backlog: {}", backlog);
        let sockmeta = self.sockmeta.lock();
        let p = sockmeta.port.clone();
        let local_end = sockmeta.local_end;
        if p.is_none() || local_end.is_none() {
            info!("[tcp listen] The socket is not bound to an address.");
            return Err(Errno::EINVAL);
        }

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.handle);
        match socket.listen(local_end.unwrap()) {
            Ok(_) => {
                info!("[tcp listen] Listening on port: {}", p.unwrap().port);
                *self.state.lock() = TcpState::Listen;
            }
            Err(_) => {
                info!("[tcp listen] Failed to listen on port: {}", p.unwrap().port);
                return Err(Errno::EINVAL);
            }
        }

        Ok(())
    }
    async fn accept(&self, _addr: Option<&mut SockAddr>) -> SysResult<Arc<dyn Socket>> {
        unimplemented!()
    }
    fn connect(&self, _addr: &SockAddr) -> SysResult<()> {
        unimplemented!()
    }
    fn set_recv_buf_size(&self, size: usize) -> SysResult<()> {
        self.sockmeta.lock().recv_buf_size = size;
        Ok(())
    }
    fn set_send_buf_size(&self, size: usize) -> SysResult<()> {
        self.sockmeta.lock().send_buf_size = size;
        Ok(())
    }
}

#[async_trait]
impl FileTrait for TcpSocket {
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