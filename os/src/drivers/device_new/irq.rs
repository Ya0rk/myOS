pub struct Plic;

pub trait ExtIrqHandler {
    fn handle_irq(&self);
}