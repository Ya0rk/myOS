#[cfg(target_arch = "loongarch64")]
use loongarch64::register::ecfg::LineBasedInterrupt;
#[cfg(target_arch = "loongarch64")]
use loongarch64::register::*;

#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub fn enable_supervisor_interrupt() {
    unsafe {
        // 开启全局中断
        crmd::set_ie(true);
        // prmd::set_pie(true);
    }
}

#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub fn disable_supervisor_interrupt() {
    unsafe {
        // 这个应该是设置中断设置寄存器
        // ecfg::set_lie(LineBasedInterrupt::empty());
        // 关闭全局中断
        crmd::set_ie(false);
        // prmd::set_pie(false);
    }
}

#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub fn supervisor_interrupt_is_enabled() -> bool {
    crmd::read().ie()
}

#[cfg(target_arch = "loongarch64")]
#[inline(always)]
pub unsafe fn enable_supervisor_timer_interrupt() {
    unsafe {
        ecfg::set_lie(LineBasedInterrupt::TIMER | LineBasedInterrupt::HWI0);
        crmd::set_ie(true);
    }
}

#[inline(always)]
pub unsafe fn enable_supervisor_extern_interrupt() {
    // unsafe {
    //     sie::set_sext();
    // }
}

/// A guard that disable interrupt when it is created and enable interrupt when it is dropped.
pub struct InterruptGuard {
    interrupt_before: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        let interrupt_before = supervisor_interrupt_is_enabled();
        disable_supervisor_interrupt();
        Self { interrupt_before }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.interrupt_before {
            enable_supervisor_interrupt();
        }
    }
}
