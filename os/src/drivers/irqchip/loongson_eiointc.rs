//! Written by Sean Lin
// QEMU extioi 驱动，未实现轮转分发

use alloc::{sync::Arc, vec};
use bitflags::bitflags;
use log::error;
use loongarch64::iocsr::{iocsr_read_b, iocsr_read_d, iocsr_read_h, iocsr_read_w, iocsr_write_b, iocsr_write_d, iocsr_write_h, iocsr_write_w};
use core::{arch::asm, error, ptr::{read_volatile, write_volatile}};

use crate::drivers::irqchip::{loongson_pch_pic::PCHIntController, IrqController};

const OFFSET_MISC_FUNC_REG: usize = 0x420;
const SHIFT_EXTIOI_EN: usize = 48;

const OFFSET_EXT_IOIEN: usize = 0x1600;
const OFFSET_EXT_IOIBOUNCE: usize = 0x1680;
const OFFSET_EXT_IOISR: usize = 0x1700;
const OFFSET_CORE_EXT_IOISR: [usize; 4] = [0x1800, 0x1900, 0x1a00, 0x1b00];
const OFFSET_EXT_IOIMAP: [usize; 8] = [0x14c0, 0x14c1, 0x14c2, 0x14c3, 0x14c4, 0x14c5, 0x14c6, 0x14c7];
const OFFSET_EXT_IOIMAP_CORE: usize = 0x1c00;
const OFFSET_EXT_NODE_TYPE: [usize; 16] = [0x14a0, 0x14a2, 0x14a4, 0x14a6, 0x14a8, 0x14aa, 0x14ac, 0x14ae, 0x14b0, 0x14b2, 0x14b4, 0x14b6, 0x14b8, 0x14ba, 0x14bc, 0x14be];

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct IOIMapVector: u8 {
        const PIN_0 = 1 << 0;
        const PIN_1 = 1 << 1;
        const PIN_2 = 1 << 2;
        const PIN_3 = 1 << 3;
    }
}

impl IOIMapVector {
    pub fn new(pin: u8) -> Option<Self> {
        if pin > 3 {
            return None;
        }
        Some(Self::from_bits_truncate(1 << pin))
    }
    
}


bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct NodeVector: u16 {
        const MAP_0 = 1 << 0;
        const MAP_1 = 1 << 1;
        const MAP_2 = 1 << 2;
        const MAP_3 = 1 << 3;
        const MAP_4 = 1 << 4;
        const MAP_5 = 1 << 5;
        const MAP_6 = 1 << 6;
        const MAP_7 = 1 << 7;
    }
}


// TODO: bounce
// 目前不支持轮转，必须只有一个置位
impl NodeVector {
    pub fn new(map: u8) -> Option<Self> {
        if map > 7 {
            return None;
        }
        Some(Self::from_bits_truncate(1 << map))
    }
    pub fn map(&self) -> u8 {
        match *self {
            NodeVector::MAP_0 => 0,
            NodeVector::MAP_1 => 1,
            NodeVector::MAP_2 => 2,
            NodeVector::MAP_3 => 3,
            NodeVector::MAP_4 => 4,
            NodeVector::MAP_5 => 5,
            NodeVector::MAP_6 => 6,
            NodeVector::MAP_7 => 7,
            _ => 0
        }
    }
}

bitflags! {
    struct ConfigVector: u8 {
        const CORE_0 = 1 << 0;
        const CORE_1 = 1 << 1;
        const CORE_2 = 1 << 2;
        const CORE_3 = 1 << 3;
    }
}

impl ConfigVector {
    pub fn new(core: u8, node: u8) -> Option<Self> {
        if core > 3 || node > 15 {
            return None;
        }
        let core_flag = 1 << core;
        let node_data = node << 4;
        Some(Self::from_bits_truncate(core_flag | node_data))
    }

    pub fn node(&self) -> u8 {
        (self.bits() >> 4) as u8
    }

}


pub struct ExtIOIntController {
    base_addr: usize,
    use_iocsr: bool,
    next_icu: Arc<PCHIntController>,
}


