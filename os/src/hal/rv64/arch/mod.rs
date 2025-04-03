pub mod sbi;
pub mod sstatus;
pub mod interrupt;
use core::arch::asm;
use riscv::register::satp;

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
/// riscv::register::satp::read().bits()
pub fn satp_read() -> usize {
    riscv::register::satp::read().bits()
}

pub fn satp_write(satp: usize) {
    unsafe {
        satp::write(satp);
    }
}



pub fn sfence() {
    unsafe {
        asm!("sfence.vma");
    }
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

pub fn shutdown(failuer: bool) -> ! {
    sbi::shutdown(failuer)
}

/// use sbi call to start the specific core
pub fn hart_start_success(hartid: usize, start_addr: usize) -> bool {
    sbi::hart_start(hartid, start_addr)
}