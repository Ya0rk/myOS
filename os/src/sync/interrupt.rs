use riscv::register::sstatus;

#[inline(always)]
pub unsafe fn enable_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}

#[inline(always)]
pub unsafe fn disable_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}

#[inline(always)]
pub fn interrupt_is_enabled() -> bool {
    sstatus::read().sie()
}