impl ExtIOIntController {
    pub const fn new(base_addr: usize, next_icu: Arc<PCHIntController>) -> Self {
        Self { base_addr, use_iocsr: true, next_icu}
    }

    #[inline(always)]
    pub fn read_reg8(&self, off_addr: usize) -> u8 {
        if self.use_iocsr{
            iocsr_read_b(off_addr)
        }
        else {
            unsafe { read_volatile((self.base_addr + off_addr) as *const u8) }
        }
    }

    #[inline(always)]
    pub fn write_reg8(&self, off_addr: usize, value: u8) {
        if self.use_iocsr{
            iocsr_write_b(off_addr, value);
        }
        else {
            unsafe { write_volatile((self.base_addr + off_addr) as *mut u8, value) };
        }
    }

    #[inline(always)]
    pub fn read_reg16(&self, off_addr: usize) -> u16 {
        if self.use_iocsr{
            iocsr_read_h(off_addr)
        }
        else {
            unsafe { read_volatile((self.base_addr + off_addr) as *const u16) }
        }
    }

    #[inline(always)]
    pub fn write_reg16(&self, off_addr: usize, value: u16) {
        if self.use_iocsr{
            iocsr_write_h(off_addr, value);
        }
        else {
            unsafe { write_volatile((self.base_addr + off_addr) as *mut u16, value) };
        }
    }

    #[inline(always)]
    pub fn read_reg32(&self, off_addr: usize) -> u32 {
        if self.use_iocsr{
            iocsr_read_w(off_addr)
        }
        else {
            unsafe { read_volatile((self.base_addr + off_addr) as *const u32) }
        }
    }

    #[inline(always)]
    pub fn write_reg32(&self, off_addr: usize, value: u32) {
        if self.use_iocsr{
            iocsr_write_w(off_addr, value);
        }
        else {
            unsafe { write_volatile((self.base_addr + off_addr) as *mut u32, value) };
        }
    }

    #[inline(always)]
    pub fn read_reg64(&self, off_addr: usize) -> u64 {
        if self.use_iocsr{
            iocsr_read_d(off_addr)
        }
        else {
            unsafe { read_volatile((self.base_addr + off_addr) as *const u64) }
        }
    }

    #[inline(always)]
    pub fn write_reg64(&self, off_addr: usize, value: u64) {
        if self.use_iocsr{
            iocsr_write_d(off_addr, value);
        }
        else {
            unsafe { write_volatile((self.base_addr + off_addr) as *mut u64, value) };
        }
    }

