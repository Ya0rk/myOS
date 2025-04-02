pub mod sbi;
pub mod sstatus;
pub mod uart;

pub fn fp_read() -> usize {
    unimplemented!("loongarch64")
}

pub fn tp_read() -> usize {
    unimplemented!("loongarch64")
}
/// riscv::register::satp::read().bits()
pub fn satp_read() -> usize {
    unimplemented!("loongarch64")
}

pub fn satp_write(satp: usize) {
    unimplemented!("loongarch64")
}



pub fn sfence() {
    unimplemented!("loongarch64")
}