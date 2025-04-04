#![allow(unused_import_braces)]
#![allow(unused)]
use log::info;
use crate::sync::{set_next_trigger, yield_now};
use crate::syscall::syscall;
use crate::task::{current_task, current_trap_cx, executor, get_current_hart_id};
use super::{__return_to_user, set_trap_handler, IndertifyMode};
/// 导入riscv架构相关的包
#[cfg(target_arch = "riscv64")]
use riscv::register::stval;
#[cfg(target_arch = "riscv64")]
use riscv::register::scause::{self, Exception, Interrupt, Trap};

#[cfg(target_arch = "riscv64")]
#[no_mangle]
/// handle user interrupt, exception, or system call from user space
pub async fn user_trap_handler() {
    // 设置kernel的trap handler entry
    set_trap_handler(IndertifyMode::Kernel);
    let scause = scause::read();
    let stval = stval::read();
    let task = current_task().unwrap();

    if task.get_time_data().usedout_timeslice() && executor::has_task() {
        // log::info!("time slice used up, yield now");
        yield_now().await;
    }

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => { // 7
            let mut cx = current_trap_cx();
            let old_sepc: usize = cx.get_sepc();
            let syscall_id = cx.user_x[17];
            cx.set_sepc(old_sepc + 4);

            let result = syscall(
                syscall_id, 
                [cx.user_x[10], 
                cx.user_x[11], 
                cx.user_x[12], 
                cx.user_x[13], 
                cx.user_x[14],
                cx.user_x[15]]
            ).await;

            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            
            match result {
                Ok(ret) => {
                    cx.user_x[10] = ret as usize;
                }
                Err(err) => {
                    // TODO：这里单独处理的waitpid返回值情况，后序要修改
                    if err as isize == -1 || err as isize == -2 {
                        cx.user_x[10] = err as usize;
                    } else {
                        cx.user_x[10] = -(err as isize) as usize;
                        info!("[syscall ret] sysID = {}, errmsg: {}", syscall_id, err.get_info());
                    }
                }
            }
            
            
        }
        Trap::Exception(Exception::StoreFault) // 6
        | Trap::Exception(Exception::StorePageFault) // 11
        | Trap::Exception(Exception::InstructionFault) // 1
        | Trap::Exception(Exception::InstructionPageFault) // 9
        | Trap::Exception(Exception::LoadFault) // 4
        | Trap::Exception(Exception::LoadPageFault) => { // 10
            println!(
                "[kernel] hart_id = {:?}, {:?} = {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                get_current_hart_id(),
                scause.bits(),
                scause.cause(),
                stval,
                current_trap_cx().get_sepc(),
            );
            // page fault exit code
            // exit_current_and_run_next(-2);
            task.set_zombie();
        }
        Trap::Exception(Exception::IllegalInstruction) => { // 2
            println!("[kernel] hart_id = {:?}, IllegalInstruction in application, kernel killed it.",
                get_current_hart_id()
            );
            // illegal instruction exit code
            // exit_current_and_run_next(-3);
            task.set_zombie();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
            set_next_trigger();
            yield_now().await;
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
#[cfg(target_arch = "loongarch64")]
pub async fn user_trap_handler() {
    unimplemented!()
}

#[no_mangle]
pub fn user_trap_return() {
    // 重新修改stvec设置 user 的trap handler entry
    set_trap_handler(IndertifyMode::User);

    let trap_cx = current_trap_cx();
    trap_cx.float_regs.trap_out_do_with_freg();

    let task = current_task().unwrap();

    task.get_time_data_mut().set_trap_out_time();
    unsafe { __return_to_user(trap_cx); }
    task.get_time_data_mut().set_trap_in_time();

    trap_cx.float_regs.trap_in_do_with_freg(trap_cx.sstatus);
}
