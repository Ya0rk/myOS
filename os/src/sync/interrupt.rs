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

/// A guard that disable interrupt when it is created and enable interrupt when it is dropped.
pub struct InterruptGuard {
    interrupt_before: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        let interrupt_before = interrupt_is_enabled();
        unsafe { disable_interrupt() };
        Self { interrupt_before }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.interrupt_before {
            unsafe { enable_interrupt() };
        }
    }
}