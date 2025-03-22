use alloc::string::String;
use crate::{fs::{FileTrait, Kstat}, mm::UserBuffer, utils::SysResult};
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
    fn readable(&self) -> SysResult<bool> {
        Ok(true)
    }
    fn writable(&self) -> SysResult<bool> {
        Ok(true)
    }
    async fn read(&self, mut user_buf: UserBuffer) -> SysResult<usize> {
        Ok(user_buf.clear())
    }
    async fn write(&self, user_buf: UserBuffer) -> SysResult<usize> {
        // do nothing
        Ok(user_buf.len())
    }
    
    fn get_name(&self) -> SysResult<String> {
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
}