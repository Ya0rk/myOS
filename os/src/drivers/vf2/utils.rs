use crate::{board::CLOCK_FREQ, hal::arch::get_time};


pub fn sleep_ms(ms: usize) {
    let start = get_time();
    while get_time() - start < ms * CLOCK_FREQ / 1000 {
        core::hint::spin_loop();
    }
}


pub fn sleep_ms_until(ms: usize, mut f: impl FnMut() -> bool) {
    let start = get_time();
    while get_time() - start < ms * CLOCK_FREQ / 1000 {
        if f() {
            return;
        }
        core::hint::spin_loop();
    }
}