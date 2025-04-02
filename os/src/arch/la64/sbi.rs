#![allow(unused)]
use crate::arch::uart::{Uart, UART};

pub fn set_timer(time: usize) {
    unimplemented!()
}

pub fn hart_start(hartid: usize, start_addr: usize) -> bool {
    unimplemented!()
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    UART.lock().put(c as u8);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    loop {
        let c = UART.lock().get();
        if let Some(ch) = c {
            return ch as usize;
        }
    }
    
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    unimplemented!()
}