use core::{error, task::Waker};

use super::{
    addr::{IpType, Sock, SockAddr},
    SockMeta, Socket,
};
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
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
    async fn accept(&self, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        unimplemented!()
    }
    fn bind(&self, _addr: &SockAddr) -> SysResult<()> {
        unimplemented!()
    }
    async fn connect(&self, _addr: &SockAddr) -> SysResult<()> {
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
    fn pollin(&self, waker: Waker) -> SysResult<bool> {
        warn!("UnixSocket::pollin not implemented");
        todo!()
    }
    fn pollout(&self, waker: Waker) -> SysResult<bool> {
        warn!("UnixSocket::pollout not implemented");
        todo!()
    }
    fn get_flags(&self) -> SysResult<OpenFlags> {
        todo!()
    }
}

// #[async_trait]
// impl FileTrait for UnixSocket {
//     fn get_socket(self: Arc<Self>) -> SysResult<Arc<dyn Socket>> {
//         Ok(self)
//     }
//     fn get_inode(&self) -> Arc<dyn InodeTrait> {
//         unimplemented!()
//     }
//     fn readable(&self) -> bool {
//         unimplemented!()
//     }
//     fn writable(&self) -> bool {
//         unimplemented!()
//     }
//     fn executable(&self) -> bool {
//         false
//     }
//     async fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
//         let res = self.read_end.read(buf).await?;
//         Ok(res)
//     }
//     async fn write(&self, buf: &[u8]) -> SysResult<usize> {
//         let res = self.write_end.write(buf).await?;
//         Ok(res)
//     }
//     fn get_name(&self) -> SysResult<String> {
//         unimplemented!()
//     }
//     fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
//         unimplemented!()
//     }
//     fn fstat(&self, _stat: &mut Kstat) -> SysResult {
//         unimplemented!()
//     }
//     fn is_dir(&self) -> bool {
//         false
//     }
//     async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
//         unimplemented!()
//     }
// }
