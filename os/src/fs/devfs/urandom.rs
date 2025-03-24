use crate::{fs::{FileTrait, Kstat}, mm::UserBuffer, utils::RNG};
use alloc::string::String;

pub struct DevRandom;

impl DevRandom {
    pub fn new() -> Self {
        Self
    }
}

impl FileTrait for DevRandom {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, user_buf: UserBuffer) -> usize {
        unsafe { RNG.fill_buf(user_buf) }
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