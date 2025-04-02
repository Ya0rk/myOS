use riscv::register::{scause::{self, Exception, Interrupt, Trap}, sepc, stval};
use crate::{mm::memory_space::PageFaultAccessType, sync::set_next_trigger, task::{current_task, current_trap_cx}, utils::backtrace};

#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn kernel_trap_handler() {
    // backtrace();
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();
    let cause = scause.cause();
    match cause {
        Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
            set_next_trigger();
        },
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

                let result = current_task().unwrap().with_mut_memory_space(|m| {
                    m.handle_page_fault(stval.into(), access_type)
                });
                // if let Err(_e) = result {
                //     log::warn!(
                //         "[trap_handler] encounter page fault, addr {stval:#x}, instruction {sepc:#x} scause {cause:?}",
                //     );
                //     log::warn!("{:x?}", current_task().unwrap().trap_context_mut());
                //     log::warn!("bad memory access, send SIGSEGV to task");
                //     current_task_ref().receive_siginfo(
                //         SigInfo {
                //             sig: Sig::SIGSEGV,
                //             code: SigInfo::KERNEL,
                //             details: SigDetails::None,
                //         },
                //         false,
                //     );
                // }
            },
            _ => {
                panic!("a trap {:?} from kernel!", scause::read().cause());
            },
        },
        _ => {
            // println!("nihoa : a trap {:?} = {:?} from kernel! stval = {:?}, sepc = {:#x}",
            //     scause::read().cause(),
            //     scause::read().bits(),
            //     stval::read(),
            //     current_trap_cx().get_sepc(),
            // );
            panic!("a trap {:?} from kernel!", scause::read().cause());
        }
    }

    
   
}