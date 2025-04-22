//！ `hal/la/trap/kernel_trap.rs`

use log::info;
use loongarch64::register::*;
use loongarch64::register::ecfg::LineBasedInterrupt;
use loongarch64::register::estat::{Exception, Interrupt, Trap};

#[no_mangle]
pub fn kernel_trap_handler() {
    info!("kernel trap");
    let estat = estat::read();
    let crmd = crmd::read();
    let era = era::read();
    if crmd.plv() != CpuMode::Ring0 {
        // 只有在内核态才会触发中断
        panic!("{:?}", estat.cause());
    }
    match estat.cause() {
        Trap::Interrupt(Interrupt::Timer) => {
            // 清除时钟专断
            info!("timer interrupt from kernel");
            ticlr::clear_timer_interrupt();
        }
        Trap::Interrupt(Interrupt::HWI0) => {
            // 中断0 --- 外部中断处理
            unimplemented!("loongarch64 Trap::Interrupt(Interrupt::HWI0)");
        }
        _ => {
            panic!("{:?} pc: {:#x} BADV: {:#x}", estat.cause(), era.pc(), badv::read().vaddr());
        }
    }
    era::set_pc(era.pc());
    info!("kernel trap end");
}