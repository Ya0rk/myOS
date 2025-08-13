use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use crate::{fs::{open, procfs::irqtable::IRQTABLE, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags}, utils::SysResult};

pub struct InterruptInode(pub InodeMeta);

impl InterruptInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self(InodeMeta::new(
            crate::fs::InodeType::File, 
            0, 
            "/proc/interrupts".into()
        )))
    }
}

#[async_trait]
impl InodeTrait for InterruptInode {
    fn metadata(&self) ->  &InodeMeta {
        &self.0
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Ok(Vec::from(IRQTABLE.lock().tostring()))
    }

    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        0
    }
    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        let irqinfo = IRQTABLE.lock().tostring();
        let irqinfo = Vec::from(irqinfo);
        let len = irqinfo.len();
        if offset < len {
            let read_len = core::cmp::min(len - offset, buf.len());
            buf[..read_len].copy_from_slice(&irqinfo[offset..offset + read_len]);
            read_len
        } else {
            0
        }
    }

    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();

        res.st_mode = InodeType::File as u32;
        res.st_nlink = 1;
        res.st_size = IRQTABLE.lock().tostring().len() as i64;
        res
    }
    fn get_size(&self) -> usize {
        IRQTABLE.lock().tostring().len()
    }
}