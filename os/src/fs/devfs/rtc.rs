use core::{cmp::min, fmt::{Formatter, Debug}};
use alloc::{format, string::String, sync::Arc};
use async_trait::async_trait;
use alloc::boxed::Box;
use crate::{fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat}, mm::{UserBuffer, page::Page}, utils::SysResult};

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
        todo!()
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    // fn poll(&self, events: PollEvents) -> PollEvents {
    //     let mut revents = PollEvents::empty();
    //     if events.contains(PollEvents::IN) {
    //         revents |= PollEvents::IN;
    //     }
    //     if events.contains(PollEvents::OUT) {
    //         revents |= PollEvents::OUT;
    //     }
    //     revents
    // }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }
    fn is_dir(&self) -> bool {
        todo!()
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
