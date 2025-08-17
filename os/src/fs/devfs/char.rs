use core::{future::Future, task::Waker};
use alloc::{boxed::Box, string::{String, ToString}, sync::Arc, vec::Vec};
use async_trait::async_trait;

use crate::{drivers::{device_new::{dev_number::{CharMajorNum, MajorNumber}, manager::DEVICE_MANAGER}, tty::tty_core::{CharDevice, TtyIoctlCmd}}, fs::{Dirent, FileMeta, FileTrait, InodeMeta, InodeTrait, InodeType, Kstat, OpenFlags, Page, PageCache, S_IFCHR}, sync::{block_on, SpinNoIrqLock, TimeStamp}, utils::{downcast::Downcast, Errno, SysResult}};

pub struct CharDev {
    metadata: FileMeta,
}

impl CharDev {
    pub fn new_in() -> Self {
        Self {
            metadata: FileMeta::new(
                OpenFlags::O_RDONLY,
                CharDevInode::new()
            )
        }
    }
    pub fn new_out() -> Self {
        Self {
            metadata: FileMeta::new(
                OpenFlags::O_WRONLY,
                CharDevInode::new()
            )
        }
    }
}

#[async_trait]
impl FileTrait for CharDev {
    fn metadata(&self) -> &FileMeta {
        &self.metadata
    }
    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        assert!(self.metadata.flags.read().readable());
        if user_buf.is_empty() {
            return Ok(0);
        }
        Ok(self.metadata.inode.read_dirctly(0, user_buf).await)
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        assert!(self.metadata.flags.read().writable());
        Ok(self.metadata.inode.write_directly(0, user_buf).await)
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    async fn pollin(&self) -> SysResult<bool> {
        Ok(self.metadata
            .inode
            .clone()
            .downcast_arc::<CharDevInode>()
            .ok_or(Errno::ENODEV)?
            .poll_in()
            .await
        )
    }
    async fn pollout(&self) -> SysResult<bool> {
        Ok(self.metadata
            .inode
            .clone()
            .downcast_arc::<CharDevInode>()
            .ok_or(Errno::ENODEV)?
            .poll_out()
            .await
        )
    }
    fn abspath(&self) -> String {
        "/dev/tty".to_string()
    }
}


pub struct CharDevInode {
    metadata: InodeMeta,
    dev: Arc<dyn CharDevice>,
}

impl CharDevInode {
    pub fn new() -> Arc<dyn InodeTrait> {
        Arc::new(Self {
            metadata: InodeMeta::new(
                InodeType::CharDevice,
                0,
                "/dev/tty",
            ),
            dev: DEVICE_MANAGER.read()
                .get_char_dev(CharMajorNum::Tty, 64)
                .unwrap()
                // .as_char()
                // .unwrap()
        })
    }
    pub fn poll_in(&self) -> impl Future<Output = bool> + use<'_> {
        self.dev.poll_in()
    }
    pub fn poll_out(&self) -> impl Future<Output = bool> + use<'_> {
        self.dev.poll_out()
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
    
}

#[async_trait]
impl InodeTrait for CharDevInode {
    fn metadata(&self) -> &InodeMeta {
        &self.metadata
    }

    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        self.dev.read(buf).await
    }
    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        self.dev.write(buf).await
    }
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }
    fn fstat(&self) -> Kstat {
        let mut res = Kstat::new();
        res.st_ino = self.metadata.ino as u64;
        res.st_mode = InodeType::CharDevice as u32;
        res.st_nlink = 1;
        res
    }
    fn read_dents(&self) -> Option<Vec<Dirent>> {
        None
    }
    fn ioctl(&self, op: usize, arg: usize) -> SysResult<usize> {
        block_on(async{
            self.dev.ioctl(TtyIoctlCmd::try_from(op).map_err(|_| Errno::EINVAL)?, arg).await
        })
    }
}
