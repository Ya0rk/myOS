use core::cell::UnsafeCell;
use core::task::Waker;
use core::time;
use core::time::Duration;
use super::addr::IpType;
use super::addr::SockAddr;
use super::net_async::TcpSendFuture;
use super::NetDev;
use super::SockMeta;
use super::Socket;
use super::TcpState;
use super::AF_INET;
use super::BUFF_SIZE;
use super::NET_DEV;
use crate::fs::FileMeta;
use crate::fs::FileTrait;
use crate::fs::OpenFlags;
use crate::fs::RenameFlags;
use crate::mm::UserBuffer;
use crate::net::addr::do_addr127;
use crate::net::addr::SockIpv4;
use crate::net::addr::SockIpv6;
use crate::net::addr::Sock;
use crate::net::do_port_aloc;
use crate::net::net_async::TcpAcceptFuture;
use crate::net::net_async::TcpRecvFuture;
use crate::net::PORT_FD_MANAMER;
use crate::net::PORT_MANAGER;
use crate::net::PORT_START;
use crate::net::SOCKET_SET;
use crate::sync::get_waker;
use crate::sync::yield_now;
use crate::sync::SpinNoIrqLock;
use crate::syscall::ShutHow;
use crate::task::exchange_sock_fdinfo;
use crate::task::sock_map_fd;
use crate::utils::Errno;
use crate::utils::SysResult;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use async_trait::async_trait;
use log::info;
use log::trace;
use lwext4_rust::bindings::int_least16_t;
use lwext4_rust::bindings::BUFSIZ;
use smoltcp::iface::SocketHandle;
use smoltcp::socket;
use smoltcp::socket::tcp;
use smoltcp::socket::tcp::ConnectError;
use smoltcp::wire::IpAddress;
use smoltcp::wire::IpEndpoint;
use smoltcp::wire::IpListenEndpoint;
use spin::Spin;

/// TCP 是一种面向连接的字节流套接字
pub struct TcpSocket {
    pub handle: SocketHandle,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub state: SpinNoIrqLock<TcpState>,
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        info!("[TcpSocket::drop] start");
        NET_DEV.lock().poll();
        self.with_socket(|socket| {
            if socket.state() == TcpState::Established {
                socket.close();
            }
        });
        let mut binding = SOCKET_SET.lock();
        binding.remove(self.handle);
        drop(binding);

        // 释放端口
        self.with_sockmeta(|sockmeta| {
            sockmeta.port
            .filter(|&port| port > 0)  // 仅处理 port > 0 的情况
            .inspect(|port| info!("[TcpSocket::drop] dealloc port: {}", port))
            .map(|port| PORT_MANAGER.lock().dealloc(sockmeta.domain, port));
        });

