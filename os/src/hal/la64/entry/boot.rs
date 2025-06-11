#![allow(unused)]
use crate::hal::config::{HART_NUM, HART_START_ADDR, KERNEL_ADDR_OFFSET};
use crate::{hal::arch::hart_start_success, mm::VirtAddr};
use core::arch::asm;

pub fn logo() {
    println!(
        r#"
                       
    `YMM'   `MM'                   .g8""8q.    .M"""bgd 
      VMA   ,V                   .dP'    `YM. ,MI    "Y 
       VMA ,V ,pW"Wq.   ,pW"Wq.  dM'      `MM `MMb.     
        VMMP 6W'   `Wb 6W'   `Wb MM        MM   `YMMNq. 
         MM  8M     M8 8M     M8 MM.      ,MP .     `MM 
         MM  YA.   ,A9 YA.   ,A9 `Mb.    ,dP' Mb     dM 
       .JMML. `Ybmd9'   `YooOS'    `"bmmd"'   P"Ybmmd"  
                                                        
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
