//! EXT4 文件系统实现
//! 
//! 本模块提供了 EXT4 文件操作的实现，包括文件操作、目录操作和 inode 管理。
//! 它为 Rust 接口与底层 EXT4 C 实现之间提供了桥接。

use crate::bindings::*;
use alloc::{ffi::CString, vec::Vec};

/// 表示 EXT4 文件系统中的一个文件。
///
/// 该结构体维护文件描述符和与 EXT4 文件系统中文件相关的元数据。
/// 它提供了一个 Rust 友好的接口来访问底层 C 实现。
pub struct Ext4File {
    /// EXT4 C 实现中的文件描述符
    file_desc: ext4_file,
    /// 文件在文件系统中的路径
    file_path: CString,
    /// inode 的类型（文件、目录等）
    this_type: InodeTypes,
}

impl Ext4File {
    /// 根据路径和类型创建一个新的 EXT4 文件实例。
    ///
    /// # 参数
    ///
    /// * `path` - 文件在文件系统中的路径
    /// * `types` - 要创建的 inode 类型
    ///
    /// # 返回
    ///
    /// 返回一个新的 `Ext4File` 实例
    pub fn new(path: &str, types: InodeTypes) -> Self {
        Self {
            file_desc: ext4_file {
                mp: core::ptr::null_mut(),
                inode: 0,
                flags: 0,
                fsize: 0,
                fpos: 0,
            },
            file_path: CString::new(path).expect("CString::new Ext4File path failed"),
            this_type: types,
        }
    }

    /// 获取文件路径。
    ///
    /// # 返回
    ///
    /// 返回文件的路径
    pub fn get_path(&self) -> CString {
        self.file_path.clone()
    }

    /// 获取文件类型。
    ///
    /// # 返回
    ///
    /// 返回 inode 的类型
    pub fn get_type(&self) -> InodeTypes {
        self.this_type.clone()
    }

    /// 打开文件的函数。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    /// * `flags` - 打开标志（见下表）
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    ///
    /// # 标志表
    ///
    /// | 模式          | 标志                         |
    /// |--------------|-------------------------------|
    /// | r 或 rb      | O_RDONLY                     |
    /// | w 或 wb      | O_WRONLY|O_CREAT|O_TRUNC    |
    /// | a 或 ab      | O_WRONLY|O_CREAT|O_APPEND   |
    /// | r+ 或 rb+    | O_RDWR                      |
    /// | w+ 或 wb+    | O_RDWR|O_CREAT|O_TRUNC     |
    /// | a+ 或 ab+    | O_RDWR|O_CREAT|O_APPEND    |
    pub fn file_open(&mut self, path: &str, flags: u32) -> Result<usize, i32> {
        let c_path = CString::new(path).expect("CString::new failed");
        if c_path != self.get_path() {
            debug!(
                "Ext4File file_open, cur path={}, new path={}",
                self.file_path.to_str().unwrap(),
                path
            );
        }
        //let to_map = c_path.clone();
        let c_path = c_path.into_raw();
        let flags = Self::flags_to_cstring(flags);
        let flags = flags.into_raw();

        let r = unsafe { ext4_fopen(&mut self.file_desc, c_path, flags) };
        unsafe {
            // deallocate the CString
            drop(CString::from_raw(c_path));
            drop(CString::from_raw(flags));
        }
        if r != EOK as i32 {
            error!("ext4_fopen: {}, rc = {}", path, r);
            return Err(r);
        }
        //self.file_desc_map.insert(to_map, fd); // store c_path
        debug!("file_open {}, mp={:#x}", path, self.file_desc.mp as usize);
        Ok(EOK as usize)
    }

