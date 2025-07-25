use super::{tcp::TcpSocket, udp::UdpSocket, TcpState, NET_DEV};
use crate::{
    console::print, fs::OpenFlags, net::{addr::SockAddr, Socket, MAX_BUFFER_SIZE, SOCKET_SET}, sync::yield_now, utils::{Errno, SysResult}
};
use core::{future::Future, task::Poll};
use log::info;
use smoltcp::{
    socket::{
        tcp::{self},
        udp::{self, UdpMetadata},
    }, wire::IpEndpoint}
;

pub struct TcpAcceptFuture<'a> {
    /// 正在阻塞等待accept的socket
    socket: &'a TcpSocket,
}

impl<'a> TcpAcceptFuture<'a> {
    pub fn new(socket: &'a TcpSocket) -> Self {
        Self { socket }
    }
}

impl<'a> Future for TcpAcceptFuture<'a> {
    type Output = SysResult<IpEndpoint>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        NET_DEV.lock().poll();

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.socket.handle);
        let cur_state = socket.state();
        match cur_state {
            TcpState::Closed => return Poll::Ready(Err(Errno::EINVAL)),
            TcpState::TimeWait => return Poll::Ready(Err(Errno::EINVAL)),
            TcpState::Established | TcpState::SynReceived => {
                // 代表已经建立好链接，此时服务器知道了远端链接的地址，可以返回远端
                self.socket.set_state(cur_state);
                let remote_end = socket
                    .remote_endpoint()
                    .expect("[tcpacceptFuture] poll fail: remote is none.");
                return Poll::Ready(Ok(remote_end));
            }
            _ => {
                // The socket is marked nonblocking and no connections are present to be accepted.
                if self.socket.get_flags()?.contains(OpenFlags::O_NONBLOCK) {
                    return Poll::Ready(Err(Errno::EAGAIN));
                }
                // 注册waker，当socket状态改变时会重新唤醒任务执行，执行poll，直到返回Ready
                socket.register_recv_waker(cx.waker());
                drop(binding);
                NET_DEV.lock().poll();
                cx.waker().clone().wake();

                return Poll::Pending;
            }
        }
    }
}

pub struct TcpSendFuture<'a> {
    pub msg_buf: &'a [u8],
    pub tcpsocket: &'a TcpSocket,
}

impl<'a> TcpSendFuture<'a> {
    pub fn new(msg_buf: &'a [u8], socket: &'a TcpSocket) -> Self {
        Self {
            msg_buf,
            tcpsocket: socket,
        }
    }
}

impl<'a> Future for TcpSendFuture<'a> {
    type Output = SysResult<usize>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        info!("[TcpSendFuture] start");
        NET_DEV.lock().poll();
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.tcpsocket.handle);

        if !socket.is_open() {
            info!("[TcpSendFuture] socket state {:?}", socket.state());
            return Poll::Ready(Err(Errno::ENOTCONN));
        }
        if !socket.can_send() {
            if self.tcpsocket.get_flags()?.contains(OpenFlags::O_NONBLOCK) {
                return Poll::Ready(Err(Errno::EAGAIN));
            }
            socket.register_send_waker(cx.waker());
            drop(binding);
            NET_DEV.lock().poll();
            cx.waker().clone().wake();

            return Poll::Pending;
        }

        info!("[TcpSendFuture] can send");
        match socket.send_slice(self.msg_buf) {
            Ok(size) => {
                return Poll::Ready(Ok(size));
            }
            Err(_) => return Poll::Ready(Err(Errno::ENOBUFS)),
        };
        
    }
}

pub struct UdpSendFuture<'a> {
    pub msg_buf: &'a [u8],
    pub udpsocket: &'a UdpSocket,
    pub remote_end: IpEndpoint,
}

impl<'a> UdpSendFuture<'a> {
    pub fn new(msg_buf: &'a [u8], socket: &'a UdpSocket, remote_end: IpEndpoint) -> Self {
        Self {
            msg_buf,
            udpsocket: socket,
            remote_end,
        }
    }
}

