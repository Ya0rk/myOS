use super::{tcp::TcpSocket, udp::UdpSocket, TcpState, NET_DEV};
use crate::{
    fs::OpenFlags,
    net::SOCKET_SET,
    utils::{Errno, SysResult},
};
use core::{future::Future, task::Poll};
use log::info;
use smoltcp::{
    socket::{
        tcp::{self, Socket},
        udp::{self, UdpMetadata},
    },
    wire::IpEndpoint,
};

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
        let ret = self.socket.with_socket(|socket| {
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
                    if self.socket.flags.contains(OpenFlags::O_NONBLOCK) {
                        return Poll::Ready(Err(Errno::EAGAIN));
                    }
                    // 注册waker，当socket状态改变时会重新唤醒任务执行，执行poll，直到返回Ready
                    socket.register_recv_waker(cx.waker());
                    return Poll::Pending;
                }
            }
        });
        NET_DEV.lock().poll();
        ret
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
        let ret = self.tcpsocket.with_socket(|socket| {
            if !socket.is_open() {
                return Poll::Ready(Err(Errno::ENOTCONN));
            }
            if !socket.can_send() {
                if self.tcpsocket.flags.contains(OpenFlags::O_NONBLOCK) {
                    return Poll::Ready(Err(Errno::EAGAIN));
                }
                socket.register_send_waker(cx.waker());
                return Poll::Pending;
            }

            match socket.send_slice(self.msg_buf) {
                Ok(size) => {
                    NET_DEV.lock().poll();
                    return Poll::Ready(Ok(size));
                }
                Err(_) => return Poll::Ready(Err(Errno::ENOBUFS)),
            };
        });
        ret
    }
}

pub struct UdpSendFuture<'a> {
    pub msg_buf: &'a [u8],
    pub udpsocket: &'a UdpSocket,
    pub meta: &'a UdpMetadata,
}

impl<'a> UdpSendFuture<'a> {
    pub fn new(msg_buf: &'a [u8], socket: &'a UdpSocket, meta: &'a UdpMetadata) -> Self {
        Self {
            msg_buf,
            udpsocket: socket,
            meta,
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
        let ret = {
            if !socket.can_send() {
                if self.udpsocket.flags.contains(OpenFlags::O_NONBLOCK) {
                    return Poll::Ready(Err(Errno::EAGAIN));
                }
                socket.register_send_waker(cx.waker());
                return Poll::Pending;
            }
            match socket.send_slice(self.msg_buf, *self.meta) {
                Ok(_) => {
                    drop(binding);
                    NET_DEV.lock().poll();
                    info!("[UdpSendFuture] finish, msg = {:?}", self.msg_buf);
                    return Poll::Ready(Ok(self.msg_buf.len()));
                }
                Err(_) => return Poll::Ready(Err(Errno::ENOBUFS)),
            }
        };

        ret
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
    type Output = SysResult<usize>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        NET_DEV.lock().poll();
        let res = self.tcpsocket.with_socket(|socket| {
            if socket.state() == TcpState::CloseWait || socket.state() == TcpState::TimeWait {
                return Poll::Ready(Ok(0));
            }
            if !socket.may_recv() {
                if self.tcpsocket.flags.contains(OpenFlags::O_NONBLOCK) {
                    return Poll::Ready(Err(Errno::EAGAIN));
                }
                return Poll::Ready(Err(Errno::ENOTCONN));
            }

            match socket.recv_slice(self.msg_buf) {
                Ok(size) => {
                    if size > 0 {
                        return Poll::Ready(Ok(size));
                    }
                    return Poll::Pending;
                }
                Err(tcp::RecvError::Finished) => {
                    return Poll::Ready(Err(Errno::ENOTCONN));
                }
                Err(tcp::RecvError::InvalidState) => {
                    return Poll::Ready(Err(Errno::ENOTCONN));
                }
            }
        });

        res
    }
}