    /// 关闭文件。
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 0）
    /// * `Err(i32)` - 错误代码
    pub fn file_close(&mut self) -> Result<usize, i32> {
        if self.file_desc.mp != core::ptr::null_mut() {
            debug!("file_close {:?}", self.get_path());
            // self.file_cache_flush()?;
            unsafe {
                ext4_fclose(&mut self.file_desc);
            }
        }
        Ok(0)
    }
    /// 将标志转换为 C 字符串。
    ///
    /// # 参数
    ///
    /// * `flags` - 文件打开标志
    ///
    /// # 返回
    ///
    /// 返回转换后的 C 字符串
    pub fn flags_to_cstring(flags: u32) -> CString {
        let cstr = match flags {
            O_RDONLY => "rb",
            O_RDWR => "r+",
            0x241 => "wb", // O_WRONLY | O_CREAT | O_TRUNC
            0x441 => "ab", // O_WRONLY | O_CREAT | O_APPEND
            0x242 => "w+", // O_RDWR | O_CREAT | O_TRUNC
            0x442 => "a+", // O_RDWR | O_CREAT | O_APPEND
            _ => {
                warn!("Unknown File Open Flags: {:#x}", flags);
                "r+"
            }
        };
        debug!("flags_to_cstring: {}", cstr);
        CString::new(cstr).expect("CString::new OpenFlags failed")
    }

    /// Inode types:
    /// EXT4_DIRENTRY_UNKNOWN
    /// EXT4_DE_REG_FILE
    /// EXT4_DE_DIR
    /// EXT4_DE_CHRDEV
    /// EXT4_DE_BLKDEV
    /// EXT4_DE_FIFO
    /// EXT4_DE_SOCK
    /// EXT4_DE_SYMLINK
    ///
    /// Check if inode exists.
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    /// * `types` - inode 类型
    ///
    /// # 返回
    ///
    /// 如果 inode 存在，返回 `true`，否则返回 `false`
    pub fn check_inode_exist(&mut self, path: &str, types: InodeTypes) -> bool {
        let c_path = CString::new(path).expect("CString::new failed");
        let c_path = c_path.into_raw();
        let mtype = types.clone();
        let r = unsafe { ext4_inode_exist(c_path, types as i32) }; //eg: types: EXT4_DE_REG_FILE

        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r == EOK as i32 {
            debug!("{:?} {} Exist", mtype, path);
            true //Exist
        } else {
            debug!("{:?} {} No Exist. ext4_inode_exist rc = {}", mtype, path, r);
            false
        }
    }

