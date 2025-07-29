use core::{cell::{SyncUnsafeCell, UnsafeCell}, fmt::write, marker::PhantomData};

use async_trait::async_trait;
use alloc::{boxed::Box, sync::Arc};
use spin::RwLock;
use crate::{drivers::{device_new::{dev_number::MajorNumber, Device, DeviceType}, tty::{self,  termios}}, sync::{new_shared, Shared, SleepShared}, utils::{container::ring_buffer::LineBuffer, SysResult}};

#[async_trait]
pub trait CharDevice : Device + Send + Sync + 'static {
    async fn read(&self, buf: &mut [u8]) -> usize;
    async fn write(&self, buf: &[u8]) -> usize;
    async fn ioctl(&self, op: TtyIoctlCmd, arg: usize) -> SysResult<usize>;
    // // poll if input is available
    // async fn poll_in(&self) -> bool;
    // // poll if output is available
    // async fn poll_out(&self) -> bool;
}

// lazy_static! {
//     pub static ref TTY: Arc<TtyBase> = Arc::new(TtyBase::new(SERIAL_DRIVER.clone(), MajorNumber::Serial, 64));
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
    async fn read(&self, tty: &TtyBase, buf: &mut [u8]) -> usize;
    async fn write(&self, tty: &TtyBase, buf: &[u8]) -> usize;
    async fn poll_in(&self, tty: &TtyBase) -> bool;
    async fn poll_out(&self, tty: &TtyBase) -> bool;
    /// TODO: validate_termios is a more adaptable choice
    async fn set_mode(&self, tty: &TtyBase, mode: TtyLineDiscMode);
}


pub struct TtyLineDisc;

impl TtyLineDisc {
    pub async fn read_raw(tty: &TtyBase, buf: &mut [u8]) -> usize {
        tty.driver.read(buf).await
    }
}

/// TODO：暂未完善
impl TtyLineDisc {
    pub async fn read_canonical(tty: &TtyBase, buf: &mut [u8]) -> usize {

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
    async fn read(&self, tty: &TtyBase, buf: &mut [u8]) -> usize {
        let mode = tty.n_tty_mode.read().clone();
        match mode {
            TtyLineDiscMode::Raw => TtyLineDisc::read_raw(tty, buf).await,
            TtyLineDiscMode::Canonical => TtyLineDisc::read_canonical(tty, buf).await,
        }
    }
    async fn write(&self, tty: &TtyBase, buf: &[u8]) -> usize {
            
        tty.driver.write(buf).await
    }
    async fn poll_in(&self, tty: &TtyBase) -> bool {
        tty.driver.poll_in().await
    }
    async fn poll_out(&self, tty: &TtyBase) -> bool {
        tty.driver.poll_out().await
        // true
    }
    async fn set_mode(&self, tty: &TtyBase, mode: TtyLineDiscMode) {
        *tty.n_tty_mode.write() = mode;
    }
}

// pub struct LineDiscipline<P: LineDiscPolicy> {
//     _null: u8,
//     _marker: PhantomData<P>
// }

// impl<P> LineDiscipline<P>
// where P: LineDiscPolicy {
//     pub fn new() -> LineDiscipline<P> {
//         LineDiscipline {
//             _null: 0,
//             _marker: PhantomData
//         }
//     }
//     pub async fn read(tty: &TtyBase, buf: &mut [u8]) -> usize {
//         P::read(tty, buf).await
//     }
//     pub async fn write(tty: &TtyBase, buf: &[u8]) -> usize {
//         P::write(tty, buf).await
//     }
//     pub async fn poll_in(tty: &TtyBase) -> bool {
//         P::poll_in(tty).await
//     }
//     pub async fn poll_out(tty: &TtyBase) -> bool {
//         P::poll_out(tty).await
//     }
//     pub async fn set_mode(tty: &TtyBase, mode: TtyLineDiscMode) {
//         P::set_mode(tty, mode).await
//     }
    
// }


pub struct TtyBase {
    pub driver: Arc<dyn TtyDriver>,

    pub termios: RwLock<termios::Termios>,

    pub n_tty_mode: RwLock<TtyLineDiscMode>,
    pub ldisc: SyncUnsafeCell<Arc<dyn LineDiscPolicy>>,

    pub lbuffer: Shared<LineBuffer>,

    pub major: MajorNumber,

    pub minor: usize,
    
}



impl TtyBase {
    pub fn new(driver: Arc<dyn TtyDriver>, major: MajorNumber, minor: usize) -> TtyBase {
        TtyBase {
            driver,
            termios: RwLock::new(termios::Termios::new()),
            n_tty_mode: RwLock::new(TtyLineDiscMode::Raw),
            ldisc: SyncUnsafeCell::new(Arc::new(TtyLineDisc)),
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


impl Device for TtyBase {
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
}


#[async_trait]
impl CharDevice for TtyBase {
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
    async fn ioctl(&self, request: TtyIoctlCmd, arg: usize) -> SysResult<usize> {
        Ok(0)
    }
}