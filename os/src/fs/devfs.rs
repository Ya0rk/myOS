use crate::{
    fs::FileClass,
    mm::{translated_byte_buffer, UserBuffer},
    syscall::{IoctlCommand, PollEvents},
    task::{current_task, INITPROC},
    utils::SyscallRet,
};
use alloc::{
    collections::BTreeSet,
    fmt::{Debug, Formatter},
    format,
    string::{String, ToString},
    sync::Arc,
};
use core::{cmp::min, mem::size_of};
use lazy_static::lazy_static;
use spin::Mutex;

use super::{File, Ioctl, Stdout};

pub struct DevZero;
pub struct DevNull;
pub struct DevRtc;
pub struct DevRandom;

pub struct DevTty;

lazy_static! {
    pub static ref DEVICES: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
}

pub fn register_device(abs_path: &str) {
    DEVICES.lock().insert(abs_path.to_string());
}

pub fn unregister_device(abs_path: &str) {
    DEVICES.lock().remove(&abs_path.to_string());
}

pub fn find_device(abs_path: &str) -> bool {
    DEVICES.lock().contains(&abs_path.to_string())
}

pub fn open_device_file(abs_path: &str) -> Option<Arc<dyn File>> {
    // warning: just a fake implementation
    if abs_path == "/dev/zero" {
        Some(Arc::new(DevZero::new()))
    } else if abs_path == "/dev/null" {
        Some(Arc::new(DevNull::new()))
    } else if abs_path == "/dev/rtc" || abs_path == "/dev/rtc0" || abs_path == "/dev/misc/rtc" {
        Some(Arc::new(DevRtc::new()))
    } else if abs_path == "/dev/random" {
        Some(Arc::new(DevRandom::new()))
    } else if abs_path == "/dev/tty" {
        Some(Arc::new(DevTty::new()))
    } else {
        None
    }
}

impl DevZero {
    pub fn new() -> Self {
        Self
    }
}

impl File for DevZero {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut user_buf: UserBuffer) -> SyscallRet {
        Ok(user_buf.fill0())
    }
    fn write(&self, user_buf: UserBuffer) -> SyscallRet {
        // do nothing
        Ok(user_buf.len())
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            revents |= PollEvents::IN;
        }
        if events.contains(PollEvents::OUT) {
            revents |= PollEvents::OUT;
        }
        revents
    }
}

impl DevNull {
    pub fn new() -> Self {
        Self
    }
}

impl File for DevNull {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut _user_buf: UserBuffer) -> SyscallRet {
        // do nothing
        Ok(0)
    }
    fn write(&self, user_buf: UserBuffer) -> SyscallRet {
        // do nothing
        Ok(user_buf.len())
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            revents |= PollEvents::IN;
        }
        if events.contains(PollEvents::OUT) {
            revents |= PollEvents::OUT;
        }
        revents
    }
}

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

impl DevRtc {
    pub fn new() -> Self {
        Self
    }
}

impl File for DevRtc {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut user_buf: UserBuffer) -> SyscallRet {
        let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
        let str = format!("{:?}", time);
        let bytes = str.as_bytes();
        let len = min(user_buf.len(), bytes.len());
        user_buf.write(bytes);
        Ok(len)
    }
    fn write(&self, user_buf: UserBuffer) -> SyscallRet {
        // do nothing
        Ok(user_buf.len())
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            revents |= PollEvents::IN;
        }
        if events.contains(PollEvents::OUT) {
            revents |= PollEvents::OUT;
        }
        revents
    }
}

impl Ioctl for DevRtc {
    fn ioctl(&self, cmd: usize, arg: usize) -> isize {
        let cmd = IoctlCommand::from(cmd);
        let task = current_task().unwrap();
        let mut inner = task.inner_lock();
        let token = inner.user_token();

        match cmd {
            IoctlCommand::RTC_RD_TIME => {
                let time = RtcTime::new(2000, 1, 1, 0, 0, 0);
                let mut arg = UserBuffer::new(
                    translated_byte_buffer(token, arg as *const u8, size_of::<RtcTime>()).unwrap(),
                );
                arg.write(time.as_bytes());
            }
            _ => return -1,
        }
        0
    }
}

impl DevRandom {
    pub fn new() -> Self {
        Self
    }
}

impl File for DevRandom {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut user_buf: UserBuffer) -> SyscallRet {
        Ok(user_buf.fillrandom())
    }
    fn write(&self, user_buf: UserBuffer) -> SyscallRet {
        // do nothing
        Ok(user_buf.len())
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            revents |= PollEvents::IN;
        }
        if events.contains(PollEvents::OUT) {
            revents |= PollEvents::OUT;
        }
        revents
    }
}

impl DevTty {
    pub fn new() -> Self {
        Self
    }
}

impl File for DevTty {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut user_buf: UserBuffer) -> SyscallRet {
        if let Some(FileClass::Abs(tty_device)) = INITPROC.inner_lock().fd_table.try_get_file(0) {
            tty_device.read(user_buf)
        } else {
            panic!("get Stdin error!");
        }
    }
    fn write(&self, user_buf: UserBuffer) -> SyscallRet {
        if let Some(FileClass::Abs(tty_device)) = INITPROC.inner_lock().fd_table.try_get_file(1) {
            tty_device.write(user_buf)
        } else {
            panic!("get Stdout error!");
        }
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            revents |= PollEvents::IN;
        }
        if events.contains(PollEvents::OUT) {
            revents |= PollEvents::OUT;
        }
        revents
    }
}

impl Ioctl for DevTty {
    fn ioctl(&self, cmd: usize, arg: usize) -> isize {
        if let Some(FileClass::Abs(tty_device)) = INITPROC.inner_lock().fd_table.try_get_file(1) {
            // tty_device.ioctl(cmd, arg)
            let tty_device = unsafe { Arc::from_raw(Arc::into_raw(tty_device) as *const Stdout) };
            tty_device.ioctl(cmd, arg)
        } else {
            panic!("get Stdout error!");
        }
    }
}
