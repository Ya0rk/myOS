use super::ffi::RenameFlags;
use super::FileTrait;
use super::InodeTrait;
use super::Kstat;
use super::OpenFlags;
use crate::fs::page_cache::PageCache;
use crate::fs::Dirent;
use crate::fs::Page;
use crate::hal::arch::console_getchar;
use crate::mm::user_ptr::{user_mut_ptr, user_ref, user_ref_mut};

use crate::sync::{SpinNoIrqLock, TimeStamp};
use crate::utils::{Errno, SysResult};

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_trait::async_trait;
use bitflags::bitflags;
use lazy_static::lazy_static;
use log::{error, info};

type Pid = u32;

lazy_static! {
    /// The global singleton TTY Inode, shared by Stdin, Stdout, and Stderr.
    pub static ref TTY_INODE: Arc<TtyInode> = Arc::new(TtyInode::new());
}

// --- Stdin ---

pub struct Stdin {
    inode: Arc<TtyInode>,
}

impl Stdin {
    pub fn new() -> Self {
        Self {
            inode: TTY_INODE.clone(),
        }
    }
}

#[async_trait]
impl FileTrait for Stdin {
    fn readable(&self) -> bool { true }
    fn writable(&self) -> bool { false }
    fn executable(&self) -> bool { false }
    fn get_flags(&self) -> OpenFlags { OpenFlags::O_RDONLY }
    fn is_dir(&self) -> bool { false }
    fn get_inode(&self) -> Arc<dyn InodeTrait> { self.inode.clone() }

    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        if user_buf.is_empty() {
            return Ok(0);
        }
        let res = self.inode.read_dirctly(0, user_buf).await;
        Ok(res)
    }

    async fn write(&self, _user_buf: &[u8]) -> SysResult<usize> {
        Err(Errno::EINVAL)
    }

    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        // Stdin is a special file, fstat can be a no-op returning Ok.
        Ok(())
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("stdin".into())
    }

    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        Err(Errno::EPERM)
    }

    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> { None }
}

// --- Stdout ---

pub struct Stdout {
    inode: Arc<TtyInode>,
}

impl Stdout {
    pub fn new() -> Self {
        Self {
            inode: TTY_INODE.clone(),
        }
    }
}

#[async_trait]
impl FileTrait for Stdout {
    fn readable(&self) -> bool { false }
    fn writable(&self) -> bool { true }
    fn executable(&self) -> bool { false }
    fn get_flags(&self) -> OpenFlags { OpenFlags::O_WRONLY }
    fn is_dir(&self) -> bool { false }
    fn get_inode(&self) -> Arc<dyn InodeTrait> { self.inode.clone() }

    async fn read(&self, _user_buf: &mut [u8]) -> SysResult<usize> {
        Err(Errno::EINVAL)
    }

    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        let res = self.inode.write_directly(0, user_buf).await;
        Ok(res)
    }

    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        // Stdout is a special file, fstat can be a no-op returning Ok.
        Ok(())
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok("stdout".into())
    }

    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        Err(Errno::EPERM)
    }
    
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> { None }
}


// --- TtyInode ---

/// Represents the shared state and logic for a Terminal Device (TTY).
pub struct TtyInode {
    inner: SpinNoIrqLock<TtyInodeInner>,
}

impl TtyInode {
    fn new() -> Self {
        Self {
            inner: SpinNoIrqLock::new(TtyInodeInner::new()),
        }
    }
}

#[async_trait]
impl InodeTrait for TtyInode {
    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        // This is a simplified, blocking implementation.
        // A real-world kernel would use wait queues and interrupts here.
        let mut count: usize = 0;
        let vmin = self.inner.lock().termios.cc[5] as usize; // VMIN

        // while count < buf.len() {
            let mut ch = console_getchar() as u8;

            let termios = self.inner.lock().termios;

            // Handle ICRNL (translate carriage return to newline)
            if termios.is_icrnl() && ch == b'\r' {
                ch = b'\n';
            }

            // Handle ECHO
            if termios.is_echo() {
                // Here we directly print, a more complex driver might queue this output
                print!("{}", ch as char);
            }

            buf[count] = ch;
            count += 1;
            
