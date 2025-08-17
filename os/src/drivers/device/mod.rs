
use alloc::sync::Arc;

use crate::drivers::{device::dev_number::MajorNumber, tty::tty_core::CharDevice};

pub mod dev_core;
pub mod dev_number;
pub mod irq;
pub mod manager;
pub mod uart;
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceType {
    Block,
    Char,
    Unknown,
    // Net,
    // Display
}



pub trait Device : Send + Sync {
    fn get_type(&self) -> DeviceType;
    fn get_major(&self) -> MajorNumber;
    fn get_minor(&self) -> usize;
    fn as_char(self: Arc<Self>) -> Option<Arc<dyn CharDevice>> {
        None
    }
    // TODO: BlockDriver -> BlockDevice
    fn as_block(self: Arc<Self>) -> Option<Arc<dyn BlockDevice>> {
        None
    }
    // fn as_abs(self: Arc<Self>) -> Option<Arc<dyn BlockDevice>> {
    //     None
    // }
}
pub type DevResult<T = ()> = Result<T, DevError>;
/// A specialized `Result` type for device operations.
// pub type DevResult<T = ()> = Result<T, DevError>;
/// The error type for device operation failures.
#[derive(Debug)]
pub enum DevError {
    /// An entity already exists.
    AlreadyExists,
    /// Try again, for non-blocking APIs.
    Again,
    /// Bad internal state.
    BadState,
    /// Invalid parameter/argument.
    InvalidParam,
    /// Input/output error.
    Io,
    /// Not enough space/cannot allocate memory (DMA).
    NoMemory,
    /// Device or resource is busy.
    ResourceBusy,
    /// This operation is unsupported or unimplemented.
    Unsupported,
}

// TODO: make it async
/// Operations that require a block storage device driver to implement.
pub trait BlockDevice: Device {
    /// The number of blocks in this storage device.
    ///
    /// The total size of the device is `num_blocks() * block_size()`.
    fn num_blocks(&self) -> usize;
    /// The size of each block in bytes.
    fn block_size(&self) -> usize;

    /// Reads blocked data from the given block.
    ///
    /// The size of the buffer may exceed the block size, in which case multiple
    /// contiguous blocks will be read.
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> DevResult;

    /// Writes blocked data to the given block.
    ///
    /// The size of the buffer may exceed the block size, in which case multiple
    /// contiguous blocks will be written.
    fn write_block(&self, block_id: usize, buf: &[u8]) -> DevResult;

    /// Flushes the device to write all pending data to the storage.
    fn flush(&self) -> DevResult;
}
