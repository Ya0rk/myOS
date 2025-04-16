use core::cell::UnsafeCell;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use async_trait::async_trait;
use log::info;
use lwext4_rust::bindings::BUFSIZ;
use smoltcp::iface::SocketHandle;
use smoltcp::socket;
use smoltcp::socket::tcp;
use smoltcp::socket::tcp::ConnectError;
use smoltcp::wire::IpAddress;
use smoltcp::wire::IpEndpoint;
use spin::Spin;
use crate::fs::FileMeta;
use crate::fs::FileTrait;
use crate::fs::OpenFlags;
use crate::fs::RenameFlags;
use crate::mm::UserBuffer;
use crate::net::addr::Sock;
use crate::net::alloc_port;
use crate::net::net_async::TcpAcceptFuture;
use crate::net::PORT_MANAGER;
use crate::net::SOCKET_SET;
use crate::sync::yield_now;
use crate::sync::SpinNoIrqLock;
use crate::syscall::ShutHow;
use crate::utils::Errno;
use crate::utils::SysResult;
use super::addr::IpType;
use super::addr::SockAddr;
use super::NetDev;
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
    pub flags: OpenFlags,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub state: SpinNoIrqLock<TcpState>,
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    pub fn new(iptype: IpType, non_block_flags: Option<OpenFlags>) -> Self {
        let flags = match non_block_flags {
            Some(noblock) => noblock | OpenFlags::O_RDWR,
            None => OpenFlags::O_RDWR,
        };
        let socket = Self::new_sock();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(
            SockMeta::new(
                Sock::Tcp,
                iptype,
                BUFF_SIZE,
                BUFF_SIZE,
        ));
        // TODO(YJJ): maybe bug
        // NET_DEV.lock().poll();
        Self {
            handle,
            flags,
            sockmeta,
            state: SpinNoIrqLock::new(TcpState::Closed),
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

    fn do_connect(&self, remote_point: IpEndpoint) -> SysResult<TcpState> {
        let local_end = self.whit_sockmeta(|sockmeta| {
            sockmeta.remote_end = Some(remote_point);
            sockmeta.local_end.unwrap()
        });

        let res = self.with_socket(|socket| {
            let mut binding = NET_DEV.lock();
            let context = binding.iface.context();
            match socket.connect(
            context,
            local_end,
            remote_point,
            ) {
                Err(ConnectError::InvalidState) => return Err(Errno::EISCONN),
                Err(ConnectError::Unaddressable) => return Err(Errno::EADDRNOTAVAIL),
                _ => return Ok(()),
            }
        });
        match res {
            Err(e) => {
                info!("[do_connect] tcp connect err");
                return Err(e);
            }
            _ => {}
        }

        let stat = self.with_socket(|socket| socket.state());

        Ok(stat)
    }

    fn check_addr(&self, mut endpoint: IpEndpoint) -> SysResult<()> {
        let mut sockmeta = self.sockmeta.lock();
        match sockmeta.domain {
            Sock::Tcp => {
                if endpoint.addr.is_unspecified() {
                    match sockmeta.iptype {
                        IpType::Ipv4 => endpoint.addr = IpAddress::v4(127, 0, 0, 1),
                        IpType::Ipv6 => endpoint.addr = IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1),
                    }
                }

                
            }
            _ => return Err(Errno::EAFNOSUPPORT),
        }
        Ok(())
    }

    pub fn set_state(&self, state: TcpState) {
        *self.state.lock() = state;
    }
}

// 这里是一些闭包函数实现
impl TcpSocket {
    /// 对SOCKET_SET进行加锁，获取socket句柄，然后在回调中使用
    pub fn with_socket<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut tcp::Socket<'_>) -> R,
    {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.handle);
        f(socket)
    }

    fn whit_sockmeta<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut SockMeta) -> R,
    {
        let mut sockmeta = self.sockmeta.lock();
        f(&mut sockmeta)
    }
}

