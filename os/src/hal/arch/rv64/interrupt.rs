use riscv::register::{sie, sstatus};

#[inline(always)]
pub unsafe fn enable_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}

#[inline(always)]
pub unsafe fn disable_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}

#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    sstatus::read().sie()
}

/// enable timer interrupt in sie CSR
#[inline(always)]
pub unsafe fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}