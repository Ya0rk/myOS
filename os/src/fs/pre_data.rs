use alloc::{sync::Arc, vec::Vec};
use log::info;
// use virtio_drivers::transport::pci::bus;
use crate::{fs::{open, FileClass, FileTrait, OpenFlags}, mm::UserBuffer};

use super::ext4::NormalFile;

#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(include_str!("preload_rv.S"));


#[cfg(target_arch = "loongarch64")]
core::arch::global_asm!(include_str!("preload_la.S"));

//将预加载到内存中的程序写入文件根目录
/// 自动测试
pub async fn autorun() -> Arc<NormalFile> {
    extern "C" {
        fn autorun_start();
        fn autorun_end();
    }

    if let Ok(FileClass::File(autorun)) = open("/autorun".into(), OpenFlags::O_CREAT) {
        let len = autorun_end as usize - autorun_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(autorun_start as *mut u8, len) as &'static [u8] };
        info!("data len ={}", data.len());
        info!("autorun size = {}", autorun.metadata.inode.get_size());
        autorun.write(data).await;
        return autorun;
    }
    else {
        panic!("[flush_preload] open autorun failed");
    }
}

/// 使用musl busybox的shell
pub async fn mbshell() -> Arc<NormalFile> {
    extern "C" {
        fn mbshell_start();
        fn mbshell_end();
    }

    if let Ok(FileClass::File(gbshell)) = open("/mbshell".into(), OpenFlags::O_CREAT) {
        let len = mbshell_end as usize - mbshell_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(mbshell_start as *mut u8, len) as &'static [u8] };
        info!("data len ={}", data.len());
        info!("gbshell size = {}", gbshell.metadata.inode.get_size());
        gbshell.write(data).await;
        return gbshell;
    }
    else {
        panic!("[flush_preload] open gbshell failed");
    }
}

/// 使用glibc busybox的shell
pub async fn gbshell() -> Arc<NormalFile> {
    extern "C" {
        fn gbshell_start();
        fn gbshell_end();
    }

    if let Ok(FileClass::File(gbshell)) = open("/gbshell".into(), OpenFlags::O_CREAT) {
        let len = gbshell_end as usize - gbshell_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(gbshell_start as *mut u8, len) as &'static [u8] };
        info!("data len ={}", data.len());
        info!("gbshell size = {}", gbshell.metadata.inode.get_size());
        gbshell.write(data).await;
        return gbshell;
    }
    else {
        panic!("[flush_preload] open gbshell failed");
    }
}

/// 使用自己的shell
pub async fn initproc() -> Arc<NormalFile> {
extern "C" {
        fn initproc_start();
        fn initproc_end();
        fn user_shell_start();
        fn user_shell_end();
    }

    // user_shell
    if let Ok(FileClass::File(user_shell)) = open("/user_shell".into(), OpenFlags::O_CREAT) {
        let len = user_shell_end as usize - user_shell_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(user_shell_start as *mut u8, len) as &'static [u8] };
        user_shell.metadata.inode.write_directly(0, &data).await;
    }
    else {
        panic!("[flush_preload] open user_shell failed");
    }

    // initproc
    if let Ok(FileClass::File(initproc)) = open("/initproc".into(), OpenFlags::O_CREAT) {
        let len = initproc_end as usize - initproc_start as usize;
        let data = unsafe { core::slice::from_raw_parts_mut(initproc_start as *mut u8, len) as &'static [u8] };
        initproc.write(data).await;
        return initproc;
    }
    else {
        panic!("[flush_preload] open initproc failed");
    }
}