        NET_DEV.lock().poll();
    }
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    pub fn new(iptype: IpType, flags: OpenFlags) -> Self {
        let socket = Self::new_sock();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(SockMeta::new(Sock::Tcp, iptype, BUFF_SIZE, BUFF_SIZE, flags));
        // TODO(YJJ): maybe bug
        NET_DEV.lock().poll();
        Self {
            handle,
            sockmeta,
            state: SpinNoIrqLock::new(TcpState::Closed),
        }
    }

    fn new_sock() -> tcp::Socket<'static> {
        let recv_buf = tcp::SocketBuffer::new(vec![0; BUFF_SIZE]);
        let send_buf = tcp::SocketBuffer::new(vec![0; BUFF_SIZE]);
        tcp::Socket::new(recv_buf, send_buf)
    }

    ///根据参数 local_point 绑定 self->sockmeta->local_end
    pub fn do_bind(&self, mut local_point: IpEndpoint) -> SysResult<()> {
        info!("[do_bind]");
        self.with_sockmeta(|sockmeta| -> SysResult<()> {
            let mut p: u16;

            // addr = 0.0.0.0代表本地
            do_addr127(&mut local_point);
            // 分配port
            // 存在问题? 有的情况下不需要做这个动作?或者说do_port不够满足要求?
            // 因为同一时间可能会有多个socket在使用某个local end?只是remote end不一样而已
            // 特别是在accept中会产生多个同一local end的socket
            p = do_port_aloc(&mut local_point, sockmeta.domain)?;

            let mut listen_point;
            if local_point.addr.is_unspecified() {
                listen_point = IpListenEndpoint::from(local_point.port);
            } else {
                listen_point = IpListenEndpoint::from(local_point);
            }

            sockmeta.local_end = Some(listen_point);
            info!("[do_bind] after bind, local_end = {:?}", sockmeta.local_end);
            sockmeta.port = Some(p);
            return Ok(())
        })?;

        trace!("[do_bind] addr: port = {}", local_point);
        info!("[TCP::do_bind] bind to port: {}", local_point.port);
        Ok(())
    }

    fn do_connect(&self, remote_point: IpEndpoint) -> SysResult<TcpState> {
        info!("[do_connect] start, remote_point = {:?}", remote_point);
        let local_end = self.with_sockmeta(|sockmeta| {
            sockmeta.remote_end = Some(remote_point);
            sockmeta.local_end.unwrap()
        });

        self.with_socket(|socket| -> SysResult<()>{
            let mut binding = NET_DEV.lock();
            let context = binding.iface.context();
            match socket.connect(context, remote_point, local_end) {
                Err(ConnectError::InvalidState) => return Err(Errno::EISCONN),
                Err(ConnectError::Unaddressable) => return Err(Errno::EADDRNOTAVAIL),
                _ => return Ok(()),
            }
        })?;

        let stat = self.with_socket(|socket| socket.state());

        Ok(stat)
    }

    /// 查看socket的状态
    /// 注意到当前实现没有错误处理(不会)
    /// 注意到这里复用了 do_connect 的代码,其实应当将 do_connect 的查看状态功能全放在这里
    /// 形成 connect 函数的工作流
    /// 1. 发出建立连接申请
    /// 2. 不断查看连接状态
    fn check_stat(&self) -> SysResult<TcpState> {
        Ok(self.with_socket(|socket| socket.state()))
    }

    /// 和udp的check addr相同
    fn check_addr(&self, sockfd: usize, mut endpoint: IpEndpoint) -> SysResult<()> {
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
                    info!("[check_addr] call bind");
                    self.bind(sockfd, &addr);
                }
                IpType::Ipv6 => todo!(),
            };
        }

        Ok(())
    }

    pub fn set_state(&self, state: TcpState) {
        *self.state.lock() = state;
    }

    pub fn set_remote_point(&self) {
        self.with_socket(|socket| {
            self.sockmeta.lock().remote_end = socket.remote_endpoint();
        })
    }

    // 判断是否可以从tcp socket中读取数据
    pub fn shoule_return_ready(&self) -> bool {
        let res = self.with_socket(|socket| -> bool {
            if socket.can_recv() {
                info!("[TcpSocket::pollin] can recv");
                return true;
            }
            // TODO(YJJ):maybe bug
            info!("[should_return_ready] last state: {:?}, state: {:?} ", *self.state.lock(), socket.state());
            if socket.state() == TcpState::CloseWait
                || socket.state() == TcpState::FinWait2
                || socket.state() == TcpState::FinWait1
                || socket.state() == TcpState::TimeWait
                || (*self.state.lock() == TcpState::Listen
                    && socket.state() == TcpState::Established)
                || socket.state() == TcpState::SynReceived
            {
                info!(
                    "[TcpSocket::pollin]  can recv, state become {:?}",
                    socket.state()
                );
                return true;
            }

            return false;
        });

        info!("[TcpSocket::shoule_return_ready] res = {}", res);
        res
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

    fn with_sockmeta<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut SockMeta) -> R,
    {
        let mut sockmeta = self.sockmeta.lock();
        f(&mut sockmeta)
    }
}

