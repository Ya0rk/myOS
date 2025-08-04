use alloc::vec::Vec;
use core::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Termios {
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
    pub fn new() -> Self {
        Self {
            iflag: IFlag::IMAXBEL | IFlag::IUTF8 | IFlag::IXON | IFlag::ICRNL | IFlag::BRKINT,
            oflag: OFlag::OPOST | OFlag::ONLCR,
            cflag: CFlag::CREAD | CFlag::CS8 | CFlag::HUPCL,
            lflag: LFlag::ISIG | LFlag::ICANON | LFlag::ECHO | LFlag::ECHOE | LFlag::ECHOK | LFlag::ECHOKE | LFlag::ECHOCTL,
            line: 0,
            cc: [3, 28, 127, 21, 4, 1, 0, 0, 17, 19, 26, 255, 18, 15, 23, 22, 255, 0, 0],
        }
    }
    pub fn is_icrnl(&self) -> bool {
        self.iflag.contains(IFlag::ICRNL)
    }
    pub fn is_echo(&self) -> bool {
        self.lflag.contains(LFlag::ECHO)
    }
    pub fn is_onlcr(&self) -> bool {
        self.oflag.contains(OFlag::ONLCR)
    }
    pub fn is_opost(&self) -> bool {
        self.oflag.contains(OFlag::OPOST)
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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct WinSize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

impl WinSize {
    pub fn new() -> Self {
        Self {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}