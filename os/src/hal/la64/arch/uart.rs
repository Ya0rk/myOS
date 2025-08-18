//! 在这个文件中发挥作用的只有uart_put和uart_get两个函数。
//! 其他的都是没有用的
// #[cfg(feature = "2k1000la")]
use embedded_hal::serial::nb::{Read, Write};

// #[cfg(feature = "2k1000la")]
use crate::{boards::UART_BASE, drivers::tty::serial::ns16550a::Uart16550Driver};
use crate::drivers::tty::serial::UartDriver;
// #[cfg(feature = "2k1000la")]
// lazy_static!{
//     static ref UART: Uart16550Driver = Uart16550Driver::new(
//         UART_BASE,
//         0,
//         115200,
//         1,
//         0,
//         false,
//         None
//     );
// }

// #[cfg(feature = "board_qemu")]
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

// #[cfg(feature = "board_qemu")]
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

// #[cfg(feature = "2k1000la")]
// pub fn uart_put(c: usize) {
//     let mut retry = 0;
//     unsafe {
//         UART.putc(c as u8);
//     }
// }

// #[cfg(feature = "2k1000la")]
// pub fn uart_get() -> usize {
//     unsafe {
//         loop {
//             if UART.poll_in() {
//                 return UART.getc() as usize;
//             }
//         }
//     }
// }


// use core::fmt::Error;
// use core::fmt::Write;

// pub struct Uart {
//     base_address: usize,
// }

// impl Uart {
//     fn new(base_address: usize) -> Self {
//         Uart { base_address }
//     }

//     fn put(&mut self, c: u8) {
//         let mut ptr = self.base_address as *mut u8;
//         loop {
//             unsafe {
//                 let c = ptr.add(5).read_volatile();
//                 if c & (1 << 5) != 0 {
//                     break;
//                 }
//             }
//         }
//         ptr = self.base_address as *mut u8;
//         unsafe {
//             ptr.add(0).write_volatile(c);
//         }
//     }

//     fn get(&mut self) -> Option<u8> {
//         let ptr = self.base_address as *mut u8;
//         unsafe {
//             if ptr.add(5).read_volatile() & 1 == 0 {
//                 // The DR bit is 0, meaning no data
//                 None
//             } else {
//                 // The DR bit is 1, meaning data!
//                 Some(ptr.add(0).read_volatile())
//             }
//         }
//     }
// }
