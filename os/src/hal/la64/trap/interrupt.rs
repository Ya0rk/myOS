// use riscv::register::{sie, sstatus};

use loongarch64::register::*;
use loongarch64::register::ecfg::LineBasedInterrupt;
use loongarch64::register::estat::{Exception, Interrupt, Trap};
use loongarch64::time::get_timer_freq;

#[inline(always)]
pub unsafe fn enable_interrupt() {
    crmd::set_ie(true);
}

#[inline(always)]
pub unsafe fn disable_interrupt() {
    // 关闭全局中断
    crmd::set_ie(false);
}

#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    crmd::ie()
}

/// enable timer interrupt in sie CSR
#[inline(always)]
pub unsafe fn enable_timer_interrupt() {
    let timer_freq = get_timer_freq();
    // Ticlr::read().clear_timer().write(); //清除时钟专断
    ticlr::clear_timer_interrupt();
    // 设置计时器的配置
    tcfg::set_init_val(timer_freq / TICKS_PER_SEC);
    tcfg::set_en(true);
    tcfg::set_periodic(true);
    // Tcfg::read()
    //     .set_enable(true)
    //     .set_loop(true)
    //     .set_tval(timer_freq / TICKS_PER_SEC)
    //     .write();
    // Ecfg::read()
    //     .set_lie_with_index(11, true)
    //     .set_lie_with_index(2, true)
    //     .write();
    // 开启全局中断
    ecfg::set_lie(LineBasedInterrupt::TIMER|LineBasedInterrupt::HWI0);
    crmd::set_ie(true);
    // Crmd::read().set_ie(true).write();
    info!("interrupt enable: {:?}", ecfg::read().lie());
}