#[cfg(target_arch = "riscv64")]
use riscv::register::{sie, sstatus};

#[cfg(target_arch = "loongarch64")]
use loongarch64::register::ecfg::LineBasedInterrupt;
#[cfg(target_arch = "loongarch64")]
use loongarch64::register::*;

#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn enable_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}
#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub fn enable_interrupt() {
    unsafe {
        // 开启全局中断
        crmd::set_ie(true);
        // prmd::set_pie(true);
    }
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn disable_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}
#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub fn disable_interrupt() {
    unsafe {
        // 这个应该是设置中断设置寄存器
        // ecfg::set_lie(LineBasedInterrupt::empty());
        // 关闭全局中断
        crmd::set_ie(false);
        // prmd::set_pie(false);
    }
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    sstatus::read().sie()
}
#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    crmd::read().ie()
}

/// enable timer interrupt in sie CSR
#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub unsafe fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub unsafe fn enable_timer_interrupt() {
    unsafe {
        ecfg::set_lie(LineBasedInterrupt::TIMER | LineBasedInterrupt::HWI0);
        crmd::set_ie(true);
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
