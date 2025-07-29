use alloc::{sync::Arc, vec::Vec, boxed::Box};
use async_trait::async_trait;

use crate::{drivers::tty::tty_core::{CharDevice, TTY}, fs::{Dirent, InodeTrait, InodeType, PageCache}, sync::{SpinNoIrqLock, TimeStamp}, utils::SysResult};


lazy_static!{
    pub static ref TTY_INODE1: Arc<CharDevInode> = Arc::new(CharDevInode::new(TTY.clone()));
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
}

impl CharDevInode {
    pub fn new(dev: Arc<dyn CharDevice>) -> Self {
        CharDevInode { dev }
    }
}