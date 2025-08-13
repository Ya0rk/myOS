
use lwext4_rust::{bindings::O_RDONLY, file};

use crate::{fs::{open, FileClass, FileTrait, OpenFlags}, hal::{config::{PAGE_SIZE_BITS, USER_SPACE_TOP, USER_STACK_SIZE, U_SEG_STACK_BEG, U_SEG_STACK_END}, trap::TrapContext}, mm::{user_ptr::user_ref, Direct, Paged, PhysAddr}, sync::block_on, utils::{Errno, SysResult}};



pub fn user_backtrace(cx: &TrapContext) -> Result<(), ()> {

    let mut pc = cx.get_sepc();
    let mut fp = cx.get_fp();

    let stack_range = U_SEG_STACK_END - USER_STACK_SIZE..U_SEG_STACK_END;
    log::error!("[user_backtrace] pc: {pc:#x}, fp: {fp:#x}");

    while stack_range.contains(&fp) {
        let former_fp: usize = *user_ref((fp - 16).into()).map_err(|_| ())?.ok_or(())?;
        let ra: usize = *user_ref((fp - 8).into()).map_err(|_| ())?.ok_or(())?;
        pc = ra;
        fp = former_fp;
        log::error!("[user_backtrace] pc: {pc:#x}, fp: {fp:#x}");
    }
    Ok(())
}

pub fn print_file_at() {
    if let Ok(file) = open("/glibc/lib/libc.so.6".into(), OpenFlags::O_RDONLY) {
        let page = block_on( {file.get_page_at(0x1dbe30)});
        if let Some(page) = page {
            // let buf = &page.get_bytes_array()[0xe30..];
            let pa_base = page.ppn().0 << PAGE_SIZE_BITS;
            for i in (0xe30..0x1000).step_by(16) {
                let pa_former_8: PhysAddr = (pa_base + i).into();
                let former_8: usize = *user_ref(pa_former_8.direct_va()).expect("inaccessible").expect("null pointer");
                let pa_latter_8: PhysAddr = (pa_base + i + 8).into();
                let latter_8: usize = *user_ref(pa_latter_8.direct_va()).expect("inaccessible").expect("null pointer");
                log::error!("[print_file_at] former_8: {former_8:#x}, latter_8: {latter_8:#x}");
            }
        }
    }

}