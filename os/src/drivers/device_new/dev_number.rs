#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(usize)]
pub enum MajorNumber {
    /// Memory devices (/dev/null, /dev/zero, /dev/random, etc.)
    Mem = 1,
    /// TTY devices (serial ports, virtual consoles)
    Tty = 4,
    /// TTY auxiliary devices
    TtyAux = 5,
    /// Loopback devices (/dev/loopX)
    Loop = 7,
    /// SCSI disk devices (/dev/sdX)
    ScsiDisk = 8,
    /// Miscellaneous character devices (/dev/fuse, /dev/psaux, etc.)
    Misc = 10,
    /// Input devices (/dev/input/*)
    Input = 13,
    /// Framebuffer devices (/dev/fb*)
    Framebuffer = 29,
    /// UNIX98 PTY slaves (/dev/pts/*)
    PtySlave = 136,
    /// MMC/SD card block devices (/dev/mmcblkX)
    MmcBlock = 179,
    /// Universal Serial Bus (USB) character devices
    UsbChar = 180,
    /// Direct Rendering Manager (modern graphics) (/dev/dri/*)
    Drm = 226,
    /// Extended block devices (NVMe, etc.)
    BlockExt = 259,
}

