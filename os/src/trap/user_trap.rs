use riscv::register::stval;
use riscv::register::scause::{self, Exception, Interrupt, Trap};
use crate::syscall::syscall;
use crate::timer::set_next_trigger;
use crate::task::{current_trap_cx, exit_current_and_run_next, get_current_hart_id, suspend_current_and_run_next};
use super::{__return_to_user, set_trap_handler, IndertifyMode};

#[no_mangle]
/// handle user interrupt, exception, or system call from user space
pub fn user_trap_handler() {
    // 设置kernel的trap handler entry
    set_trap_handler(IndertifyMode::Kernel);
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => { // 7
            let mut cx = current_trap_cx();
            let old_sepc: usize = cx.get_sepc();
            cx.set_sepc(old_sepc + 4);

            let result = syscall(cx.user_x[17], [cx.user_x[10], cx.user_x[11], cx.user_x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.user_x[10] = result as usize;
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
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => { // 2
            println!("[kernel] hart_id = {:?}, IllegalInstruction in application, kernel killed it.",
                get_current_hart_id()
            );
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
            set_next_trigger();
            suspend_current_and_run_next();
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
    unsafe {
        __return_to_user(trap_cx);
    }
}
