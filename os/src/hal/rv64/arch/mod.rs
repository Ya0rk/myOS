pub mod interrupt;
pub mod sbi;
pub mod sstatus;

use core::arch::asm;
use log::info;
use riscv::register::satp;
pub use riscv::register::scause;
// pub use core::arch::riscv64::*;

pub fn tp_read() -> usize {
    unsafe {
        let mut tp: usize;
        asm!("mv {}, tp", out(reg) tp);
        tp
    }
}

pub fn fp_read() -> usize {
    unsafe {
        let mut fp: usize;
        asm!("mv {}, fp", out(reg) fp);
        fp
    }
}

pub fn ra_read() -> usize {
    unsafe {
        let mut ra: usize;
        asm!("mv {}, ra", out(reg) ra);
        ra
    }
}
/// riscv::register::satp::read().bits()
pub fn satp_read() -> usize {
    riscv::register::satp::read().bits()
}

pub fn satp_write(satp: usize) {
    unsafe {
        satp::write(satp);
    }
}

pub fn user_token_write(token: usize) {
    unsafe {
        satp::write(token);
    }
}

pub fn user_token_read() -> usize {
    unsafe { satp_read() }
}

pub fn kernel_token_write(token: usize) {
    unsafe {
        satp::write(token);
    }
}

pub fn kernel_token_read() -> usize {
    unsafe { satp_read() }
}

pub fn sfence() {
    unsafe {
        asm!("sfence.vma");
    }
}

pub fn sfence_vma_vaddr(vaddr: usize) {
    unsafe { asm!("sfence.vma {}, x0", in(reg) vaddr, options(nostack)) }
}

pub fn console_putchar(c: usize) {
    sbi::console_putchar(c);
}

pub fn console_getchar() -> usize {
    sbi::console_getchar()
}

pub fn set_timer(timer: usize) {
    sbi::set_timer(timer);
}

pub fn get_time() -> usize {
    riscv::register::time::read()
}

pub fn shutdown(failuer: bool) -> ! {
    sbi::shutdown(failuer)
}

/// use sbi call to start the specific core
pub fn hart_start_success(hartid: usize, start_addr: usize) -> bool {
    info!(
        "[hart_start_success] hart_start_success: hartid = {}, start_addr = {:#x}",
        hartid, start_addr
    );
    sbi::hart_start(hartid, start_addr)
}

/// 让内核态可以直接访问用户态地址空间
pub fn set_sum() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}
