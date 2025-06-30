#[cfg(target_arch = "riscv64")]
use riscv::register::{sie, sstatus};



#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn enable_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}


#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn disable_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}


#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    sstatus::read().sie()
}


/// enable timer interrupt in sie CSR
#[cfg(target_arch = "riscv64")]
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
        disable_interrupt();
        Self { interrupt_before }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.interrupt_before {
            enable_interrupt();
        }
    }
}
