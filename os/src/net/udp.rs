use super::{
    addr::{IpType, Ipv4, Ipv6, Sock, SockAddr},
    SockMeta, Socket, AF_INET, BUFF_SIZE, META_SIZE, NET_DEV, PORT_MANAGER, SOCKET_SET,
};
use crate::fs::FileTrait;
// use crate::mm::UserBuffer;
use crate::{
    fs::{FileMeta, OpenFlags, RenameFlags},
    net::{addr::do_addr127, do_port, net_async::UdpSendFuture, MAX_BUFFER_SIZE},
    sync::{get_waker, yield_now, NullFuture, SpinNoIrqLock, TimeoutFuture},
    syscall::ShutHow,
    task::current_task,
    utils::{Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::{string::String, sync::Arc, vec};
use async_trait::async_trait;
use core::{net::Ipv4Addr, task::Waker, time::Duration};
use log::info;
use smoltcp::{
    iface::SocketHandle,
    socket::udp::{self, PacketMetadata, UdpMetadata},
    storage::PacketBuffer,
    wire::{IpAddress, IpEndpoint},
};

/// UDP 是一种无连接的报文套接字
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub flags: OpenFlags,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
}

impl UdpSocket {
    pub fn new(iptype: IpType) -> Self {
        let socket = Self::new_socket();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(SockMeta::new(Sock::Udp, iptype, BUFF_SIZE, BUFF_SIZE));
        // TODO(YJJ): maybe bug
        NET_DEV.lock().poll();
        Self {
            handle,
            flags: OpenFlags::O_RDWR,
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
    fn check_addr(&self, mut endpoint: IpEndpoint) -> SysResult<()> {
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
                    let addr = SockAddr::Inet4(Ipv4 {
                        family: AF_INET,
                        port: 0,
                        addr: [127, 0, 0, 1],
                        zero: [0u8; 8],
                    });
                    drop(sockmeta);
                    self.bind(&addr);
                }
                IpType::Ipv6 => {
                    let addr = SockAddr::Inet6(Ipv6 {
                        family: AF_INET,
                        port: 0,
                        flowinfo: 0,
                        addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                        scope_id: 0,
                    });
                    drop(sockmeta);
                    self.bind(&addr);
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
}

#[async_trait]
impl Socket for UdpSocket {
    fn bind(&self, addr: &SockAddr) -> SysResult<()> {
        info!("[Udp::bind] start, addr = {:?}", addr);
        let mut endpoint = IpEndpoint::try_from(addr.clone())?;
        let mut sockmeta = self.sockmeta.lock();
        if sockmeta.port.is_some() {
            return Err(Errno::EADDRINUSE);
        }
        let mut p: u16;

        // addr == 0.0.0.0代表本地广播
        do_addr127(&mut endpoint);
        // 分配port
        p = do_port(&mut endpoint, Sock::Udp)?;
        // 记录port
        sockmeta.port = Some(p);

        NET_DEV.lock().poll();

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        match socket.bind(endpoint) {
            Ok(_) => {
                info!("[Udp::bind] bind to port: {}", endpoint.port);
                sockmeta.local_end = Some(endpoint);
            }
            Err(_) => {
                return Err(Errno::EADDRINUSE);
            }
        }

        Ok(())
    }
    fn listen(&self, _backlog: usize) -> SysResult<()> {
        Err(Errno::EOPNOTSUPP)
    }
    async fn accept(&self, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        Err(Errno::EOPNOTSUPP)
    }
    async fn connect(&self, addr: &SockAddr) -> SysResult<()> {
        info!("[Udp::connect] start, addr = {:?}", addr);
        /// 与TCP不同，UDP的connect函数不会引发三次握手，而是将目标IP和端口记录下来
        let remote_endpoint = IpEndpoint::try_from(addr.clone())?;
        self.check_addr(remote_endpoint);
        self.sockmeta.lock().remote_end = Some(remote_endpoint);
        Ok(())
    }
    async fn send_msg(&self, buf: &[u8], dest_addr: &SockAddr) -> SysResult<usize> {
        // 如果没有远程地址，就先和远程地址建立连接
        // 就算有远程地址，也要覆盖，使用提供的dest_addr
        info!("[Udp::send_msg] start, dest_addr = {:?}", dest_addr);
        // self.connect(dest_addr).await;
        let dest = match self.sockmeta.lock().remote_end {
            Some(a) => a,
            None => return Err(Errno::EISCONN),
        };
        let meta = UdpMetadata::from(dest);

        let res = UdpSendFuture::new(buf, self, &meta).await?;
        Ok(res)
    }
    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)> {
        // 完成timer 队列后再来实现，需要控制接收速度
        // NET_DEV.lock().poll();
        // TODO: (YJJ)将这里修改为future形式
        loop {
            NET_DEV.lock().poll(); // 更新队列，保持队列是最新状态
            let res = self.with_socket(|socket| {
                if socket.can_recv() {
                    info!("[Udp::recv_msg] can recv");
                    if let Ok((size, metadata)) = socket.recv_slice(buf) {
                        info!("[Udp::recv_msg] recv_slice, size:{}", size);
                        if size > (MAX_BUFFER_SIZE / 2) as usize {
                            // need to be slow
                            let task = current_task().unwrap();
                            let span = Duration::from_millis(2);
                            let nullfuture = NullFuture::new(task, span);
                            // TimeoutFuture::new(nullfuture, span).await;
                        }

                        let remote = metadata.endpoint;
                        let port = remote.port;
                        let addr = remote.addr;
                        let res = match addr {
                            IpAddress::Ipv4(addr) => {
                                SockAddr::Inet4(Ipv4::new(port, addr.octets()))
                            }
                            IpAddress::Ipv6(addr) => {
                                SockAddr::Inet6(Ipv6::new(port, addr.octets()))
                            }
                        };

                        return Some((size, res));
                    }
                    info!("[Udp::recv_msg] do not recvslice");
                    return None;
                }
                return None;
            });

            info!("[Udp::recv_msg] do not recv");
            match res {
                Some(res) => return Ok(res),
                None => yield_now().await,
            }
        }
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
        let addr = local_end.addr;
        info!(
            "[udp]get_sockname local end port: {}, addr = {}",
            port, addr
        );
        match addr {
            IpAddress::Ipv4(addr) => {
                let res = SockAddr::Inet4(Ipv4::new(port, addr.octets()));
                return Ok(res);
            }
            IpAddress::Ipv6(addr) => {
                let res = SockAddr::Inet6(Ipv6::new(port, addr.octets()));
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
            "[udp]get_sockname local end port: {}, addr = {}",
            port, addr
        );
        match addr {
            IpAddress::Ipv4(addr) => {
                let res = SockAddr::Inet4(Ipv4::new(port, addr.octets()));
                return Ok(res);
            }
            IpAddress::Ipv6(addr) => {
                let res = SockAddr::Inet6(Ipv6::new(port, addr.octets()));
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
}

#[async_trait]
impl FileTrait for UdpSocket {
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
}
