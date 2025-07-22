#![allow(unused_import_braces)]
#![allow(unused)]
use super::{__return_to_user, set_trap_handler, IndertifyMode, TrapContext};
use crate::hal::arch::sstatus::FS;
use crate::mm::memory_space::PageFaultAccessType;
use crate::sync::{set_next_trigger, yield_now};
use crate::syscall::syscall;
use crate::task::{current_task, current_trap_cx, executor, get_current_cpu, get_current_hart_id};
use log::info;
#[cfg(target_arch = "riscv64")]
use riscv::register::scause::{self, Exception, Interrupt, Trap};
/// 导入riscv架构相关的包
#[cfg(target_arch = "riscv64")]
use riscv::register::{sepc, stval};

#[cfg(target_arch = "riscv64")]
#[no_mangle]
/// handle user interrupt, exception, or system call from user space
pub async fn user_trap_handler() {
    // 设置kernel的trap handler entry

    use crate::sync::TIMER_QUEUE;
    set_trap_handler(IndertifyMode::Kernel);
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();
    let cause = scause.cause();
    let task = current_task().unwrap();
    // println!("stval = {:#x}", stval);

    // if task.get_time_data().usedout_timeslice() && executor::has_task() {
    //     yield_now().await;
    // }

    match cause {
        Trap::Exception(Exception::UserEnvCall) => { // 7
            let mut cx = current_trap_cx();
            let old_sepc: usize = cx.get_sepc();
            let syscall_id = cx.user_gp.a7;
            cx.set_sepc(old_sepc + 4);

            let result = syscall(
                syscall_id, 
                [cx.user_gp.a0, 
                cx.user_gp.a1, 
                cx.user_gp.a2, 
                cx.user_gp.a3, 
                cx.user_gp.a4,
                cx.user_gp.a5]
            ).await;

            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();

            match result {
                Ok(ret) => {
                    cx.user_gp.a0 = ret as usize;
                }
                Err(err) => {
                    // TODO：这里单独处理的waitpid返回值情况，后序要修改
                    if (err as isize) < 0 {
                        cx.user_gp.a0 = err as usize;
                    } else {
                        cx.user_gp.a0 = (-(err as isize)) as usize;
                        info!("[syscall ret] sysID = {}, errmsg: {}", syscall_id, err.get_info());
                    }
                }
            }
        }
        Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadPageFault)
        | Trap::Exception(Exception::InstructionPageFault) => {
            let access_type = match cause {
                Trap::Exception(Exception::InstructionPageFault) => PageFaultAccessType::RX,
                Trap::Exception(Exception::LoadPageFault) => PageFaultAccessType::RO,
                Trap::Exception(Exception::StorePageFault) => PageFaultAccessType::RW,
                _ => unreachable!(),
            };

            let result = current_task().unwrap().with_mut_memory_space(|m| {
                m.handle_page_fault(stval.into(), access_type)
            }).unwrap_or_else(|e| {
                use log::error;
                task.set_zombie();
                // println!("task {} 's children len = {}", task.get_pid(), task.children.lock().len());
                error!("user trap: {:?} pc: {:#x} BADV: {:#x}", cause, sepc, stval);
            });;


        }
        Trap::Exception(Exception::StoreFault) // 6
        | Trap::Exception(Exception::InstructionFault) // 1
        | Trap::Exception(Exception::LoadFault) => { // 10
            println!(
                "[user_trap] hart_id = {:?}, {:?} = {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                get_current_hart_id(),
                scause.bits(),
                scause.cause(),
                stval,
                current_trap_cx().get_sepc(),
            );
            task.set_zombie();
        }
        Trap::Exception(Exception::IllegalInstruction) => { // 2
            println!("[user_trap] hart_id = {:?}, IllegalInstruction in application, stval = {:#x}, sepc = {:#x}, kernel killed it.",
                get_current_hart_id(), stval, sepc
            );
            task.set_zombie();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
            TIMER_QUEUE.handle_expired();
            set_next_trigger();
            yield_now().await;
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            use log::error;
            info!("got a supervisor external interrupt. do nothing");
            crate::hal::arch::interrupt::irq_handler();
        }
        _ => {
            panic!(
                "hart_id = {:?}, Unsupported trap {:?}, stval = {:#x}!",
                get_current_hart_id(),
                scause.cause(),
                stval
            );
        }
    }
}

#[no_mangle]
pub fn user_trap_return() {
    // 重新修改stvec设置 user 的trap handler entry
    set_trap_handler(IndertifyMode::User);

    let trap_cx = current_trap_cx();
    trap_cx.float_regs.trap_out_do_with_freg();
    trap_cx.sstatus.set_fs(FS::Clean);

    get_current_cpu().timer_irq_reset();
    let task = current_task().unwrap();
    task.get_time_data_mut().set_trap_out_time();
    unsafe {
        __return_to_user(trap_cx);
    }
    task.get_time_data_mut().set_trap_in_time();

    trap_cx.float_regs.trap_in_do_with_freg(trap_cx.sstatus);
}
