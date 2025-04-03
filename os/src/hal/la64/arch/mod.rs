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

pub fn console_putchar(c: usize) {
    uart::uart_put(c);
}

pub fn console_getchar() -> usize {
    uart::uart_get()
}

pub fn set_timer(timer: usize) {
    unimplemented!("loongarch64");
}

pub fn shutdown(failuer: bool) -> ! {
    unimplemented!("loongarch64");
}

/// use sbi call to start the specific core
pub fn hart_start_success(hartid: usize, start_addr: usize) -> bool {
    unimplemented!("loongarch64");
}