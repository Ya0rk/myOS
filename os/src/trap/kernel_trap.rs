use riscv::register::{scause::{self, Interrupt, Trap}, stval};
use crate::{sync::set_next_trigger, task::current_trap_cx, utils::backtrace};

#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn kernel_trap_handler() {
    backtrace();
    let scause = scause::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
            set_next_trigger();
        }
        _ => {
            println!("nihoa : a trap {:?} = {:?} from kernel! stval = {:?}, sepc = {:#x}",
                scause::read().cause(),
                scause::read().bits(),
                stval::read(),
                current_trap_cx().get_sepc(),
            );
            panic!("a trap {:?} from kernel!", scause::read().cause());
        }
    }

    
   
}