    pub fn enable(&self, irq: u32) {
        let group = irq as usize / 64;
        let shift = irq as usize - group * 64;
        let off_addr = OFFSET_EXT_IOIEN + group * 8;
        let old_irq_mask = self.read_reg64(off_addr);
        // let old_irq_mask = unsafe { read_volatile(addr) };
        let irq_mask = 1 << shift | old_irq_mask;
        error!("group = {}, shift = {}, irq_mask = {:#x}", group, shift, irq_mask);
        self.write_reg64(off_addr, irq_mask);
        // unsafe { write_volatile(addr, irq_mask) };
    }
    pub fn disable(&self, irq: u32) {
        let group = irq as usize / 64;
        let shift = irq as usize - group * 64;
        // let addr = (self.base_addr + OFFSET_EXT_IOIEN + group * 8) as *mut u64;
        let off_addr = OFFSET_EXT_IOIEN + group * 8;
        // let old_irq_mask = unsafe { read_volatile(addr) };
        let old_irq_mask = self.read_reg64(off_addr);
        let irq_mask = !(1 << shift) & old_irq_mask;
        // unsafe { write_volatile(addr, irq_mask) };
        self.write_reg64(off_addr, irq_mask);
    }
    pub fn is_enabled(&self, irq: u32) -> bool {
        let group = irq as usize / 64;
        let shift = irq as usize - group * 64;
        // let addr = (self.base_addr + OFFSET_EXT_IOIEN + group * 8) as *const u64;
        let off_addr = OFFSET_EXT_IOIEN + group * 8;
        // let enabled_mask = unsafe { read_volatile(addr) };
        let enabled_mask = self.read_reg64(off_addr);
        (enabled_mask & (1 << shift)) != 0
    }
    pub fn is_pending(&self, irq: u32) -> bool {
        let group = irq as usize / 64;
        let shift = irq as usize - group * 64;
        // let addr = (self.base_addr + OFFSET_EXT_IOISR + group * 8) as *const u64;
        let off_addr = OFFSET_EXT_IOISR + group * 8;
        // let pending_mask = unsafe { read_volatile(addr) };
        let pending_mask = self.read_reg64(off_addr);
        (pending_mask & (1 << shift)) != 0
    }
    // pub fn get_core_pending_mask
    pub fn get_core_pendings(&self, core_id: u8) -> Option<[u64; 4]> {
        if core_id > 3 {
           return None;
        }
        // let core_off = self.base_addr + OFFSET_CORE_EXT_IOISR[core_id as usize];
        let core_off = OFFSET_CORE_EXT_IOISR[core_id as usize];
        let mut core_pendings = [0; 4];
        // core_pendings[0] = unsafe { read_volatile((core_off + 0) as *const u64) };
        // core_pendings[1] = unsafe { read_volatile((core_off + 8) as *const u64) };
        // core_pendings[2] = unsafe { read_volatile((core_off + 16) as *const u64) };
        // core_pendings[3] = unsafe { read_volatile((core_off + 24) as *const u64) };
        core_pendings[0] = self.read_reg64(core_off + 0);
        core_pendings[1] = self.read_reg64(core_off + 8);
        core_pendings[2] = self.read_reg64(core_off + 16);
        core_pendings[3] = self.read_reg64(core_off + 24);
        Some(core_pendings)
    }
    pub fn clear_core_pending(&self, irq: u32, core_id: u8) {
        if core_id > 3 || irq > 255 {
            return;
        }
        // let core_base = self.base_addr + OFFSET_CORE_EXT_IOISR[core_id as usize];
        let group = irq as usize / 64;
        let shift = irq as usize - group * 64;
        // let addr = (core_base + group * 8) as *mut u64;
        let core_off = OFFSET_CORE_EXT_IOISR[core_id as usize] + group * 8;
        let bit = 1u64 << shift;
        // let old_irq_mask = unsafe { read_volatile(addr) };
        // let irq_mask = !(1 << shift) & old_irq_mask;
        // unsafe { write_volatile(addr, bit) };
        self.write_reg64(core_off, bit);    
    }
    pub fn set_cfg(&self, irq: u32, config: ConfigVector) {
        let entry_offset = irq;
        // let cfg_addr = (self.base_addr + OFFSET_EXT_IOIMAP_CORE + entry_offset as usize) as *mut u8;
        let cfg_off = OFFSET_EXT_IOIMAP_CORE + entry_offset as usize;
        // let node = config
        // unsafe { write_volatile(cfg_addr, config.bits()) };
        self.write_reg8(cfg_off, config.bits());
    }
    pub fn get_cfg(&self, irq: u32) -> ConfigVector {
        let entry_offset = irq;
        // let cfg_addr = (self.base_addr + OFFSET_EXT_IOIMAP_CORE + entry_offset as usize) as *const u8;
        let cfg_off = OFFSET_EXT_IOIMAP_CORE + entry_offset as usize;
        // let bits = unsafe { read_volatile(cfg_addr) };
        let bits = self.read_reg8(cfg_off);
        ConfigVector::from_bits_truncate(bits)
    }
    pub fn set_node(&self, node: u8, vector: NodeVector) {
        // let node_addr = (self.base_addr + OFFSET_EXT_NODE_TYPE[node as usize]) as *mut u16;
        let node_off = OFFSET_EXT_NODE_TYPE[node as usize];
        // unsafe { write_volatile(node_addr, vector.bits()) };
        self.write_reg16(node_off, vector.bits());
    }
    pub fn get_node(&self, node: u8) -> NodeVector {
        // let node_addr = (self.base_addr + OFFSET_EXT_NODE_TYPE[node as usize]) as *const u16;
        let node_off = OFFSET_EXT_NODE_TYPE[node as usize];
        // let bits = unsafe { read_volatile(node_addr) };
        let bits = self.read_reg16(node_off);
        NodeVector::from_bits_truncate(bits)
    }
    pub fn set_map(&self, map: u8, vector: IOIMapVector) {
        // let imap_addr = (self.base_addr + OFFSET_EXT_IOIMAP[map as usize]) as *mut u8;
        let imap_addr = OFFSET_EXT_IOIMAP[map as usize];
        // unsafe { write_volatile(imap_addr, vector.bits()) };
        self.write_reg8(imap_addr, vector.bits());
    }

