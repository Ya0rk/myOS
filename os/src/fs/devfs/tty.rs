// deprecated
use crate::{
    drivers::{uart::UART_DEVICE, tty::{termios::Termios, tty_core::TtyIoctlCmd}},
    fs::{
        page_cache::PageCache, Dirent, FileMeta, FileTrait, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags, Page, S_IFCHR
    },
    mm::user_ptr::{user_mut_ptr, user_ref},
    sync::SpinNoIrqLock,
    utils::{Errno, SysResult},
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use async_trait::async_trait;
use bitflags::bitflags;
use core::{error, fmt};
use lazy_static::lazy_static;
use log::{error, info};

type Pid = u32;

pub struct DevTty {
    metadata: FileMeta,
}

impl DevTty {
    pub fn new_in() -> Self {
        Self {
            metadata: FileMeta::new(
                OpenFlags::O_RDONLY,
                DevTtyInode::new(),
            ),
        }
    }
    pub fn new_out() -> Self {
        Self {
            metadata: FileMeta::new(
                OpenFlags::O_WRONLY,
                DevTtyInode::new(),
            ),
        }
    }
}

#[async_trait]
impl FileTrait for DevTty {
    fn metadata(&self) -> &FileMeta {
        &self.metadata
    }
    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        assert!(self.metadata.flags.read().readable());
        if user_buf.is_empty() {
            return Ok(0);
        }
        Ok(self.metadata.inode.read_dirctly(0, user_buf).await)
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        assert!(self.metadata.flags.read().writable());
        Ok(self.metadata.inode.write_directly(0, user_buf).await)
    }
    fn abspath(&self) -> String {
        "/dev/tty".to_string()
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
}

pub struct DevTtyInode {
    metadata: InodeMeta,
    inner: SpinNoIrqLock<DevTtyInner>,
}

impl DevTtyInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::CharDevice,
                0,
                "/dev/tty",
            ),
            inner: SpinNoIrqLock::new(DevTtyInner::new()),
        })
    }
}

#[async_trait]
impl InodeTrait for DevTtyInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }
    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        // error!("getchar");
        if buf.is_empty() {
            return 0;
        }
        let mut ch = UART_DEVICE.getchar();
        let termios = self.inner.lock().termios;
        if termios.is_icrnl() && ch == b'\r' {
            ch = b'\n';
        }
        if termios.is_echo() {
            // error!("ECHO");
            print!("{}", ch as char);
        }
        buf[0] = ch;
        1
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        let termios = self.inner.lock().termios;
        if termios.is_opost() {
        // if false {
            for &byte in buf {
                if termios.is_onlcr() && byte == b'\n' {
                    UART_DEVICE.putchar(b'\r');
                    UART_DEVICE.putchar(b'\n');
                } else {
                    UART_DEVICE.putchar(byte);
                }
            }
        } else {
            for &byte in buf {
                UART_DEVICE.putchar(byte);
            }
        }
        buf.len()
    }

    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        let cmd = TtyIoctlCmd::try_from(op).map_err(|_| Errno::EINVAL)?;
        info!("[DevTty::ioctl] cmd: {:?}, arg: {:#x}", cmd, arg);
        unsafe {
            match cmd {
                TtyIoctlCmd::TCGETS | TtyIoctlCmd::TCGETA => {
                    let mut user_termios_ptr = user_mut_ptr(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_termios_ptr = self.inner.lock().termios.clone();
                    Ok(0)
                }
                TtyIoctlCmd::TIOCGPGRP => {
                    let mut user_pid_ptr = user_mut_ptr(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_pid_ptr = self.inner.lock().fg_pgid;
                    Ok(0)
                }
                TtyIoctlCmd::TIOCGWINSZ => {
                    let mut user_winsize_ptr = user_mut_ptr(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_winsize_ptr = self.inner.lock().win_size;
                    Ok(0)
                }
                TtyIoctlCmd::TCSETS | TtyIoctlCmd::TCSETSW | TtyIoctlCmd::TCSETSF => {
                    let user_termios_ref: &Termios = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    self.inner.lock().termios = user_termios_ref.clone();
                    Ok(0)
                }
                TtyIoctlCmd::TIOCSPGRP => {
                    let user_pid_ref: &Pid = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    self.inner.lock().fg_pgid = *user_pid_ref;
                    Ok(0)
                }
                TtyIoctlCmd::TIOCSWINSZ => {
                    let user_winsize_ref: &WinSize = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    self.inner.lock().win_size = *user_winsize_ref;
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

    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }
    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_ino = self.metadata.ino as u64;
        res.st_mode = S_IFCHR;
        res.st_nlink = 1;
        res
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }
}

struct DevTtyInner {
    fg_pgid: Pid,
    win_size: WinSize,
    termios: Termios,
}

impl DevTtyInner {
    fn new() -> Self {
        Self {
            fg_pgid: 1,
            win_size: WinSize::new(),
            termios: Termios::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct WinSize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

impl WinSize {
    fn new() -> Self {
        Self {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

