use lwext4_rust::bindings::ext4_inode_stat;

use crate::{config::BLOCK_SIZE, sync::TimeSepc};

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
    pub st_blksize: i64, // 文件系统 I/O 的块大小
    pub __pad2: i32,
    pub st_blocks: u64,     // 分配的 512B 块数
    pub st_atime_sec: i64,  // 上次访问时间
    pub st_atime_nsec: i64, // 上次访问时间（纳秒精度）
    pub st_mtime_sec: i64,  // 上次修改时间
    pub st_mtime_nsec: i64, // 上次修改时间（纳秒精度）
    pub st_ctime_sec: i64,  // 上次状态变化的时间
    pub st_ctime_nsec: i64, // 上次状态变化的时间（纳秒精度）
    pub __unused: [u32; 2],
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
            __unused: [0; 2],
        }
    }

    pub fn init(&mut self, st_size: i64, st_blksize: i64, st_blocks: u64) {
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
pub(crate) fn as_inode_stat(stat: ext4_inode_stat, atime: TimeSepc, mtime: TimeSepc, ctime: TimeSepc) -> Kstat {
    Kstat {
        st_dev: stat.st_dev as u32,
        st_ino: stat.st_ino as u64,
        st_mode: stat.st_mode,
        st_nlink: stat.st_nlink,
        st_uid: stat.st_uid,
        st_gid: stat.st_gid,
        st_rdev: 0, // 如果需要，可以根据具体情况设置
        __pad: 0,   // 填充字段
        st_size: stat.st_size as i64,
        st_blksize: BLOCK_SIZE as i64,
        __pad2: 0,  // 填充字段
        st_blocks: stat.st_blocks as u64,
        st_atime_sec: atime.tv_sec as i64,
        st_atime_nsec: atime.tv_nsec as i64,
        st_mtime_sec: mtime.tv_sec as i64,
        st_mtime_nsec: mtime.tv_nsec as i64,
        st_ctime_sec: ctime.tv_sec as i64,
        st_ctime_nsec: ctime.tv_nsec as i64,
        __unused: [0; 2], // 填充字段
    }
}