use core::{cell::{SyncUnsafeCell, UnsafeCell}, fmt::write, marker::PhantomData};

use async_trait::async_trait;
use alloc::{boxed::Box, sync::Arc, vec::{self, Vec}};
use log::{error, info};
use spin::RwLock;
use crate::{drivers::{device::{dev_number::MajorNumber, Device, DeviceType}, tty::{self, termios::{self, Termios, WinSize}}}, mm::user_ptr::{user_ref, user_ref_mut}, sync::{new_shared, Shared, SleepShared}, utils::{container::ring_buffer::LineBuffer, Errno, SysResult}};

#[async_trait]
pub trait CharDevice : Device + Send + Sync + 'static {
    async fn read(&self, buf: &mut [u8]) -> usize;
    async fn write(&self, buf: &[u8]) -> usize;
    async fn ioctl(&self, op: TtyIoctlCmd, arg: usize) -> SysResult<usize>;
    // // poll if input is available
    async fn poll_in(&self) -> bool;
    // // poll if output is available
    async fn poll_out(&self) -> bool;
}

// lazy_static! {
//     pub static ref TTY: Arc<TtyBase> = Arc::new(TtyBase::new(SERIAL_DRIVER.clone(), MajorNumber::Tty, 64));
// }


#[async_trait]
pub trait TtyDriver : Send + Sync {
    async fn read(&self, buf: &mut [u8]) -> usize;
    async fn readc(&self) -> u8;
    async fn write(&self, buf: &[u8]) -> usize;
    // poll if input is available
    async fn poll_in(&self) -> bool;
    // poll if output is available
    async fn poll_out(&self) -> bool;

    async fn stop(&self);

    async fn start(&self);

    async fn validate_termios(&self, termios: &termios::Termios) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum TtyIoctlCmd {
    TCGETS = 0x5401, TCSETS = 0x5402, TCSETSW = 0x5403, TCSETSF = 0x5404,
    TCGETA = 0x5405, TCSETA = 0x5406, TCSETAW = 0x5407, TCSETAF = 0x5408,
    TCSBRK = 0x5409, TIOCGPGRP = 0x540F, TIOCSPGRP = 0x5410,
    TIOCGWINSZ = 0x5413, TIOCSWINSZ = 0x5414,
}

impl TryFrom<usize> for TtyIoctlCmd {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0x5401 => Ok(Self::TCGETS), 0x5402 => Ok(Self::TCSETS), 0x5403 => Ok(Self::TCSETSW),
            0x5404 => Ok(Self::TCSETSF), 0x5405 => Ok(Self::TCGETA), 0x5406 => Ok(Self::TCSETA),
            0x5407 => Ok(Self::TCSETAW), 0x5408 => Ok(Self::TCSETAF), 0x5409 => Ok(Self::TCSBRK),
            0x540F => Ok(Self::TIOCGPGRP), 0x5410 => Ok(Self::TIOCSPGRP), 0x5413 => Ok(Self::TIOCGWINSZ),
            0x5414 => Ok(Self::TIOCSWINSZ), _ => Err(()),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyLineDiscMode {
    Raw,
    Canonical,
}


#[async_trait]
pub trait LineDiscPolicy : Sync + Send + 'static {
    async fn read(&self, tty: &TtyStruct, buf: &mut [u8]) -> usize;
    async fn write(&self, tty: &TtyStruct, buf: &[u8]) -> usize;
    async fn poll_in(&self, tty: &TtyStruct) -> bool;
    async fn poll_out(&self, tty: &TtyStruct) -> bool;
    /// TODO: validate_termios is a more adaptable choice
    async fn set_mode(&self, tty: &TtyStruct, mode: TtyLineDiscMode);
}


pub struct TtyLineDisc;

impl TtyLineDisc {
    pub async fn read_raw(tty: &TtyStruct, buf: &mut [u8]) -> usize {
        tty.driver.read(buf).await
    }
}

/// TODO：暂未完善
impl TtyLineDisc {
    pub async fn read_canonical(tty: &TtyStruct, buf: &mut [u8]) -> usize {

        loop {
            let c = tty.driver.readc().await;
            match c {
                b'\n' => {
                    tty.lbuffer.lock().push(b'\n');
                    return 1;
                }
                _ => {
                    tty.lbuffer.lock().push(c);
                    return 1;
                }
            }
        }
    }
}

#[async_trait]
impl LineDiscPolicy for TtyLineDisc {
    async fn read(&self, tty: &TtyStruct, buf: &mut [u8]) -> usize {
        let mode = tty.n_tty_mode.read().clone();
        match mode {
            TtyLineDiscMode::Raw => TtyLineDisc::read_raw(tty, buf).await,
            TtyLineDiscMode::Canonical => TtyLineDisc::read_canonical(tty, buf).await,
        }
    }
    async fn write(&self, tty: &TtyStruct, _buf: &[u8]) -> usize {
        let mut buf = Vec::<u8>::new();
        let opost = tty.termios.read().is_opost();
        let onlcr = tty.termios.read().is_onlcr();
        let len = _buf.len();
        for &c in _buf {
            #[cfg(any(feature = "2k1000la", feature = "vf2"))]
            {
                if opost {
                    if onlcr && c == b'\n' {
                        buf.push(b'\r');
                        buf.push(b'\n');
                    } else {
                        buf.push(c);
                    }
                }
                else {
                    buf.push(c);
                }
            }
            #[cfg(feature = "board_qemu")]
            buf.push(c);

        }
        tty.driver.write(&buf).await;
        len
    }
    async fn poll_in(&self, tty: &TtyStruct) -> bool {
        tty.driver.poll_in().await
    }
    async fn poll_out(&self, tty: &TtyStruct) -> bool {
        tty.driver.poll_out().await
        // true
    }
    async fn set_mode(&self, tty: &TtyStruct, mode: TtyLineDiscMode) {
        *tty.n_tty_mode.write() = mode;
    }
}

pub struct TtyStruct {
    // 对接下一层，SerialDriver实现这个trait
    pub driver: Arc<dyn TtyDriver>,
    // 用于设置行规程、输入输出格式和设备控制信息
    pub termios: RwLock<termios::Termios>,
    // 为 N_TTY 行规程使用，判断当前是否为行编辑模式
    pub n_tty_mode: RwLock<TtyLineDiscMode>,
    // 存储行规程，行规程使用策略模式
    pub ldisc: SyncUnsafeCell<Arc<dyn LineDiscPolicy>>,
    // 前台进程组
    pub fg_pgid: RwLock<u32>,
    // 终端窗口尺寸
    pub win_size: RwLock<WinSize>,
    // 行缓冲区
    pub lbuffer: Shared<LineBuffer>,
    // 主设备号
    pub major: MajorNumber,
    // 次设备号
    pub minor: usize,
    
}



impl TtyStruct {
    pub fn new(driver: Arc<dyn TtyDriver>, major: MajorNumber, minor: usize) -> TtyStruct {
        TtyStruct {
            driver,
            termios: RwLock::new(termios::Termios::new()),
            n_tty_mode: RwLock::new(TtyLineDiscMode::Raw),
            ldisc: SyncUnsafeCell::new(Arc::new(TtyLineDisc)),
            fg_pgid: RwLock::new(1),
            win_size: RwLock::new(WinSize::new()),
            lbuffer: new_shared(LineBuffer::new(4096)),
            major,
            minor,
        }
    }
    fn with_ldisc<'a, T>(&'a self, f: impl FnOnce(&'a Arc<dyn LineDiscPolicy>) -> T) -> T {
        unsafe {
            f(&(*self.ldisc.get()))
        }
    }
    fn with_mut_ldisc<T>(&self, f: impl FnOnce(&mut Arc<dyn LineDiscPolicy>) -> T) -> T {
        unsafe {
            f(&mut (*self.ldisc.get()))
        }
    }
}


impl Device for TtyStruct {
    fn get_type(&self) -> DeviceType {
        DeviceType::Char
    }

