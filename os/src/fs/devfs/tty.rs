use alloc::{string::String, sync::Arc};
use crate::{fs::{ffi::RenameFlags, FileTrait, InodeTrait, Kstat}, mm::UserBuffer, task::get_init_proc, utils::SysResult};
use async_trait::async_trait;
use alloc::boxed::Box;


pub struct DevTty;

impl DevTty {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileTrait for DevTty {
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
        let init_proc = get_init_proc();
        if let Some(tty_device) = init_proc.get_file_by_fd(0) {
            tty_device.read(user_buf).await
        } else {
            panic!("get Stdin error!");
        }
    }
    async fn write(&self, user_buf: UserBuffer) -> SysResult<usize> {
        let init_proc = get_init_proc();
        if let Some(tty_device) = init_proc.get_file_by_fd(1) {
            tty_device.write(user_buf).await
        } else {
            panic!("get Stdout error!");
        }
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