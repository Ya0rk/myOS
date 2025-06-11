//! 在这个文件中发挥作用的只有uart_put和uart_get两个函数。
//! 其他的都是没有用的

pub fn uart_put(c: usize) {
    let mut ptr = crate::hal::config::UART_ADDR as *mut u8;
    loop {
        unsafe {
            let c = ptr.add(5).read_volatile();
            if c & (1 << 5) != 0 {
                break;
            }
        }
    }
    ptr = crate::hal::config::UART_ADDR as *mut u8;
    unsafe {
        ptr.add(0).write_volatile(c as u8);
    }
}

pub fn uart_get() -> usize {
    let ptr = crate::hal::config::UART_ADDR as *mut u8;
    loop {
        let c = unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        };
        if let Some(ch) = c {
            return ch as usize;
        }
    }
}

use core::fmt::Error;
use core::fmt::Write;

pub struct Uart {
    base_address: usize,
}

impl Uart {
    fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    fn put(&mut self, c: u8) {
        let mut ptr = self.base_address as *mut u8;
        loop {
            unsafe {
                let c = ptr.add(5).read_volatile();
                if c & (1 << 5) != 0 {
                    break;
                }
            }
        }
        ptr = self.base_address as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    fn get(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}
