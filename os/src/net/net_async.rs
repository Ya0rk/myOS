use core::{future::Future, task::Poll};
use log::info;
use smoltcp::wire::IpEndpoint;
use crate::{fs::OpenFlags, utils::{Errno, SysResult}};
use super::{tcp::TcpSocket, TcpState, NET_DEV};


pub struct TcpAcceptFuture<'a> {
    /// 正在阻塞等待accept的socket
    socket: &'a TcpSocket,
}

impl<'a> TcpAcceptFuture<'a> {
    pub fn new(socket: &'a TcpSocket) -> Self {
        Self {
            socket, 
        }
    }
}

impl<'a> Future for TcpAcceptFuture<'a> {
    type Output = SysResult<IpEndpoint>;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        NET_DEV.lock().poll();
        let ret = self.socket.with_socket(|socket| {
            let cur_state = socket.state();
            match cur_state {
                TcpState::Closed => return Poll::Ready(Err(Errno::EINVAL)),
                TcpState::TimeWait => return Poll::Ready(Err(Errno::EINVAL)),
                TcpState::Established | TcpState::SynReceived => {
                    // 代表已经建立好链接，此时服务器知道了远端链接的地址，可以返回远端
                    self.socket.set_state(cur_state);
                    let remote_end = socket.remote_endpoint().expect("[tcpacceptFuture] poll fail: remote is none.");
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