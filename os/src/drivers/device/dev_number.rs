#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
// #[repr(usize)]
pub enum MajorNumber {
    Char(CharMajorNum),
    Block(BlockMajorNum),    
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(usize)]
pub enum CharMajorNum {
    /// Memory devices (/dev/null, /dev/zero, /dev/random, etc.)
    Mem = 1,
    /// TTY devices (serial ports, virtual consoles)
    Tty = 4,
    /// TTY auxiliary devices
    TtyAux = 5,
    /// Miscellaneous character devices (/dev/fuse, /dev/psaux, etc.)
    Misc = 10,
    /// Input devices (/dev/input/*)
    Input = 13,
    /// Framebuffer devices (/dev/fb*)
    Framebuffer = 29,
    /// UNIX98 PTY slaves (/dev/pts/*)
    PtySlave = 136,
    /// Universal Serial Bus (USB) character devices
    UsbChar = 180,
    /// Direct Rendering Manager (modern graphics) (/dev/dri/*)
    Drm = 226,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(usize)]
pub enum BlockMajorNum {
    /// RAM disk devices (/dev/ramX)
    RamDisk = 1,
    /// Loopback devices (/dev/loopX)
    Loop = 7,
    /// SCSI disk devices (/dev/sdX)
    ScsiDisk = 8,
    /// MMC/SD card block devices (/dev/mmcblkX)
    MmcBlock = 179,
    /// Virtual block devices (/dev/vdX)
    VirtBlock = 253,
    /// Extended block devices (NVMe, etc.)
    BlockExt = 259,

}

