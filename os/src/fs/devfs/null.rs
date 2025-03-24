use crate::{fs::{FileTrait, Kstat}, mm::UserBuffer};
use alloc::string::String;

pub struct DevNull;

impl DevNull {
    pub fn new() -> Self {
        Self
    }
}

impl FileTrait for DevNull {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut _user_buf: UserBuffer) -> usize {
        // do nothing
        0
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        // do nothing
        user_buf.len()
    }
    
    fn get_name(&self) -> String {
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

    /// 这里并没有实现
    fn fstat(&self, _stat: &mut Kstat) -> () {
        todo!()
    }
}