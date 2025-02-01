use core::arch::asm;
use crate::config::KERNEL_ADDR_OFFSET;

/// 这里是一个简单的启动代码，它将在启动时运行。
#[no_mangle]
pub fn jump_helper() {
    unsafe { // 调整栈指针 加上偏移，跳转到 rust_main
        asm!(
            "add sp, sp, {offset}",
            "la t0, rust_main",
            "add t0, t0, {offset}",
            "jalr zero, 0(t0)",
            offset = in(reg) KERNEL_ADDR_OFFSET,
            options(noreturn)
        );
    }
}

pub fn logo() {
    println!(r#"
                       
    `YMM'   `MM'                   .g8""8q.    .M"""bgd 
      VMA   ,V                   .dP'    `YM. ,MI    "Y 
       VMA ,V ,pW"Wq.   ,pW"Wq.  dM'      `MM `MMb.     
        VMMP 6W'   `Wb 6W'   `Wb MM        MM   `YMMNq. 
         MM  8M     M8 8M     M8 MM.      ,MP .     `MM 
         MM  YA.   ,A9 YA.   ,A9 `Mb.    ,dP' Mb     dM 
       .JMML. `Ybmd9'   `Ybmd9'    `"bmmd"'   P"Ybmmd"  
                                                        
    "#);
}

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}