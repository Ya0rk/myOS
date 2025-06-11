#![allow(unused)]

// use core::arch::asm;
// use riscv::{addr::BitField, register::sstatus::FS};

use loongarch64::register::prmd;

/// Floating-point extension state
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FS {
    Off = 0,
    Initial = 1,
    Clean = 2,
    Dirty = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct Sstatus {
    bits: usize,
}

/// Supervisor Previous Privilege Mode
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SPP {
    Supervisor = 1,
    User = 0,
}

/// Prmd in LA
impl Sstatus {
    /// Returns the contents of the register as raw bits
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }

    /// User Interrupt Enable
    #[inline]
    pub fn uie(&self) -> bool {
        unimplemented!()
    }

    /// Supervisor Interrupt Enable
    #[inline]
    pub fn sie(&self) -> bool {
        unimplemented!()
    }

    /// User Previous Interrupt Enable
    #[inline]
    pub fn upie(&self) -> bool {
        unimplemented!()
    }

    /// Supervisor Previous Interrupt Enable
    #[inline]
    pub fn spie(&self) -> bool {
        unimplemented!()
    }

    /// Supervisor Previous Privilege Mode
    #[inline]
    pub fn spp(&self) -> SPP {
        unimplemented!()
    }

    /// The status of the floating-point unit
    /// TODO: implement
    #[inline]
    pub fn fs(&self) -> FS {
        unimplemented!()
    }

    /// The status of additional user-mode extensions
    /// and associated state
    #[inline]
    pub fn xs(&self) -> FS {
        unimplemented!()
    }

    /// Permit Supervisor User Memory access
    #[inline]
    pub fn sum(&self) -> bool {
        unimplemented!()
    }

    /// Make eXecutable Readable
    #[inline]
    pub fn mxr(&self) -> bool {
        unimplemented!()
    }

    /// Whether either the FS field or XS field
    /// signals the presence of some dirty state
    #[inline]
    pub fn sd(&self) -> bool {
        unimplemented!()
    }

    #[inline]
    pub fn set_spie(&mut self, val: bool) {
        unimplemented!()
    }

    #[inline]
    pub fn set_pie(&mut self, val: bool) {
        // unimplemented!()
        if val {
            self.bits |= 1 << 2;
        } else {
            self.bits &= !(1 << 2);
        }
    }

    #[inline]
    pub fn set_spp(&mut self, val: SPP) {
        // unimplemented!()
        if val == SPP::Supervisor {
            self.bits &= !0b11;
        } else {
            self.bits |= 0b11;
        }
    }
}

pub fn read() -> Sstatus {
    // unimplemented!()
    Sstatus {
        bits: unsafe {
            let bits: usize;
            core::arch::asm!("csrrd {},0x1", out(reg) bits);
            bits
        },
    }
    // Sstatus {bits:prmd::read()}
}
