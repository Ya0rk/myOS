use alloc::{string::{String, ToString}, sync::Arc, vec::Vec};
use log::info;
use crate::{fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat, OpenFlags}, mm::{page::Page, UserBuffer}, utils::SysResult};
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
    async fn read(&self, mut user_buf: UserBuffer) -> SysResult<usize> {
        Ok(user_buf.clear())
    }
    /// 填满0
    async fn pread(&self, mut user_buf: UserBuffer, offset: usize, len: usize) -> SysResult<usize> {
        info!("[pread] from zerofs, fill 0");
        let zero: Vec<u8> = (0..user_buf.buffers.len()).map(|_| 0).collect();
        user_buf.write(&zero);
        Ok(len)
    }
    async fn write(&self, user_buf: UserBuffer) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }
    
    fn get_name(&self) -> SysResult<String> {
        Ok("/dev/zero".to_string())
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    // fn poll(&self, events: PollEvents) -> PollEvents {
    //     let mut revents = PollEvents::empty();
    //     if events.contains(PollEvents::IN) {
    //         revents |= PollEvents::IN;
    //     }
    //     if events.contains(PollEvents::OUT) {
    //         revents |= PollEvents::OUT;
    //     }
    //     revents
    // }
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }
    fn is_dir(&self) -> bool {
        todo!()
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        // self.metadata.inode.get_page_cache().unwrap().get_page(offset).unwrap()
        Some(Page::new())
    }
}