use core::arch::asm;




#[inline(always)]
pub fn ioscr_read64(addr: usize) -> u64 {
    unsafe {
        let mut ret: u64;
        asm!(
            "iocsrrd.d {}, {}", out(reg) ret, in(reg) addr
        );
        ret
    }
}

#[inline(always)]
pub fn ioscr_write64(addr: usize, val: u64) {
    unsafe {
        asm!(
            "iocsrwr.d {}, {}", in(reg) val, in(reg) addr
        );
    }
}

#[inline(always)]
pub fn ioscr_read32(addr: usize) -> u32 {
    unsafe {
        let mut ret: u32;
        asm!(
            "iocsrrd.w {}, {}", out(reg) ret, in(reg) addr
        );
        ret
    }
}

#[inline(always)]
pub fn ioscr_write32(addr: usize, val: u32) {
    unsafe {
        asm!(
            "iocsrwr.w {}, {}", in(reg) val, in(reg) addr
        );
    }
}

#[inline(always)]
pub fn ioscr_read16(addr: usize) -> u16 {
    unsafe {
        let mut ret: u16;
        asm!(
            "iocsrrd.h {}, {}", out(reg) ret, in(reg) addr
        );
        ret
    }
}

#[inline(always)]
pub fn ioscr_write16(addr: usize, val: u16) {
    unsafe {
        asm!(
            "iocsrwr.h {}, {}", in(reg) val, in(reg) addr
        );
    }
}

#[inline(always)]
pub fn ioscr_read8(addr: usize) -> u8 {
    unsafe {
        let mut ret: u8;
        asm!(
            "iocsrrd.b {}, {}", out(reg) ret, in(reg) addr
        );
        ret
    }
}

#[inline(always)]
pub fn ioscr_write8(addr: usize, val: u8) {
    unsafe {
        asm!(
            "iocsrwr.b {}, {}", in(reg) val, in(reg) addr
        );
    }
}