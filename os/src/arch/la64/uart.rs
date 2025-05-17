use core::fmt::Error;
use core::fmt::Write;

pub struct Uart {
    base_address: usize,
} 

impl Write for Uart {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Uart {base_address}
    }

    pub fn put(&self, c: u8) {
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

    pub fn get(&mut self) -> Option<u8> {
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

use lazy_static::*;
use spin::Mutex;

const UART_ADDR: usize = 0x0_1FE0_01E0 | 0x9000_0000_0000_0000;

/// LA的UART设备，地址写死为uart_addr
lazy_static! {
    pub static ref UART: SpinNoIrqLock<Uart> = SpinNoIrqLock::new(Uart::new(UART_ADDR));
}