    pub fn debug_send(&self, irq: u32) {
        const OFFSET_IOI_SEND: usize = 0x1140;
        if irq > 7 {
            return;
        }
        let vector: u8 = 1 << irq;
        // let addr = (self.base_addr + OFFSET_IOI_SEND) as *mut u8;
        // unsafe { write_volatile(addr, vector) };
        self.write_reg8(OFFSET_IOI_SEND, vector);
    }

    pub fn route(&self, irq: u32, core_id: u8, pin_id: u8) {
        // if core_id > 3 || pin_id > 3 {
        //     return;
        // }
        let map = 0;
        let node = 0;
        let Some(map_vector) = IOIMapVector::new(pin_id) else { return; };
        self.set_map(map, map_vector);
        let Some(node_vector) = NodeVector::new(map) else { return; };
        self.set_node(node, node_vector);
        let Some(cfg_vector) = ConfigVector::new(core_id, node) else { return; };
        self.set_cfg(irq, cfg_vector);
        error!("finish route");
    }

    pub fn device_enable(&self) {
        // let addr = (self.base_addr + OFFSET_MISC_FUNC_REG) as *mut u64;
        // let mut misc_func = unsafe { read_volatile(addr) };
        // misc_func |= 1 << SHIFT_EXTIOI_EN;
        // unsafe { write_volatile(addr, misc_func) };
        let misc  = iocsr_read_d(OFFSET_MISC_FUNC_REG);
        iocsr_write_d(OFFSET_MISC_FUNC_REG, misc | (1 << SHIFT_EXTIOI_EN));
    }

}

impl IrqController for ExtIOIntController {
    // 坏实现
    fn enable_irq(&self, hart_id: usize, irq_no: usize) {
        // todo!()
        error!("[ExtIOIntController] enable_irq; out_pin: {}, irq: {}", hart_id, irq_no);
        let in_pin_id = 0;
        let out_pin_id = 0;
        self.enable(in_pin_id as u32);
        self.route(in_pin_id as u32, hart_id as u8, out_pin_id);

        self.next_icu.enable_irq(in_pin_id, irq_no);
    }

    fn disable_irq(&self, hart_id: usize, irq_no: usize) {
        // todo!()
        error!("[ExtIOIntController] disable_irq; out_pin: {}, irq: {}", hart_id, irq_no);
        let in_pin_id = 0;
        self.next_icu.disable_irq(in_pin_id, irq_no);
    }

    // one irq only
    fn claim_irq(&self, hart_id: usize) -> Option<usize> {
        // todo!()
        let pendings_group = self.get_core_pendings(hart_id as u8)
            .expect("[ExtIOIntController::claim_irq] Bad hart id");
        for i in 0..4 {
            let pendings = pendings_group[i];
            if pendings == 0 { continue;}
            let pending = pendings.trailing_zeros() as usize + i * 64;
            if pending == 0 {
                return Some(self.next_icu.claim_irq(hart_id)?);
            }
            return Some(pending);
        }
        None
    }

    fn finish_irq(&self, hart_id: usize, irq_no: usize) {
        // todo!()
        let in_pin_id = 0;
        self.next_icu.finish_irq(in_pin_id, irq_no);
        self.clear_core_pending(irq_no as u32, hart_id as u8);
    }
}