use crate::{
    fs::{
        ffi::RenameFlags, Dirent, FileTrait, InodeTrait, InodeType, Kstat, OpenFlags, S_IFBLK,
        S_IFCHR,
    },
    mm::{
        page::Page,
        user_ptr::{user_ref_mut, user_slice_mut},
        UserBuffer,
    },
    sync::{SpinNoIrqLock, TimeStamp},
    utils::{Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use async_trait::async_trait;
use log::{error, info};
use spin::Spin;

lazy_static! {
    pub static ref DEVLOOP: Arc<DevLoop> = DevLoop::new();
}

pub struct DevLoop {
    inode: Arc<DevLoopInode>,
}

impl DevLoop {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inode: Arc::new(DevLoopInode::new()),
        })
    }
}

#[async_trait]
impl FileTrait for DevLoop {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    async fn read(&self, mut user_buf: &mut [u8]) -> SysResult<usize> {
        let len = user_buf.len();
        user_buf.fill(0);
        Ok(len)
    }
    /// 填满0
    async fn pread(&self, mut user_buf: &mut [u8], offset: usize, len: usize) -> SysResult<usize> {
        info!("[pread] from zerofs, fill 0");
        user_buf.fill(0);
        Ok(len)
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/loop0".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn read_dents(&self, mut ub: usize, len: usize) -> usize {
        0
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = Kstat::new();
        stat.st_mode = S_IFBLK + 0o666;
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        // self.metadata.inode.get_page_cache().unwrap().get_page(offset).unwrap()
        Some(Page::new())
    }
    fn is_deivce(&self) -> bool {
        true
    }
}

struct DevLoopInode {
    timestamp: SpinNoIrqLock<TimeStamp>,
    meta: SpinNoIrqLock<DevLoopInodeMeta>,
}

impl DevLoopInode {
    fn new() -> Self {
        Self {
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
            meta: SpinNoIrqLock::new(DevLoopInodeMeta::new()),
        }
    }
}

struct DevLoopInodeMeta {
    /// 与之绑定的文件描述符, 不绑定文件(Arc<dyn Filetrait>是因为希望控制文件的生命周期)
    fd: Option<usize>,
    // 信息
    info: LoopInfo,
}

impl DevLoopInodeMeta {
    fn new() -> Self {
        Self {
            fd: None,
            info: LoopInfo::new(),
        }
    }
}

#[async_trait]
impl InodeTrait for DevLoopInode {
    fn get_page_cache(&self) -> Option<Arc<crate::fs::page_cache::PageCache>> {
        None
    }

    fn get_size(&self) -> usize {
        0
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }

    fn node_type(&self) -> InodeType {
        InodeType::BlockDevice
    }

    async fn read_at(&self, _offset: usize, buf: &mut [u8]) -> usize {
        buf.fill(0);
        buf.len()
    }

    async fn write_at(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len()
    }

    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        buf.len()
    }

    fn truncate(&self, _size: usize) -> usize {
        0
    }

    async fn sync(&self) {}

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Ok(Vec::new())
    }

    fn look_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        stat
    }

    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        // 你可以给 DevLoop 加一个 timestamp 字段并返回它
        // unimplemented!("DevLoop does not have a timestamp")
        &self.timestamp
    }

    fn is_dir(&self) -> bool {
        false
    }

    // fn rename(&self, _old_path: &String, _new_path: &String) {}

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }

    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        buf.fill(0);
        buf.len()
    }

    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        let cmd = LoopIoctl::from_bits(op as u32).ok_or(Errno::EINVAL)?;
        let user_ptr: &mut LoopInfo = user_ref_mut(arg.into())?.unwrap();
        info!("[DevLoopInode_ioctl] {}, {:?}", &cmd, user_ptr);
        match cmd {
            cmd if cmd.bits() == LoopIoctl::LOOP_GET_STATUS.bits() => {
                // Handle LOOP_GET_STATUS
                info!("[DevLoopInode_ioctl]LOOP_GET_STATUS ");
                {
                    *user_ptr = self.meta.lock().info;
                }
                info!("[DevLoopInode_ioctl] return\n {:?}", user_ptr);
                if self.meta.lock().fd.is_none() {
                    return Err(Errno::ENXIO);
                } else {
                    return Ok(0);
                }
            }
            cmd if cmd.bits() == LoopIoctl::LOOP_SET_STATUS.bits() => {
                // Handle LOOP_SET_STATUS
                self.meta.lock().info = *user_ptr;
                Ok(0)
            }
            cmd if cmd.bits() == LoopIoctl::LOOP_SET_FD.bits() => {
                // Handle LOOP_SET_FD
                self.meta.lock().fd = Some(arg);
                Ok(0)
            }
            cmd if cmd.bits() == LoopIoctl::LOOP_CLR_FD.bits() => {
                self.meta.lock().fd = None;
                Ok(0)
            }
            _ => {
                // Handle other cases
                error!("no valid cmd");
                Err(Errno::EINVAL)
            }
        }
    }
}

