use core::future::Future;
use alloc::{sync::Arc, vec::Vec, boxed::Box};
use async_trait::async_trait;

use crate::{drivers::tty::tty_core::{CharDevice, TtyIoctlCmd, TTY}, fs::{Dirent, InodeTrait, InodeType, PageCache}, sync::{block_on, SpinNoIrqLock, TimeStamp}, utils::{Errno, SysResult}};


lazy_static!{
    pub static ref VF2_TTY_INODE: Arc<CharDevInode> = Arc::new(CharDevInode::new(TTY.clone()));
}

pub struct CharDevInode {
    dev: Arc<dyn CharDevice>,
}

#[async_trait]
impl InodeTrait for CharDevInode {
    async fn read_dirctly(&self, _offset: usize, buf: &mut [u8]) -> usize {
        self.dev.read(buf).await
    }
    async fn write_directly(&self, _offset: usize, buf: &[u8]) -> usize {
        self.dev.write(buf).await
    }
    fn get_page_cache(&self) -> Option<Arc<PageCache>> {
        None
    }
    fn set_size(&self, _new_size: usize) -> SysResult {
        Ok(())
    }
    fn node_type(&self) -> InodeType {
        InodeType::CharDevice
    }
    fn get_timestamp(&self) -> &SpinNoIrqLock<TimeStamp> {
        unimplemented!("DevTty does not have a timestamp")
    }
    fn is_dir(&self) -> bool {
        false
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

impl CharDevInode {
    pub fn new(dev: Arc<dyn CharDevice>) -> Self {
        CharDevInode { dev }
    }
    pub fn poll_in(&self) -> impl Future<Output = bool> + use<'_> {
        self.dev.poll_in()
    }
    pub fn poll_out(&self) -> impl Future<Output = bool> + use<'_> {
        self.dev.poll_out()
    }
}