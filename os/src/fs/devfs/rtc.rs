use core::{cmp::min, fmt::{Formatter, Debug}};

use alloc::{format, string::String};

use crate::{fs::{FileTrait, Kstat}, mm::UserBuffer};

pub struct DevRtc;

impl DevRtc {
    pub fn new() -> Self {
        Self
    }
}

impl FileTrait for DevRtc {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
        let str = format!("{:?}", time);
        let bytes = str.as_bytes();
        let len = min(user_buf.len(), bytes.len());
        user_buf.write(bytes);
        len
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        // do nothing
        user_buf.len()
    }
    
    fn get_name(&self) -> String {
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
    fn fstat(&self, _stat: &mut Kstat) -> () {
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
