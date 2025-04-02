// use riscv::register::{sie, sstatus};


#[inline(always)]
pub unsafe fn enable_interrupt() {
    unimplemented!();
}

#[inline(always)]
pub unsafe fn disable_interrupt() {
    unimplemented!();
}

#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    unimplemented!();
}

/// enable timer interrupt in sie CSR
#[inline(always)]
pub unsafe fn enable_timer_interrupt() {
    unimplemented!();
}