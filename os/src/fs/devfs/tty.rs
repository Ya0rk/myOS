use crate::{
    drivers::device::uart::UART_DEVICE,
    fs::{
        ffi::RenameFlags, page_cache::PageCache, Dirent, FileTrait, InodeTrait, InodeType, Kstat,
        OpenFlags, Page, S_IFCHR,
    },
    mm::user_ptr::{user_mut_ptr, user_ref},
    sync::{SpinNoIrqLock, TimeStamp},
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
use core::fmt;
use lazy_static::lazy_static;
use log::{error, info};

type Pid = u32;

lazy_static! {
    pub static ref TTY_INODE: Arc<DevTty> = Arc::new(DevTty::new());
}

pub struct DevTty {
    inner: SpinNoIrqLock<DevTtyInner>,
}

impl DevTty {
    pub fn new() -> Self {
        Self {
            inner: SpinNoIrqLock::new(DevTtyInner::new()),
        }
    }
}

#[async_trait]
impl FileTrait for DevTty {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        TTY_INODE.clone()
    }
    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        Ok(self.read_dirctly(0, user_buf).await)
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        Ok(self.write_directly(0, user_buf).await)
    }
    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/tty".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        Err(Errno::EPERM)
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
    fn is_device(&self) -> bool {
        true
    }
}

#[async_trait]
impl InodeTrait for DevTty {
    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }
        let mut ch = UART_DEVICE.getchar();
        let termios = self.inner.lock().termios;
        if termios.is_icrnl() && ch == b'\r' {
            ch = b'\n';
        }
        if termios.is_echo() {
            print!("{}", ch as char);
        }
        buf[0] = ch;
        1
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        let termios = self.inner.lock().termios;
        if termios.is_opost() {
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
                    error!("[DevTty::ioctl] Unsupported command: {:?}", cmd);
                    Err(Errno::EINVAL)
                }
            }
        }
    }

    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }
    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }
    fn node_type(&self) -> InodeType {
        InodeType::CharDevice
    }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        unimplemented!("DevTty does not have a timestamp")
    }
    fn is_dir(&self) -> bool {
        false
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

#[derive(Clone, Copy)]
#[repr(C)]
struct Termios {
    pub iflag: IFlag,
    pub oflag: OFlag,
    pub cflag: CFlag,
    pub lflag: LFlag,
    pub line: u8,
    pub cc: [u8; 19],
}

impl fmt::Debug for Termios {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Termios")
            .field("iflag", &format_args!("{}", self.iflag))
            .field("oflag", &format_args!("{}", self.oflag))
            .field("cflag", &format_args!("{}", self.cflag))
            .field("lflag", &format_args!("{}", self.lflag))
            .field("line", &self.line)
            .field("cc", &self.cc)
            .finish()
    }
}

impl Termios {
    fn new() -> Self {
        Self {
            iflag: IFlag::IMAXBEL | IFlag::IUTF8 | IFlag::IXON | IFlag::ICRNL | IFlag::BRKINT,
            oflag: OFlag::OPOST | OFlag::ONLCR,
            cflag: CFlag::CREAD | CFlag::CS8 | CFlag::HUPCL,
            lflag: LFlag::ISIG | LFlag::ICANON | LFlag::ECHO | LFlag::ECHOE | LFlag::ECHOK | LFlag::ECHOKE | LFlag::ECHOCTL,
            line: 0,
            cc: [3, 28, 127, 21, 4, 1, 0, 0, 17, 19, 26, 255, 18, 15, 23, 22, 255, 0, 0],
        }
    }
    fn is_icrnl(&self) -> bool {
        self.iflag.contains(IFlag::ICRNL)
    }
    fn is_echo(&self) -> bool {
        self.lflag.contains(LFlag::ECHO)
    }
    fn is_onlcr(&self) -> bool {
        self.oflag.contains(OFlag::ONLCR)
    }
    fn is_opost(&self) -> bool {
        self.oflag.contains(OFlag::OPOST)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum TtyIoctlCmd {
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

bitflags! {
    #[derive(Clone, Copy)] struct IFlag: u32 {
        const IGNBRK = 0o0000001; const BRKINT = 0o0000002; const IGNPAR = 0o0000004;
        const PARMRK = 0o0000010; const INPCK = 0o0000020; const ISTRIP = 0o0000040;
        const INLCR = 0o0000100; const IGNCR = 0o0000200; const ICRNL = 0o0000400;
        const IUCLC = 0o0001000; const IXON = 0o0002000; const IXANY = 0o0004000;
        const IXOFF = 0o0010000; const IMAXBEL = 0o0020000; const IUTF8 = 0o0040000;
    }
    #[derive(Clone, Copy)] struct OFlag: u32 {
        const OPOST = 0o0000001; const OLCUC = 0o0000002; const ONLCR = 0o0000004;
        const OCRNL = 0o0000010; const ONOCR = 0o0000020; const ONLRET = 0o0000040;
        const OFILL = 0o0000100; const OFDEL = 0o0000200; const NLDLY = 0o0000400;
        const NL0 = 0o0000000; const NL1 = 0o0000400; const CRDLY = 0o0003000;
        const CR0 = 0o0000000; const CR1 = 0o0001000; const CR2 = 0o0002000;
        const CR3 = 0o0003000; const TABDLY = 0o0014000; const TAB0 = 0o0000000;
        const TAB1 = 0o0004000; const TAB2 = 0o0010000; const TAB3 = 0o0014000;
        const BSDLY = 0o0020000; const BS0 = 0o0000000; const BS1 = 0o0020000;
        const FFDLY = 0o0100000; const FF0 = 0o0000000; const FF1 = 0o0100000;
        const VTDLY = 0o0040000; const VT0 = 0o0000000; const VT1 = 0o0040000;
    }
    #[derive(Clone, Copy)] struct CFlag: u32 {
        const CSIZE = 0o0000060; const CS5 = 0o0000000; const CS6 = 0o0000020;
        const CS7 = 0o0000040; const CS8 = 0o0000060; const CSTOPB = 0o0000100;
        const CREAD = 0o0000200; const PARENB = 0o0000400; const PARODD = 0o0001000;
        const HUPCL = 0o0002000; const CLOCAL = 0o0004000;
    }
    #[derive(Clone, Copy)] struct LFlag: u32 {
        const ISIG = 0o0000001; const ICANON = 0o0000002; const ECHO = 0o0000010;
        const ECHOE = 0o0000020; const ECHOK = 0o0000040; const ECHONL = 0o0000100;
        const NOFLSH = 0o0000200; const TOSTOP = 0o0000400; const ECHOCTL = 0o0001000;
        const ECHOPRT = 0o0002000; const ECHOKE = 0o0004000; const FLUSHO = 0o0010000;
        const PENDIN = 0o0040000; const IEXTEN = 0o0100000; const EXTPROC = 0o0200000;
    }
}

impl fmt::Display for IFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(" | ")
            .fmt(f)
    }
}
impl fmt::Display for OFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(" | ")
            .fmt(f)
    }
}
impl fmt::Display for CFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(" | ")
            .fmt(f)
    }
}
impl fmt::Display for LFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(" | ")
            .fmt(f)
    }
}