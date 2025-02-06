mod rv64;

use rv64::sbi;

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