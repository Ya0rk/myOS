use alloc::sync::Arc;
use crate::{fs::InodeTrait, net::Socket};
use async_trait::async_trait;
use alloc::boxed::Box;

pub struct SocketInode {
    pub socket: Arc<dyn Socket>,
}

impl SocketInode {
    pub fn new(socket: Arc<dyn Socket>) -> Self {
        Self { socket }
    }
}

#[async_trait]
impl InodeTrait for SocketInode {}