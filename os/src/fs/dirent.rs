use alloc::string::String;

// pub const NAME_LIMIT: usize = 253;

/// 存储目录中的文件信息
#[repr(C)]
#[derive(Debug)]
pub struct Dirent {
    d_ino: u64,        // 索引节点号
    d_off: i64,        // 从 0 开始到下一个 dirent 的偏移
    d_reclen: u16,     // 当前 dirent 的长度
    d_type: u8,        // 文件类型
    pub d_name: [u8; 256], // 文件名
}

impl Dirent {
    pub fn new(name: [u8; 256], off: i64, ino: u64, dtype: u8, reclen: u16) -> Self {
        //对齐 align8
        // name += "\0";
        // let mut len = name.len() + 19;
        // let align = 8 - len % 8;
        // len += align;
        // for _ in 0..align {
        //     name.push('\0');
        // }
        Self {
            d_ino: ino,
            d_off: off,
            d_reclen: reclen,
            d_type: dtype,
            // d_name: {
            //     let mut tmp: [u8; 256] = [0; 256];
            //     tmp[..name.len()].copy_from_slice(name.as_bytes());
            //     tmp
            // },
            d_name: name
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
