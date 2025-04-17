use alloc::{string::String, sync::Arc};
use log::info;
use smoltcp::{iface::SocketHandle, wire::IpEndpoint};
use crate::{
    fs::{FileMeta, InodeTrait, Kstat, OpenFlags, Page, RenameFlags}, 
    sync::SpinNoIrqLock, syscall::ShutHow, utils::SysResult
};
use super::{addr::{IpType, SockAddr}, SockMeta, Socket};
use alloc::boxed::Box;
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
use async_trait::async_trait;

/// UnixSocket 是一种本地通信的字节流套接字
pub struct UnixSocket {
    pub filemeta: FileMeta,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
}

#[async_trait]
impl Socket for UnixSocket {
    async fn accept(&self) -> SysResult<(IpEndpoint, usize)> {
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
    fn set_recv_buf_size(&self, _size: usize) -> SysResult<()> {
        unimplemented!()
    }
    fn set_send_buf_size(&self, _size: usize) -> SysResult<()> {
        unimplemented!()
    }
    fn shutdown(&self, how: ShutHow) -> SysResult<()> {
        info!("[unix socket] shutdown: {:?}, not implemented!", how);
        unimplemented!()
    }
}

#[async_trait]
impl FileTrait for UnixSocket {
    fn get_socket(self: Arc<Self>) -> Arc<dyn Socket> {
        self
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
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
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        unimplemented!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        unimplemented!()
    }
}