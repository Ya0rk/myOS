pub mod sbi;
pub mod sstatus;
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