use crate::{
    fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat, OpenFlags, S_IFCHR},
    mm::{page::Page, UserBuffer},
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

pub struct DevRtc;

impl DevRtc {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevRtc {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        todo!()
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
    async fn read(&self, mut user_buf: &mut [u8]) -> SysResult<usize> {
        let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
        let str = format!("{:?}", time);
        let bytes = str.as_bytes();
        let len = min(user_buf.len(), bytes.len());
        // user_buf.write(bytes);
        Ok(len)
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/rtc".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }

    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        // self.metadata.inode.get_page_cache().unwrap().get_page(offset).unwrap()
        todo!()
    }
}

// impl Ioctl for DevRtc {
//     fn ioctl(&self, cmd: usize, arg: usize) -> isize {
//         let cmd = IoctlCommand::from(cmd);
//         let task = current_task().unwrap();
//         let mut inner = task.inner_lock();
//         let token = inner.get_user_token();

//         match cmd {
//             IoctlCommand::RTC_RD_TIME => {
//                 let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
//                 let mut arg = UserBuffer::new(
//                     translated_byte_buffer(token, arg as *const u8, size_of::<RtcTime>()),
//                 );
//                 arg.write(time.as_bytes());
//             }
//             _ => return -1,
//         }
//         0
//     }
// }

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

#[async_trait]
impl InodeTrait for DevRtc {
    fn get_page_cache(&self) -> Option<Arc<crate::fs::page_cache::PageCache>> {
        None
    }

    fn get_size(&self) -> usize {
        0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }


    fn node_type(&self) -> crate::fs::InodeType {
        crate::fs::InodeType::CharDevice
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

    fn loop_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = crate::fs::InodeType::CharDevice as u32;
        stat.st_nlink = 1;
        stat.st_size = 0;
        stat
    }


    fn get_timestamp(&self) -> &crate::sync::SpinNoIrqLock<crate::sync::TimeStamp> {
        // 如果需要返回一个实际值，需要给 DevRtc 加 timestamp 字段
        unimplemented!("DevRtc does not have a timestamp")
    }

    fn is_dir(&self) -> bool {
        false
    }

    // fn rename(&self, _old_path: &String, _new_path: &String) {}

    fn read_dents(&self) -> Option<Vec<crate::fs::Dirent>> {
        None
    }
}
