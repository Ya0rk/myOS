use crate::{fs::{ffi::RenameFlags, FileTrait, Kstat}, mm::UserBuffer, utils::SysResult};
use alloc::string::String;
use async_trait::async_trait;
use alloc::boxed::Box;

pub struct DevNull;

impl DevNull {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevNull {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    async fn read(&self, mut _user_buf: UserBuffer) -> SysResult<usize> {
        // do nothing
        Ok(0)
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
}