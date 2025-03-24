use alloc::string::String;
use crate::{fs::{FileTrait, Kstat}, mm::UserBuffer};


pub struct DevZero;

impl DevZero {
    pub fn new() -> Self {
        Self
    }
}

impl FileTrait for DevZero {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        user_buf.clear()
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
    fn fstat(&self, _stat: &mut Kstat) -> () {
        todo!()
    }
}