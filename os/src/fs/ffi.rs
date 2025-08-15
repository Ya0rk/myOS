use alloc::vec::Vec;
use bitflags::*;
use lwext4_rust::Ext4InodeType;

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
        /// 目前用于socket文件，检测accept是否为非阻塞
        const O_NONBLOCK    = 0o4000;
        const O_DSYNC       = 0o10000;
        const O_SYNC        = 0o4010000;
        const O_RSYNC       = 0o4010000;
        const O_DIRECTORY   = 0o200000;
        const O_NOFOLLOW    = 0o400000;
        /// set close_on_exec
        /// O_CLOEXEC是一个open函数的选项，它决定了新打开的文件描述符是否会在调用execve后自动关闭
        const O_CLOEXEC     = 0o2000000;

        const O_ASYNC       = 0o20000;
        const O_DIRECT      = 0o40000;
        const O_LARGEFILE   = 0o100000;
        const O_NOATIME     = 0o1000000;
        const O_PATH        = 0o10000000;
        const O_TMPFILE     = 0o20200000;
        const FCNTL_MASK = Self::O_RDONLY.bits()
                          | Self::O_WRONLY.bits()
                          | Self::O_RDWR.bits()
                          | Self::O_CREAT.bits()
                          | Self::O_EXCL.bits()
                          | Self::O_NOCTTY.bits()
                          | Self::O_TRUNC.bits();
    }

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

    pub struct RenameFlags: u32 {
        const RENAME_NOREPLACE	=   1 << 0;	// Don't overwrite target
        const RENAME_EXCHANGE	= 	1 << 1;	// Exchange source and dest
        const RENAME_WHITEOUT	= 	1 << 2;	// Whiteout source
    }

    pub struct StxMask: u32 {
        /// Request file type (stx_mode & S_IFMT)
        const STATX_TYPE          = 0x00000001;
        /// Request file permissions (stx_mode & ~S_IFMT)
        const STATX_MODE          = 0x00000002;
        /// Request number of hard links (stx_nlink)
        const STATX_NLINK         = 0x00000004;
        /// Request owner UID (stx_uid)
        const STATX_UID           = 0x00000008;
        /// Request group GID (stx_gid)
        const STATX_GID           = 0x00000010;
        /// Request last access time (stx_atime)
        const STATX_ATIME         = 0x00000020;
        /// Request last modification time (stx_mtime)
        const STATX_MTIME         = 0x00000040;
        /// Request last status change time (stx_ctime)
        const STATX_CTIME         = 0x00000080;
        /// Request inode number (stx_ino)
        const STATX_INO           = 0x00000100;
        /// Request file size in bytes (stx_size)
        const STATX_SIZE          = 0x00000200;
        /// Request allocated blocks count (stx_blocks)
        const STATX_BLOCKS        = 0x00000400;
        /// Combination of all basic stat fields (equivalent to traditional `stat` struct)
        const STATX_BASIC_STATS   = 0x000007FF;
        /// Request file creation/birth time (stx_btime)
        const STATX_BTIME         = 0x00000800;
        /// Request mount ID (stx_mnt_id, since Linux 5.8)
        const STATX_MNT_ID        = 0x00001000;
        /// Reserved for future expansion
        const STATX__RESERVED     = 0x80000000;
        /// Request all fields (equivalent to `statx` with all flags set)
        const STATX_ALL           = 0x00000fff;
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

    pub fn node_type(&self) -> Ext4InodeType {
        if self.contains(OpenFlags::O_DIRECTORY) {
            Ext4InodeType::EXT4_DE_DIR
        } else {
            Ext4InodeType::EXT4_DE_REG_FILE
        }
    }
}

//
pub const MOUNTS: &str = " ext4 / ext4 rw 0 0\n";
pub const MEMINFO: &str = r"
MemTotal:        8135008 kB
MemFree:         1478028 kB
MemAvailable:    3026828 kB
Buffers:          352200 kB
Cached:          1363936 kB
SwapCached:            0 kB
Active:           937352 kB
Inactive:        4971712 kB
Active(anon):       3104 kB
Inactive(anon):  4193520 kB
Active(file):     934248 kB
Inactive(file):   778192 kB
Unevictable:           0 kB
Mlocked:               0 kB
SwapTotal:             0 kB
SwapFree:              0 kB
Dirty:               268 kB
Writeback:             0 kB
AnonPages:       4189292 kB
Mapped:           340688 kB
Shmem:              3696 kB
KReclaimable:     104972 kB
Slab:             215164 kB
SReclaimable:     104972 kB
SUnreclaim:       110192 kB
KernelStack:        9856 kB
PageTables:        42096 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:     4067504 kB
Committed_AS:    2763488 kB
VmallocTotal:   34359738367 kB
VmallocUsed:      124444 kB
VmallocChunk:          0 kB
Percpu:             7680 kB
AnonHugePages:    376832 kB
ShmemHugePages:        0 kB
ShmemPmdMapped:        0 kB
FileHugePages:         0 kB
FilePmdMapped:         0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
DirectMap4k:      107520 kB
DirectMap2M:     7231488 kB
DirectMap1G:    10485760 kB
";

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum InodeType {
    Unknown = 0o0,
    /// FIFO (named pipe)
    Fifo = 0o1,
    /// 字符设备
    CharDevice = 0o2,
    /// 目录
    Dir = 0o4,
    /// 块设备
    BlockDevice = 0o6,
    /// 文件
    File = 0o10,
    /// 符号链接文件
    SymLink = 0o12,
    /// Socket
    Socket = 0o14,
}