impl<'a> Future for UdpSendFuture<'a> {
    type Output = SysResult<usize>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        NET_DEV.lock().poll();
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.udpsocket.handle);

        if !socket.can_send() {
            if self.udpsocket.get_flags()?.contains(OpenFlags::O_NONBLOCK) {
                return Poll::Ready(Err(Errno::EAGAIN));
            }
            // socket.register_send_waker(cx.waker());
            info!("[UdpSendFuture] socket can't send");
            return Poll::Ready(Err(Errno::ENOBUFS));
            // return Poll::Pending;
        }
        match socket.send_slice(self.msg_buf, self.remote_end) {
            Ok(_) => {
                drop(binding);
                info!("[UdpSendFuture] finish, msg = {:?}", self.msg_buf);
                NET_DEV.lock().poll();
                return Poll::Ready(Ok(self.msg_buf.len()));
            }
            Err(_) => return Poll::Ready(Err(Errno::ENOBUFS)),
        }
    }
}

pub struct TcpRecvFuture<'a> {
    pub msg_buf: &'a mut [u8],
    pub tcpsocket: &'a TcpSocket,
}

impl<'a> TcpRecvFuture<'a> {
    pub fn new(msg_buf: &'a mut [u8], tcpsocket: &'a TcpSocket) -> Self {
        Self { msg_buf, tcpsocket }
    }
}

impl<'a> Future for TcpRecvFuture<'a> {
    type Output = SysResult<(usize, SockAddr)>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        NET_DEV.lock().poll();
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<tcp::Socket>(self.tcpsocket.handle);
        
        if socket.state() == TcpState::CloseWait || socket.state() == TcpState::TimeWait {
            return Poll::Ready(Ok((0, SockAddr::Unspec)));
        }
        
        if !socket.may_recv() {
            if self.tcpsocket.get_flags()?.contains(OpenFlags::O_NONBLOCK) {
                return Poll::Ready(Err(Errno::EAGAIN));
            }
            return Poll::Ready(Err(Errno::ENOTCONN));
        }

        match socket.recv_slice(self.msg_buf) {
            Ok(size) => {
                if size > 0 {
                    let Some(remote_end) = socket.remote_endpoint() else {
                        // 如果 remote_endpoint() 返回 None，则执行这里的代码
                        return Poll::Ready(Err(Errno::ENOTCONN));
                    };
                    info!("[TcpRecvFuture] success recv msg, remote end is {:?}", remote_end);

                    NET_DEV.lock().poll();
                    return Poll::Ready(Ok((size, remote_end.into())));
                }
                socket.register_recv_waker(cx.waker());
                drop(binding);
                NET_DEV.lock().poll();
                cx.waker().clone().wake();

                return Poll::Pending;
            }
            Err(tcp::RecvError::Finished) => {
                return Poll::Ready(Err(Errno::ENOTCONN));
            }
            Err(tcp::RecvError::InvalidState) => {
                return Poll::Ready(Err(Errno::ENOTCONN));
            }
        }

    }
}

pub struct UdpRecvFuture<'a> {
    pub msg_buf: &'a mut [u8],
    pub udpsocket: &'a UdpSocket,
}

impl<'a> UdpRecvFuture<'a> {
    pub fn new(msg_buf: &'a mut [u8], udpsocket: &'a UdpSocket) -> Self {
        Self {
            msg_buf,
            udpsocket
        }
    }
}

impl<'a> Future for UdpRecvFuture<'a> {
    type Output = SysResult<(usize, SockAddr)>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>
    )-> Poll<Self::Output> {
        NET_DEV.lock().poll();

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.udpsocket.handle);

        if !socket.can_recv() {
            let flags = self.udpsocket.get_flags()?;
            if flags.contains(OpenFlags::O_NONBLOCK) {
                info!("[UdpRecvFuture] can't recv, flag = {:?}", flags);
                return Poll::Ready(Err(Errno::EAGAIN));
            }
            socket.register_recv_waker(cx.waker());
            drop(binding);
            NET_DEV.lock().poll();
            cx.waker().clone().wake();
            return Poll::Pending;
        }

        if let Ok((size, metadata)) = socket.recv_slice(self.msg_buf) {
            if size > (MAX_BUFFER_SIZE / 2) as usize {
                // need to impl null sleep
            }
            drop(binding);
            NET_DEV.lock().poll();
            return Poll::Ready(Ok((size, metadata.endpoint.into())));
        } else {
            return Poll::Ready(Err(Errno::ENOTCONN));
        }

    }
}