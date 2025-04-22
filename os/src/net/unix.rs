use alloc::{string::String, sync::Arc};
use log::info;
use smoltcp::{iface::SocketHandle, wire::IpEndpoint};
use crate::{
    fs::{FileMeta, InodeTrait, Kstat, OpenFlags, Page, Pipe, RenameFlags}, 
    sync::SpinNoIrqLock, syscall::ShutHow, utils::SysResult
};
use super::{addr::{IpType, SockAddr}, SockMeta, Socket};
use alloc::boxed::Box;
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
use async_trait::async_trait;

/// UnixSocket 是一种本地通信的字节流套接字
/// 使用管道来本地通信
pub struct UnixSocket {
    pub filemeta: FileMeta,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
    pub read_end: Arc<Pipe>,
    pub write_end: Arc<Pipe>
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
    async fn send_msg(&self, buf: &[u8], dest_addr: &SockAddr) -> SysResult<usize> {
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
    async fn read(&self, buf: UserBuffer) -> SysResult<usize> {
        let res = self.read_end.read(buf).await?;
        Ok(res)
    }
    async fn write(&self, buf: UserBuffer) -> SysResult<usize> {
        let res = self.write_end.write(buf).await?;
        Ok(res)
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