impl InodeType {
    /// Tests whether this node type represents a regular file.
    pub const fn is_device(self) -> bool {
        match self {
            Self::BlockDevice | Self::CharDevice => true,
            _ => false,
        }
    }
    pub const fn is_file(self) -> bool {
        matches!(self, Self::File)
    }
    /// Tests whether this node type represents a directory.
    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
    /// Tests whether this node type represents a symbolic link.
    pub const fn is_symlink(self) -> bool {
        matches!(self, Self::SymLink)
    }
    /// Returns `true` if this node type is a block device.
    pub const fn is_block_device(self) -> bool {
        matches!(self, Self::BlockDevice)
    }
    /// Returns `true` if this node type is a char device.
    pub const fn is_char_device(self) -> bool {
        matches!(self, Self::CharDevice)
    }
    /// Returns `true` if this node type is a fifo.
    pub const fn is_fifo(self) -> bool {
        matches!(self, Self::Fifo)
    }
    /// Returns `true` if this node type is a socket.
    pub const fn is_socket(self) -> bool {
        matches!(self, Self::Socket)
    }
}

pub fn as_ext4_de_type(types: InodeType) -> Ext4InodeType {
    match types {
        InodeType::BlockDevice => Ext4InodeType::EXT4_DE_BLKDEV,
        InodeType::CharDevice => Ext4InodeType::EXT4_DE_CHRDEV,
        InodeType::Dir => Ext4InodeType::EXT4_DE_DIR,
        InodeType::Fifo => Ext4InodeType::EXT4_DE_FIFO,
        InodeType::File => Ext4InodeType::EXT4_DE_REG_FILE,
        InodeType::Socket => Ext4InodeType::EXT4_DE_SOCK,
        InodeType::SymLink => Ext4InodeType::EXT4_DE_SYMLINK,
        InodeType::Unknown => Ext4InodeType::EXT4_DE_UNKNOWN,
    }
}

impl From<Ext4InodeType> for InodeType {
    fn from(types: Ext4InodeType) -> Self {
        match types {
            Ext4InodeType::EXT4_DE_FIFO => InodeType::Fifo,
            Ext4InodeType::EXT4_DE_CHRDEV => InodeType::CharDevice,
            Ext4InodeType::EXT4_DE_DIR => InodeType::Dir,
            Ext4InodeType::EXT4_DE_BLKDEV => InodeType::BlockDevice,
            Ext4InodeType::EXT4_DE_REG_FILE => InodeType::File,
            Ext4InodeType::EXT4_DE_SYMLINK => InodeType::SymLink,
            Ext4InodeType::EXT4_DE_SOCK => InodeType::Socket,
            Ext4InodeType::EXT4_INODE_MODE_DIRECTORY => InodeType::Dir,
            Ext4InodeType::EXT4_INODE_MODE_FILE => InodeType::File,
            _ => unreachable!(),
        }
    }
}

pub fn as_inode_type(types: Ext4InodeType) -> InodeType {
    match types {
        Ext4InodeType::EXT4_DE_FIFO => InodeType::Fifo,
        Ext4InodeType::EXT4_DE_CHRDEV => InodeType::CharDevice,
        Ext4InodeType::EXT4_DE_DIR => InodeType::Dir,
        Ext4InodeType::EXT4_DE_BLKDEV => InodeType::BlockDevice,
        Ext4InodeType::EXT4_DE_REG_FILE => InodeType::File,
        Ext4InodeType::EXT4_DE_SYMLINK => InodeType::SymLink,
        Ext4InodeType::EXT4_DE_SOCK => InodeType::Socket,

        Ext4InodeType::EXT4_INODE_MODE_DIRECTORY => InodeType::Dir,
        Ext4InodeType::EXT4_INODE_MODE_FILE => InodeType::File,

        _ => {
            log::warn!("unknown file type: {:?}", types);
            unreachable!()
        }
    }
}

