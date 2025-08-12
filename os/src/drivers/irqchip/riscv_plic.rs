//! Adapted from rCore
//! Refactored by Sean Lin


use alloc::sync::Arc;
use sbi_spec::pmu::NUM_COUNTERS;

use crate::{drivers::{device_new::dev_core::{PhysDevice, PhysDriver, PhysDriverProbe}, irqchip::IrqController}, hal::config::KERNEL_ADDR_OFFSET};

#[allow(clippy::upper_case_acronyms)]
pub struct PLIC {
    base_addr: usize,
}

#[derive(Copy, Clone)]
pub enum IntrTargetPriority {
    Machine = 0,
    Supervisor = 1,
}

impl IntrTargetPriority {
    pub fn supported_number() -> usize {
        2
    }
}

impl PLIC {
    fn priority_ptr(&self, intr_source_id: usize) -> *mut u32 {
        assert!(intr_source_id > 0 && intr_source_id <= 132);
        (self.base_addr + intr_source_id * 4) as *mut u32
    }
    fn hart_id_with_priority(hart_id: usize, target_priority: IntrTargetPriority) -> usize {
        let priority_num = IntrTargetPriority::supported_number();
        hart_id * priority_num + target_priority as usize
    }
    fn enable_ptr(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        intr_source_id: usize,
    ) -> (*mut u32, usize) {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        let (reg_id, reg_shift) = (intr_source_id / 32, intr_source_id % 32);
        (
            (self.base_addr + 0x2000 + 0x80 * id + 0x4 * reg_id) as *mut u32,
            reg_shift,
        )
    }
    fn threshold_ptr_of_hart_with_priority(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
    ) -> *mut u32 {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        (self.base_addr + 0x20_0000 + 0x1000 * id) as *mut u32
    }
    fn claim_comp_ptr_of_hart_with_priority(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
    ) -> *mut u32 {
        let id = Self::hart_id_with_priority(hart_id, target_priority);
        (self.base_addr + 0x20_0004 + 0x1000 * id) as *mut u32
    }
    pub unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }
    pub fn set_priority(&self, intr_source_id: usize, priority: u32) {
        assert!(priority < 8);
        unsafe {
            self.priority_ptr(intr_source_id).write_volatile(priority);
        }
    }
    #[allow(unused)]
    pub fn get_priority(&self, intr_source_id: usize) -> u32 {
        unsafe { self.priority_ptr(intr_source_id).read_volatile() & 7 }
    }
    pub fn enable(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        intr_source_id: usize,
    ) {
        let (reg_ptr, shift) = self.enable_ptr(hart_id, target_priority, intr_source_id);
        unsafe {
            reg_ptr.write_volatile(reg_ptr.read_volatile() | 1 << shift);
        }
    }
    #[allow(unused)]
    pub fn disable(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        intr_source_id: usize,
    ) {
        let (reg_ptr, shift) = self.enable_ptr(hart_id, target_priority, intr_source_id);
        unsafe {
            reg_ptr.write_volatile(reg_ptr.read_volatile() & (!(1u32 << shift)));
        }
    }
    pub fn set_threshold(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        threshold: u32,
    ) {
        assert!(threshold < 8);
        let threshold_ptr = self.threshold_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe {
            threshold_ptr.write_volatile(threshold);
        }
    }
    #[allow(unused)]
    pub fn get_threshold(&self, hart_id: usize, target_priority: IntrTargetPriority) -> u32 {
        let threshold_ptr = self.threshold_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe { threshold_ptr.read_volatile() & 7 }
    }
    pub fn claim(&self, hart_id: usize, target_priority: IntrTargetPriority) -> u32 {
        let claim_comp_ptr = self.claim_comp_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe { claim_comp_ptr.read_volatile() }
    }
    pub fn complete(
        &self,
        hart_id: usize,
        target_priority: IntrTargetPriority,
        completion: u32,
    ) {
        let claim_comp_ptr = self.claim_comp_ptr_of_hart_with_priority(hart_id, target_priority);
        unsafe {
            claim_comp_ptr.write_volatile(completion);
        }
    }
}

impl IrqController for PLIC {
    fn enable_irq(&self, hart_id: usize, irq_no: usize) {
        // todo!()

        self.enable(hart_id, IntrTargetPriority::Machine, irq_no);
        self.enable(hart_id, IntrTargetPriority::Supervisor, irq_no);
        self.set_priority(irq_no, 6);
    }

    fn disable_irq(&self, hart_id: usize, irq_no: usize) {
        // todo!()

        self.disable(hart_id, IntrTargetPriority::Machine, irq_no);
        self.disable(hart_id, IntrTargetPriority::Supervisor, irq_no);
    }

    fn claim_irq(&self, hart_id: usize) -> Option<usize> {
        // todo!()
        Some(self.claim(hart_id, IntrTargetPriority::Supervisor) as usize)
    }

    fn finish_irq(&self, hart_id: usize, irq_no: usize) {
        // todo!()
        self.complete(hart_id, IntrTargetPriority::Supervisor, irq_no as u32);
    }
}

impl PhysDriver for PLIC {
    fn irq_number(&self) -> Option<usize> {
        None
    }
}

impl<'b, 'a> PhysDriverProbe<'b, 'a> for PLIC {
    fn probe(fdt: &'b flat_device_tree::Fdt<'a>) -> Option<Arc<Self>> {
        // todo!()
        let plic_node = fdt.find_node("/soc/plic@c000000")
        .or( fdt.find_compatible(&["riscv,plic0", "sifive,plic-1.0.0"]) )?;
        
        let mmio_base = plic_node.reg().next().unwrap().starting_address as usize + KERNEL_ADDR_OFFSET;
        let plic = unsafe{ PLIC::new(mmio_base) };

        #[cfg(feature = "vf2")]
        let NUM_HARTS = 2;
        #[cfg(feature = "board_qemu")]
        let NUM_HARTS = 1;

        let supervisor = IntrTargetPriority::Supervisor;
        let machine = IntrTargetPriority::Machine;
        let threshold = 0;

        for hart_id in 0..NUM_HARTS {
            plic.set_threshold(hart_id, machine, threshold);
            plic.set_threshold(hart_id, supervisor, threshold);
        }

        Some(Arc::new(plic))
    }
}