bitflags! {
    /// 环回设备的 ioctl 命令
    pub struct LoopIoctl: u32 {
        /// 设置环回设备的文件描述符
        const LOOP_SET_FD = 0x4C00;
        /// 清除环回设备的文件描述符
        const LOOP_CLR_FD = 0x4C01;
        /// 设置环回设备的状态
        const LOOP_SET_STATUS = 0x4C02;
        /// 获取环回设备的状态
        const LOOP_GET_STATUS = 0x4C03;
        /// 设置环回设备的状态（64 位版本）
        const LOOP_SET_STATUS64 = 0x4C04;
        /// 获取环回设备的状态（64 位版本）
        const LOOP_GET_STATUS64 = 0x4C05;
        /// 更改环回设备的文件描述符
        const LOOP_CHANGE_FD = 0x4C06;
        /// 设置环回设备的容量
        const LOOP_SET_CAPACITY = 0x4C07;
        /// 启用或禁用环回设备的直接 I/O
        const LOOP_SET_DIRECT_IO = 0x4C08;
        /// 设置环回设备的块大小
        const LOOP_SET_BLOCK_SIZE = 0x4C09;
        /// 配置环回设备
        const LOOP_CONFIGURE = 0x4C0A;
        /// 添加一个新的环回设备
        const LOOP_CTL_ADD = 0x4C80;
        /// 移除一个现有的环回设备
        const LOOP_CTL_REMOVE = 0x4C81;
        /// 获取第一个空闲的环回设备
        const LOOP_CTL_GET_FREE = 0x4C82;
    }
}

use core::fmt;

impl fmt::Display for LoopIoctl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.bits() {
            0x4C00 => "LOOP_SET_FD",
            0x4C01 => "LOOP_CLR_FD",
            0x4C02 => "LOOP_SET_STATUS",
            0x4C03 => "LOOP_GET_STATUS",
            0x4C04 => "LOOP_SET_STATUS64",
            0x4C05 => "LOOP_GET_STATUS64",
            0x4C06 => "LOOP_CHANGE_FD",
            0x4C07 => "LOOP_SET_CAPACITY",
            0x4C08 => "LOOP_SET_DIRECT_IO",
            0x4C09 => "LOOP_SET_BLOCK_SIZE",
            0x4C0A => "LOOP_CONFIGURE",
            0x4C80 => "LOOP_CTL_ADD",
            0x4C81 => "LOOP_CTL_REMOVE",
            0x4C82 => "LOOP_CTL_GET_FREE",
            _ => "UNKNOWN_IOCTL",
        };
        write!(f, "{}", name)
    }
}

use core::mem::size_of;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// 表示环回设备的信息。
pub struct LoopInfo {
    /// 环回设备编号（通过 ioctl 只读）。
    pub lo_number: i32,
    /// 设备标识符（__kernel_old_dev_t，通过 ioctl 只读）。
    pub lo_device: u16,
    /// 与环回设备关联的 inode 编号（通过 ioctl 只读）。
    pub lo_inode: u64,
    /// 实际设备标识符（__kernel_old_dev_t，通过 ioctl 只读）。
    pub lo_rdevice: u16,
    /// 在后备文件或设备中的偏移量。
    pub lo_offset: i32,
    /// 环回设备使用的加密类型。
    pub lo_encrypt_type: i32,
    /// 加密密钥的大小（通过 ioctl 只写）。
    pub lo_encrypt_key_size: i32,
    /// 与环回设备关联的标志。
    pub lo_flags: i32,
    /// 后备文件或设备的名称。
    pub lo_name: [u8; LO_NAME_SIZE],
    /// 环回设备使用的加密密钥。
    pub lo_encrypt_key: [u8; LO_KEY_SIZE],
    /// 环回设备的初始化数据。
    pub lo_init: [u64; 2],
    /// 保留供将来使用。
    pub reserved: [u8; 4],
}

// 定义常量，确保与 C 语言的宏定义一致
pub const LO_NAME_SIZE: usize = 64;
pub const LO_KEY_SIZE: usize = 32;

impl LoopInfo {
    /// 创建一个新的 `LoopInfo` 实例，所有字段初始化为默认值
    pub fn new() -> Self {
        Self {
            lo_number: 1,
            lo_device: 1,
            lo_inode: 0,
            lo_rdevice: 1,
            lo_offset: 0,
            lo_encrypt_type: 0,
            lo_encrypt_key_size: 0,
            lo_flags: 0,
            lo_name: [0; LO_NAME_SIZE],
            lo_encrypt_key: [0; LO_KEY_SIZE],
            lo_init: [0; 2],
            reserved: [0; 4],
        }
    }

    /// 将结构体转换为字节数组，便于与 C 语言交互
    pub fn as_bytes(&self) -> &[u8] {
        let size = size_of::<Self>();
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, size) }
    }

    /// 从字节数组创建 `LoopInfo` 实例
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), size_of::<Self>(), "Invalid byte slice size");
        unsafe { *(bytes.as_ptr() as *const Self) }
    }

    pub fn sizeof() -> usize {
        size_of::<Self>()
    }
}