            // For raw mode, VMIN is often 1, so we return after one character.
            // if count >= vmin {
            //     break;
            // }
        // }
        count
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        match core::str::from_utf8(buf) {
            Ok(text) => {
                let filtered: String = text.chars().filter(|&c| c != '\x1b').collect();
                print!("{}", filtered);
                text.len()
            }
            Err(_) => {
                // For non-utf8 data, print what we can lossily.
                let lossy_string = String::from_utf8_lossy(buf);
                let filtered: String = lossy_string.chars().filter(|&c| c != '\x1b').collect();
                // print!("{}", lossy_string);
                print!("{}", filtered);
                lossy_string.len()
            }
        }
    }
    

    
    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        let cmd = TtyIoctlCmd::try_from(op).map_err(|_| Errno::EINVAL)?;
        
        info!("[TtyInode::ioctl] cmd: {:?}, arg: {:#x}", cmd, arg);
        unsafe {
            match cmd {
                // --- GET 操作：从内核复制到用户空间 ---
                TtyIoctlCmd::TCGETS | TtyIoctlCmd::TCGETA => {
                    debug_point!("he wanto get ================== TCGETS");
                    let mut user_termios_ptr = user_mut_ptr(arg.into())?.ok_or(Errno::EFAULT)?;
                    *user_termios_ptr = self.inner.lock().termios.clone();
                    info!("get from {:?} to {:?}", self.inner.lock().termios, user_termios_ptr);
                    Ok(0)
                }
                TtyIoctlCmd::TIOCGPGRP => {
                    debug_point!("TIOCGPGRP");
                    let mut user_pid_ptr = user_mut_ptr(arg.into())?.ok_or(Errno::EFAULT)?;
                    let fg_pgid = self.inner.lock().fg_pgid;
                    log::info!("[TtyFile::ioctl] get fg pgid {:?}", fg_pgid);
                    *user_pid_ptr = fg_pgid;
                    Ok(0)
                }
                TtyIoctlCmd::TIOCGWINSZ => {
                    debug_point!("TIOCGWINSZ");
                    let mut user_winsize_ptr = user_mut_ptr(arg.into())?.ok_or(Errno::EFAULT)?;
                    let win_size = self.inner.lock().win_size;
                    log::info!("[TtyFile::ioctl] get window size {win_size:?}");
                    *user_winsize_ptr = win_size;
                    Ok(0)
                }
        
                // --- SET 操作：从用户空间复制到内核 ---
                TtyIoctlCmd::TCSETS | TtyIoctlCmd::TCSETSW | TtyIoctlCmd::TCSETSF => {
                    debug_point!("he want to set ================== TCSETS");
                    let user_termios_ref: &Termios = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    // info!("he want to set {:?}", user_termios_ref);
                    self.inner.lock().termios = user_termios_ref.clone();
                    log::info!("[TtyFile::ioctl] set termios {:?} to {:?}", user_termios_ref, self.inner.lock().termios);
                    Ok(0)
                }
                TtyIoctlCmd::TIOCSPGRP => {
                    debug_point!("TIOCSPGRP");
                    let user_pid_ref: &Pid = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    let new_pgid = *user_pid_ref;
                    self.inner.lock().fg_pgid = new_pgid;
                    log::info!("[TtyFile::ioctl] set fg pgid {:?}", new_pgid);
                    Ok(0)
                }
                TtyIoctlCmd::TIOCSWINSZ => {
                    debug_point!("TIOCSWINSZ");
                    let user_winsize_ref: &WinSize = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                    self.inner.lock().win_size = *user_winsize_ref;
                    log::info!("[TtyFile::ioctl] set window size {:?}", *user_winsize_ref);
                    Ok(0)
                }
        
                // --- 其他操作 ---
                TtyIoctlCmd::TCSBRK => {
                    // No-op for now, sending a break is UART specific.
                    Ok(0)
                }
                _ => {
                    error!("[TtyInode::ioctl] Unsupported command: {:?}", cmd);
                    Err(Errno::EINVAL)
                }
            }
        }
    }

    fn set_size(&self, _new_size: usize) -> SysResult { Ok(()) }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> { todo!() }
    fn is_dir(&self) -> bool { false }
    fn get_page_cache(&self) -> Option<Arc<PageCache>> { None }
    fn read_dents(&self) -> Option<Vec<Dirent>> { None }
}

// --- TTY Internals ---

struct TtyInodeInner {
    fg_pgid: Pid,
    win_size: WinSize,
    termios: Termios,
}

