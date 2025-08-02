use super::ffi::RenameFlags;
use super::devfs::tty::TTY_INODE;
use super::devfs::char::TTY_INODE1;
use super::FileTrait;
use super::InodeTrait;
use super::Kstat;
use super::OpenFlags;
use crate::fs::devfs::char::CharDevInode;
use crate::fs::Page;
use crate::utils::downcast::Downcast;
use crate::utils::{Errno, SysResult};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use async_trait::async_trait;

// --- Stdin ---

pub struct Stdin {
    inode: Arc<dyn InodeTrait>,
}

impl Stdin {
    pub fn new() -> Self {
        Self {
            inode: TTY_INODE1.clone(),
        }
    }
}

#[async_trait]
impl FileTrait for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_flags(&self) -> OpenFlags {
        OpenFlags::O_RDONLY
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }

    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        if user_buf.is_empty() {
            return Ok(0);
        }
        let res = self.inode.read_dirctly(0, user_buf).await;
        Ok(res)
    }

    async fn write(&self, _user_buf: &[u8]) -> SysResult<usize> {
        Err(Errno::EINVAL)
    }

    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        Ok(())
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("stdin".into())
    }

    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        Err(Errno::EPERM)
    }

    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
    fn is_device(&self) -> bool {
        true
    }
    async fn pollin(&self) -> SysResult<bool> {
        Ok(self.inode
            .clone()
            .downcast_arc::<CharDevInode>()
            .ok_or(Errno::ENODEV)?
            .poll_in()
            .await
        )
    }
    async fn pollout(&self) -> SysResult<bool> {
        Ok(self.inode
            .clone()
            .downcast_arc::<CharDevInode>()
            .ok_or(Errno::ENODEV)?
            .poll_out()
            .await
        )
    }
}

// --- Stdout ---

pub struct Stdout {
    inode: Arc<dyn InodeTrait>,
}

impl Stdout {
    pub fn new() -> Self {
        Self {
            inode: TTY_INODE1.clone(),
        }
    }
}

#[async_trait]
impl FileTrait for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn executable(&self) -> bool {
        false
    }
    fn get_flags(&self) -> OpenFlags {
        OpenFlags::O_WRONLY
    }
    fn is_dir(&self) -> bool {
        false
    }
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        self.inode.clone()
    }

    async fn read(&self, _user_buf: &mut [u8]) -> SysResult<usize> {
        Err(Errno::EINVAL)
    }

    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        let res = self.inode.write_directly(0, user_buf).await;
        Ok(res)
    }

    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        Ok(())
    }

    fn get_name(&self) -> SysResult<String> {
        Ok("stdout".into())
    }

    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        Err(Errno::EPERM)
    }

    async fn get_page_at(&self, _offset: usize) -> Option<Arc<Page>> {
        None
    }
}