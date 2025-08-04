use core::{error, task::Waker};

use super::{
    addr::{IpType, Sock, SockAddr},
    SockMeta, Socket,
};
use crate::fs::FileTrait;
// use crate::mm::UserBuffer;
use crate::{
    fs::{FileMeta, InodeTrait, Kstat, OpenFlags, Page, Pipe, RenameFlags},
    sync::SpinNoIrqLock,
    syscall::ShutHow,
    utils::SysResult,
};
use alloc::boxed::Box;
use alloc::{string::String, sync::Arc};
use async_trait::async_trait;
use log::{info, warn};
use sbi_spec::pmu::hardware_event::STALLED_CYCLES_FRONTEND;
use smoltcp::{iface::SocketHandle, wire::IpEndpoint};

/// UnixSocket 是一种本地通信的字节流套接字
/// 使用管道来本地通信
pub struct UnixSocket {
    pub filemeta: FileMeta,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub read_end: Arc<Pipe>,
    pub write_end: Arc<Pipe>,
}

#[async_trait]
impl Socket for UnixSocket {
    async fn accept(&self, sockfd: usize, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        unimplemented!()
    }
    fn bind(&self, sockfd: usize, _addr: &SockAddr) -> SysResult<()> {
        unimplemented!()
    }
    async fn connect(&self, sockfd: usize, _addr: &SockAddr) -> SysResult<()> {
        unimplemented!()
    }
    fn listen(&self, _backlog: usize) -> SysResult<()> {
        unimplemented!()
    }
    fn set_recv_buf_size(&self, _size: u32) -> SysResult<()> {
        unimplemented!()
    }
    fn set_send_buf_size(&self, _size: u32) -> SysResult<()> {
        unimplemented!()
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
        info!("[unix socket] shutdown: {:?}, not implemented!", how);
        unimplemented!()
    }
    fn get_sockname(&self) -> SysResult<SockAddr> {
        todo!()
    }
    fn get_peername(&self) -> SysResult<SockAddr> {
        todo!()
    }
    async fn send_msg(&self, buf: &[u8], dest_addr: Option<SockAddr>) -> SysResult<usize> {
        todo!()
    }
    async fn recv_msg(&self, buf: &mut [u8]) -> SysResult<(usize, SockAddr)> {
        todo!()
    }
    fn set_keep_alive(&self, action: u32) -> SysResult<()> {
        todo!()
    }
    fn enable_nagle(&self, action: u32) -> SysResult<()> {
        todo!()
    }
    fn get_socktype(&self) -> SysResult<Sock> {
        Ok(Sock::Unix)
    }
    async fn pollin(&self) -> SysResult<bool> {
        warn!("UnixSocket::pollin not implemented");
        todo!()
    }
    async fn pollout(&self) -> SysResult<bool> {
        warn!("UnixSocket::pollout not implemented");
        todo!()
    }
    fn get_flags(&self) -> SysResult<OpenFlags> {
        todo!()
    }
}
