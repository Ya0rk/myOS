pub mod sbi;
pub mod context;
pub mod kernel_trap;
pub mod user_trap;
pub mod interrupt;
pub mod timer;

pub use context::TrapContext;
use user_trap::{user_trap_handler, user_trap_return};
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



global_asm!(include_str!("trap.S"));

// 申明外部函数，这些函数是在汇编代码中实现的，用于从用户态和内核态切换
extern "C" {
    fn __trap_from_user();
    fn __trap_from_kernel();
    #[allow(improper_ctypes)]
    fn __return_to_user(ctx: *mut TrapContext);
}

pub fn trap_init() {
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

/// 更换页表，刷新TLB，开启内存屏障
/// 传入的是satp的值
pub fn switch_pagetable(satp: usize) {
    unsafe {
        satp::write(satp);
        core::arch::asm!("sfence.vma");
    }
}
