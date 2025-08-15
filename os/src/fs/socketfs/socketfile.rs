use core::task::Waker;
use alloc::{string::{String, ToString}, sync::Arc};
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
    fn metadata(&self) -> &FileMeta {
        &self.metadata
    }

    async fn read(&self, buf: &mut [u8]) -> SysResult<usize> {
        let inode = self.metadata().inode.clone();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        let (res, _) = socketinode.socket.recv_msg(buf).await?;
        Ok(res)
    }

    async fn write(&self, buf: &[u8]) -> SysResult<usize> {
        let inode = self.metadata().inode.clone();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        let res = socketinode.socket.send_msg(buf, None).await?;
        Ok(res)
    }

    async fn pollin(&self) -> SysResult<bool> {
        let inode = self.metadata().inode.clone();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        socketinode.socket.pollin().await
    }

    async fn pollout(&self) -> SysResult<bool> {
        let inode = self.metadata().inode.clone();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        socketinode.socket.pollout().await
    }

    fn get_socket(&self) -> SysResult<Arc<dyn Socket>> {
        let inode = self.metadata().inode.clone();
        let socketinode = inode.downcast_arc::<SocketInode>().unwrap();
        Ok(socketinode.socket.clone())
    }

    fn abspath(&self) -> String {
        "SocketFile".to_string()
    }
}