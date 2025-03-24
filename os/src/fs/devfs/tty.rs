use alloc::string::String;
use crate::{fs::{FileTrait, Kstat}, mm::UserBuffer, task::INITPROC};


pub struct DevTty;

impl DevTty {
    pub fn new() -> Self {
        Self
    }
}

impl FileTrait for DevTty {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, user_buf: UserBuffer) -> usize {
        if let Some(tty_device) = INITPROC.get_file_by_fd(0) {
            tty_device.read(user_buf)
        } else {
            panic!("get Stdin error!");
        }
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        if let Some(tty_device) = INITPROC.get_file_by_fd(1) {
            tty_device.write(user_buf)
        } else {
            panic!("get Stdout error!");
        }
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