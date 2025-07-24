use super::{
    addr::{IpType, SockIpv4, SockIpv6, Sock, SockAddr},
    SockMeta, Socket, AF_INET, BUFF_SIZE, META_SIZE, NET_DEV, PORT_MANAGER, SOCKET_SET,
};
use crate::{fs::FileTrait, net::{net_async::UdpRecvFuture, PORT_FD_MANAMER, PORT_START}};
use crate::mm::UserBuffer;
use crate::{
    fs::{FileMeta, OpenFlags, RenameFlags},
    net::{addr::do_addr127, do_port_aloc, net_async::UdpSendFuture, MAX_BUFFER_SIZE},
    sync::{get_waker, yield_now, NullFuture, SpinNoIrqLock, TimeoutFuture},
    syscall::ShutHow,
    task::current_task,
    utils::{Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::{string::String, sync::Arc, vec};
use async_trait::async_trait;
use core::{net::Ipv4Addr, task::Waker, time::Duration};
use log::{info, trace};
use smoltcp::{
    iface::SocketHandle,
    socket::udp::{self, PacketMetadata, UdpMetadata},
    storage::PacketBuffer,
    wire::{IpAddress, IpEndpoint, IpListenEndpoint},
};

/// UDP 是一种无连接的报文套接字
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        info!("[UdpSocket::drop] start");
        NET_DEV.lock().poll();
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        socket.close();
        binding.remove(self.handle);
        drop(binding);

        // 释放端口
        self.with_sockmeta(|sockmeta| {
            sockmeta.port
            .filter(|&port| port > 0)  // 仅处理 port > 0 的情况
            .inspect(|port| info!("[UdpSocket::drop] dealloc port: {}", port))
            .map(|port| PORT_MANAGER.lock().dealloc(sockmeta.domain, port));
        });
        NET_DEV.lock().poll();
    }
}

impl UdpSocket {
    pub fn new(iptype: IpType) -> Self {
        let socket = Self::new_socket();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(SockMeta::new(Sock::Udp, iptype, BUFF_SIZE, BUFF_SIZE, OpenFlags::O_RDWR));
        // TODO(YJJ): maybe bug
        NET_DEV.lock().poll();
        Self {
            handle,
            sockmeta,
        }
    }

    fn new_socket() -> udp::Socket<'static> {
        let recv_buf =
            PacketBuffer::new(vec![PacketMetadata::EMPTY; META_SIZE], vec![0; BUFF_SIZE]);
        let send_buf =
            PacketBuffer::new(vec![PacketMetadata::EMPTY; META_SIZE], vec![0; BUFF_SIZE]);
        udp::Socket::new(recv_buf, send_buf)
    }

    /// 这里不只是要检查地址，还要本地是否绑定local end，没有的话就bind一个
    fn check_addr(&self, sockfd: usize, mut endpoint: IpEndpoint) -> SysResult<()> {
        let mut sockmeta = self.sockmeta.lock();
        match sockmeta.domain {
            Sock::Udp => {
                if endpoint.addr.is_unspecified() {
                    match sockmeta.iptype {
                        IpType::Ipv4 => endpoint.addr = IpAddress::v4(127, 0, 0, 1),
                        IpType::Ipv6 => endpoint.addr = IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1),
                    }
                }
            }
            _ => return Err(Errno::EAFNOSUPPORT),
        }

        if sockmeta.local_end.is_none() {
            match sockmeta.iptype {
                IpType::Ipv4 => {
                    let addr = SockAddr::Inet4(SockIpv4 {
                        family: AF_INET,
                        port: 0,
                        addr: [0, 0, 0, 0],
                        zero: [0u8; 8],
                    });
                    drop(sockmeta);
                    self.bind(sockfd, &addr);
                }
                IpType::Ipv6 => {
                    let addr = SockAddr::Inet6(SockIpv6 {
                        family: AF_INET,
                        port: 0,
                        flowinfo: 0,
                        addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        scope_id: 0,
                    });
                    drop(sockmeta);
                    self.bind(sockfd, &addr);
                }
            };
        }

        Ok(())
    }
}

impl UdpSocket {
    pub fn with_socket<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut udp::Socket<'_>) -> R,
    {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        f(socket)
    }
    
    fn with_sockmeta<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut SockMeta) -> R,
    {
        let mut sockmeta = self.sockmeta.lock();
        f(&mut sockmeta)
    }
}

