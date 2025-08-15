use crate::{
    fs::{ffi::RenameFlags, FileMeta, FileTrait, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags, S_IFCHR},
    mm::page::Page,
    utils::SysResult,
};
use alloc::boxed::Box;
use alloc::{
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use async_trait::async_trait;
use core::{
    cmp::min,
    fmt::{Debug, Formatter},
};

pub struct RtcTime {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl RtcTime {
    pub fn new(year: u32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
    #[allow(unused)]
    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size) }
    }
}

impl Debug for RtcTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}:{}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

pub struct DevRtcInode {
    metadata: InodeMeta,
}

impl DevRtcInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::CharDevice, 
                0, 
                "/dev/rtc"
            ),
        })
    }
}

#[async_trait]
impl InodeTrait for DevRtcInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    fn get_page_cache(&self) -> Option<Arc<crate::fs::page_cache::PageCache>> {
        None
    }

    fn get_size(&self) -> usize {
        0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }

    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
        let str = format!("{:?}", time);
        let bytes = str.as_bytes();
        if offset >= bytes.len() {
            return 0;
        }
        let len = core::cmp::min(buf.len(), bytes.len() - offset);
        buf[..len].copy_from_slice(&bytes[offset..offset + len]);
        len
    }

    async fn read_dirctly(&self, _offset: usize, _buf: &mut [u8]) -> usize {
        0
    }

    async fn write_at(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len()
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len()
    }

    fn truncate(&self, _size: usize) -> usize {
        0
    }

    async fn sync(&self) {}

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
        let str = format!("{:?}", time);
        Ok(str.as_bytes().to_vec())
    }

    fn look_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_ino = self.metadata.ino as u64;
        stat.st_mode = S_IFCHR;
        stat.st_nlink = 1;
        stat.st_size = 0;
        stat
    }

    fn get_timestamp(&self) -> &crate::sync::SpinNoIrqLock<crate::sync::TimeStamp> {
        // 如果需要返回一个实际值，需要给 DevRtc 加 timestamp 字段
        unimplemented!("DevRtc does not have a timestamp")
    }

    fn read_dents(&self) -> Option<Vec<crate::fs::Dirent>> {
        None
    }
}
