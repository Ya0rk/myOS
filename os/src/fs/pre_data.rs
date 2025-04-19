use alloc::{sync::Arc, vec::Vec};
use log::info;
use crate::{fs::{open_file, FileClass, FileTrait, OpenFlags}, mm::UserBuffer};

use super::ext4::NormalFile;

core::arch::global_asm!(include_str!("preload.S"));


//将预加载到内存中的程序写入文件根目录
pub async fn flush_preload() -> Arc<NormalFile> {
    extern "C" {
        fn initproc_start();
        fn initproc_end();
        fn user_shell_start();
        fn user_shell_end();
        fn busybox_start();
        fn busybox_end();
    }

    // busybox
    // if let Some(FileClass::File(busybox)) = open_file("musl/busybox", OpenFlags::O_CREAT) {
    //     let mut v = Vec::new();
    //     let len = busybox_end as usize - busybox_start as usize;
    //     let data = unsafe { core::slice::from_raw_parts_mut(busybox_start as *mut u8, len) as &'static [u8] };
    //     let buf = UserBuffer::new(v);
    //     info!("data len ={}", data.len());
    //     info!("busybox size = {}", busybox.metadata.inode.size());
    //     busybox.write(data).await;
    //     return busybox;
    // }
    // else {
    //     panic!("[flush_preload] open busybox failed");
    // }
    
    // user_shell
    if let Some(FileClass::File(user_shell)) = open_file("user_shell", OpenFlags::O_CREAT) {
        // v.push(data);
        // let mut v = Vec::new();
        let len = user_shell_end as usize - user_shell_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(user_shell_start as *mut u8, len) as &'static [u8] };
        // let buf = UserBuffer::new(v);
        // user_shell.write(data).await;
        user_shell.metadata.inode.write_directly(0, &data).await;
    }
    else {
        panic!("[flush_preload] open user_shell failed");
    }

    // initproc
    if let Some(FileClass::File(initproc)) = open_file("initproc", OpenFlags::O_CREAT) {
        // let mut v = Vec::new();
        let len = initproc_end as usize - initproc_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(initproc_start as *mut u8, len) as &'static [u8] };
        // let buf = UserBuffer::new(v);
        // initproc.metadata.inode.write_directly(0, &data).await;
        initproc.write(data).await;
        return initproc;
    }
    else {
        panic!("[flush_preload] open initproc failed");
    }
}