#[async_trait]
impl Socket for UdpSocket {
    fn bind(&self, sockfd: usize, addr: &SockAddr) -> SysResult<()> {
        info!("[Udp::bind] start, addr = {:?}", addr);

        NET_DEV.lock().poll();

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        let mut endpoint = IpEndpoint::try_from(addr.clone())?;

        let mut localpoint;
        let res = if endpoint.addr.is_unspecified() {
            if endpoint.port == 0 {
                let p = do_port_aloc(&mut endpoint, Sock::Udp)?;
            }
            localpoint = IpListenEndpoint::from(endpoint.port);
            socket.bind(localpoint)
        } else {
            localpoint = IpListenEndpoint::from(endpoint);
            socket.bind(localpoint)
        };

        match res {
            Ok(_) => {
                let task = current_task().unwrap();
                self.sockmeta.lock().local_end = Some(localpoint);
                self.sockmeta.lock().port = Some(endpoint.port);
                PORT_FD_MANAMER.lock().insert(task.get_pid(), endpoint.port, sockfd);

                info!("[Udp::bind] bind to port: {}", endpoint.port);
                Ok(())
            }
            Err(_) => {
                info!("[Udp::bind] bind failed, port may be in use");
                Err(Errno::EINVAL)
            }
        }
    }
    fn listen(&self, _backlog: usize) -> SysResult<()> {
        Err(Errno::EOPNOTSUPP)
    }
    async fn accept(&self, sockfd: usize, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        Err(Errno::EOPNOTSUPP)
    }
    async fn connect(&self, sockfd: usize, addr: &SockAddr) -> SysResult<()> {
        info!("[Udp::connect] start, connect to remote_addr = {:?}", addr);
        /// 与TCP不同，UDP的connect函数不会引发三次握手，而是将目标IP和端口记录下来
        let remote_endpoint = IpEndpoint::try_from(addr.clone())?;
        self.check_addr(sockfd, remote_endpoint);
        self.sockmeta.lock().remote_end = Some(remote_endpoint);
        NET_DEV.lock().poll();
        Ok(())
    }
    async fn send_msg(&self, buf: &[u8], dest_addr: Option<SockAddr>) -> SysResult<usize> {
        // 如果没有远程地址，就先和远程地址建立连接
        // 就算有远程地址，也要覆盖，使用提供的dest_addr
        info!("[Udp::send_msg] start, dest_addr = {:?}", dest_addr);
        let remote_endpoint = match dest_addr{
            Some(addr) => IpEndpoint::try_from(addr)?,
            None => {
                let remote_end = self.sockmeta.lock().remote_end.ok_or(Errno::ENOTCONN)?;
                remote_end
            }
        };

        UdpSendFuture::new(buf, self, remote_endpoint).await

        // let remote_endpoint = match dest_addr{
        //     Some(addr) => IpEndpoint::try_from(addr)?,
        //     None => {
        //         let remote_end = self.sockmeta.lock().remote_end.ok_or(Errno::ENOTCONN)?;
        //         remote_end
        //     }
        // };

        // NET_DEV.lock().poll();
        // let mut sockets = SOCKET_SET.lock();
        // let handle = self.handle;
        // let socket = sockets.get_mut::<udp::Socket>(handle);
        // let flags = self.get_flags()?;
        // // todo:maybe要完善查看bind的端口是否为0，Tanix
        // if socket.can_send() {
        //     info!("[UdpSocket::write] UdpSocket::write: can send");
        //     match socket.send_slice(buf, remote_endpoint) {
        //         Ok(()) => {
        //             info!("[UdpSocket::write] UdpSocket::write: send slice");
        //             drop(sockets);
        //             // yield_now().await;
        //             NET_DEV.lock().poll();
        //             info!(
        //                 "[UdpSocket::write] UdpSocket::write: send slice ok,size:{}",
        //                 buf.len()
        //             );
        //             return Ok(buf.len());
        //         }
        //         Err(_) => {
        //             info!("[UdpSocket::write] UdpSocket::write: send error");
        //             // debug!("udp write: send err");
        //             return Err(Errno::ENOBUFS);
        //         }
        //     }
        // } else {
        //     if flags.contains(OpenFlags::O_NONBLOCK) {
        //         info!("[UdpSendFuture::poll] already set nonblock");
        //         return Err(Errno::EAGAIN);
        //     }
        //     info!("[UdpSocket::write] UdpSocket::write: cant send");
        //     return Err(Errno::ENOBUFS);
        // };

    }
    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)> {
        info!("[Udp::recv_msg] start");
        UdpRecvFuture::new(buf, self).await

        // loop {
        //     NET_DEV.lock().poll();
        //     let mut sockets = SOCKET_SET.lock();
        //     let handle = self.handle;
        //     let socket = sockets.get_mut::<udp::Socket>(handle);
        //     if socket.can_recv() {
        //         info!("[UdpSocket::read] UdpSocket::read can_recv");
        //         if let Ok((size, metadata)) = socket.recv_slice(buf) {
        //             info!("UdpSocket::read recv_slice,size:{}", size);
        //             drop(sockets);
        //             if size > (MAX_BUFFER_SIZE / 2) as usize {
        //                 // need to be slow
        //                 // ksleep(Duration::from_millis(2)).await;
        //             }
        //             match metadata.endpoint.addr {
        //                 IpAddress::Ipv4(addr) => {
        //                     let res = SockAddr::Inet4(SockIpv4::new(metadata.endpoint.port, addr.octets()));
        //                     return Ok((size, res));
        //                 }
        //                 IpAddress::Ipv6(addr) => {
        //                     let res = SockAddr::Inet6(SockIpv6::new(metadata.endpoint.port, addr.octets()));
        //                     return Ok((size, res));
        //                 }
        //             }
        //         }
        //         info!("[UdpSocket::read] UdpSocket::read dont recv_slice");
        //     } else {
        //         let flags = self.get_flags()?;
        //         if flags.contains(OpenFlags::O_NONBLOCK) {
        //             return Err(Errno::EAGAIN);
        //         }
        //     }
        //     drop(sockets);
        //     info!("[UdpSocket::read] UdpSocket::read dont can_recv,entering yiled_now");
        //     yield_now().await;
        // }


    }
    fn set_recv_buf_size(&self, size: u32) -> SysResult<()> {
        self.sockmeta.lock().recv_buf_size = size as usize;
        Ok(())
    }
    fn set_send_buf_size(&self, size: u32) -> SysResult<()> {
        self.sockmeta.lock().send_buf_size = size as usize;
        Ok(())
    }
    fn get_recv_buf_size(&self) -> SysResult<usize> {
        let res = self.sockmeta.lock().recv_buf_size;
        Ok(res)
    }
    fn get_send_buf_size(&self) -> SysResult<usize> {
        let res = self.sockmeta.lock().send_buf_size;
        Ok(res)
    }
    fn shutdown(&self, how: ShutHow) -> SysResult<()> {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        socket.close();
        NET_DEV.lock().poll();
        Ok(())
    }
    fn get_sockname(&self) -> SysResult<SockAddr> {
        let local_end = self
            .sockmeta
            .lock()
            .local_end
            .expect("[udp] get_sockname no local_end");
        let port = local_end.port;
        // let addr = local_end.addr.unwrap();
        let addr = match local_end.addr {
            None => IpAddress::v4(127, 0, 0, 1),
            Some(addr) => addr,
        };
        info!(
            "[udp]get_sockname local end port: {}, addr = {}",
            port, addr
        );

        match addr {
            IpAddress::Ipv4(addr) => {
                let res = SockAddr::Inet4(SockIpv4::new(port, addr.octets()));
                return Ok(res);
            }
            IpAddress::Ipv6(addr) => {
                let res = SockAddr::Inet6(SockIpv6::new(port, addr.octets()));
                return Ok(res);
            }
        }
        Ok(SockAddr::Unspec)
    }
    fn get_peername(&self) -> SysResult<SockAddr> {
        let remote_end = self.sockmeta.lock().remote_end.ok_or(Errno::ENOTCONN)?;
        let port = remote_end.port;
        let addr = remote_end.addr;
        info!(
            "[udp]get_peername local end port: {}, addr = {}",
            port, addr
        );
        match addr {
            IpAddress::Ipv4(addr) => {
                let res = SockAddr::Inet4(SockIpv4::new(port, addr.octets()));
                return Ok(res);
            }
            IpAddress::Ipv6(addr) => {
                let res = SockAddr::Inet6(SockIpv6::new(port, addr.octets()));
                return Ok(res);
            }
        }
        Ok(SockAddr::Unspec)
    }
    fn set_keep_alive(&self, action: u32) -> SysResult<()> {
        todo!()
    }
    fn enable_nagle(&self, action: u32) -> SysResult<()> {
        todo!()
    }
    fn get_socktype(&self) -> SysResult<Sock> {
        Ok(Sock::Udp)
    }
    fn pollin(&self, waker: Waker) -> SysResult<bool> {
        NET_DEV.lock().poll();
        let res = self.with_socket(|socket| {
            if socket.can_recv() {
                info!("[UdpSocket::pollin] have data can recv");
                return Ok(true);
            }
            info!("[UdpSocket::pollin] don't have data, nothing to recv");
            socket.register_recv_waker(&waker);
            return Ok(false);
        });
        res
    }
    fn pollout(&self, waker: Waker) -> SysResult<bool> {
        NET_DEV.lock().poll();
        let res = self.with_socket(|socket| {
            if socket.can_send() {
                info!("[UdpSocket::pollout] have data to send");
                return Ok(true);
            }
            info!("[UdpSocket::pollout] nothing to send");
            socket.register_send_waker(&waker);
            return Ok(false);
        });
        res
    }
    fn get_flags(&self) -> SysResult<OpenFlags> {
        Ok(self.sockmeta.lock().flags)
    }
}