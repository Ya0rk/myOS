#![allow(unused)]

pub fn set_timer(time: usize) {
    unimplemented!()
}

pub fn hart_start(hartid: usize, start_addr: usize) -> bool {
    unimplemented!()
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    unimplemented!()
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    unimplemented!()
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    unimplemented!()
}