#[async_trait]
impl Socket for TcpSocket {
    fn bind(&self, sockfd:usize, addr: &SockAddr) -> SysResult<()> {
        info!("[TcpSocket::bind]");
        // 先建立一个local_end,会将服务器绑定到这个地址上
        let mut local_end = IpEndpoint::try_from(addr.clone())?;
        self.with_sockmeta(|sockmeta| -> SysResult<()> {
            if sockmeta.local_end.is_some() {
                info!("[bind] The socket is already bound to an address.");
                return Err(Errno::EINVAL);
            }
            Ok(())
        })?;
        info!("[TcpSocket::bind] before bind, local_end = {:?}", local_end);
        self.do_bind(local_end)?;
        // FD_PORT_MANAMER.lock().insert(local_end.port, sockfd);
        Ok(())
    }
    fn listen(&self, backlog: usize) -> SysResult<()> {
        info!("[tcp listen] backlog: {}", backlog);
        let sockmeta = self.sockmeta.lock();
        info!("[tcp listen] stage 1");
        let local_end = sockmeta.local_end.ok_or(Errno::EINVAL)?;
        info!("[tcp listen] stage 2");
        let res = self.with_socket(|socket| match socket.listen(local_end) {
            Ok(_) => {
                info!("[tcp listen] Listening on {:?}, state = {:?}", socket.listen_endpoint(), socket.state());
                self.set_state(socket.state());
                return Ok(());
            }
            Err(_) => {
                let p = sockmeta.port.clone().ok_or(Errno::EINVAL)?;
                info!("[tcp listen] Failed to listen on port: {}", p);
                return Err(Errno::EINVAL);
            }
        });

        res
    }
    async fn accept(&self, sockfd: usize, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        info!("[TcpSocket::accept] flags: {:?}", flags);
        if self.state.lock().clone() != TcpState::Listen {
            info!("[TcpSocket::accept] Socket is not in listening state");
            return Err(Errno::EINVAL);
        }

        let cloexec_enable = flags.contains(OpenFlags::O_CLOEXEC);
        let remote_end = TcpAcceptFuture::new(self).await?; // 这里的remote end是客户端
        let ip_type = self.sockmeta.lock().iptype;
        let local_end = self
            .sockmeta
            .lock()
            .local_end
            .expect("[tcp accept] no local end");
        let newsock = TcpSocket::new(ip_type, flags);
        {
            newsock.sockmeta.lock().port = Some(local_end.port);
        }
        {
            newsock.sockmeta.lock().local_end = Some(local_end);
        }
        // newsock.do_bind(local_end)?;
        newsock.listen(0)?;
        // newsock.set_state(TcpState::Established);
        let newsock = Arc::new(newsock);
        let newfd = sock_map_fd(newsock, cloexec_enable).map_err(|_| Errno::EAFNOSUPPORT)?;
        // 在accept成功后，oldsock的状态会转化为established，我们后面只能用oldsock来进行数据传输
        // 然后服务器端任然需要一个socket来监听，所以使用newsock来监听
        // 所以这里需要将oldsock对应fd的file和newsock对应的file进行交换
        // 这样，我们返回newfd回去后，用户通过newfd可以访问到oldfile，通过oldfile获取到oldsock
        exchange_sock_fdinfo(sockfd, newfd)?;

        Ok((remote_end, newfd))
    }
    async fn connect(&self, sockfd: usize, addr: &SockAddr) -> SysResult<()> {
        info!("[Tcp::connect] start, remoteaddr = {:?}", addr);
        let mut remote_endpoint = IpEndpoint::try_from(addr.clone())?;
        self.check_addr(sockfd, remote_endpoint)?;
        // yield_now().await;

        let mut state = self.do_connect(remote_endpoint)?;
        loop {
            NET_DEV.lock().poll(); // poll 会修改socket的状态
            state = self.check_stat()?;
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
    async fn send_msg(&self, buf: &[u8], dest_addr: Option<SockAddr>) -> SysResult<usize> {
        info!("[Tcp::send_msg] start, dest_addr = {:?}", dest_addr);
        let res = TcpSendFuture::new(buf, self).await?;
        Ok(res)

        // NET_DEV.lock().poll();
        // let mut binding = SOCKET_SET.lock();
        // let mut socket = binding.get_mut::<tcp::Socket>(self.handle);

        // if socket.can_send() {
        //     info!("[TcpSocket::send_msg] TcpSocket can send");
        //     match socket.send_slice(buf) {
        //         Ok(size) => {
        //             info!("[TcpSocket::send_msg] TcpSocket is send slice,size:{}", size);
        //             drop(binding);
        //             NET_DEV.lock().poll();
        //             yield_now().await;
        //             return Ok(size);
        //         }
        //         Err(e) => {
        //             info!("[TcpSocket::send_msg] Tcp Socket Write Error {e:?}");
        //             println!("aaa");
        //             return Err(Errno::ENOBUFS);
        //         }
        //     }
        // }

        // if self.get_flags()?.contains(OpenFlags::O_NONBLOCK) {
        //     info!("[TcpSendFuture::poll] already set nonblock");
        //     return Err(Errno::EAGAIN);
        // }
        // info!("[TcpSocket::send_msg] TcpSocket can not send, state = {:?}", socket.state());
        // return Err(Errno::ENOBUFS);
    }
    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)> {
        self.set_remote_point();
        TcpRecvFuture::new(buf, self).await

        // self.set_remote_point();
        // loop {
        //     NET_DEV.lock().poll();
        //     let mut sockets = SOCKET_SET.lock();
        //     let socket = sockets.get_mut::<tcp::Socket>(self.handle);
        //     info!("[TcpSocket::read] state become {:?}", socket.state());
        //     if socket.state() == tcp::State::CloseWait || socket.state() == tcp::State::TimeWait
        //     {
        //         return Ok((0, SockAddr::Unspec));
        //     }
        //     // 使用may_recv()检查是否可以接收数据，和recv_slice()接收数据
        //     if socket.may_recv() {
        //         info!("[TcpSocket::read] Tcp Socket Read, may_recv");
        //         match socket.recv_slice(buf) {
        //             Ok(size) => {
        //                 info!("[TcpSocket::read] Tcp Socket recv_slice, size:{}", size);
        //                 if size > 0 {
        //                     // 将远端的地址返回
        //                     let Some(remote_end) = socket.remote_endpoint() else {
        //                         // 如果 remote_endpoint() 返回 None
        //                         return Err(Errno::ENOTCONN);
        //                     };
        //                     info!("[TcpRecvFuture] success recv msg, remote end is {:?}", remote_end);
        //                     return Ok((size, remote_end.into()));
        //                 } else {
        //                     drop(sockets);
        //                     yield_now().await;
        //                 }
        //             }
        //             Err(tcp::RecvError::InvalidState) => {
        //                 return Err(Errno::ENOTCONN);
        //             }
        //             Err(tcp::RecvError::Finished) => {
        //                 return Err(Errno::ENOTCONN);
        //             }
        //         }
        //     } else {
        //         let flags = self.get_flags()?;
        //         if flags.contains(OpenFlags::O_NONBLOCK) {
        //             info!("[TcpRecvFuture::poll] already set nonblock");
        //             return Err(Errno::EAGAIN);
        //         }
        //         info!("[TcpSocket::read] Tcp Socket Read, dont may_recv,retrun error");
        //         drop(sockets);
        //         return Err(Errno::ENOTCONN);
        //     }
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
    fn get_sockname(&self) -> SysResult<SockAddr> {
        let local_end = self
            .sockmeta
            .lock()
            .local_end
            .expect("[tcp] get_sockname no local_end");
        let port = local_end.port;
        // let addr = local_end.addr.unwrap();
        let addr = match local_end.addr {
            None => IpAddress::v4(127, 0, 0, 1),
            Some(addr) => addr,
        };
        info!(
            "[tcp]get_sockname local end port: {}, addr = {}",
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
        // 获取远程连接节点
        NET_DEV.lock().poll();
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.handle);
        let remote_end: SockAddr = socket.remote_endpoint().unwrap().into();
        Ok(remote_end)
        // let remote_end = self.sockmeta.lock().remote_end.ok_or(Errno::ENOTCONN)?;
        // let port = remote_end.port;
        // let addr = remote_end.addr;
        // info!(
        //     "[tcp]get_peername local end port: {}, addr = {}",
        //     port, addr
        // );
        // match addr {
        //     IpAddress::Ipv4(addr) => {
        //         let res = SockAddr::Inet4(SockIpv4::new(port, addr.octets()));
        //         return Ok(res);
        //     }
        //     IpAddress::Ipv6(addr) => {
        //         let res = SockAddr::Inet6(SockIpv6::new(port, addr.octets()));
        //         return Ok(res);
        //     }
        // }
        // Ok(SockAddr::Unspec)
    }
    fn set_keep_alive(&self, action: u32) -> SysResult<()> {
        match action {
            1 => self.with_socket(|socket| {
                let interval = Duration::from_secs(1).into();
                socket.set_keep_alive(Some(interval));
            }),
            _ => {}
        }
        Ok(())
    }
    fn enable_nagle(&self, action: u32) -> SysResult<()> {
        match action {
            1 => {
                self.with_socket(|socket| {
                    // nagle算法可以阻塞小packet的发送
                    socket.set_nagle_enabled(true);
                })
            }
            _ => {}
        }
        Ok(())
    }

    fn get_socktype(&self) -> SysResult<Sock> {
        Ok(Sock::Tcp)
    }

    fn pollin(&self, waker: Waker) -> SysResult<bool> {
        info!("[TcpSocket::pollin] start");
        // 调用底层网络接口轮询机制，处理待处理的网络事件, 确保 Socket 状态和数据缓冲区是最新的
        NET_DEV.lock().poll();
        if self.shoule_return_ready() {
            return Ok(true);
        }
        self.with_socket(|socket| {
            info!(
                "[TcpSocket::pollin] nothing to read, state {:?}",
                socket.state()
            );
            socket.register_recv_waker(&waker);
        });

        Ok(false)
    }
    
    fn pollout(&self, waker: Waker) -> SysResult<bool> {
        info!("[TcpSocket::pollout] start");
        NET_DEV.lock().poll();
        let res = self.with_socket(|socket| {
            if socket.can_send() {
                info!("[TcpSocket::pollout] can send");
                return true;
            }
            socket.register_send_waker(&waker);
            return false;
        });
        Ok(res)
    }

    fn get_flags(&self) -> SysResult<OpenFlags> {
        Ok(self.sockmeta.lock().flags)
    }
}
