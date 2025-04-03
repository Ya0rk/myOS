pub mod entry;
// pub mod mm;
pub mod sbi;
pub mod trap;
pub mod config;



pub use trap::user_trap::{user_trap_handler, user_trap_return};
use core::arch::global_asm;
use core::fmt::Display;
use riscv::register::mtvec::TrapMode;
use riscv::register::{satp, sstatus, stvec};


pub fn console_putchar(c: usize) {
    sbi::console_putchar(c);
}

pub fn console_getchar() -> usize {
    sbi::console_getchar()
}

pub fn set_timer(timer: usize) {
    sbi::set_timer(timer);
}

pub fn shutdown(failuer: bool) -> ! {
    sbi::shutdown(failuer)
}

/// use sbi call to start the specific core
pub fn hart_start_success(hartid: usize, start_addr: usize) -> bool {
    sbi::hart_start(hartid, start_addr)
}

/// Allows the current CPU to respond to interrupts
#[inline]
pub fn enable_irqs() {
    unsafe {sstatus::set_sie();}
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    unsafe { sstatus::clear_sie() }
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    sstatus::read().sie()
}

/// Relaxes the current CPU and waits for interrupts.
///
/// It must be called with interrupts enabled, otherwise it will never return.
#[inline]
pub fn wait_for_irqs() {
    unsafe {
        riscv::asm::wfi();   
    }
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
    satp::read().bits()
}

/// 获取当前CPU ID
#[inline]
pub fn get_current_hart_id() -> usize {
    let hartid: usize;
    unsafe {
        core::arch::asm!(
            "mv {}, tp",
            out(reg) hartid
        );
    }
    hartid
}

