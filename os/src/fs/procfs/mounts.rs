use alloc::{string::String, sync::Arc, boxed::Box};
use async_trait::async_trait;
use crate::fs::{InodeMeta, InodeTrait, InodeType, Kstat};


pub struct MountsInode(pub InodeMeta);

impl MountsInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self(InodeMeta::new(
            InodeType::File, 
            0, 
            "/proc/mounts".into()
        )))
    }
}

#[async_trait]
impl InodeTrait for MountsInode {
    fn metadata(&self) ->  &InodeMeta {
        &self.0
    }
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        // 非常重要
        return 0;
    }
    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        // 非常重要
        return 0;
    }
    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_mode = 16877;
        res.st_ino = self.0.ino as u64;
        res.st_nlink = 1;
        res
    }
    fn get_size(&self) -> usize {
        4000
    }
}