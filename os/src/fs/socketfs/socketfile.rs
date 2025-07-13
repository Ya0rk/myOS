use core::task::Waker;
use alloc::{string::String, sync::Arc};
use async_trait::async_trait;
use crate::{fs::{socketfs::socketinode::SocketInode, FileMeta, FileTrait, InodeTrait, OpenFlags}, net::Socket, utils::{downcast::Downcast, SysResult}};
use alloc::boxed::Box;

pub struct SocketFile {
    pub metadata: FileMeta
}

impl SocketFile {
    pub fn new(flags: OpenFlags, inode: Arc<dyn InodeTrait>) -> Self {
        Self {
            metadata: FileMeta::new(flags, inode),
        }
    }
}

#[async_trait]
impl FileTrait for SocketFile {
    fn get_inode(&self) -> Arc<dyn InodeTrait>  {
        self.metadata.inode.clone()
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn executable(&self) -> bool {
        false
    }

    async fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        let inode = self.get_inode();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        let (res, _) = socketinode.socket.recv_msg(buf).await?;
        Ok(res)
    }

    async fn write(&self, buf: &[u8]) -> SysResult<usize> {
        let inode = self.get_inode();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        let res = socketinode.socket.send_msg(buf, None).await?;
        Ok(res)
    }

    fn pollin(&self, waker: Waker) -> SysResult<bool> {
        let inode = self.get_inode();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        socketinode.socket.pollin(waker)
    }

    fn pollout(&self, waker: Waker) -> SysResult<bool> {
        let inode = self.get_inode();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        socketinode.socket.pollout(waker)
    }

    fn get_socket(&self) -> SysResult<Arc<dyn Socket>> {
        let inode = self.get_inode();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        Ok(socketinode.socket.clone())
    }

    fn get_flags(&self) -> OpenFlags {
        let inode = self.get_inode();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        socketinode.socket.get_flags().unwrap_or(OpenFlags::empty())
    }
}