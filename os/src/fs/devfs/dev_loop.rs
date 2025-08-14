use crate::{
    fs::{
        ffi::RenameFlags, Dirent, FileMeta, FileTrait, InodeTrait, InodeType, Kstat, OpenFlags,
        SEEK_CUR, SEEK_END, SEEK_SET, S_IFBLK, S_IFCHR,
    },
    mm::{
        page::Page,
        user_ptr::{user_ref, user_ref_mut},
    },
    sync::{SpinNoIrqLock, TimeStamp},
    task::current_task,
    utils::{Errno, SysResult},
};
use alloc::boxed::Box;
use alloc::format;
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use async_trait::async_trait;
use bitflags::bitflags;
use core::mem::size_of;
use log::{error, info};

lazy_static! {
    // This singleton represents the platonic ideal of the device.
    // When opened, a new `DevLoop` file handle with its own state should be created.
    // For now, we create a single instance that will be shared, which might have
    // issues with concurrent access expecting different file offsets.
    pub static ref DEVLOOP: Arc<DevLoop> = DevLoop::new(0, OpenFlags::O_RDWR);
}

pub struct DevLoop {
    metadata: FileMeta,
    inode: Arc<DevLoopInode>,
}

impl DevLoop {
    pub fn new(device_num: u32, flags: OpenFlags) -> Arc<Self> {
        let inode = Arc::new(DevLoopInode::new(device_num));
        Arc::new(Self {
            metadata: FileMeta::new(flags, inode.clone()),
            inode,
        })
    }
}

#[async_trait]
impl FileTrait for DevLoop {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }

    fn readable(&self) -> bool {
        self.metadata.flags.read().readable()
    }

    fn writable(&self) -> bool {
        self.metadata.flags.read().writable()
    }

    fn executable(&self) -> bool {
        false
    }

    fn get_flags(&self) -> OpenFlags {
        self.metadata.flags.read().clone()
    }

    fn set_flags(&self, flags: OpenFlags) {
        *self.metadata.flags.write() = flags;
    }

    async fn read(&self, mut user_buf: &mut [u8]) -> SysResult<usize> {
        let offset = self.metadata.offset();
        let read_size = self.inode.read_at(offset, user_buf).await;
        self.metadata.set_offset(offset + read_size);
        Ok(read_size)
    }

    async fn pread(&self, mut user_buf: &mut [u8], offset: usize, _len: usize) -> SysResult<usize> {
        let read_len = self.inode.read_at(offset, &mut user_buf).await;
        Ok(read_len)
    }

    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        let offset = self.metadata.offset();
        let write_size = self.inode.write_at(offset, user_buf).await;
        self.metadata.set_offset(offset + write_size);
        Ok(write_size)
    }

    async fn pwrite(&self, buf: &[u8], offset: usize, _len: usize) -> SysResult<usize> {
        let write_size = self.inode.write_at(offset, buf).await;
        Ok(write_size)
    }

    fn lseek(&self, offset: isize, whence: usize) -> SysResult<usize> {
        let old_offset = self.metadata.offset();
        let new_offset = match whence {
            SEEK_SET => offset as usize,
            SEEK_CUR => (old_offset as isize + offset) as usize,
            SEEK_END => (self.inode.get_size() as isize + offset) as usize,
            _ => return Err(Errno::EINVAL),
        };
        self.metadata.set_offset(new_offset);
        Ok(new_offset)
    }

    fn get_name(&self) -> SysResult<String> {
        Ok(format!(
            "/dev/loop{}",
            self.inode.meta.lock().info.lo_number
        ))
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        Err(Errno::EPERM)
    }
    fn read_dents(&self, _ub: usize, _len: usize) -> usize {
        0
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = self.inode.fstat();
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        if let Some(file) = &self.inode.meta.lock().backing_file {
            file.get_page_at(offset).await
        } else {
            None
        }
    }
    fn is_device(&self) -> bool {
        true
    }
}

struct DevLoopInode {
    timestamp: SpinNoIrqLock<TimeStamp>,
    meta: SpinNoIrqLock<DevLoopInodeMeta>,
}

