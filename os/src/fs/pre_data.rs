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
        fn usershell_start();
        fn usershell_end();
    }
    let mut v1 = Vec::new();
    let len = usershell_end as usize - usershell_start as usize;
    let data = unsafe { core::slice::from_raw_parts_mut(usershell_start as *mut u8, len) as &'static [u8] };
    if let Some(FileClass::File(usershell)) = open_file("user_shell", OpenFlags::O_CREAT) {
        let buf = UserBuffer::new(v1);
        usershell.metadata.inode.write_directly(0, &data).await;
    }

    let mut v2 = Vec::new();
    let len = initproc_end as usize - initproc_start as usize;
    let data = unsafe { core::slice::from_raw_parts_mut(initproc_start as *mut u8, len) as &'static [u8] };
    
    if let Some(FileClass::File(initproc)) = open_file("initproc", OpenFlags::O_CREAT) {
        // v.push(data);
        let buf = UserBuffer::new(v2);
        initproc.metadata.inode.write_directly(0, &data).await;
        // info!("[flush_preload] write initproc to file");
        return initproc;
    }
    // info!("[flush_preload] open initproc failed");
    panic!("[flush_preload] open initproc failed");
}