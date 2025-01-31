mod context;
pub use context::TrapContext;
use crate::syscall::syscall;
use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next
};
use crate::timer::set_next_trigger;
use crate::utils::backtrace;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

// 申明外部函数，这些函数是在汇编代码中实现的，用于从用户态和内核态切换
extern {
    fn __trap_from_user();
    fn __trap_from_kernel();
    #[allow(improper_ctypes)]
    fn __return_to_user(ctx: *mut TrapContext, satp: usize);
}

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

// 在trap handler中设置内核态的trap entry
fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
    }
}

// 在trap return中重新修改stvec设置用户态的trap entry
fn set_user_trap_entry() {
    unsafe {
        stvec::write(__trap_from_user as usize, TrapMode::Direct);
    }
}
/// enable timer interrupt in sie CSR
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

pub fn trap_loop() {
    loop {
        trap_return();
        trap_handler(); 
    }
}

#[no_mangle]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler() {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => { // 7
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // get system call return value
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
                "[kernel] {:?} = {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.bits(),
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => { // 2
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => { // 5
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // trap_return();
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() {
    set_user_trap_entry();

    let trap_cx = current_trap_cx();
    let satp = current_user_token();
    unsafe {
        __return_to_user(trap_cx, satp);
    }
}

#[no_mangle]
/// Unimplement: traps/interrupts/exceptions from kernel mode
/// Todo: Chapter 9: I/O device
pub fn kernel_trap_handler() {
    backtrace();
    println!("nihoa : a trap {:?} = {:?} from kernel! stval = {:?}, sepc = {:#x}",
            scause::read().cause(),
            scause::read().bits(),
            stval::read(),
            current_trap_cx().sepc,
        );
    panic!("a trap {:?} from kernel!", scause::read().cause());
}