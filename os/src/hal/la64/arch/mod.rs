pub mod interrupt;
pub mod sstatus;
pub mod uart;

// 供外部使用
pub use loongarch64::register::ecfg::LineBasedInterrupt;
pub use loongarch64::register::estat::{Exception, Interrupt, Trap};
pub use loongarch64::register::*;
pub use loongarch64::time::get_timer_freq;

// 该文件使用
use core::{arch::asm, intrinsics::unreachable};
use log::{debug, info};

use crate::hal::PAGE_SIZE_BITS;

pub fn tp_read() -> usize {
    unsafe {
        let mut tp: usize;
        asm!("add.w {}, $tp, $r0", out(reg) tp);
        tp
    }
}

pub fn fp_read() -> usize {
    unsafe {
        let mut fp: usize;
        asm!("add.w {}, $fp, $r0", out(reg) fp);
        fp
    }
}

pub fn ra_read() -> usize {
    unsafe {
        let mut ra: usize;
        asm!("add.w {}, $ra, $r0", out(reg) ra);
        ra
    }
}

/// in loongarch satp named pgd
pub fn satp_read() -> usize {
    loongarch64::register::pgdl::read().base() >> 12
}
/// in loongarch satp named pgd
pub fn satp_write(satp: usize) {
    loongarch64::register::pgdl::set_base(satp << PAGE_SIZE_BITS)
}

pub fn user_token_write(token: usize) {
    // info!("[user_token_write] token: {:#x}", token);
    loongarch64::register::pgdl::set_base(token << PAGE_SIZE_BITS);
}
pub fn user_token_read() -> usize {
    loongarch64::register::pgdl::read().base() >> PAGE_SIZE_BITS
}

pub fn kernel_token_write(token: usize) {
    loongarch64::register::pgdh::set_base(token << PAGE_SIZE_BITS);
}
pub fn kernel_token_read() -> usize {
    loongarch64::register::pgdh::read().base() >> PAGE_SIZE_BITS
}

/// dbar为内存屏障功能，当执行dbar 0的时候只有当该指令前面的load/store指令执行完毕后，后面的load/store的指令才会执行
/// invtlb op, rj, rk指令用于无效TLB中的内容
/// op用于指示操作类型， rj用于存放所需的ASID信息， rk用于存放无效操作所需要的虚拟地址信息
/// 当op所表示的操作不需要ASID或虚拟地址时应当将对应的操作数设置为$r0
/// op = 0x0: 清除所有页表项
/// op = 0x5: 清除所有G=0（全局共享标志位），且ASID等于寄存器指定ASID，且VA等于寄存器指定VA的页表项
pub fn sfence() {
    unsafe {
        core::arch::asm!("dbar 0; invtlb 0x00, $r0, $r0");
    }
}

///
pub fn sfence_vma_vaddr(vaddr: usize) {
    unsafe {
        core::arch::asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr);
    }
}

pub fn console_putchar(c: usize) {
    uart::uart_put(c);
}

pub fn console_getchar() -> usize {
    uart::uart_get()
}

pub fn set_timer(timer: usize) {
    let timer_freq = get_timer_freq();
    // Ticlr::read().clear_timer().write(); //清除时钟专断
    ticlr::clear_timer_interrupt();
    // 设置计时器的配置
    tcfg::set_init_val(timer_freq / super::TICKS_PER_SEC);
    tcfg::set_en(true);
    tcfg::set_periodic(true);

    ecfg::set_lie(LineBasedInterrupt::TIMER | LineBasedInterrupt::HWI0);
    crmd::set_ie(true);
    // Crmd::read().set_ie(true).write();
    // info!("interrupt enable: {:?}", ecfg::read().lie());
}

pub fn get_time() -> usize {
    loongarch64::time::Time::read()
}

pub fn shutdown(failuer: bool) -> ! {
    if failuer {
        println!("SystemFailure core dump; shutdown(true)");
    } else {
        println!("NoReason die no failure; shutdown(false)");
    }
    loop {
        unsafe {
            println!("power off!");
            // asm!("idle 1");
            let power_off = 0x8000_0000_100e_001c as *mut u8;
            power_off.write_volatile(0x34 as u8);
        }
    }
    unreachable!()
}

extern "C" {
    static boot_stack_top: usize;
}

/// use sbi call to start the specific core
pub fn hart_start_success(hartid: usize, start_addr: usize) -> bool {
    // 来自polyhal multicore
    unsafe {
        let sp_top = boot_stack_top - 4096 * 16 * hartid;
        loongarch64::ipi::csr_mail_send(super::HART_START_ADDR as _, hartid, 0);
        loongarch64::ipi::csr_mail_send(sp_top as _, hartid, 1);
        loongarch64::ipi::send_ipi_single(1, 1);
    }

    true
}

/// 让内核态可以直接访问用户态地址空间
pub fn set_sum() {
    log::info!("loongarch do not need set_sum");
    // unimplemented!("loongarch64");
}

pub fn current_inst_len() -> usize {
    4
}