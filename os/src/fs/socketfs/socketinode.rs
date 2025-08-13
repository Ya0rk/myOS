use alloc::{string::{String, ToString}, sync::Arc};
use crate::{fs::{InodeMeta, InodeTrait, InodeType}, net::Socket};
use async_trait::async_trait;
use alloc::boxed::Box;

pub struct SocketInode {
    pub metadata: InodeMeta,
    pub socket: Arc<dyn Socket>,
}

impl SocketInode {
    pub fn new(socket: Arc<dyn Socket>) -> Self {
        Self {
            metadata: InodeMeta::new(
                InodeType::Socket,
                0,
                "/Socket".into(),
            ),
            socket,
        }
    }
}

#[async_trait]
impl InodeTrait for SocketInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
}