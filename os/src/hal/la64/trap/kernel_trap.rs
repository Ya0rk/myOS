//！ `hal/la/trap/kernel_trap.rs`

use log::info;
use loongarch64::register::ecfg::LineBasedInterrupt;
use loongarch64::register::estat::{Exception, Interrupt, Trap};
use loongarch64::register::*;

use crate::mm::memory_space::PageFaultAccessType;
use crate::sync::{set_next_trigger, TIMER_QUEUE};
use crate::task::current_task;

#[no_mangle]
pub fn kernel_trap_handler() {
    // info!("kernel trap");
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
            // info!("timer interrupt from kernel");
            // ticlr::clear_timer_interrupt();
            TIMER_QUEUE.handle_expired();
            set_next_trigger();
        }
        Trap::Interrupt(Interrupt::HWI0) => {
            // 中断0 --- 外部中断处理
            unimplemented!("loongarch64 Trap::Interrupt(Interrupt::HWI0)");
        }
        Trap::Exception(e) => {
            match e {
                Exception::LoadPageFault |
                Exception::StorePageFault |
                Exception::FetchPageFault |
                Exception::PageModifyFault |
                Exception::PageNonReadableFault |
                Exception::PageNonExecutableFault => {
                    let va = badv::read().vaddr();  
                    info!("[kernel_trap_handler] meet a pagefault {:?} at {:#x}", e, va);
                    let access_type = match e {
                        Exception::LoadPageFault | Exception::PageNonReadableFault => {
                            PageFaultAccessType::RO
                        }
                        Exception::StorePageFault | Exception::PageModifyFault => {
                            PageFaultAccessType::RW
                        }
                        Exception::FetchPageFault | Exception::PageNonExecutableFault => {
                            PageFaultAccessType::RX
                        }
                        _=> {
                            unreachable!()
                        }
                    };
                    
                    let task = current_task().unwrap();
                    task.with_mut_memory_space(|m| {
                        m.handle_page_fault(va.into(), access_type)
                    }).unwrap_or_else(|e| {
                        use log::error;
                        task.set_zombie();
                        error!("{:?} pc: {:#x} BADV: {:#x}", estat.cause(), era.pc(), badv::read().vaddr());
                    });
                }

                _ => {
                    panic!(
                        "{:?} pc: {:#x} BADV: {:#x}",
                        estat.cause(),
                        era.pc(),
                        badv::read().vaddr()
                    );
                }
            }
        }
    }
    era::set_pc(era.pc());
    // info!("kernel trap end");
}
