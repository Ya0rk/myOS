use super::{File, Ioctl};
use crate::mm::{translated_byte_buffer, translated_ref, translated_refmut};
use crate::task::current_task;
use crate::utils::{SysErrNo, SyscallRet};
use crate::{
    mm::UserBuffer,
    sbi::console_getchar,
    syscall::{IoctlCommand, PollEvents},
    task::suspend_current_and_run_next,
};
use alloc::vec::Vec;
use core::mem::size_of;
/// # 标准输入输出接口
/// `os/src/fs/stdio.rs`
/// ```
/// pub struct Stdin
/// pub struct Stdout
/// ```
//
use lazy_static::lazy_static;
use spin::Mutex;

const LF: usize = 0x0a;
const CR: usize = 0x0d;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: UserBuffer) -> SyscallRet {
        /*
        //一次读取单个字符
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
        */
        //一次读取多个字符
        let mut c: usize;
        let mut count: usize = 0;
        let mut buf = Vec::new();
        while count < user_buf.len() {
            c = console_getchar();
            match c {
                // `c > 255`是为了兼容OPENSBI，OPENSBI未获取字符时会返回-1
                0 | 256.. => {
                    suspend_current_and_run_next();
                    continue;
                }
                CR => {
                    buf.push(LF as u8);
                    count += 1;
                    break;
                }
                LF => {
                    buf.push(LF as u8);
                    count += 1;
                    break;
                }
                _ => {
                    buf.push(c as u8);
                    count += 1;
                }
            }
        }
        user_buf.write(buf.as_slice());
        Ok(count)
    }
    fn write(&self, _user_buf: UserBuffer) -> SyscallRet {
        Err(SysErrNo::EINVAL)
        // panic!("Cannot write to stdin!");
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::IN) {
            revents |= PollEvents::IN;
        }
        revents
    }
}

impl Ioctl for Stdin {
    fn ioctl(&self, cmd: usize, arg: usize) -> isize {
        let cmd = IoctlCommand::from(cmd);
        let task = current_task().unwrap();
        let mut inner = task.inner_lock();
        let token = inner.user_token();

        match cmd {
            IoctlCommand::TCGETS | IoctlCommand::TCGETA => {
                let mut arg = UserBuffer::new(
                    translated_byte_buffer(token, arg as *const u8, size_of::<Termios>()).unwrap(),
                );
                arg.write(IOINFO.lock().termios.as_bytes());
                return 0;
            }
            IoctlCommand::TCSETS | IoctlCommand::TCSETSW | IoctlCommand::TCSETSF => {
                let arg = translated_ref(token, arg as *const Termios);
                IOINFO.lock().termios.update(arg);
                return 0;
            }
            IoctlCommand::TIOCGPGRP => {
                *translated_refmut(token, arg as *mut u32) = IOINFO.lock().foreground_pgid;
                return 0;
            }
            IoctlCommand::TIOCSPGRP => {
                let arg = translated_ref(token, arg as *const u32);
                IOINFO.lock().foreground_pgid = *arg;
                return 0;
            }
            IoctlCommand::TIOCGWINSZ => {
                let mut arg = UserBuffer::new(
                    translated_byte_buffer(token, arg as *const u8, size_of::<WinSize>()).unwrap(),
                );
                arg.write(IOINFO.lock().winsize.as_bytes());
                return 0;
            }
            IoctlCommand::TIOCSWINSZ => {
                let arg = translated_ref(token, arg as *const WinSize);
                IOINFO.lock().winsize.update(arg);
                return 0;
            }
            _ => {
                return -1;
            }
        };
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> SyscallRet {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> SyscallRet {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        Ok(user_buf.len())
    }
    fn poll(&self, events: PollEvents) -> PollEvents {
        let mut revents = PollEvents::empty();
        if events.contains(PollEvents::OUT) {
            revents |= PollEvents::OUT;
        }
        revents
    }
}

impl Ioctl for Stdout {
    fn ioctl(&self, cmd: usize, arg: usize) -> isize {
        let cmd = IoctlCommand::from(cmd);
        let task = current_task().unwrap();
        let mut inner = task.inner_lock();
        let token = inner.user_token();

        match cmd {
            IoctlCommand::TCGETS | IoctlCommand::TCGETA => {
                let mut arg = UserBuffer::new(
                    translated_byte_buffer(token, arg as *const u8, size_of::<Termios>()).unwrap(),
                );
                arg.write(IOINFO.lock().termios.as_bytes());
                return 0;
            }
            IoctlCommand::TCSETS | IoctlCommand::TCSETSW | IoctlCommand::TCSETSF => {
                let arg = translated_ref(token, arg as *const Termios);
                IOINFO.lock().termios.update(arg);
                return 0;
            }
            IoctlCommand::TIOCGPGRP => {
                *translated_refmut(token, arg as *mut u32) = IOINFO.lock().foreground_pgid;
                return 0;
            }
            IoctlCommand::TIOCSPGRP => {
                let arg = translated_ref(token, arg as *const u32);
                IOINFO.lock().foreground_pgid = *arg;
                return 0;
            }
            IoctlCommand::TIOCGWINSZ => {
                let mut arg = UserBuffer::new(
                    translated_byte_buffer(token, arg as *const u8, size_of::<WinSize>()).unwrap(),
                );
                arg.write(IOINFO.lock().winsize.as_bytes());
                return 0;
            }
            IoctlCommand::TIOCSWINSZ => {
                let arg = translated_ref(token, arg as *const WinSize);
                IOINFO.lock().winsize.update(arg);
                return 0;
            }
            _ => {
                return -1;
            }
        };
    }
}

#[derive(Debug)]
pub struct IOInfo {
    foreground_pgid: u32,
    winsize: WinSize,
    termios: Termios,
}

#[derive(Clone, Copy, Debug)]
/// The termios functions describe a general terminal interface that
/// is provided to control asynchronous communications ports.
pub struct Termios {
    /// input modes
    pub iflag: u32,
    /// ouput modes
    pub oflag: u32,
    /// control modes
    pub cflag: u32,
    /// local modes
    pub lflag: u32,
    pub line: u8,
    /// terminal special characters.
    pub cc: [u8; 32],
    pub ispeed: u32,
    pub ospeed: u32,
}

impl Termios {
    pub fn new() -> Self {
        Termios {
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
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            ispeed: 0,
            ospeed: 0,
        }
    }
    pub fn update(&mut self, termios: &Self) {
        self.iflag = termios.iflag;
        self.oflag = termios.oflag;
        self.cflag = termios.cflag;
        self.lflag = termios.lflag;
        self.line = termios.line;
        self.cc.copy_from_slice(&termios.cc[..]);
        self.ispeed = termios.ispeed;
        self.ospeed = termios.ospeed;
    }
    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WinSize {
    ws_row: u16,
    ws_col: u16,
    xpixel: u16,
    ypixel: u16,
}

impl WinSize {
    pub fn new() -> Self {
        Self {
            ws_row: 24,
            ws_col: 80,
            xpixel: 0,
            ypixel: 0,
        }
    }
    pub fn update(&mut self, winsize: &Self) {
        self.ws_row = winsize.ws_row;
        self.ws_col = winsize.ws_col;
        self.xpixel = winsize.xpixel;
        self.ypixel = winsize.ypixel;
    }
    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size) }
    }
}

lazy_static! {
    pub static ref IOINFO: Mutex<IOInfo> = {
        Mutex::new(IOInfo {
            foreground_pgid: 0,
            winsize: WinSize::new(),
            termios: Termios::new(),
        })
    };
}
