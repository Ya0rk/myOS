use super::ffi::RenameFlags;
use super::FileTrait;
use super::InodeTrait;
use super::Kstat;
use super::OpenFlags;
use crate::fs::page_cache::PageCache;
use crate::fs::Dirent;
use crate::fs::Page;
use crate::hal::arch::console_getchar;
use crate::hal::arch::console_putchar;
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
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_flags(&self) -> OpenFlags {
        OpenFlags::O_RDONLY
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }

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

    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
    fn is_device(&self) -> bool {
        true
    }
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
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_flags(&self) -> OpenFlags {
        OpenFlags::O_WRONLY
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }

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

    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
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
        let termios = self.inner.lock().termios;

        // 1. 检查总开关 OPOST
        if termios.is_opost() {
            // 输出后处理已开启
            for &byte in buf {
                // 2. 检查具体的处理标志，例如 ONLCR
                if termios.is_onlcr() && byte == b'\n' {
                    console_putchar(b'\r' as usize);
                    console_putchar(b'\n' as usize);
                } else {
                    // 在这里可以添加对 OCRNL, ONOCR 等其他标志的处理
                    // 为了简化，我们暂时只处理 ONLCR
                    console_putchar(byte as usize);
                }
            }
        } else {
            // 输出后处理被关闭 (vi 的情况)
            // 直接、原始地输出所有字节
            for &byte in buf {
                console_putchar(byte as usize);
            }
        }

        // write 系统调用通常返回成功消耗的输入缓冲区字节数
        // 在这个实现中，我们总是消耗掉所有字节
        buf.len()
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
                    info!(
                        "get from \n{:?} to \n{:?}",
                        self.inner.lock().termios,
                        *user_termios_ptr
                    );
                    *user_termios_ptr = self.inner.lock().termios.clone();
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
                    log::info!(
                        "[TtyFile::ioctl] set termios \n{:?} from \n{:?}",
                        user_termios_ref,
                        self.inner.lock().termios
                    );
                    self.inner.lock().termios = user_termios_ref.clone();
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

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        todo!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }
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

impl core::fmt::Debug for Termios {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Termios")
            .field("iflag", &format_args!("{}", self.iflag)) // 打印八进制
            .field("oflag", &format_args!("{}", self.oflag)) // 打印八进制
            .field("cflag", &format_args!("{}", self.cflag)) // 打印八进制
            .field("lflag", &format_args!("{}", self.lflag)) // 打印八进制
            .field("line", &self.line)
            .field("cc", &self.cc)
            .finish()
    }
}

