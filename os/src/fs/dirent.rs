use core::fmt::{Debug, Display};

use alloc::{string::String, vec::Vec};

use super::path;

// pub const NAME_LIMIT: usize = 253;

/// 存储目录中的文件信息
#[repr(C)]
// #[derive(Debug)]
pub struct Dirent {
    d_ino: u64,            // 索引节点号
    d_off: i64,            // 从 0 开始到下一个 dirent 的偏移
    d_reclen: u16,         // 当前 dirent 的长度
    d_type: u8,            // 文件类型
    pub d_name: [u8; 256], // 文件名
}

impl Dirent {
    pub fn new(name: [u8; 256], off: i64, ino: u64, dtype: u8, reclen: u16) -> Self {
        //对齐 align8
        Self {
            d_ino: ino,
            d_off: off,
            d_reclen: reclen,
            d_type: dtype,
            d_name: name,
        }
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.d_reclen as usize
    }
    #[inline(always)]
    pub fn off(&self) -> usize {
        self.d_off as usize
    }

    pub fn as_bytes(&self) -> &[u8] {
        //特殊处理，因为名字数组大小不定
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, self.len()) }
    }
}

impl Display for Dirent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = String::from_utf8_lossy(&self.d_name);
        name.replace("\0", "");
        write!(
            f,
            "d_ino: {}, d_off: {}, d_reclen: {}, d_type: {}, d_name: {}",
            self.d_ino, self.d_off, self.d_reclen, self.d_type, name
        )
    }
}

impl Debug for Dirent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = String::from_utf8_lossy(&self.d_name);
        write!(
            f,
            "Dirent {{ d_ino: {}, d_off: {}, d_reclen: {}, d_type: {}, d_name: {} }}",
            self.d_ino, self.d_off, self.d_reclen, self.d_type, name
        )
    }
}

/// (path, ino, d_type)
///
///    DT_UNKNOWN = 0,
///
///    DT_FIFO = 1,
///
///    DT_CHR = 2,
///
///    DT_DIR = 4,
///
///    DT_BLK = 6,
///
///    DT_REG = 8,
///
///    DT_LNK = 10,
///
///    DT_SOCK = 12,
///
///    DT_WHT = 14
///  };
pub fn build_dirents(entries: Vec<(&str, u64, u8)>) -> Vec<Dirent> {
    let mut dirents = Vec::new();
    let mut current_off: i64 = 0;
    let mut offset: i64 = 0;
    for (path, ino, d_type) in entries {
        let mut name = [0u8; 256];
        let name_len = path.len();
        // 将name: &str转换为字节数组
        name[..name_len].copy_from_slice(&path.as_bytes()[..name_len]);
        let mut len = name_len + 19;
        let align = 8 - len % 8;
        len += align;
        offset += len as i64;
        let dirent = Dirent::new(name, offset, ino, d_type, len as u16);
        dirents.push(dirent);
    }
    dirents
}