#[async_trait]
impl Socket for TcpSocket {
    fn bind(&self, addr: &SockAddr) -> SysResult<()> {
        let mut local_point = IpEndpoint::try_from(addr.clone()).map_err(|_| Errno::EINVAL)?;
        let mut sockmeta = self.sockmeta.lock();
        if sockmeta.local_end.is_some() {
            info!("[bind] The socket is already bound to an address.");
            return Err(Errno::EINVAL);
        }
        let mut p: Port;
        let mut port_manager = PORT_MANAGER.lock();

        if local_point.port == 0 {
            p = alloc_port(Sock::Tcp)?;
            local_point.port = p.port;
        } else {
            if port_manager.tcp_used_ports.get(local_point.port as usize).unwrap() {
                info!("[bind] The port {} is already in use.", local_point.port);
                return Err(Errno::EADDRINUSE);
            }
            p = Port::new(Sock::Tcp, local_point.port);
            port_manager.tcp_used_ports.set(local_point.port as usize, true);
        }
        
        drop(port_manager);
        sockmeta.local_end = Some(local_point);
        sockmeta.port = Some(p);
        drop(sockmeta);

        info!("[bind] bind to port: {}", local_point.port);
        Ok(())
    }
    fn listen(&self, backlog: usize) -> SysResult<()> {
        info!("[tcp listen] backlog: {}", backlog);
        let sockmeta = self.sockmeta.lock();
        let p = sockmeta.port.clone().ok_or(Errno::EINVAL)?;
        let local_end = sockmeta.local_end.ok_or(Errno::EINVAL)?;

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.handle);
        match socket.listen(local_end) {
            Ok(_) => {
                info!("[tcp listen] Listening on port: {}", p.port);
                self.set_state(TcpState::Listen);
            }
            Err(_) => {
                info!("[tcp listen] Failed to listen on port: {}", p.port);
                return Err(Errno::EINVAL);
            }
        }

        Ok(())
    }
    async fn accept(&self, addr: Option<&mut SockAddr>) -> SysResult<Arc<dyn FileTrait>> {
        if *self.state.lock() != TcpState::Listen {
            return Err(Errno::EINVAL);
        }

        let remote_end = TcpAcceptFuture::new(self).await?;
        


        unimplemented!()
    }
    async fn connect(&self, addr: &SockAddr) -> SysResult<()> {
        let mut remote_endpoint = IpEndpoint::try_from(addr.clone()).map_err(|_| Errno::EINVAL)?;
        self.check_addr(remote_endpoint)?;
        // yield_now().await;

        loop {
            NET_DEV.lock().poll();
            let state = self.do_connect(remote_endpoint)?;
            match state {
                TcpState::Established => {
                    info!("[tcp connect] Connected to: {}", remote_endpoint);
                    break;
                }
                TcpState::SynSent => {
                    info!("[tcp connect] Connection in progress...");
                    yield_now().await;
                }
                TcpState::Closed => {
                    info!("[tcp connect] Connection closed, try again...");
                    self.do_connect(remote_endpoint);
                    yield_now().await;
                }
                _ => {
                    info!("[tcp connect] Waiting for connection...");
                    yield_now().await;
                }
            }
        }

        Ok(())
    }
    fn set_recv_buf_size(&self, size: usize) -> SysResult<()> {
        self.sockmeta.lock().recv_buf_size = size;
        Ok(())
    }
    fn set_send_buf_size(&self, size: usize) -> SysResult<()> {
        self.sockmeta.lock().send_buf_size = size;
        Ok(())
    }
    fn shutdown(&self, how: ShutHow) -> SysResult<()> {
        let mut sockmeta = self.sockmeta.lock();
        if *self.state.lock() == TcpState::Closed {
            return Err(Errno::ENOTCONN);
        }
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.handle);
        let cur_shuthow = sockmeta.shuthow;
        match cur_shuthow {
            Some(now) => sockmeta.shuthow = Some(now | how),
            None => sockmeta.shuthow = Some(how),
        }
        match how {
            ShutHow::SHUT_RD | ShutHow::SHUT_WR => socket.close(),
            ShutHow::SHUT_RDWR => socket.abort(),
            _ => info!("[shutdown] Invalid shutdown type"),
        }
        drop(binding);
        NET_DEV.lock().poll();
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