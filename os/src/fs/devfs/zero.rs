use alloc::{string::{String, ToString}, sync::Arc, vec::Vec};
use log::info;
use crate::{fs::{ffi::RenameFlags, Dirent, FileTrait, InodeTrait, Kstat, OpenFlags, S_IFCHR}, mm::{page::Page, UserBuffer}, utils::SysResult};
use async_trait::async_trait;
use alloc::boxed::Box;


pub struct DevZero;

impl DevZero {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevZero {
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
    async fn write(&self, user_buf: & [u8]) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/zero".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    fn read_dents(&self, mut ub: usize, len: usize) -> usize {
        0
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
        Some(Page::new())
    }
}