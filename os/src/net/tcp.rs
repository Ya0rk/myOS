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
use crate::net::PORT_MANAGER;
use crate::net::SOCKET_SET;
use crate::sync::get_waker;
use crate::sync::yield_now;
use crate::sync::SpinNoIrqLock;
use crate::syscall::ShutHow;
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
use spin::Spin;

/// TCP 是一种面向连接的字节流套接字
pub struct TcpSocket {
    pub handle: SocketHandle,
    pub flags: OpenFlags,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub state: SpinNoIrqLock<TcpState>,
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        info!("[TcpSocket::drop] start");
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
            if let Some(port) = sockmeta.port {
                info!("[TcpSocket::drop] dealloc port: {}", port);
                PORT_MANAGER.lock().dealloc(sockmeta.domain, port);
            }
        });

        NET_DEV.lock().poll();
    }
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    pub fn new(iptype: IpType, flags: OpenFlags) -> Self {
        let socket = Self::new_sock();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(SockMeta::new(Sock::Tcp, iptype, BUFF_SIZE, BUFF_SIZE));
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

            sockmeta.local_end = Some(local_point);
            sockmeta.port = Some(p);
            return Ok(())
        })?;

        trace!("[do_bind] addr: port = {}", local_point);
        info!("[TCP::do_bind] bind to port: {}", local_point.port);
        Ok(())
    }

    fn do_connect(&self, remote_point: IpEndpoint) -> SysResult<TcpState> {
        info!("[do_connect] start");
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

        if sockmeta.local_end.is_none() {
            match sockmeta.iptype {
                IpType::Ipv4 => {
                    let addr = SockAddr::Inet4(SockIpv4 {
                        family: AF_INET,
                        port: 0,
                        addr: [127, 0, 0, 1],
                        zero: [0u8; 8],
                    });
                    drop(sockmeta);
                    info!("[check_addr] call bind");
                    self.bind(&addr);
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
        let res = self.with_socket(|socket| {
            if socket.can_recv() {
                info!("[TcpSocket::pollin] can recv");
                return true;
            }
            // TODO(YJJ):maybe bug
            if socket.state() == TcpState::CloseWait
                || socket.state() == TcpState::FinWait2
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
    fn bind(&self, addr: &SockAddr) -> SysResult<()> {
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
        self.do_bind(local_end)?;
        Ok(())
    }
    fn listen(&self, backlog: usize) -> SysResult<()> {
        info!("[tcp listen] backlog: {}", backlog);
        let sockmeta = self.sockmeta.lock();
        let p = sockmeta.port.clone().ok_or(Errno::EINVAL)?;
        info!("[tcp listen] stage 1");
        let local_end = sockmeta.local_end.ok_or(Errno::EINVAL)?;
        info!("[tcp listen] stage 2");
        let res = self.with_socket(|socket| match socket.listen(local_end) {
            Ok(_) => {
                info!("[tcp listen] Listening on port: {}", p);
                self.set_state(socket.state());
                return Ok(());
            }
            Err(_) => {
                info!("[tcp listen] Failed to listen on port: {}", p);
                return Err(Errno::EINVAL);
            }
        });

        res
    }
    async fn accept(&self, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        info!("[TcpSocket::accept] flags: {:?}", flags);
        if self.check_stat()? != TcpState::Listen {
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
        // newsock.listen(10)?;
        newsock.set_state(TcpState::Established);
        let newsock = Arc::new(newsock);
        let newfd = sock_map_fd(newsock, cloexec_enable).map_err(|_| Errno::EAFNOSUPPORT)?;

        Ok((remote_end, newfd))
    }
    async fn connect(&self, addr: &SockAddr) -> SysResult<()> {
        info!("[Tcp::connect] start, remoteaddr = {:?}", addr);
        let mut remote_endpoint = IpEndpoint::try_from(addr.clone())?;
        self.check_addr(remote_endpoint)?;
        yield_now().await;

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
    async fn send_msg(&self, buf: &[u8], dest_addr: &SockAddr) -> SysResult<usize> {
        info!("[Tcp::send_msg] start, dest_addr = {:?}", dest_addr);
        let res = TcpSendFuture::new(buf, self).await?;
        Ok(res)
    }
    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)> {
        self.set_remote_point();
        let size = TcpRecvFuture::new(buf, self).await?;
        // 返回远端地址
        let remote_end = match self.sockmeta.lock().remote_end {
            Some(remote) => {
                let port = remote.port;
                let addr = remote.addr;
                match addr {
                    IpAddress::Ipv4(addr) => {
                        let res = SockAddr::Inet4(SockIpv4::new(port, addr.octets()));
                        res
                    }
                    IpAddress::Ipv6(addr) => {
                        let res = SockAddr::Inet6(SockIpv6::new(port, addr.octets()));
                        res
                    }
                }
            }
            None => {
                return Err(Errno::ENOTCONN);
            }
        };
        Ok((size, remote_end))
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
        let addr = local_end.addr;
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
        let remote_end = self.sockmeta.lock().remote_end.ok_or(Errno::ENOTCONN)?;
        let port = remote_end.port;
        let addr = remote_end.addr;
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
}

#[async_trait]
impl FileTrait for TcpSocket {
    fn get_socket(self: Arc<Self>) -> SysResult<Arc<dyn Socket>> {
        Ok(self)
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
    async fn read(&self, _buf: &mut [u8]) -> SysResult<usize> {
        unimplemented!()
    }
    async fn write(&self, _buf: &[u8]) -> SysResult<usize> {
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
    fn get_flags(&self) -> OpenFlags {
        self.flags
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<crate::mm::page::Page>> {
        unimplemented!()
    }
    /// TCP Socket 的异步可读性检查方法，
    /// 用于判断当前 Socket 是否有数据可读或处于特定状态（如连接关闭），
    /// 并根据情况注册 Waker 以便在数据到达时唤醒异步任务。
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
        info!("[TcpSocket::pollin] start");
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
}
