use crate::hal::arch::{console_getchar, console_putchar};
use lazy_static::lazy_static;

pub struct Uart;

impl Uart {
    pub fn new() -> Self {
        Self
    }

    pub fn getchar(&self) -> u8 {
        console_getchar() as u8
    }

    pub fn putchar(&self, c: u8) {
        console_putchar(c as usize);
    }
}

lazy_static! {
    pub static ref UART_DEVICE: Uart = Uart::new();
}