impl DevLoopInode {
    fn new(device_num: u32) -> Self {
        Self {
            timestamp: SpinNoIrqLock::new(TimeStamp::new()),
            meta: SpinNoIrqLock::new(DevLoopInodeMeta::new(device_num)),
        }
    }
}

struct DevLoopInodeMeta {
    /// With which file is the loop device associated?
    backing_file: Option<Arc<dyn FileTrait>>,
    // Information using the 64-bit structure
    info: LoopInfo64,
}

impl DevLoopInodeMeta {
    fn new(device_num: u32) -> Self {
        let mut info = LoopInfo64::new();
        info.lo_number = device_num;
        Self {
            backing_file: None,
            info,
        }
    }
}

#[async_trait]
impl InodeTrait for DevLoopInode {
    fn get_page_cache(&self) -> Option<Arc<crate::fs::page_cache::PageCache>> {
        None
    }

    fn get_size(&self) -> usize {
        let meta = self.meta.lock();
        if let Some(file) = &meta.backing_file {
            if meta.info.lo_sizelimit > 0 {
                meta.info.lo_sizelimit as usize
            } else {
                file.get_inode().get_size()
            }
        } else {
            0
        }
    }

    fn set_size(&self, _new_size: usize) -> SysResult {
        // The size is determined by the backing file and cannot be changed directly.
        Err(Errno::EPERM)
    }

    fn node_type(&self) -> InodeType {
        InodeType::BlockDevice
    }

