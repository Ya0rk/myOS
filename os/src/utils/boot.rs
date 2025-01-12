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