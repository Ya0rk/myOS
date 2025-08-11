use core::sync::atomic::Ordering;

#[cfg(target_arch = "riscv64")]
use riscv::register::{sie, sstatus};

use crate::{drivers::{device_new::manager::DEVICE_MANAGER}, START_HART_ID};

const VIRT_PLIC: usize = 0xffff_ffc0_0000_0000 + 0xC00_0000;

#[cfg(target_arch = "riscv64")]
#[inline(always)]
/// 打开 supervisor 模式的中断
pub fn enable_supervisor_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
/// 关闭 supervisor 模式的中断
pub fn disable_supervisor_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub fn supervisor_interrupt_is_enabled() -> bool {
    sstatus::read().sie()
}

/// enable timer interrupt in sie CSR
#[cfg(target_arch = "riscv64")]
#[inline(always)]
pub unsafe fn enable_supervisor_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[inline(always)]
pub unsafe fn enable_supervisor_extern_interrupt() {
    unsafe {
        sie::set_sext();
    }
}

#[inline(always)]
pub unsafe fn disenable_supervisor_extern_interrupt() {
    unsafe {
        sie::clear_sext();
    }
}

pub fn plic_init() {
    use crate::hal::arch::plic::*;
    use riscv::register::sie;
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    // let hart_id: usize = START_HART_ID.load(Ordering::SeqCst);
    let supervisor = IntrTargetPriority::Supervisor;
    let machine = IntrTargetPriority::Machine;
    // 设置PLIC中外设中断的阈值
    for hart_id in 0..2 {
        plic.set_threshold(hart_id, supervisor, 0);
        plic.set_threshold(hart_id, machine, 0);
        // 使能PLIC在CPU处于S-Mode下传递键盘/鼠标/块设备/串口外设中断
        // irq nums: 5 keyboard, 6 mouse, 8 block, 32 uart
        for intr_src_id in [5usize, 6, 8, 10, 32] {
            plic.enable(hart_id, supervisor, intr_src_id);
            plic.enable(hart_id, machine, intr_src_id);
            plic.set_priority(intr_src_id, 6);
        }
    }

    // 设置S-Mode CPU使能中断
    unsafe {
        sstatus::set_sie();
        sie::set_sext();
        // 临时启动uart设备的中断功能
        // ((0xffff_ffc0_0000_0000 as usize + 0x1000_0001) as *mut u8).write_volatile(0x01);
    }
}
pub fn irq_handler() {
    use crate::hal::arch::plic::*;
    use riscv::register::sie;
    disable_supervisor_interrupt();
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let hart_id = START_HART_ID.load(Ordering::SeqCst);
    let intr_src_id = plic.claim(hart_id, IntrTargetPriority::Supervisor);
    DEVICE_MANAGER.read().handle_irq(intr_src_id as usize);
    // use log::info;
    // match intr_src_id {
    //     5 => info!("[irq_handler] extern irq from keyboard"),
    //     6 => info!("[irq_handle] extern irq from mouse"),
    //     8 => info!("[irq_handle] extern irq from blockd evice"),
    //     32 => {
    //         // TODO: 暂时硬编码，后面需要实现
    //         info!("[irq_handler] extern irq from uart");
    //         SERIAL_DRIVER.handle_irq();
    //     },
    //     _ => panic!("unsupported IRQ {}", intr_src_id),
    // }
    plic.complete(hart_id, IntrTargetPriority::Supervisor, intr_src_id);
}

/// A guard that disable interrupt when it is created and enable interrupt when it is dropped.
pub struct InterruptGuard {
    interrupt_before: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        let interrupt_before = supervisor_interrupt_is_enabled();
        disable_supervisor_interrupt();
        Self { interrupt_before }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.interrupt_before {
            enable_supervisor_interrupt();
        }
    }
}


pub trait IrqHandler {
    fn handle_irq(&self);
}
