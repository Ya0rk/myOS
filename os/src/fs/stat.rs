use lwext4_rust::bindings::ext4_inode_stat;

use crate::hal::config::BLOCK_SIZE;
use crate::sync::TimeSpec;
#[derive(Debug)]
#[repr(C)]
pub struct Kstat {
    pub st_dev: u32,   // 包含文件的设备 ID
    pub st_ino: u64,   // 索引节点号
    pub st_mode: u32,  // 文件类型和模式
    pub st_nlink: u32, // 硬链接数
    pub st_uid: u32,   // 所有者的用户 ID
    pub st_gid: u32,   // 所有者的组 ID
    pub st_rdev: u32,  // 设备 ID（如果是特殊文件）
    pub __pad: u64,
    pub st_size: i64,    // 总大小，以字节为单位
    pub st_blksize: i32, // 文件系统 I/O 的块大小
    pub __pad2: i32,
    pub st_blocks: i64,     // 分配的 512B 块数
    pub st_atime_sec: isize,  // 上次访问时间
    pub st_atime_nsec: isize, // 上次访问时间（纳秒精度）
    pub st_mtime_sec: isize,  // 上次修改时间
    pub st_mtime_nsec: isize, // 上次修改时间（纳秒精度）
    pub st_ctime_sec: isize,  // 上次状态变化的时间
    pub st_ctime_nsec: isize, // 上次状态变化的时间（纳秒精度）
}

impl Kstat {
    pub fn new() -> Self {
        Self {
            st_dev: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: 0,
            st_blksize: 0,
            __pad2: 0,
            st_blocks: 0,
            st_atime_sec: 0,
            st_atime_nsec: 0,
            st_mtime_sec: 0,
            st_mtime_nsec: 0,
            st_ctime_sec: 0,
            st_ctime_nsec: 0,
            // __unused: [0; 2],
        }
    }

    pub fn init(&mut self, st_size: i64, st_blksize: i32, st_blocks: i64) {
        self.st_nlink = 1;
        self.st_size = st_size;
        self.st_blksize = st_blksize;
        self.st_blocks = st_blocks;
    }

    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size) }
    }
}


/// 将 ext4_inode_stat 转换为 Kstat
#[allow(unused)]
pub(crate) fn as_inode_stat(stat: ext4_inode_stat, atime: TimeSpec, mtime: TimeSpec, ctime: TimeSpec, size: usize) -> Kstat {
    Kstat {
        st_dev: stat.st_dev as u32,
        st_ino: stat.st_ino as u64,
        st_mode: stat.st_mode,
        st_nlink: stat.st_nlink,
        st_uid: stat.st_uid,
        st_gid: stat.st_gid,
        st_rdev: 0, // 如果需要，可以根据具体情况设置
        __pad: 0,   // 填充字段
        st_size: size as i64,
        st_blksize: BLOCK_SIZE as i32,
        __pad2: 0,  // 填充字段
        st_blocks: stat.st_blocks as i64,
        st_atime_sec: atime.tv_sec as isize,
        st_atime_nsec: atime.tv_nsec as isize,
        st_mtime_sec: mtime.tv_sec as isize,
        st_mtime_nsec: mtime.tv_nsec as isize,
        st_ctime_sec: ctime.tv_sec as isize,
        st_ctime_nsec: ctime.tv_nsec as isize,
        // __unused: [0; 2], // 填充字段
    }
}



/// man 2 statx 中的定义
///
// The file timestamps are structures of the following type:
///
///       struct statx_timestamp {
///           __s64 tv_sec;    /* Seconds since the Epoch (UNIX time) */
///           __u32 tv_nsec;   /* Nanoseconds since tv_sec */
///       };
#[derive(Debug)]
#[repr(C)]
pub struct Statx_timestamp {
    tv_sec: i64,
    tv_nsec: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Statx {
    /// mask of bits indicating
    stx_mask: u32,
    stx_blksize: u32,
    stx_attributes: u64,
    stx_nlink: u32,
    stx_uid: u32,
    stx_gid: u32,
    stx_mode: u16,
    stx_ino: u64,
    stx_size: u64,
    stx_blocks: u64,
    stx_attributes_mask: u64, 

    /// 最后访问时间
    stx_atime: Statx_timestamp,
    /// 创建时间
    stx_btime: Statx_timestamp,
    /// 最后状态变更时间
    stx_ctime: Statx_timestamp,
    /// 最后修改时间
    stx_mtime: Statx_timestamp,
    
    /// 主设备ID 
    stx_dev_major: u32,
    // 设备示例ID
    stx_dev_minor: u32,
}

impl From<Kstat> for Statx {
    fn from(kstat: Kstat) -> Self {
        Self {
            stx_mask: 0,
            stx_blksize: kstat.st_blksize as u32,
            stx_attributes: 0,
            stx_nlink: kstat.st_nlink,
            stx_uid: kstat.st_uid,
            stx_gid: kstat.st_gid,
            stx_mode: kstat.st_mode as u16,
            stx_ino: kstat.st_ino,
            stx_size: kstat.st_size as u64,
            stx_blocks: kstat.st_blocks as u64,
            stx_attributes_mask: 0,

            // 时间戳
            stx_atime: Statx_timestamp {
                tv_sec: kstat.st_atime_sec as i64,
                tv_nsec: kstat.st_atime_nsec as u32,
            },
            stx_btime: Statx_timestamp {
                tv_sec: 0, // 创建时间未知
                tv_nsec: 0,
            },
            stx_ctime: Statx_timestamp {
                tv_sec: kstat.st_ctime_sec as i64,
                tv_nsec: kstat.st_ctime_nsec as u32,
            },
            stx_mtime: Statx_timestamp {
                tv_sec: kstat.st_mtime_sec as i64,
                tv_nsec: kstat.st_mtime_nsec as u32,
            },
            stx_dev_major: 0,
            stx_dev_minor: 0,
        }
    }
}

impl Statx {

    pub fn set_mask(&mut self, mask: u32) {
        self.stx_mask = mask;
    }

    pub fn as_bytes(&self) -> &[u8] {
        let size = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as usize as *const u8, size) }
    }

}