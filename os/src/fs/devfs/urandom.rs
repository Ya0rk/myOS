use crate::{fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat}, mm::UserBuffer, utils::{SysResult, RNG}};
use alloc::{string::String, sync::Arc};
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
    async fn read(&self, user_buf: UserBuffer) -> SysResult<usize> {
        Ok(RNG.lock().fill_buf(user_buf))
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
}