    async fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let meta = self.meta.lock();
        if let Some(file) = &meta.backing_file {
            let inode = file.get_inode();
            let read_offset = meta.info.lo_offset as usize + offset;

            let size_limit = if meta.info.lo_sizelimit > 0 {
                meta.info.lo_sizelimit
            } else {
                inode.get_size() as u64
            };

            if offset as u64 >= size_limit {
                return 0;
            }

            let remaining_in_limit = (size_limit as usize).saturating_sub(offset);
            let read_len = buf.len().min(remaining_in_limit);

            if read_len == 0 {
                return 0;
            }
            inode.read_at(read_offset, &mut buf[..read_len]).await
        } else {
            // No backing file, should return EIO. Since we can't, return 0 bytes read.
            0
        }
    }

    async fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let meta = self.meta.lock();
        if (meta.info.lo_flags & LoopFlags::LO_FLAGS_READ_ONLY.bits()) != 0 {
            return 0; // Write on read-only device
        }
        if let Some(file) = &meta.backing_file {
            let inode = file.get_inode();
            let write_offset = meta.info.lo_offset as usize + offset;

            // let size_limit = if meta.info.lo_sizelimit > 0 {
            //     meta.info.lo_sizelimit
            // } else {
            //     inode.get_size() as u64
            // };
            //
            // if offset as u64 >= size_limit {
            //     return 0; // Attempt to write past the size limit
            // }
            //
            // let remaining_in_limit = (size_limit as usize).saturating_sub(offset);
            // let write_len = buf.len().min(remaining_in_limit);
            //
            // if write_len == 0 {
            //     return 0;
            // }
            inode.write_at(write_offset, &buf).await
        } else {
            // No backing file, should return EIO.
            0
        }
    }

    async fn write_directly(&self, offset: usize, buf: &[u8]) -> usize {
        self.write_at(offset, buf).await
    }

    fn truncate(&self, _size: usize) -> usize {
        0
    }

    async fn sync(&self) {
        if let Some(file) = &self.meta.lock().backing_file {
            file.get_inode().sync().await;
        }
    }

    async fn read_all(&self) -> SysResult<Vec<u8>> {
        Err(Errno::EPERM)
    }

    fn look_up(&self, _path: &str) -> Option<Arc<dyn InodeTrait>> {
        None
    }

    fn fstat(&self) -> Kstat {
        let mut stat = Kstat::new();
        stat.st_mode = S_IFBLK | 0o660;
        stat.st_size = self.get_size() as i64;
        // In a real scenario, major/minor numbers should be assigned.
        stat.st_dev = 0;
        stat
    }

    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        &self.timestamp
    }

    fn is_dir(&self) -> bool {
        false
    }

    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }

    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        buf.fill(0);
        buf.len()
    }

    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        // Generic Block Device ioctls
        info!("[ioctl] loop op: {:x}, arg: {:x}", op, arg);
        const BLKGETSIZE64: u32 = 0x80081272; // returns u64, size in bytes
        const BLKGETSIZE: u32 = 0x1260; // returns long, size in 512-byte sectors
        const BLKSSZGET: u32 = 0x1268; // returns int, logical block size (sector size)
        let op = op as u32;

        match op {
            BLKGETSIZE64 => {
                info!("[DevLoopInode_ioctl] BLKGETSIZE64");
                let size_in_bytes = self.get_size() as u64;
                let user_ptr: &mut u64 = user_ref_mut(arg.into())?.ok_or(Errno::EFAULT)?;
                *user_ptr = size_in_bytes;
                return Ok(0);
            }
            BLKGETSIZE => {
                info!("[DevLoopInode_ioctl] BLKGETSIZE");
                let size_in_bytes = self.get_size() as u64;
                let size_in_sectors = size_in_bytes / 512;
                return Ok(size_in_sectors as usize);
            }
            BLKSSZGET => {
                info!("[DevLoopInode_ioctl] BLKSSZGET");
                let sector_size = 512; // Logical sector size
                let user_ptr: &mut u32 = user_ref_mut(arg.into())?.ok_or(Errno::EFAULT)?;
                *user_ptr = sector_size;
                return Ok(0);
            }
            _ => {} // Fall through to loop-specific ioctls
        }

        let cmd = LoopIoctl::from_bits(op as u32).ok_or(Errno::EINVAL)?;
        info!("[DevLoopInode_ioctl] cmd: {}, arg: {:#x}", &cmd, arg);

        let task = current_task().ok_or(Errno::ESRCH)?;

        match cmd {
            LoopIoctl::LOOP_SET_FD => {
                let fd = arg as usize;
                let file = task.get_file_by_fd(fd).ok_or(Errno::EBADF)?;
                let inode = file.get_inode();
                let mut meta = self.meta.lock();

                if meta.backing_file.is_some() {
                    return Err(Errno::EBUSY);
                }

                meta.backing_file = Some(file.clone());
                meta.info.lo_sizelimit = inode.get_size() as u64;
                let name = file.get_name().unwrap_or_default();
                let name_bytes = name.as_bytes();
                let len = name_bytes.len().min(LO_NAME_SIZE - 1);
                meta.info.lo_file_name[..len].copy_from_slice(&name_bytes[..len]);
                meta.info.lo_file_name[len] = 0;
                Ok(0)
            }
            LoopIoctl::LOOP_CLR_FD => {
                let mut meta = self.meta.lock();
                if meta.backing_file.is_none() {
                    return Err(Errno::ENXIO);
                }
                meta.backing_file = None;
                let number = meta.info.lo_number;
                meta.info = LoopInfo64::new(); // Reset info
                meta.info.lo_number = number;
                Ok(0)
            }
            LoopIoctl::LOOP_GET_STATUS => {
                let user_ptr: &mut LoopInfo = user_ref_mut(arg.into())?.ok_or(Errno::EFAULT)?;
                let meta = self.meta.lock();
                if meta.backing_file.is_none() {
                    return Err(Errno::ENXIO);
                }
                *user_ptr = meta.info.to_loop_info();
                Ok(0)
            }
            LoopIoctl::LOOP_SET_STATUS => {
                let user_ptr: &LoopInfo = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                let mut meta = self.meta.lock();
                if meta.backing_file.is_none() {
                    return Err(Errno::ENXIO);
                }
                meta.info.from_loop_info(user_ptr);
                Ok(0)
            }
            LoopIoctl::LOOP_GET_STATUS64 => {
                let user_ptr: &mut LoopInfo64 = user_ref_mut(arg.into())?.ok_or(Errno::EFAULT)?;
                let meta = self.meta.lock();
                if meta.backing_file.is_none() {
                    return Err(Errno::ENXIO);
                }
                *user_ptr = meta.info;
                Ok(0)
            }
            LoopIoctl::LOOP_SET_STATUS64 => {
                let user_ptr: &LoopInfo64 = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                let mut meta = self.meta.lock();
                if meta.backing_file.is_none() {
                    return Err(Errno::ENXIO);
                }
                let old_info = meta.info;
                meta.info = *user_ptr;
                // Restore read-only fields
                meta.info.lo_device = old_info.lo_device;
                meta.info.lo_inode = old_info.lo_inode;
                meta.info.lo_rdevice = old_info.lo_rdevice;
                meta.info.lo_number = old_info.lo_number;
                Ok(0)
            }
            LoopIoctl::LOOP_CHANGE_FD => {
                let meta = self.meta.lock();
                if (meta.info.lo_flags & LoopFlags::LO_FLAGS_READ_ONLY.bits()) == 0 {
                    return Err(Errno::EINVAL);
                }
                let old_file_size = meta
                    .backing_file
                    .as_ref()
                    .map(|f| f.get_inode().get_size())
                    .unwrap_or(0);
                drop(meta);

                let fd = arg as usize;
                let file = task.get_file_by_fd(fd).ok_or(Errno::EBADF)?;
                let inode = file.get_inode();
                if inode.get_size() != old_file_size {
                    return Err(Errno::EINVAL);
                }
                self.meta.lock().backing_file = Some(file);
                Ok(0)
            }
            LoopIoctl::LOOP_SET_CAPACITY => {
                let mut meta = self.meta.lock();
                if let Some(file) = &meta.backing_file {
                    let inode = file.get_inode();
                    meta.info.lo_sizelimit = inode.get_size() as u64;
                    Ok(0)
                } else {
                    Err(Errno::ENXIO)
                }
            }
            LoopIoctl::LOOP_SET_DIRECT_IO => {
                let mut meta = self.meta.lock();
                if arg != 0 {
                    meta.info.lo_flags |= LoopFlags::LO_FLAGS_DIRECT_IO.bits();
                } else {
                    meta.info.lo_flags &= !LoopFlags::LO_FLAGS_DIRECT_IO.bits();
                }
                Ok(0)
            }
            LoopIoctl::LOOP_SET_BLOCK_SIZE => Err(Errno::EINVAL), // Not supported for now
            LoopIoctl::LOOP_CONFIGURE => {
                let user_ptr: &LoopConfig = user_ref(arg.into())?.ok_or(Errno::EFAULT)?;
                let config = *user_ptr;
                let fd = config.fd as usize;
                let file = task.get_file_by_fd(fd).ok_or(Errno::EBADF)?;
                let inode = file.get_inode();

                let mut meta = self.meta.lock();
                meta.backing_file = Some(file.clone());
                meta.info = config.info;
                if meta.info.lo_sizelimit == 0 {
                    meta.info.lo_sizelimit = inode.get_size() as u64;
                }
                let name = file.get_name().unwrap_or_default();
                let name_bytes = name.as_bytes();
                let len = name_bytes.len().min(LO_NAME_SIZE - 1);
                meta.info.lo_file_name[..len].copy_from_slice(&name_bytes[..len]);
                meta.info.lo_file_name[len] = 0;
                Ok(0)
            }
            _ => {
                error!("unimplemented or invalid loop ioctl: {}", &cmd);
                Err(Errno::EINVAL)
            }
        }
    }
}

