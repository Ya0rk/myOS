#![allow(unused)]
use loongarch64::register::euen;

use crate::hal::config::{HART_NUM, HART_START_ADDR, KERNEL_ADDR_OFFSET};
use crate::hal::mem::tlb::tlb_fill;
use crate::hal::mem::{mmu_init, tlb_init};
use crate::{hal::arch::hart_start_success, mm::VirtAddr};
use core::arch::asm;

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

pub fn platform() {
    println!(r#"
QEMU LoongArch64 virt
    "#);
}

pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        let start: VirtAddr = VirtAddr(sbss as usize);
        let end: VirtAddr = VirtAddr(ebss as usize);
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
    mmu_init();
    euen::set_fpe(true);
    tlb_init(tlb_fill as usize);
}