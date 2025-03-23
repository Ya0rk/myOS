use bitflags::*;

use super::InodeType;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    ///Open file flags
    pub struct OpenFlags: i32 {
        // reserve 3 bits for the access mode
        // NOTE: bitflags do not encourage zero bit flag, we should not directly check `O_RDONLY`
        const O_RDONLY      = 0;
        const O_WRONLY      = 1;
        /// 读取和写入
        const O_RDWR        = 2;
        const O_ACCMODE     = 3;
        /// 如果文件不存在，则创建文件。需要指定文件权限（如 0644）
        const O_CREAT       = 0o100;
        /// 与 O_CREAT 一起使用，确保文件是新建的（如果文件已存在则失败）
        const O_EXCL        = 0o200;
        const O_NOCTTY      = 0o400;
        /// 如果文件已存在，将其长度截断为 0（清空文件内容）
        const O_TRUNC       = 0o1000;
        /// 在写入时追加到文件末尾，而不是覆盖文件内容
        const O_APPEND      = 0o2000;
        /// 以非阻塞模式打开文件（通常用于设备文件或管道）
        const O_NONBLOCK    = 0o4000;
        const O_DSYNC       = 0o10000;
        const O_SYNC        = 0o4010000;
        const O_RSYNC       = 0o4010000;
        const O_DIRECTORY   = 0o200000;
        const O_NOFOLLOW    = 0o400000;
        /// set close_on_exec 
        const O_CLOEXEC     = 0o2000000;

        const O_ASYNC       = 0o20000;
        const O_DIRECT      = 0o40000;
        const O_LARGEFILE   = 0o100000;
        const O_NOATIME     = 0o1000000;
        const O_PATH        = 0o10000000;
        const O_TMPFILE     = 0o20200000;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else {
            (self.readable(), self.writable())
        }
    }

    pub fn readable(&self) -> bool {
        self.contains(OpenFlags::O_RDONLY) || self.contains(OpenFlags::O_RDWR)
    }

    pub fn writable(&self) -> bool {
        self.contains(OpenFlags::O_WRONLY) || self.contains(OpenFlags::O_RDWR)
    }

    pub fn node_type(&self) -> InodeType {
        if self.contains(OpenFlags::O_DIRECTORY) {
            InodeType::Dir
        } else {
            InodeType::File
        }
    }
}

bitflags! {
    pub struct UmountFlags: u32 {
        const MNT_FORCE = 0x0001;
        const MNT_DETACH = 0x0002;
        const MNT_EXPIRE = 0x0004;
        const UMOUNT_NOFOLLOW = 0x0008;
    }

    pub struct MountFlags: u32 {
        const MS_RDONLY = 1;
        const MS_NOSUID = 2;
        const MS_NODEV = 4;
        const MS_NOEXEC = 8;
        const MS_SYNCHRONOUS = 16;
        const MS_REMOUNT = 32;
        const MS_MOVE = 8192;
        const MS_BIND = 4096;
    }
}


//
pub const MOUNTS: &str = " ext4 / ext4 rw 0 0\n";
pub const MEMINFO: &str = r"
MemTotal:         944564 kB
MemFree:          835248 kB
MemAvailable:     873464 kB
Buffers:            6848 kB
Cached:            36684 kB
SwapCached:            0 kB
Active:            19032 kB
Inactive:          32676 kB
Active(anon):        128 kB
Inactive(anon):     8260 kB
Active(file):      18904 kB
Inactive(file):    24416 kB
Unevictable:           0 kB
Mlocked:               0 kB
SwapTotal:             0 kB
SwapFree:              0 kB
Dirty:                 0 kB
Writeback:             0 kB
AnonPages:          8172 kB
Mapped:            16376 kB
Shmem:               216 kB
KReclaimable:       9960 kB
Slab:              17868 kB
SReclaimable:       9960 kB
SUnreclaim:         7908 kB
KernelStack:        1072 kB
PageTables:          600 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:      472280 kB
Committed_AS:      64684 kB
VmallocTotal:   67108863 kB
VmallocUsed:       15740 kB
VmallocChunk:          0 kB
Percpu:              496 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
";
pub const ADJTIME: &str = "0.000000 0.000000 UTC\n";
pub const LOCALTIME: &str =
    "lrwxrwxrwx 1 root root 33 11月 18  2023 /etc/localtime -> /usr/share/zoneinfo/Asia/Shanghai\n";