impl From<InodeType> for Ext4InodeType {
    fn from(types: InodeType) -> Self {
        match types {
            InodeType::Fifo => Ext4InodeType::EXT4_DE_FIFO,
            InodeType::CharDevice => Ext4InodeType::EXT4_DE_CHRDEV,
            InodeType::Dir => Ext4InodeType::EXT4_DE_DIR,
            InodeType::BlockDevice => Ext4InodeType::EXT4_DE_BLKDEV,
            InodeType::File => Ext4InodeType::EXT4_DE_REG_FILE,
            InodeType::SymLink => Ext4InodeType::EXT4_DE_SYMLINK,
            InodeType::Socket => Ext4InodeType::EXT4_DE_SOCK,
            _ => unreachable!(),
        }
    }
}

pub const S_IFCHR: u32 = 0o0020000;
pub const S_IFDIR: u32 = 0o0040000;
pub const S_IFBLK: u32 = 0o0060000;

use bitflags::bitflags;
use core::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct StMode {
    pub mode: ModeFlag,
}

impl fmt::Debug for StMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StMode")
            .field("mode", &format_args!("{}", self.mode))
            .finish()
    }
}

impl StMode {
    pub fn new(mode: ModeFlag) -> Self {
        Self { mode }
    }

    pub fn is_regular(&self) -> bool {
        self.mode.contains(ModeFlag::S_IFREG)
    }
    pub fn is_dir(&self) -> bool {
        self.mode.contains(ModeFlag::S_IFDIR)
    }
    pub fn is_symlink(&self) -> bool {
        self.mode.contains(ModeFlag::S_IFLNK)
    }
    pub fn is_executable(&self) -> bool {
        self.mode
            .intersects(ModeFlag::S_IXUSR | ModeFlag::S_IXGRP | ModeFlag::S_IXOTH)
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct ModeFlag: u32 {
        // ===== 文件类型（高位掩码） =====
        /// 文件类型掩码（用来屏蔽出 S_IFSOCK~S_IFIFO 这几类类型位）
        const S_IFMT   = 0o170000;
        /// 套接字（socket）
        const S_IFSOCK = 0o140000;
        /// 符号链接（symbolic link）
        const S_IFLNK  = 0o120000;
        /// 普通文件（regular file）
        const S_IFREG  = 0o100000;
        /// 块设备（block device）
        const S_IFBLK  = 0o060000;
        /// 目录（directory）
        const S_IFDIR  = 0o040000;
        /// 字符设备（character device）
        const S_IFCHR  = 0o020000;
        /// FIFO / 管道（named pipe）
        const S_IFIFO  = 0o010000;

        // ===== 特殊权限位 =====
        /// 设置用户 ID（set-user-ID on execution）
        const S_ISUID  = 0o004000;
        /// 设置组 ID（set-group-ID on execution）
        const S_ISGID  = 0o002000;
        /// 粘滞位（sticky bit）
        /// 对目录：防止非目录所有者删除或重命名目录下文件
        const S_ISVTX  = 0o001000;

        // ===== 用户权限位（owner） =====
        /// 用户读权限（read permission, owner）
        const S_IRUSR  = 0o000400;
        /// 用户写权限（write permission, owner）
        const S_IWUSR  = 0o000200;
        /// 用户执行权限（execute/search permission, owner）
        const S_IXUSR  = 0o000100;

        // ===== 组权限位（group） =====
        /// 组读权限（read permission, group）
        const S_IRGRP  = 0o000040;
        /// 组写权限（write permission, group）
        const S_IWGRP  = 0o000020;
        /// 组执行权限（execute/search permission, group）
        const S_IXGRP  = 0o000010;

        // ===== 其他用户权限位（others） =====
        /// 其他用户读权限（read permission, others）
        const S_IROTH  = 0o000004;
        /// 其他用户写权限（write permission, others）
        const S_IWOTH  = 0o000002;
        /// 其他用户执行权限（execute/search permission, others）
        const S_IXOTH  = 0o000001;
    }
}

impl fmt::Display for ModeFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter_names()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
            .join(" | ")
            .fmt(f)
    }
}

impl From<u32> for StMode {
    fn from(bits: u32) -> Self {
        Self {
            mode: ModeFlag::from_bits_truncate(bits),
        }
    }
}

impl From<ModeFlag> for StMode {
    fn from(mode: ModeFlag) -> Self {
        Self { mode }
    }
}

impl From<StMode> for u32 {
    fn from(st: StMode) -> Self {
        st.mode.bits()
    }
}
