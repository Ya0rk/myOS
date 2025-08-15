use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use crate::{fs::{ffi::MEMINFO, InodeMeta, InodeTrait, InodeType, Kstat}, mm::frame_allocator::{FrameAllocator, FRAME_ALLOCATOR}, utils::SysResult};


pub struct MeminfoInode(pub InodeMeta);

impl MeminfoInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self(InodeMeta::new(
            InodeType::File, 
            0, 
            "/proc/meminfo".into())
        ))
    }
}

#[async_trait]
impl InodeTrait for MeminfoInode {
    fn metadata(&self) -> &InodeMeta {
        &self.0
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        let mut buf = Vec::from(meminfo2string());
        Ok(buf)
    }

    fn get_size(&self) -> usize {
        MEMINFO.len()
    }

    async fn read_at(&self, offset: usize, mut buf: &mut [u8]) -> usize {
        // 非常重要
        // 这里不能read_at
        let meminfo = Vec::from(meminfo2string());
        let len = meminfo.len();
        if offset < len {
            let read_len = core::cmp::min(len - offset, buf.len());
            buf[..read_len].copy_from_slice(&meminfo[offset..offset + read_len]);
            read_len
        } else {
            0
        }
    }

    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_mode = InodeType::File as u32;
        res.st_nlink = 1;
        res.st_size = MEMINFO.len() as i64;
        res
    }
}

fn meminfo2string() -> String {
    let (mem_total, mem_free, mem_available) = {
    let frame_allocator = FRAME_ALLOCATOR.lock();
        (
            frame_allocator.frame_total() * 4,
            frame_allocator.frame_free() * 4,
            frame_allocator.frame_free() * 4,
        )
    };

    let meminfo = format!(
        r"MemTotal:     {mem_total:>10} kB
MemFree:      {mem_free:>10} kB
MemAvailable: {mem_available:>10} kB
",
        mem_total = mem_total,
        mem_free = mem_free,
        mem_available = mem_available
    );
    return meminfo;
}