impl Termios {
    /// Provides a new `Termios` struct with sane default values.
    fn new() -> Self {
        Self {
            iflag: IFlag::IMAXBEL | IFlag::IUTF8 | IFlag::IXON | IFlag::ICRNL | IFlag::BRKINT, //0o66402,   // IMAXBEL | IUTF8 | IXON | IXANY | ICRNL | BRKINT
            oflag: OFlag::OPOST | OFlag::ONLCR, //0o000005,  // OPOST | ONLCR
            cflag: CFlag::CREAD | CFlag::CS8 | CFlag::HUPCL, // 0o002277,                    // CREAD | CS8 | HUPCL
            lflag: LFlag::ISIG
                | LFlag::ICANON
                | LFlag::ECHO
                | LFlag::ECHOE
                | LFlag::ECHOK
                | LFlag::ECHOKE
                | LFlag::ECHOCTL, // 0o0105073, // ISIG | ICANON | ECHO | ECHOE | ECHOK | ECHOCTL | ECHOKE
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
                255, // 11: VEOL
                18,  // 12: VREPRINT (Ctrl-R)
                15,  // 13: VDISCARD (Ctrl-O)
                23,  // 14: VWERASE (Ctrl-W)
                22,  // 15: VLNEXT (Ctrl-V)
                255, // 16: VEOL2
                0, 0, // 17, 18: Unused
            ],
        }
    }

    /// Check if ICRNL (translate carriage return to newline on input) is set.
    fn is_icrnl(&self) -> bool {
        // const ICRNL: u32 = 0o000400;
        // self.iflag & ICRNL != 0
        self.iflag.contains(IFlag::ICRNL)
    }

    /// Check if ECHO (echo input characters) is set.
    fn is_echo(&self) -> bool {
        // const ECHO: u32 = 0o000010;
        // self.lflag & ECHO != 0
        self.lflag.contains(LFlag::ECHO)
    }

    fn is_onlcr(&self) -> bool {
        // const ONLCR: u32 = 0o0000004;
        // self.oflag & ONLCR != 0
        self.oflag.contains(OFlag::ONLCR)
    }

    fn is_opost(&self) -> bool {
        // const OPOST: u32 = 0o0000001;
        // self.oflag & OPOST != 0
        self.oflag.contains(OFlag::OPOST)
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

bitflags! {
    /// Input flags for terminal I/O settings.
    #[derive(Clone, Copy)]
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

bitflags! {
    /// Output flags for terminal I/O settings.
    #[derive(Clone, Copy)]
    struct OFlag: u32 {
        const OPOST = 0o0000001;   // Post-process output.
        const OLCUC = 0o0000002;   // Map lowercase characters to uppercase on output (not in POSIX).
        const ONLCR = 0o0000004;   // Map NL to CR-NL on output.
        const OCRNL = 0o0000010;   // Map CR to NL on output.
        const ONOCR = 0o0000020;   // No CR output at column 0.
        const ONLRET = 0o0000040;  // NL performs CR function.
        const OFILL = 0o0000100;   // Use fill characters for delay.
        const OFDEL = 0o0000200;   // Fill is DEL.
        const NLDLY = 0o0000400;   // Select newline delays.
        const NL0 = 0o0000000;     // Newline type 0.
        const NL1 = 0o0000400;     // Newline type 1.
        const CRDLY = 0o0003000;   // Select carriage-return delays.
        const CR0 = 0o0000000;     // Carriage-return delay type 0.
        const CR1 = 0o0001000;     // Carriage-return delay type 1.
        const CR2 = 0o0002000;     // Carriage-return delay type 2.
        const CR3 = 0o0003000;     // Carriage-return delay type 3.
        const TABDLY = 0o0014000;  // Select horizontal-tab delays.
        const TAB0 = 0o0000000;    // Horizontal-tab delay type 0.
        const TAB1 = 0o0004000;    // Horizontal-tab delay type 1.
        const TAB2 = 0o0010000;    // Horizontal-tab delay type 2.
        const TAB3 = 0o0014000;    // Expand tabs to spaces.
        const BSDLY = 0o0020000;   // Select backspace delays.
        const BS0 = 0o0000000;     // Backspace-delay type 0.
        const BS1 = 0o0020000;     // Backspace-delay type 1.
        const FFDLY = 0o0100000;   // Select form-feed delays.
        const FF0 = 0o0000000;     // Form-feed delay type 0.
        const FF1 = 0o0100000;     // Form-feed delay type 1.
        const VTDLY = 0o0040000;   // Select vertical-tab delays.
        const VT0 = 0o0000000;     // Vertical-tab delay type 0.
        const VT1 = 0o0040000;     // Vertical-tab delay type 1.
    }
}

bitflags! {
    /// Control flags for terminal I/O settings.
    #[derive(Clone, Copy)]
    struct CFlag: u32 {
        const CSIZE = 0o0000060;   // Character size mask.
        const CS5 = 0o0000000;     // 5-bit characters.
        const CS6 = 0o0000020;     // 6-bit characters.
        const CS7 = 0o0000040;     // 7-bit characters.
        const CS8 = 0o0000060;     // 8-bit characters.
        const CSTOPB = 0o0000100;  // Send two stop bits, else one.
        const CREAD = 0o0000200;   // Enable receiver.
        const PARENB = 0o0000400;  // Enable parity generation and checking.
        const PARODD = 0o0001000;  // Use odd parity instead of even.
        const HUPCL = 0o0002000;   // Hang up on last close.
        const CLOCAL = 0o0004000;  // Ignore modem control lines.
    }
}

bitflags! {
    /// Local flags for terminal I/O settings.
    #[derive(Clone, Copy)]
    struct LFlag: u32 {
        const ISIG = 0o0000001;    // Enable signals.
        const ICANON = 0o0000002;  // Canonical input (erase and kill processing).
        const ECHO = 0o0000010;    // Enable echo.
        const ECHOE = 0o0000020;   // Echo erase character as error-correcting backspace.
        const ECHOK = 0o0000040;   // Echo KILL.
        const ECHONL = 0o0000100;  // Echo NL.
        const NOFLSH = 0o0000200;  // Disable flush after interrupt or quit.
        const TOSTOP = 0o0000400;  // Send SIGTTOU for background output.
        const ECHOCTL = 0o0001000; // Echo control characters as ^X (not in POSIX).
        const ECHOPRT = 0o0002000; // Echo characters as they are being erased (not in POSIX).
        const ECHOKE = 0o0004000;  // Echo KILL by erasing each character on the line (not in POSIX).
        const FLUSHO = 0o0010000;  // Output is being flushed (not in POSIX).
        const PENDIN = 0o0040000;  // Reprint all characters in the input queue when the next character is read (not in POSIX).
        const IEXTEN = 0o0100000;  // Enable implementation-defined input processing.
        const EXTPROC = 0o0200000; // Enable external processing (not in POSIX).
    }
}

use core::fmt;

impl fmt::Display for IFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(IFlag::IGNBRK) {
            flags.push("IGNBRK");
        }
        if self.contains(IFlag::BRKINT) {
            flags.push("BRKINT");
        }
        if self.contains(IFlag::IGNPAR) {
            flags.push("IGNPAR");
        }
        if self.contains(IFlag::PARMRK) {
            flags.push("PARMRK");
        }
        if self.contains(IFlag::INPCK) {
            flags.push("INPCK");
        }
        if self.contains(IFlag::ISTRIP) {
            flags.push("ISTRIP");
        }
        if self.contains(IFlag::INLCR) {
            flags.push("INLCR");
        }
        if self.contains(IFlag::IGNCR) {
            flags.push("IGNCR");
        }
        if self.contains(IFlag::ICRNL) {
            flags.push("ICRNL");
        }
        if self.contains(IFlag::IUCLC) {
            flags.push("IUCLC");
        }
        if self.contains(IFlag::IXON) {
            flags.push("IXON");
        }
        if self.contains(IFlag::IXANY) {
            flags.push("IXANY");
        }
        if self.contains(IFlag::IXOFF) {
            flags.push("IXOFF");
        }
        if self.contains(IFlag::IMAXBEL) {
            flags.push("IMAXBEL");
        }
        if self.contains(IFlag::IUTF8) {
            flags.push("IUTF8");
        }
        write!(f, "{}", flags.join(" | "))
    }
}

