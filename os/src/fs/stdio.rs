use super::ffi::RenameFlags;
use super::FileTrait;
use super::InodeTrait;
use super::Kstat;
use super::OpenFlags;
use crate::fs::page_cache::PageCache;
use crate::fs::Dirent;
use crate::hal::arch::console_getchar;
use crate::mm::user_ptr::user_mut_ptr;
use crate::mm::user_ptr::user_ref;
use crate::mm::user_ptr::user_ref_mut;
use crate::mm::{page::Page, UserBuffer};
use crate::sync::SpinNoIrqLock;
use crate::sync::TimeStamp;
use crate::task::get_current_hart_id;

use crate::utils::Errno;
use crate::utils::SysResult;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

const LF: usize = 0x0a;
const CR: usize = 0x0d;
type Pid = u32;

pub struct Stdin {
    inode: Arc<TtyInode>,
}

impl Stdin {
    pub fn new() -> Self {
        Self {
            inode: TTYINODE.clone(),
        }
    }
}

pub struct Stdout {
   inode: Arc<TtyInode>, 
}

impl Stdout {
    pub fn new() -> Self {
        Self {
            inode:TTYINODE.clone(), 
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
    async fn read(&self, mut user_buf: &mut [u8]) -> SysResult<usize> {
        //一次读取多个字符
        // let mut c: usize;
        // let mut count: usize = 0;
        // while count < user_buf.len() {
        //     c = console_getchar();
        //     if c > 255 {
        //         break;
        //     }
        //     user_buf[count] = c as u8;
        //     count += 1;
        // }
        // Ok(count)
        let res = {self.inode.read_dirctly(0, user_buf).await};
        let termios = {self.inode.inner.lock().termios};
        if termios.is_icrnl() {
            for i in 0..res {
                if user_buf[i] == '\r' as u8 {
                    user_buf[i] = '\n' as u8;
                }
            }
        };
        if termios.is_echo() {
            self.inode.write_directly(0, &user_buf);
        }
        Ok(res)
    }   
    async fn write(&self, _user_buf: &[u8]) -> SysResult<usize> {
        Err(Errno::EINVAL)
        // panic!("Cannot write to stdin!");
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("Stdin".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        // todo!()
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }

    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
}
///
/// 当前先记录工作行为
/// 
/// 将vi 的软件行为进行记录
/// 
/// 注意到应当去除这里对底层接口 print 的调用，转用 tty inode 进行实现
/// 
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
    async fn read(&self, _user_buf: &mut [u8]) -> SysResult<usize> {
        panic!("Cannot read from stdout!");
    }
    async fn write_at(&self, offset: usize, buf: &[u8]) -> SysResult<usize> {
        self.write(buf).await
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        // match core::str::from_utf8(user_buf) {
        //     Ok(text) => {
        //         print!("{}", text);
        //         Ok(text.len())
        //     }
        //         ,
        //     Err(e) =>  {
        //         Err(Errno::EBADCALL)
        //     }
        // }
        // print!("{}", core::str::from_utf8(user_buf).unwarp());
        // Ok(user_buf.len())
        let res = self.inode.write_directly(0, &user_buf).await;
        Ok(res)
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("Stdout".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        todo!()
    }
}

/// 临时设置，应当迁移到 tty，
/// 
/// 这里采用单例模式
pub struct TtyInode {
    inner: SpinNoIrqLock<TtyInodeInner>,
}

impl TtyInode {
    fn new() -> Self {
        Self {
            inner: SpinNoIrqLock::new(TtyInodeInner::new())
        }
    }
}

impl InodeTrait for TtyInode {
    #[doc = " 设置大小"]
    fn set_size(&self,new_size:usize) -> SysResult {
        Ok(())
    }

    #[doc = " 绕过cache，直接从磁盘读"]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn read_dirctly<'life0,'life1,'async_trait>(&'life0 self,_offset:usize,_buf: &'life1 mut [u8]) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = usize> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        // 一次读取多个字符
        let mut c: usize;
        let mut count: usize = 0;
        while count < _buf.len() {
            c = console_getchar();
            if c > 255 {
                break;
            }
            _buf[count] = c as u8;
            count += 1;
        }
        Box::pin(async move { count })
    }

    #[doc = " 直接写"]
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn write_directly<'life0,'life1,'async_trait>(&'life0 self,_offset:usize,_buf: &'life1[u8]) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = usize> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,Self:'async_trait {
        let res = match core::str::from_utf8(_buf) {
            Ok(text) => {
                print!("{}", text);
                text.len()
            }
                ,
            Err(e) =>  {
                0
            }
        };
        // print!("{}", core::str::from_utf8(_buf).expect("no utf8"));
        Box::pin(async move { res })
    }


    #[doc = " 获取时间戳，用于修改或访问"]
    fn get_timestamp(&self) ->  &SpinNoIrqLock<TimeStamp>  {
        todo!()
    }

    fn is_dir(&self) -> bool {
        todo!()
    }

    #[doc = " get page cache from ext4 file"]
    fn get_page_cache(&self) -> Option<Arc<PageCache> >  {
        todo!()
    }

    #[doc = " 获得目录项"]
    fn read_dents(&self) -> Option<Vec<Dirent> >  {
        todo!()
    }

    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        let cmd = TtyIoctl::from_bits(op as u32).ok_or(Errno::EINVAL)?;
        debug_point!("[tty_ioctl]");
        log::info!("[TtyFile::ioctl] cmd {:?}, value {:#x}", cmd, arg);
        match cmd {
            TtyIoctl::TCGETS | TtyIoctl::TCGETA => {
                debug_point!("TCGETS");
                unsafe {
                    *(arg as *mut Termios) = self.inner.lock().termios;
                }
                Ok(0)
            }
            TtyIoctl::TCSETS | TtyIoctl::TCSETSW | TtyIoctl::TCSETSF => {
                debug_point!("TCSETS");
                unsafe {
                    self.inner.lock().termios = *(arg as *const Termios);
                    log::info!("termios {:#x?}", self.inner.lock().termios);
                }
                Ok(0)
            }
            TtyIoctl::TIOCGPGRP => {
                debug_point!("TIOCGPGRP");
                let fg_pgid = self.inner.lock().fg_pgid.clone();
                debug_point!("TIOCGPGRP");
                log::info!("[TtyFile::ioctl] get fg pgid {:?}", fg_pgid);
                unsafe {
                    *(arg as *mut Pid) = fg_pgid;
                }
                debug_point!("TIOCGPGRP");
                Ok(0)
            }
            TtyIoctl::TIOCSPGRP => {
                debug_point!("TIOCSPGRP");
                let user_ptr: &Pid = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                unsafe {
                    self.inner.lock().fg_pgid = user_ptr.clone();
                }
                log::info!("[TtyFile::ioctl] set fg pgid {:?}", user_ptr);
                Ok(0)
            }
            TtyIoctl::TIOCGWINSZ => {
                debug_point!("TIOCGWINSZ");
                let win_size = self.inner.lock().win_size;
                log::info!("[TtyFile::ioctl] get window size {win_size:?}");
                unsafe {
                    *(arg as *mut WinSize) = win_size;
                }
                Ok(0)
            }
            TtyIoctl::TIOCSWINSZ => {
                debug_point!("TIOCSWINSZ");
                unsafe {
                    self.inner.lock().win_size = *(arg as *const WinSize);
                }
                Ok(0)
            }
            TtyIoctl::TCSBRK => Ok(0),
            _ => {
                log::error!("[TtyFile::ioctl] Unsupported command: {cmd:?}");
                Err(Errno::EINVAL)
            }
        }
    }

}

lazy_static! {
    pub static ref TTYINODE: Arc<TtyInode> = Arc::new(TtyInode::new());
}

struct TtyInodeInner {
    fg_pgid: Pid,
    win_size: WinSize,
    termios: Termios,
}

impl TtyInodeInner {
    fn new() -> Self {
        Self {
            // TODO 可能在龙芯会出错？
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
            ws_row: 67,
            ws_col: 120,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

// 定义 Termios 数据结构
/// Defined in <asm-generic/termbits.h>
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Termios {
    /// Input mode flags.
    pub iflag: u32,
    /// Output mode flags.
    pub oflag: u32,
    /// Control mode flags.
    pub cflag: u32,
    /// Local mode flags.
    pub lflag: u32,
    /// Line discipline.
    pub line: u8,
    /// control characters.
    pub cc: [u8; 19],
}

impl Termios {
    fn new() -> Self {
        Self {
            // IMAXBEL | IUTF8 | IXON | IXANY | ICRNL | BRKINT
            iflag: 0o66402,
            // OPOST | ONLCR
            oflag: 0o5,
            // HUPCL | CREAD | CSIZE | EXTB
            cflag: 0o2277,
            // IEXTEN | ECHOTCL | ECHOKE ECHO | ECHOE | ECHOK | ISIG | ICANON
            lflag: 0o105073,
            line: 0,
            cc: [
                3,   // VINTR Ctrl-C
                28,  // VQUIT
                127, // VERASE
                21,  // VKILL
                4,   // VEOF Ctrl-D
                0,   // VTIME
                1,   // VMIN
                0,   // VSWTC
                17,  // VSTART
                19,  // VSTOP
                26,  // VSUSP Ctrl-Z
                255, // VEOL
                18,  // VREPAINT
                15,  // VDISCARD
                23,  // VWERASE
                22,  // VLNEXT
                255, // VEOL2
                0, 0,
            ],
        }
    }

    fn is_icrnl(&self) -> bool {
        const ICRNL: u32 = 0o0000400;
        self.iflag & ICRNL != 0
    }

    fn is_echo(&self) -> bool {
        const ECHO: u32 = 0o0000010;
        self.lflag & ECHO != 0
    }
}


/// Defined in <asm-generic/ioctls.h>
#[derive(Debug)]
#[repr(usize)]
enum TtyIoctlCmd {
    // For struct termios
    /// Gets the current serial port settings.
    TCGETS = 0x5401,
    /// Sets the serial port settings immediately.
    TCSETS = 0x5402,
    /// Sets the serial port settings after allowing the input and output
    /// buffers to drain/empty.
    TCSETSW = 0x5403,
    /// Sets the serial port settings after flushing the input and output
    /// buffers.
    TCSETSF = 0x5404,
    /// For struct termio
    /// Gets the current serial port settings.
    TCGETA = 0x5405,
    /// Sets the serial port settings immediately.
    #[allow(unused)]
    TCSETA = 0x5406,
    /// Sets the serial port settings after allowing the input and output
    /// buffers to drain/empty.
    #[allow(unused)]
    TCSETAW = 0x5407,
    /// Sets the serial port settings after flushing the input and output
    /// buffers.
    #[allow(unused)]
    TCSETAF = 0x5408,
    /// If the terminal is using asynchronous serial data transmission, and arg
    /// is zero, then send a break (a stream of zero bits) for between 0.25
    /// and 0.5 seconds.
    TCSBRK = 0x5409,
    /// Get the process group ID of the foreground process group on this
    /// terminal.
    TIOCGPGRP = 0x540F,
    /// Set the foreground process group ID of this terminal.
    TIOCSPGRP = 0x5410,
    /// Get window size.
    TIOCGWINSZ = 0x5413,
    /// Set window size.
    TIOCSWINSZ = 0x5414,
    UNSUPPORT,
}


bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct TtyIoctl: u32 {
        /// Gets the current serial port settings.
        const TCGETS = 0x5401;
        /// Sets the serial port settings immediately.
        const TCSETS = 0x5402;
        /// Sets the serial port settings after allowing the input and output
        /// buffers to drain/empty.
        const TCSETSW = 0x5403;
        /// Sets the serial port settings after flushing the input and output
        /// buffers.
        const TCSETSF = 0x5404;
        /// Gets the current serial port settings (termio).
        const TCGETA = 0x5405;
        /// Sets the serial port settings immediately (termio).
        const TCSETA = 0x5406;
        /// Sets the serial port settings after allowing the input and output
        /// buffers to drain/empty (termio).
        const TCSETAW = 0x5407;
        /// Sets the serial port settings after flushing the input and output
        /// buffers (termio).
        const TCSETAF = 0x5408;
        /// Sends a break signal for asynchronous serial data transmission.
        const TCSBRK = 0x5409;
        /// Gets the process group ID of the foreground process group on this terminal.
        const TIOCGPGRP = 0x540F;
        /// Sets the foreground process group ID of this terminal.
        const TIOCSPGRP = 0x5410;
        /// Gets the window size of the terminal.
        const TIOCGWINSZ = 0x5413;
        /// Sets the window size of the terminal.
        const TIOCSWINSZ = 0x5414;
    }
}
