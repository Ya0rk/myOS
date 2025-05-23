use crate::{fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat, OpenFlags, S_IFCHR}, mm::{page::Page, UserBuffer}, utils::{SysResult, RNG}};
use alloc::{string::{String, ToString}, sync::Arc};
use async_trait::async_trait;
use alloc::boxed::Box;

pub struct DevRandom;

impl DevRandom {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevRandom {
    fn get_inode(&self) -> Arc<dyn InodeTrait> {
        todo!()
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
    async fn read(&self, user_buf: &mut [u8]) -> SysResult<usize> {
        Ok(RNG.lock().fill_buf(user_buf))
    }
    async fn write(&self, user_buf: &[u8]) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/random".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn fstat(&self, stat: &mut Kstat) -> SysResult {
        *stat = Kstat::new();
        stat.st_mode = S_IFCHR;
        Ok(())
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        // self.metadata.inode.get_page_cache().unwrap().get_page(offset).unwrap()
        todo!()
    }
}