bitflags! {
    /// `ioctl` commands for the loop device
    #[derive(PartialEq)]
    pub struct LoopIoctl: u32 {
        const LOOP_SET_FD = 0x4C00;
        const LOOP_CLR_FD = 0x4C01;
        const LOOP_SET_STATUS = 0x4C02;
        const LOOP_GET_STATUS = 0x4C03;
        const LOOP_SET_STATUS64 = 0x4C04;
        const LOOP_GET_STATUS64 = 0x4C05;
        const LOOP_CHANGE_FD = 0x4C06;
        const LOOP_SET_CAPACITY = 0x4C07;
        const LOOP_SET_DIRECT_IO = 0x4C08;
        const LOOP_SET_BLOCK_SIZE = 0x4C09;
        const LOOP_CONFIGURE = 0x4C0A;
        const LOOP_CTL_ADD = 0x4C80;
        const LOOP_CTL_REMOVE = 0x4C81;
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

bitflags! {
    pub struct LoopFlags: u32 {
        const LO_FLAGS_READ_ONLY = 1;
        const LO_FLAGS_AUTOCLEAR = 4;
        const LO_FLAGS_PARTSCAN = 8;
        const LO_FLAGS_DIRECT_IO = 16;
    }
}

pub const LO_NAME_SIZE: usize = 64;
pub const LO_KEY_SIZE: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// 32-bit version of loop device info
pub struct LoopInfo {
    pub lo_number: i32,
    pub lo_device: u16,
    pub lo_inode: u64,
    pub lo_rdevice: u16,
    pub lo_offset: i32,
    pub lo_encrypt_type: i32,
    pub lo_encrypt_key_size: i32,
    pub lo_flags: i32,
    pub lo_name: [u8; LO_NAME_SIZE],
    pub lo_encrypt_key: [u8; LO_KEY_SIZE],
    pub lo_init: [u64; 2],
    pub reserved: [u8; 4],
}

impl LoopInfo {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// 64-bit version of loop device info
pub struct LoopInfo64 {
    pub lo_device: u64,
    pub lo_inode: u64,
    pub lo_rdevice: u64,
    pub lo_offset: u64,
    pub lo_sizelimit: u64,
    pub lo_number: u32,
    pub lo_encrypt_type: u32,
    pub lo_encrypt_key_size: u32,
    pub lo_flags: u32,
    pub lo_file_name: [u8; LO_NAME_SIZE],
    pub lo_crypt_name: [u8; LO_NAME_SIZE],
    pub lo_encrypt_key: [u8; LO_KEY_SIZE],
    pub lo_init: [u64; 2],
}

impl LoopInfo64 {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn to_loop_info(&self) -> LoopInfo {
        let mut info = LoopInfo::new();
        info.lo_number = self.lo_number as i32;
        info.lo_device = self.lo_device as u16;
        info.lo_inode = self.lo_inode;
        info.lo_rdevice = self.lo_rdevice as u16;
        info.lo_offset = self.lo_offset as i32;
        info.lo_encrypt_type = self.lo_encrypt_type as i32;
        info.lo_encrypt_key_size = self.lo_encrypt_key_size as i32;
        info.lo_flags = self.lo_flags as i32;
        info.lo_name.copy_from_slice(&self.lo_file_name);
        info.lo_encrypt_key.copy_from_slice(&self.lo_encrypt_key);
        info.lo_init = self.lo_init;
        info
    }

    pub fn from_loop_info(&mut self, info: &LoopInfo) {
        const MODIFIABLE_FLAGS: u32 =
            LoopFlags::LO_FLAGS_AUTOCLEAR.bits() | LoopFlags::LO_FLAGS_PARTSCAN.bits();

        self.lo_offset = info.lo_offset as u64;
        let new_flags = info.lo_flags as u32;
        self.lo_flags = (self.lo_flags & !MODIFIABLE_FLAGS) | (new_flags & MODIFIABLE_FLAGS);
        self.lo_file_name.copy_from_slice(&info.lo_name);
        self.lo_encrypt_type = info.lo_encrypt_type as u32;
        self.lo_encrypt_key_size = info.lo_encrypt_key_size as u32;
        self.lo_encrypt_key.copy_from_slice(&info.lo_encrypt_key);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LoopConfig {
    pub fd: u32,
    pub block_size: u32,
    pub info: LoopInfo64,
    pub __reserved: [u8; 8],
}

