use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use crate::{fs::{open, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags}, utils::SysResult};


pub struct ExeInode(pub InodeMeta);

impl ExeInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self(InodeMeta::new(
            InodeType::File,
            0,
            "/proc/self/exe".into(),
        )))
    }
}

#[async_trait]
impl InodeTrait for ExeInode {
    fn metadata(&self) ->  &InodeMeta {
        &self.0
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        // Ok(alloc::vec![])
        if let Ok(exe) = open("/bin/sh".into(), OpenFlags::O_RDONLY) {
            exe.metadata().inode.read_all().await
        } else {
            Err(crate::utils::Errno::EACCES)
        }
    }

    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        // 非常重要
        if let Ok(exe) = open("/bin/sh".into(), OpenFlags::O_RDONLY) {
            exe.metadata().inode.read_at(offset, &mut buf).await
        } else {
            0
        }
    }

    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();

        if let Ok(exe) = open("/bin/sh".into(), OpenFlags::O_RDONLY) {
            exe.metadata().inode.fstat()
        } else {
            // error!("open /bin/sh failed");
            res.st_mode = InodeType::File as u32;
            res
        }
    }
}