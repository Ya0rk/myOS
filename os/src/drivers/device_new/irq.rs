use alloc::sync::Arc;

pub struct Plic;

pub trait HandleHardIrq : Send + Sync {
    fn handle_irq(&self);
}

#[derive(Clone)]
pub struct HardIrqHandler {
    handler: Option<Arc<dyn HandleHardIrq>>,
}

impl Default for HardIrqHandler {
    fn default() -> Self {
        Self { handler: None }
    }
}

impl HardIrqHandler {
    pub fn new() -> Self {
        Self { handler: None }
    }
    pub fn register(&mut self, handler: Arc<dyn HandleHardIrq>) {
        self.handler = Some(handler);
    }
    pub fn unregister(&mut self) {
        self.handler = None;
    }
    pub fn handle_irq(&self) {
        if let Some(handler) = &self.handler {
            handler.handle_irq();
        }
    }
}

