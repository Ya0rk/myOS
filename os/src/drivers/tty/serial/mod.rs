use crate::{drivers::{tty::termios, uart}, hal::{arch::interrupt::IrqHandler, config::KERNEL_ADDR_OFFSET}, sync::{get_waker, new_shared}};
use core::task::Waker;

use alloc::{collections::vec_deque::VecDeque, sync::Arc, boxed::Box};
use async_trait::async_trait;

use crate::{drivers::tty::tty_core::TtyDriver, sync::{suspend_now, Shared}, utils::container::ring_buffer::CharBuffer};

pub mod ns16550a;

lazy_static! {
    pub static ref UART_DRIVER: Arc<ns16550a::Uart16550Driver> = Arc::new(ns16550a::Uart16550Driver::new(
        KERNEL_ADDR_OFFSET + 0x1000_0000,
        0x16e3600,
        115200,
        4,
        2,
        true
    ));

    pub static ref SERIAL_DRIVER: Arc<SerialDriver> = Arc::new(SerialDriver::new(UART_DRIVER.clone()));
}

pub trait UartDriver : Send + Sync + 'static {
    fn getc(&self) -> u8;
    fn putc(&self, c: u8);
    fn poll_in(&self) -> bool;
    fn poll_out(&self) -> bool;
}


pub struct SerialDriver {
    pub uart: Arc<dyn UartDriver>,
    
    pub icbuffer: Shared<CharBuffer>,
    pub ocbuffer: Shared<CharBuffer>,

    /// TODO:还没做
    pub read_queue: Shared<VecDeque<Waker>>,
    pub write_queue: Shared<VecDeque<Waker>>,
}

#[async_trait]
impl TtyDriver for SerialDriver {
    async fn read(&self, buf: &mut [u8]) -> usize {
        while !self.poll_in().await {
            suspend_now().await
        }
        let mut len = self.icbuffer.lock().read(buf);
        // self.with_mut_inner(|inner| {
        //     len = inner.read_buf.read(buf);
        // });
        // let uart = self.uart();
        while self.uart.poll_in() && len < buf.len() {
            let c = self.uart.getc();
            buf[len] = c;
            len += 1;
        }
        len
    }
    async fn readc(&self) -> u8 {
        while !self.poll_in().await {
            suspend_now().await
        }
        let c = self.icbuffer.lock().pop().unwrap_or_else( || {
            self.uart.getc()
        });
        c
    }
    async fn write(&self, buf: &[u8]) -> usize {
        // TODO: use irq and buffer
        for &c in buf {
            self.uart.putc(c);
        }
        buf.len()
    }
    // poll if input is available
    async fn poll_in(&self) -> bool {
        // if 
        if !self.icbuffer.lock().is_empty() || self.uart.poll_in() {
            true
        }
        else {
            let waker = get_waker().await;
            self.read_queue.lock().push_back(waker);
            false
        }
    }
    // poll if output is available
    async fn poll_out(&self) -> bool {
        true
    }

    async fn stop(&self) {
        unimplemented!()
    }

    async fn start(&self) {
        unimplemented!()
    }

    async fn validate_termios(&self, termios: &termios::Termios) -> bool {
        unimplemented!()
    }
}


impl SerialDriver {
    pub fn new(uart: Arc<dyn UartDriver>) -> Self {
        Self {
            uart,
            icbuffer: new_shared(CharBuffer::new(4096)),
            ocbuffer: new_shared(CharBuffer::new(4096)),
            read_queue: new_shared(VecDeque::new()),
            write_queue: new_shared(VecDeque::new()),
        }
    }
}

impl IrqHandler for SerialDriver {
    fn handle_irq(&self) {
        while self.uart.poll_in() {
            let c = self.uart.getc();
            self.icbuffer.lock().push(c);
        }
        if let Some(waker) = self.read_queue.lock().pop_front() {
            waker.wake();
        }
    }
}