#![allow(unused_variables)]

pub mod context;
pub mod uart;
pub mod user_trap;
pub mod timer;
pub mod interrupt;
#[macro_use]
pub mod print;

pub use context::TrapContext;
use user_trap::{user_trap_handler, user_trap_return};
use core::fmt::Display;
/// 在qemu模拟器中URAT的地址为这个
pub const UART_ADDR: usize = 0x1FE001E0;

core::arch::global_asm!(include_str!("trap.s"));

// 申明外部函数，这些函数是在汇编代码中实现的，用于从用户态和内核态的切换
extern "C" {
    fn __trap_from_user();
    fn __trap_from_kernel();
    #[allow(improper_ctypes)]
    fn __return_to_user(ctx: *mut TrapContext);
}

pub fn console_putchar(c: usize) {
    print!("{}", c);
}

pub fn console_getchar() -> usize {
    print::get_char() as usize
}


pub fn set_timer(time: usize) {
    unimplemented!()
}

pub fn shutdown(failuer: bool) -> ! {
    unimplemented!()
}

pub fn hart_start_success(hartid: usize, start_addr: usize) -> bool {
    true
}

pub fn trap_init() {
    unimplemented!()
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

#[allow(unused)]
fn set_trap_handler(mode: IndertifyMode) {
    match mode {
        IndertifyMode::User => {
            unimplemented!()
        },
        IndertifyMode::Kernel => {
            unimplemented!()
        },
    }
}


/// Allows the current CPU to respond to interrupts
#[inline]
pub fn enable_irqs() {
    unimplemented!()
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    unimplemented!()
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    unimplemented!()
}

/// Relaxes the current CPU and waits for interrupts.
///
/// It must be called with interrupts enabled, otherwise it will never return.
#[inline]
pub fn wait_for_irqs() {
    unimplemented!()
}

/// CPU停顿，不再取指
#[inline]
pub fn halt() {
    disable_irqs();
    wait_for_irqs();
}

/// 获取当前程序的页表的物理地址
#[inline]
pub fn read_page_table_root() -> usize {
    unimplemented!()
}

/// 获取当前CPU ID
/// unimplemented!
#[inline]
pub fn get_current_hart_id() -> usize {
    unimplemented!()
}

/// 更换页表，刷新TLB，开启内存屏障
/// 传入的是satp的值
pub fn switch_pagetable(satp: usize) {
    unimplemented!()
}