    fn get_major(&self) -> MajorNumber {
        // TODO
        self.major
    }

    fn get_minor(&self) -> usize {
        // TODO
        self.minor
    }
    fn as_char(self: Arc<Self>) -> Option<Arc<dyn CharDevice>> {
        Some(self)
    }
}


#[async_trait]
impl CharDevice for TtyStruct {
    async fn read(&self, buf: &mut [u8]) -> usize {
        self.with_ldisc( |ldisc| {
            ldisc.read(self, buf)
        }).await
    }
    async fn write(&self, buf: &[u8]) -> usize {
        self.with_ldisc( |ldisc| {
            ldisc.write(self, buf)
        }).await
    }
    async fn ioctl(&self, op: TtyIoctlCmd, arg: usize) -> SysResult<usize> {
        let cmd = TtyIoctlCmd::try_from(op).map_err(|_| Errno::EINVAL)?;
        info!("[DevTty::ioctl] cmd: {:?}, arg: {:#x}", cmd, arg);
        unsafe {
            match cmd {
                TtyIoctlCmd::TCGETS | TtyIoctlCmd::TCGETA => {
                    let user_termios_ptr = user_ref_mut::<Termios>(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_termios_ptr = self.termios.read().clone();
                    Ok(0)
                }
                TtyIoctlCmd::TIOCGPGRP => {
                    let mut user_pgid_ptr = user_ref_mut::<u32>(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_pgid_ptr = *self.fg_pgid.read();
                    Ok(0)
                }
                TtyIoctlCmd::TIOCGWINSZ => {
                    let mut user_winsize_ptr = user_ref_mut::<WinSize>(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_winsize_ptr = self.win_size.read().clone();
                    Ok(0)
                }
                TtyIoctlCmd::TCSETS | TtyIoctlCmd::TCSETSW | TtyIoctlCmd::TCSETSF => {
                    let user_termios_ref: &Termios = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    *self.termios.write() = user_termios_ref.clone();
                    Ok(0)
                }
                TtyIoctlCmd::TIOCSPGRP => {
                    let user_pgid_ref: &u32 = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    *self.fg_pgid.write() = *user_pgid_ref;
                    Ok(0)
                }
                TtyIoctlCmd::TIOCSWINSZ => {
                    let user_winsize_ref: &WinSize = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    *self.win_size.write() = *user_winsize_ref;
                    Ok(0)
                }
                TtyIoctlCmd::TCSBRK => Ok(0),
                _ => {
                    // error!("[DevTty::ioctl] Unsupported command: {:?}", cmd);
                    Err(Errno::EINVAL)
                }
            }
        }
    }
    async fn poll_in(&self) -> bool {
        self.with_ldisc( |ldisc| {
            ldisc.poll_in(self)
        }).await
    }
    async fn poll_out(&self) -> bool {
        self.with_ldisc( |ldisc| {
            ldisc.poll_out(self)
        }).await
    }
}