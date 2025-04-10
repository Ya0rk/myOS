use alloc::{string::String, sync::Arc};
use smoltcp::iface::SocketHandle;
use crate::{fs::{FileMeta, RenameFlags}, utils::SysResult};
use super::{addr::DomainType, SockMeta, Socket};
use alloc::boxed::Box;
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
use async_trait::async_trait;

/// UDP 是一种无连接的报文套接字
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub filemeta: FileMeta,
    pub sockmeta: SockMeta,
}

#[async_trait]
impl Socket for UdpSocket {
    async fn accept(&self, _addr: Option<&mut DomainType>) -> SysResult<Arc<dyn Socket>> {
        unimplemented!()
    }
    fn bind(&self, _addr: &DomainType) -> SysResult<()> {
        unimplemented!()
    }
    fn connect(&self, _addr: &DomainType) -> SysResult<()> {
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
}

#[async_trait]
impl FileTrait for UdpSocket {
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
    fn fstat(&self, _stat: &mut crate::fs::Kstat) -> SysResult {
        unimplemented!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<crate::mm::page::Page>> {
        unimplemented!()
    }
}