    /// 重命名文件和目录。
    ///
    /// # 参数
    ///
    /// * `path` - 当前文件路径
    /// * `new_path` - 新文件路径
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn file_rename(&mut self, path: &str, new_path: &str) -> Result<usize, i32> {
        let c_path = CString::new(path).expect("CString::new failed");
        let c_path = c_path.into_raw();
        let c_new_path = CString::new(new_path).expect("CString::new failed");
        let c_new_path = c_new_path.into_raw();
        let r = unsafe { ext4_frename(c_path, c_new_path) };
        unsafe {
            drop(CString::from_raw(c_path));
            drop(CString::from_raw(c_new_path));
        }
        if r != EOK as i32 {
            error!("ext4_frename error: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 根据路径删除文件。
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn file_remove(&mut self, path: &str) -> Result<usize, i32> {
        debug!("file_remove {}", path);

        let c_path = CString::new(path).expect("CString::new failed");
        let c_path = c_path.into_raw();

        let r = unsafe { ext4_fremove(c_path) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if (r != EOK as i32) && (r != ENOENT as i32) {
            error!("ext4_fremove error: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 设置文件偏移量。
    ///
    /// # 参数
    ///
    /// * `offset` - 偏移量
    /// * `seek_type` - 寻址类型
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn file_seek(&mut self, offset: i64, seek_type: u32) -> Result<usize, i32> {
        let mut offset = offset;
        let size = self.file_size() as i64;

        if offset > size {
            warn!("Seek beyond the end of the file");
            offset = size;
        }

        let r = unsafe { ext4_fseek(&mut self.file_desc, offset, seek_type) };
        if r != EOK as i32 {
            error!("ext4_fseek: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 读取文件。
    ///
    /// # 参数
    ///
    /// * `buff` - 用于存储读取数据的缓冲区
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 读取的字节数
    /// * `Err(i32)` - 错误代码
    pub fn file_read(&mut self, buff: &mut [u8]) -> Result<usize, i32> {
        let mut rw_count = 0;
        let r = unsafe {
            ext4_fread(
                &mut self.file_desc,
                buff.as_mut_ptr() as _,
                buff.len(),
                &mut rw_count,
            )
        };

        if r != EOK as i32 {
            error!("ext4_fread: rc = {}", r);
            return Err(r);
        }

        debug!("file_read {:?}, len={}", self.get_path(), rw_count);

        Ok(rw_count)
    }

    /*
    pub fn file_close(&mut self, path: &str) -> Result<usize, i32> {
        let cstr_path = CString::new(path).unwrap();
        if let Some(mut fd) = self.file_desc_map.remove(&cstr_path) {
            unsafe {
                ext4_fclose(&mut fd);
            }
            Ok(0)
        } else {
            error!("Can't find file descriptor of {}", path);
            Err(-1)
        }
    }
    */

    /// 写入数据到文件。
    ///
    /// # 参数
    ///
    /// * `buf` - 包含要写入数据的缓冲区
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 写入的字节数
    /// * `Err(i32)` - 错误代码
    pub fn file_write(&mut self, buf: &[u8]) -> Result<usize, i32> {
        let mut rw_count = 0;
        let r = unsafe {
            ext4_fwrite(
                &mut self.file_desc,
                buf.as_ptr() as _,
                buf.len(),
                &mut rw_count,
            )
        };

        if r != EOK as i32 {
            error!("ext4_fwrite: rc = {}", r);
            return Err(r);
        }
        debug!("file_write {:?}, len={}", self.get_path(), rw_count);
        Ok(rw_count)
    }

    /// 截断文件到指定大小。
    ///
    /// # 参数
    ///
    /// * `size` - 新的文件大小
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn file_truncate(&mut self, size: u64) -> Result<usize, i32> {
        debug!("file_truncate to {}", size);
        let r = unsafe { ext4_ftruncate(&mut self.file_desc, size) };
        if r != EOK as i32 {
            error!("ext4_ftruncate: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }
    /// 获取文件大小。
    ///
    /// # 返回
    ///
    /// 返回文件的大小（字节）
    pub fn file_size(&mut self) -> u64 {
        //注，记得先 O_RDONLY 打开文件
        unsafe { ext4_fsize(&mut self.file_desc) }
    }


    /// 获取文件的状态信息。
    ///
    /// 返回一个包含文件元数据的 `ext4_inode_stat` 结构体,包括:
    /// - 文件大小
    /// - 所有者 ID
    /// - 组 ID 
    /// - 访问权限
    /// - 时间戳等
    ///
    /// # 返回
    ///
    /// * `Ok(ext4_inode_stat)` - 成功获取文件状态
    /// * `Err(i32)` - 错误代码
    pub fn fstat(&mut self) -> Result<ext4_inode_stat, i32> {
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        let mut stat = ext4_inode_stat::default();
        let r = unsafe { ext4_stat_get(c_path, &mut stat) };

        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            error!("ext4_stat_get: rc = {}", r);
            return Err(r);
        }
        Ok(stat)
    }

    /// 设置文件的时间戳。
    ///
    /// # 参数
    ///
    /// * `atime` - 可选的访问时间戳
    /// * `mtime` - 可选的修改时间戳  
    /// * `ctime` - 可选的创建时间戳
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功设置时间戳(返回 EOK)
    /// * `Err(i32)` - 错误代码
    pub fn set_time(
        &mut self,
        atime: Option<u64>,
        mtime: Option<u64>,
        ctime: Option<u64>,
    ) -> Result<usize, i32> {
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        let mut r = 0;
        if let Some(atime) = atime {
            r = unsafe { ext4_atime_set(c_path, atime) }
        }
        if let Some(mtime) = mtime {
            r = unsafe { ext4_mtime_set(c_path, mtime) }
        }
        if let Some(ctime) = ctime {
            r = unsafe { ext4_ctime_set(c_path, ctime) }
        }
        // unsafe { ext4_mode_set(c_path, mode) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            error!("ext4_time_set: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 刷新文件缓存。
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 0）
    /// * `Err(i32)` - 错误代码
    pub fn file_cache_flush(&mut self) -> Result<usize, i32> {
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        unsafe {
            let r = ext4_cache_flush(c_path);
            if r != EOK as i32 {
                error!("ext4_cache_flush: rc = {}", r);
                return Err(r);
            }
            drop(CString::from_raw(c_path));
        }
        Ok(0)
    }
    /// 获取文件模式。
    ///
    /// # 返回
    ///
    /// * `Ok(u32)` - 文件模式
    /// * `Err(i32)` - 错误代码
    pub fn file_mode_get(&mut self) -> Result<u32, i32> {
        // 0o777 (octal) == rwxrwxrwx
        let mut mode: u32 = 0o777;
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        let r = unsafe { ext4_mode_get(c_path, &mut mode) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            error!("ext4_mode_get: rc = {}", r);
            return Err(r);
        }
        debug!("Got file mode={:#x}", mode);
        Ok(mode)
    }
    /// 设置文件模式。
    ///
    /// # 参数
    ///
    /// * `mode` - 要设置的文件模式
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn file_mode_set(&mut self, mode: u32) -> Result<usize, i32> {
        debug!("file_mode_set to {:#x}", mode);

        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        let r = unsafe { ext4_mode_set(c_path, mode) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            error!("ext4_mode_set: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }
    /// 获取文件类型。
    ///
    /// # 返回
    ///
    /// 返回 inode 的类型
    pub fn file_type_get(&mut self) -> InodeTypes {
        let mode = self.file_mode_get().unwrap();
        // 0o777 (octal) == rwxrwxrwx
        // if filetype == EXT4_DE_SYMLINK;
        // mode = 0777;
        // mode |= EXT4_INODE_MODE_SOFTLINK;
        let cal: u32 = 0o777;
        let types = mode & (!cal);
        let itypes = match types {
            0x1000 => InodeTypes::EXT4_INODE_MODE_FIFO,
            0x2000 => InodeTypes::EXT4_INODE_MODE_CHARDEV,
            0x4000 => InodeTypes::EXT4_INODE_MODE_DIRECTORY,
            0x6000 => InodeTypes::EXT4_INODE_MODE_BLOCKDEV,
            0x8000 => InodeTypes::EXT4_INODE_MODE_FILE,
            0xA000 => InodeTypes::EXT4_INODE_MODE_SOFTLINK,
            0xC000 => InodeTypes::EXT4_INODE_MODE_SOCKET,
            0xF000 => InodeTypes::EXT4_INODE_MODE_TYPE_MASK,
            _ => {
                warn!("Unknown inode mode type {:x}", types);
                InodeTypes::EXT4_INODE_MODE_FILE
            }
        };
        debug!("Inode mode types: {:?}", itypes);

        itypes
    }

    //int ext4_owner_set(const char *path, uint32_t uid, uint32_t gid)
    pub fn set_owner(&mut self, uid: u32, gid: u32) -> Result<usize, i32> {
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();

        let r = unsafe { ext4_owner_set(c_path, uid, gid) };

        // unsafe { ext4_mode_set(c_path, mode) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            error!("ext4_owner_set: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 获取文件的硬链接数量
    ///
    /// # 返回值
    /// - `Ok(u32)`: 成功时返回文件的硬链接数量
    /// - `Err(i32)`: 失败时返回错误码
    pub fn links_cnt(&mut self) -> Result<u32, i32> {
        let mut cnt: u32 = 0;
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        let r = unsafe { ext4_get_links_cnt(c_path, &mut cnt) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            // error!("ext4_links_cnt_get: rc = {}", r);
            return Err(r);
        }
        Ok(cnt)
    }

    pub fn link(&self, newpath: &str) -> Result<usize, i32> {
        let old_path = self.file_path.clone();
        let new_path = CString::new(newpath).expect("[link] CString::new failed");

        let r = unsafe { ext4_flink(old_path.as_ptr(), new_path.as_ptr()) };
        if r != EOK as i32 {
            error!("ext4_dir_mk: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    

    /********* DIRECTORY OPERATION *********/

    /// 创建新目录。
    ///
    /// # 参数
    ///
    /// * `path` - 新目录的路径
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn dir_mk(&mut self, path: &str) -> Result<usize, i32> {
        debug!("directory create: {}", path);
        let c_path = CString::new(path).expect("CString::new failed");
        let c_path = c_path.into_raw();

        let r = unsafe { ext4_dir_mk(c_path) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if r != EOK as i32 {
            error!("ext4_dir_mk: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 重命名/移动目录。
    ///
    /// # 参数
    ///
    /// * `path` - 当前目录路径
    /// * `new_path` - 新目录路径
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn dir_mv(&mut self, path: &str, new_path: &str) -> Result<usize, i32> {
        debug!("directory move from {} to {}", path, new_path);

        let c_path = CString::new(path).expect("CString::new failed");
        let c_path = c_path.into_raw();
        let c_new_path = CString::new(new_path).expect("CString::new failed");
        let c_new_path = c_new_path.into_raw();

        let r = unsafe { ext4_dir_mv(c_path, c_new_path) };
        unsafe {
            drop(CString::from_raw(c_path));
            drop(CString::from_raw(c_new_path));
        }
        if r != EOK as i32 {
            error!("ext4_dir_mv: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 递归删除目录。
    ///
    /// # 参数
    ///
    /// * `path` - 要删除的目录路径
    ///
    /// # 返回
    ///
    /// * `Ok(usize)` - 成功（返回 EOK）
    /// * `Err(i32)` - 错误代码
    pub fn dir_rm(&mut self, path: &str) -> Result<usize, i32> {
        debug!("directory recursive remove: {}", path);

        let c_path = CString::new(path).expect("CString::new failed");
        let c_path = c_path.into_raw();

        let r = unsafe { ext4_dir_rm(c_path) };
        unsafe {
            drop(CString::from_raw(c_path));
        }
        if (r != EOK as i32) && (r != ENOENT as i32) {
            error!("ext4_fremove ext4_dir_rm: rc = {}", r);
            return Err(r);
        }
        Ok(EOK as usize)
    }

    /// 从指定偏移量开始读取目录内容。
    ///
    /// # 参数
    ///
    /// * `off` - 目录偏移量,用于指定从哪个位置开始读取
    ///
    /// # 返回
    ///
    /// * `Ok(Vec<OsDirent>)` - 成功,返回目录项列表
    /// * `Err(i32)` - 错误代码,如果不是目录则返回 22 (EINVAL)
    ///
    /// # 安全性
    ///
    /// 该函数包含不安全代码块,用于调用底层 ext4 文件系统接口
    pub fn read_dir_from(&self, _off: u64) -> Result<Vec<OsDirent>, i32> {
        if self.this_type != InodeTypes::EXT4_DE_DIR {
            return Err(22);
        }
        let c_path = self.file_path.clone();
        let c_path = c_path.into_raw();
        let mut d: ext4_dir = unsafe { core::mem::zeroed() };
        let mut entries: Vec<_> = Vec::new();

        unsafe {
            ext4_dir_open(&mut d, c_path);
            drop(CString::from_raw(c_path));
            let mut offset = 0;
            let mut de = ext4_dir_entry_next(&mut d);
            while !de.is_null() {
                let dentry = &(*de);
                // 创建 8 字节对齐的目录项
                let mut name = [0u8; 256];
                let name_len = dentry.name_length as usize;
                name[0..name_len].copy_from_slice(&dentry.name[0..name_len]);
                let mut len = name_len + 19;
                let align = 8 - len % 8;
                len += align;
                offset += dentry.entry_length;
                entries.push(OsDirent {
                    d_ino: dentry.inode as u64,
                    d_off: offset as i64,
                    d_reclen: len as u16,
                    d_type: dentry.inode_type,
                    d_name: name,
                });
                de = ext4_dir_entry_next(&mut d);
            }
            ext4_dir_close(&mut d);
        }
        Ok(entries)
    }
}

/*
pub enum OpenFlags {
O_RDONLY = 0,
O_WRONLY = 0x1,
O_RDWR = 0x2,
O_CREAT = 0x40,
O_TRUNC = 0x200,
O_APPEND = 0x400,
}
*/

/// 表示 EXT4 文件系统中的各种 inode 类型。
///
/// 此枚举包括目录项类型和 inode 模式，为 EXT4 文件系统条目提供了完整的类型系统。
#[derive(PartialEq, Clone, Debug)]
pub enum InodeTypes {
    /// 未知文件类型
    EXT4_DE_UNKNOWN = 0,
    /// 常规文件
    EXT4_DE_REG_FILE = 1,
    /// 目录
    EXT4_DE_DIR = 2,
    /// 字符设备
    EXT4_DE_CHRDEV = 3,
    /// 块设备
    EXT4_DE_BLKDEV = 4,
    /// FIFO
    EXT4_DE_FIFO = 5,
    /// 套接字
    EXT4_DE_SOCK = 6,
    /// 符号链接
    EXT4_DE_SYMLINK = 7,

    // inode 模式
    EXT4_INODE_MODE_FIFO = 0x1000,
    EXT4_INODE_MODE_CHARDEV = 0x2000,
    EXT4_INODE_MODE_DIRECTORY = 0x4000,
    EXT4_INODE_MODE_BLOCKDEV = 0x6000,
    EXT4_INODE_MODE_FILE = 0x8000,
    EXT4_INODE_MODE_SOFTLINK = 0xA000,
    EXT4_INODE_MODE_SOCKET = 0xC000,
    EXT4_INODE_MODE_TYPE_MASK = 0xF000,
}

impl From<usize> for InodeTypes {
    /// 将数字值转换为相应的 InodeType。
    ///
    /// # 参数
    ///
    /// * `num` - 要转换的数字值
    ///
    /// # 返回
    ///
    /// 返回相应的 InodeType，如果未知则返回 EXT4_DE_UNKNOWN
    fn from(num: usize) -> InodeTypes {
        match num {
            0 => InodeTypes::EXT4_DE_UNKNOWN,
            1 => InodeTypes::EXT4_DE_REG_FILE,
            2 => InodeTypes::EXT4_DE_DIR,
            3 => InodeTypes::EXT4_DE_CHRDEV,
            4 => InodeTypes::EXT4_DE_BLKDEV,
            5 => InodeTypes::EXT4_DE_FIFO,
            6 => InodeTypes::EXT4_DE_SOCK,
            7 => InodeTypes::EXT4_DE_SYMLINK,
            0x1000 => InodeTypes::EXT4_INODE_MODE_FIFO,
            0x2000 => InodeTypes::EXT4_INODE_MODE_CHARDEV,
            0x4000 => InodeTypes::EXT4_INODE_MODE_DIRECTORY,
            0x6000 => InodeTypes::EXT4_INODE_MODE_BLOCKDEV,
            0x8000 => InodeTypes::EXT4_INODE_MODE_FILE,
            0xA000 => InodeTypes::EXT4_INODE_MODE_SOFTLINK,
            0xC000 => InodeTypes::EXT4_INODE_MODE_SOCKET,
            0xF000 => InodeTypes::EXT4_INODE_MODE_TYPE_MASK,
            _ => {
                warn!("Unknown ext4 inode type: {}", num);
                InodeTypes::EXT4_DE_UNKNOWN
            }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct OsDirent {
    pub d_ino: u64,        // 索引节点号
    pub d_off: i64,        // 从 0 开始到下一个 dirent 的偏移
    pub d_reclen: u16,     // 当前 dirent 的长度
    pub d_type: u8,        // 文件类型
    pub d_name: [u8; 256], // 文件名
}
