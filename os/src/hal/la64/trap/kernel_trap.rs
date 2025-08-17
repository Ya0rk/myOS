//！ `hal/la/trap/kernel_trap.rs`

use log::info;
use loongarch64::register::ecfg::LineBasedInterrupt;
use loongarch64::register::estat::{Exception, Interrupt, Trap};
use loongarch64::register::*;

use crate::drivers::device::manager::DEVICE_MANAGER;
use crate::mm::memory_space::PageFaultAccessType;
use crate::sync::{disable_supervisor_interrupt, set_next_trigger, TIMER_QUEUE};
use crate::task::{current_task, get_current_cpu, get_current_hart_id, set_ktrap_ret};
use crate::utils::SysResult;

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
    let mut result: SysResult<()> = Ok(());
    match estat.cause() {
        Trap::Interrupt(Interrupt::Timer) => {
            // 清除时钟专断
            // info!("timer interrupt from kernel");
            // ticlr::clear_timer_interrupt();
            TIMER_QUEUE.handle_expired();
            get_current_cpu().timer_irq_inc();
            set_next_trigger();
        }
        Trap::Interrupt(Interrupt::HWI0) => {
            // 中断0 --- 外部中断处理
            // unimplemented!("loongarch64 Trap::Interrupt(Interrupt::HWI0)");
            // disable_supervisor_interrupt();
            let hart_id = get_current_hart_id();
            DEVICE_MANAGER.read().handle_irq(hart_id);
        }
        Trap::Exception(e) => {
            match e {
                Exception::LoadPageFault
                | Exception::StorePageFault
                | Exception::FetchPageFault
                | Exception::PageModifyFault
                | Exception::PageNonReadableFault
                | Exception::PageNonExecutableFault => {
                    let va = badv::read().vaddr();
                    info!(
                        "[kernel_trap_handler] meet a pagefault {:?} at {:#x}",
                        e, va
                    );
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
                        _ => {
                            unreachable!()
                        }
                    };

                    let task = current_task().unwrap_or_else(
                        || {
                            panic!(
                                "{:?} pc: {:#x} BADV: {:#x}",
                                estat.cause(),
                                era.pc(),
                                badv::read().vaddr()
                            )
                        },
                    );
                    result =
                        task.with_mut_memory_space(|m| m.handle_page_fault(va.into(), access_type));

                    result.is_err().then(|| {
                        use crate::hal::arch::current_inst_len;
                        info!("skip inst");
                        era::set_pc(era.pc() + current_inst_len());
                    });

                    // .unwrap_or_else(|e| {
                    //     use log::error;
                    //     task.set_zombie();
                    //     error!("{:?} pc: {:#x} BADV: {:#x}", estat.cause(), era.pc(), badv::read().vaddr());
                    // });
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
        _ => {
            // 其他中断
            panic!(
                "Unhandled trap {:?} pc: {:#x} BADV: {:#x}",
                estat.cause(),
                era.pc(),
                badv::read().vaddr()
            );
        }
    }
    // era::set_pc(era.pc());
    set_ktrap_ret(result);
    // info!("kernel trap end");
}