impl TtyInodeInner {
    fn new() -> Self {
        Self {
            fg_pgid: 0,
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
    ws_xpixel: u16, // Unused
    ws_ypixel: u16, // Unused
}

impl WinSize {
    fn new() -> Self {
        Self {
            ws_row: 24, // A more standard default height
            ws_col: 80, // A more standard default width
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

/// Terminal I/O settings, defined in `<asm-generic/termbits.h>`
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Termios {
    pub iflag: u32,
    pub oflag: u32,
    pub cflag: u32,
    pub lflag: u32,
    pub line: u8,
    pub cc: [u8; 19],
}

impl Termios {
    /// Provides a new `Termios` struct with sane default values.
    fn new() -> Self {
        Self {
            iflag: 0o002402, // BRKINT | IXON
            oflag: 0o000005, // OPOST | ONLCR
            cflag: 0o002277, // CREAD | CS8 | HUPCL
            lflag: 0o0105073,// ISIG | ICANON | ECHO | ECHOE | ECHOK | ECHOCTL | ECHOKE
            line: 0,
            cc: [
                3,   // 0: VINTR (Ctrl-C)
                28,  // 1: VQUIT (Ctrl-\)
                127, // 2: VERASE (Backspace)
                21,  // 3: VKILL (Ctrl-U)
                4,   // 4: VEOF (Ctrl-D)
                1,   // 5: VMIN
                0,   // 6: VTIME
                0,   // 7: VSWTC
                17,  // 8: VSTART (Ctrl-Q)
                19,  // 9: VSTOP (Ctrl-S)
                26,  // 10: VSUSP (Ctrl-Z)
                0,   // 11: VEOL
                18,  // 12: VREPRINT (Ctrl-R)
                15,  // 13: VDISCARD (Ctrl-O)
                23,  // 14: VWERASE (Ctrl-W)
                22,  // 15: VLNEXT (Ctrl-V)
                0,   // 16: VEOL2
                0, 0, // 17, 18: Unused
            ],
        }
    }

    /// Check if ICRNL (translate carriage return to newline on input) is set.
    fn is_icrnl(&self) -> bool {
        const ICRNL: u32 = 0o000400;
        self.iflag & ICRNL != 0
    }

    /// Check if ECHO (echo input characters) is set.
    fn is_echo(&self) -> bool {
        const ECHO: u32 = 0o000010;
        self.lflag & ECHO != 0
    }
}

/// TTY ioctl commands, defined in `<asm-generic/ioctls.h>`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum TtyIoctlCmd {
    TCGETS = 0x5401,
    TCSETS = 0x5402,
    TCSETSW = 0x5403,
    TCSETSF = 0x5404,
    TCGETA = 0x5405,
    TCSETA = 0x5406,
    TCSETAW = 0x5407,
    TCSETAF = 0x5408,
    TCSBRK = 0x5409,
    TIOCGPGRP = 0x540F,
    TIOCSPGRP = 0x5410,
    TIOCGWINSZ = 0x5413,
    TIOCSWINSZ = 0x5414,
}

impl TryFrom<usize> for TtyIoctlCmd {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0x5401 => Ok(Self::TCGETS),
            0x5402 => Ok(Self::TCSETS),
            0x5403 => Ok(Self::TCSETSW),
            0x5404 => Ok(Self::TCSETSF),
            0x5405 => Ok(Self::TCGETA),
            0x5406 => Ok(Self::TCSETA),
            0x5407 => Ok(Self::TCSETAW),
            0x5408 => Ok(Self::TCSETAF),
            0x5409 => Ok(Self::TCSBRK),
            0x540F => Ok(Self::TIOCGPGRP),
            0x5410 => Ok(Self::TIOCSPGRP),
            0x5413 => Ok(Self::TIOCGWINSZ),
            0x5414 => Ok(Self::TIOCSWINSZ),
            _ => Err(()),
        }
    }
}

// C language definitions for terminal I/O input flags.
// #define IGNBRK 0o0000001  // Ignore break condition.
// #define BRKINT 0o0000002  // Signal interrupt on break.
// #define IGNPAR 0o0000004  // Ignore characters with parity errors.
// #define PARMRK 0o0000010  // Mark parity and framing errors.
// #define INPCK 0o0000020   // Enable input parity check.
// #define ISTRIP 0o0000040  // Strip 8th bit off characters.
// #define INLCR 0o0000100   // Map NL to CR on input.
// #define IGNCR 0o0000200   // Ignore CR.
// #define ICRNL 0o0000400   // Map CR to NL on input.
// #define IUCLC 0o0001000   // Map uppercase characters to lowercase on input (not in POSIX).
// #define IXON 0o0002000    // Enable start/stop output control.
// #define IXANY 0o0004000   // Enable any character to restart output.
// #define IXOFF 0o0010000   // Enable start/stop input control.
// #define IMAXBEL 0o0020000 // Ring bell when input queue is full (not in POSIX).
// #define IUTF8 0o0040000   // Input is UTF8 (not in POSIX).

bitflags! {
    /// Input flags for terminal I/O settings.
    struct IFlag: u32 {
        const IGNBRK = 0o0000001; // Ignore break condition.
        const BRKINT = 0o0000002; // Signal interrupt on break.
        const IGNPAR = 0o0000004; // Ignore characters with parity errors.
        const PARMRK = 0o0000010; // Mark parity and framing errors.
        const INPCK = 0o0000020;  // Enable input parity check.
        const ISTRIP = 0o0000040; // Strip 8th bit off characters.
        const INLCR = 0o0000100;  // Map NL to CR on input.
        const IGNCR = 0o0000200;  // Ignore CR.
        const ICRNL = 0o0000400;  // Map CR to NL on input.
        const IUCLC = 0o0001000;  // Map uppercase characters to lowercase on input (not in POSIX).
        const IXON = 0o0002000;   // Enable start/stop output control.
        const IXANY = 0o0004000;  // Enable any character to restart output.
        const IXOFF = 0o0010000;  // Enable start/stop input control.
        const IMAXBEL = 0o0020000;// Ring bell when input queue is full (not in POSIX).
        const IUTF8 = 0o0040000;  // Input is UTF8 (not in POSIX).
    }
}
