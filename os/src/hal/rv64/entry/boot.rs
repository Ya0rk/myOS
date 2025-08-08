#![allow(unused)]
use core::arch::asm;
use log::info;
use lwext4_rust::bindings::int_fast16_t;

use crate::hal::config::{HART_NUM, HART_START_ADDR, KERNEL_ADDR_OFFSET};
use crate::{hal::arch::hart_start_success, mm::VirtAddr};

/// 这里是一个简单的启动代码，它将在启动时运行。
#[cfg(target_arch = "riscv64")]
#[no_mangle]
pub fn jump_helper(hart_id: usize, dtb_ptr: usize) {
    unsafe {
        // 调整栈指针 加上偏移，跳转到 rust_main
        asm!(
            "add sp, sp, {offset}",
            "la t0, rust_main",
            "add t0, t0, {offset}",
            "mv a0, {hartid}",
            "mv a1, {dtb}",
            "jalr zero, 0(t0)",
            hartid = in(reg) hart_id,
            dtb = in(reg) dtb_ptr,
            offset = in(reg) KERNEL_ADDR_OFFSET,
            options(noreturn)
        );
    }
    // println!("hello");
}
#[cfg(target_arch = "loongarch64")]
#[no_mangle]
pub fn jump_helper(_hart_id: usize) {
    unimplemented!()
}

pub fn logo() {
    println!(
        r#"
                       
oooooooooo.             oooo    .oooo.                 .o              
`888'   `Y8b            `888   d8P'`Y8b              o888              
 888      888  .ooooo.   888  888    888 ooo. .oo.    888  oooo    ooo 
 888      888 d88' `88b  888  888    888 `888P"Y88b   888   `88b..8P'  
 888      888 888ooo888  888  888    888  888   888   888     Y888'    
 888     d88' 888    .o  888  `88b  d88'  888   888   888   .o8"'88b   
o888bood8P'   `Y8bod8P' o888o  `Y8bd8P'  o888o o888o o888o o88'   888o
                                                        
    "#
    );
}

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        let start: VirtAddr = VirtAddr(sbss as usize);
        let end: VirtAddr = VirtAddr(ebss as usize);
        info!("end = {}, start = {}", end.0, start.0);
        let len: usize = end.0 - start.0;
        core::slice::from_raw_parts_mut(start.as_ptr(), len).fill(0);
    }
}

/// boot start_hart之外的所有 hart
pub fn boot_all_harts(hartid: usize) {
    for i in (0..HART_NUM).filter(|id| *id != hartid) {
        if !hart_start_success(i, HART_START_ADDR) {
            println!(
                "[kernel] ---------- hart {} start failed!!!... ----------",
                i
            );
        } else {
            println!("[kernel] ---------- hart {} is starting... ----------", i);
        }
    }
}

pub fn arch_init() {
}

#[inline(always)]
pub fn print_checkpoint(num: usize) {
    unsafe {
        asm!(
            "mv t0, a0",
            "li a7, 1",
            "mv a0, {num}",
            "addi a0, a0, 48",
            "ecall",
            "mv a0, t0",
            num = in(reg) num,
        );
    }
    
}