use crate::{fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat, OpenFlags}, mm::{page::Page, UserBuffer}, utils::SysResult};
use alloc::{string::String, sync::Arc, vec::Vec};
use async_trait::async_trait;
use alloc::boxed::Box;
use log::info;

pub struct DevNull;

impl DevNull {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevNull {
    fn set_flags(&self, _flags: OpenFlags) {
        todo!()
    }
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
    async fn read(&self, mut _user_buf: UserBuffer) -> SysResult<usize> {
        // do nothing
        Ok(0)
    }
    /// 填满0
    async fn pread(&self, mut user_buf: UserBuffer, offset: usize, len: usize) -> SysResult<usize> {
        info!("[pread] from nullfs, fill 0");
        let zero: Vec<u8> = (0..user_buf.buffers.len()).map(|_| 0).collect();
        user_buf.write(&zero);
        Ok(len)
    }
    async fn write(&self, user_buf: UserBuffer) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }
    
    fn get_name(&self) -> SysResult<String> {
        todo!()
    }

    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        todo!()
    }
    // fn poll(&self , events: PollEvents) -> PollEvents {
    //     let mut revents = PollEvents::empty();
    //     if events.contains(PollEvents::IN) {
    //         revents |= PollEvents::IN;
    //     }
    //     if events.contains(PollEvents::OUT) {
    //         revents |= PollEvents::OUT;
    //     }
    //     revents
    // }

    /// 这里并没有实现
    fn fstat(&self, _stat: &mut Kstat) -> SysResult {
        todo!()
    }
    fn is_dir(&self) -> bool {
        todo!()
    }
    async fn get_page_at(&self, offset: usize) -> Option<Arc<Page>> {
        // self.metadata.inode.get_page_cache().unwrap().get_page(offset).unwrap()
        todo!()
    }
}