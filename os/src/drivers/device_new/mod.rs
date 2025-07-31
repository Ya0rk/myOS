use crate::drivers::device_new::dev_number::MajorNumber;

pub mod dev_core;
pub mod dev_number;
pub mod irq;
pub mod manager;
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceType {
    Block,
    Char,
    Net,
    Display
}



pub trait Device {
    fn get_type(&self) -> DeviceType;
    fn get_major(&self) -> MajorNumber;
    fn get_minor(&self) -> usize;
    // TODO
}