use riscv::register::{scause, stval};
use crate::{task::current_trap_cx, utils::backtrace};

#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn kernel_trap_handler() {
    backtrace();
    println!("nihoa : a trap {:?} = {:?} from kernel! stval = {:?}, sepc = {:#x}",
            scause::read().cause(),
            scause::read().bits(),
            stval::read(),
            current_trap_cx().get_sepc(),
        );
    panic!("a trap {:?} from kernel!", scause::read().cause());
}