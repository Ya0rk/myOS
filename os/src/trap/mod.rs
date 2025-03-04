mod context;
mod user_trap;
mod kernel_trap;

pub use context::TrapContext;
use user_trap::{user_trap_handler, user_trap_return};
use core::arch::global_asm;
use core::fmt::Display;
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec;

global_asm!(include_str!("trap.S"));

// 申明外部函数，这些函数是在汇编代码中实现的，用于从用户态和内核态切换
extern "C" {
    fn __trap_from_user();
    fn __trap_from_kernel();
    #[allow(improper_ctypes)]
    fn __return_to_user(ctx: *mut TrapContext);
}

pub fn init() {
    set_trap_handler(IndertifyMode::Kernel);
}

pub fn trap_loop() {
    loop {
        user_trap_return();
        user_trap_handler(); 
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
enum IndertifyMode {
    User,
    Kernel,
}

impl Display for IndertifyMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IndertifyMode::User => write!(f, "User"),
            IndertifyMode::Kernel => write!(f, "Kernel"),
        }
    }
}

fn set_trap_handler(mode: IndertifyMode) {
    match mode {
        IndertifyMode::User => {
            unsafe {
                stvec::write(__trap_from_user as usize, TrapMode::Direct);
            }
        },
        IndertifyMode::Kernel => {
            unsafe {
                stvec::write(__trap_from_kernel as usize, TrapMode::Direct);
            }
        },
    }
}