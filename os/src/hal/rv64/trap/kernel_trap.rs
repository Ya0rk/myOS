use crate::{
    mm::memory_space::PageFaultAccessType,
    sync::set_next_trigger,
    task::{current_task, current_trap_cx, set_ktrap_ret},
    utils::{backtrace, Errno, SysResult},
};

#[cfg(target_arch = "riscv64")]
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sepc, stval,
};

#[cfg(target_arch = "riscv64")]
#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn kernel_trap_handler() {
    use core::result;

    use log::info;

    use crate::{sync::TIMER_QUEUE, task::get_current_cpu, utils::SysResult};
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();
    let cause = scause.cause();
    let mut result: SysResult<()> = Ok(());
    match cause {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 5
            // info!("[kernel_trap_handler] kernel timer interrupt");

            use log::error;

            error!("[kernel_trap_handler] kernel timer interrupt");
            TIMER_QUEUE.handle_expired();
            get_current_cpu().timer_irq_inc();
            set_next_trigger();
        }
        Trap::Exception(e) => match e {
            Exception::StorePageFault
            | Exception::InstructionPageFault
            | Exception::LoadPageFault => {
                // log::info!(
                //         "[kernel_trap_handler] encounter page fault, addr {stval:#x}, instruction {sepc:#x} scause {cause:?}",
                // );
                let access_type = match e {
                    Exception::InstructionPageFault => PageFaultAccessType::RX,
                    Exception::LoadPageFault => PageFaultAccessType::RO,
                    Exception::StorePageFault => PageFaultAccessType::RW,
                    _ => unreachable!(),
                };

                let task = current_task().unwrap_or_else(
                    || {
                        panic!("No task! bad addr:{:#x}", stval);
                    }
                );
                // task.switch_pgtable();
                result = task.with_mut_memory_space(|m| {
                    // info!("[kernel_trap_page_fault] task id = {}", task.get_pid());
                    m.handle_page_fault(stval.into(), access_type)
                });
                result.is_err().then(|| {
                    use crate::hal::arch::current_inst_len;

                    sepc::write(sepc + current_inst_len());
                });
                // .unwrap_or_else(|e| {
                //     use log::error;

                //     task.set_zombie();
                //     error!("kernel trap:{:?} pc: {:#x} BADV: {:#x}", cause, sepc, stval);
                // });
            }
            _ => {
                result = Err(Errno::EINVAL);
                panic!("a trap {:?} from kernel!", scause::read().cause());
            }
        },
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            use log::error;
            error!("got a supervisor external interrupt. do nothing");
            crate::hal::arch::interrupt::irq_handler();
        },
        _ => {
            result = Err(Errno::EINVAL);
            panic!("a trap {:?} from kernel!", scause::read().cause());
        }
    }
    set_ktrap_ret(result);
}

#[cfg(target_arch = "loongarch64")]
#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn kernel_trap_handler() {
    unimplemented!()
}