impl fmt::Display for OFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(OFlag::OPOST) {
            flags.push("OPOST");
        }
        if self.contains(OFlag::OLCUC) {
            flags.push("OLCUC");
        }
        if self.contains(OFlag::ONLCR) {
            flags.push("ONLCR");
        }
        if self.contains(OFlag::OCRNL) {
            flags.push("OCRNL");
        }
        if self.contains(OFlag::ONOCR) {
            flags.push("ONOCR");
        }
        if self.contains(OFlag::ONLRET) {
            flags.push("ONLRET");
        }
        if self.contains(OFlag::OFILL) {
            flags.push("OFILL");
        }
        if self.contains(OFlag::OFDEL) {
            flags.push("OFDEL");
        }
        if self.contains(OFlag::NLDLY) {
            flags.push("NLDLY");
        }
        if self.contains(OFlag::CRDLY) {
            flags.push("CRDLY");
        }
        if self.contains(OFlag::TABDLY) {
            flags.push("TABDLY");
        }
        if self.contains(OFlag::BSDLY) {
            flags.push("BSDLY");
        }
        if self.contains(OFlag::FFDLY) {
            flags.push("FFDLY");
        }
        if self.contains(OFlag::VTDLY) {
            flags.push("VTDLY");
        }
        write!(f, "{}", flags.join(" | "))
    }
}

impl fmt::Display for CFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(CFlag::CSIZE) {
            flags.push("CSIZE");
        }
        if self.contains(CFlag::CS5) {
            flags.push("CS5");
        }
        if self.contains(CFlag::CS6) {
            flags.push("CS6");
        }
        if self.contains(CFlag::CS7) {
            flags.push("CS7");
        }
        if self.contains(CFlag::CS8) {
            flags.push("CS8");
        }
        if self.contains(CFlag::CSTOPB) {
            flags.push("CSTOPB");
        }
        if self.contains(CFlag::CREAD) {
            flags.push("CREAD");
        }
        if self.contains(CFlag::PARENB) {
            flags.push("PARENB");
        }
        if self.contains(CFlag::PARODD) {
            flags.push("PARODD");
        }
        if self.contains(CFlag::HUPCL) {
            flags.push("HUPCL");
        }
        if self.contains(CFlag::CLOCAL) {
            flags.push("CLOCAL");
        }
        write!(f, "{}", flags.join(" | "))
    }
}

impl fmt::Display for LFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(LFlag::ISIG) {
            flags.push("ISIG");
        }
        if self.contains(LFlag::ICANON) {
            flags.push("ICANON");
        }
        if self.contains(LFlag::ECHO) {
            flags.push("ECHO");
        }
        if self.contains(LFlag::ECHOE) {
            flags.push("ECHOE");
        }
        if self.contains(LFlag::ECHOK) {
            flags.push("ECHOK");
        }
        if self.contains(LFlag::ECHONL) {
            flags.push("ECHONL");
        }
        if self.contains(LFlag::NOFLSH) {
            flags.push("NOFLSH");
        }
        if self.contains(LFlag::TOSTOP) {
            flags.push("TOSTOP");
        }
        if self.contains(LFlag::ECHOCTL) {
            flags.push("ECHOCTL");
        }
        if self.contains(LFlag::ECHOPRT) {
            flags.push("ECHOPRT");
        }
        if self.contains(LFlag::ECHOKE) {
            flags.push("ECHOKE");
        }
        if self.contains(LFlag::FLUSHO) {
            flags.push("FLUSHO");
        }
        if self.contains(LFlag::PENDIN) {
            flags.push("PENDIN");
        }
        if self.contains(LFlag::IEXTEN) {
            flags.push("IEXTEN");
        }
        if self.contains(LFlag::EXTPROC) {
            flags.push("EXTPROC");
        }
        write!(f, "{}", flags.join(" | "))
    }
}
