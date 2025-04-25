use log::{debug, error, info, warn};
use lwext4_rust::KernelDevOp;
use alloc::sync::Arc;

use crate::{drivers::BlockDriver, utils::logger};

use super::{BlockDeviceImpl, DevResult};

const BLOCK_SIZE: usize = 512;

/// A disk device with a cursor.
pub struct Disk {
    block_id: usize,
    offset: usize,
    dev: Arc<dyn BlockDriver>,
}

impl Disk {
    /// Create a new disk.
    pub fn new(dev: Arc<dyn BlockDriver>) -> Self {
        log::info!("create a new disk");
        assert_eq!(BLOCK_SIZE, dev.block_size());
        Self {
            block_id: 0,
            offset: 0,
            dev,
        }
    }

    /// Get the size of the disk.
    pub fn size(&self) -> usize {
        self.dev.num_blocks() * BLOCK_SIZE
    }

    /// Get the position of the cursor.
    pub fn position(&self) -> usize {
        self.block_id * BLOCK_SIZE + self.offset
    }

    /// Set the position of the cursor.
    pub fn set_position(&mut self, pos: usize) {
        self.block_id = pos / BLOCK_SIZE;
        self.offset = pos as usize % BLOCK_SIZE;
    }

    /// Read within one block, returns the number of bytes read.
    pub fn read_one(&mut self, buf: &mut [u8]) -> DevResult<usize> {
        // info!("block id: {}", self.block_id);
        let read_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev
                .read_block(self.block_id, &mut buf[0..BLOCK_SIZE])?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);
            if start > BLOCK_SIZE {
                info!("block size: {} start {}", BLOCK_SIZE, start);
            }

            self.dev.read_block(self.block_id, &mut data)?;
            buf[..count].copy_from_slice(&data[start..start + count]);

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(read_size)
    }

    /// Write within one block, returns the number of bytes written.
    pub fn write_one(&mut self, buf: &[u8]) -> DevResult<usize> {
        let write_size = if self.offset == 0 && buf.len() >= BLOCK_SIZE {
            // whole block
            self.dev.write_block(self.block_id, &buf[0..BLOCK_SIZE])?;
            self.block_id += 1;
            BLOCK_SIZE
        } else {
            // partial block
            let mut data = [0u8; BLOCK_SIZE];
            let start = self.offset;
            let count = buf.len().min(BLOCK_SIZE - self.offset);

            self.dev.read_block(self.block_id, &mut data)?;
            data[start..start + count].copy_from_slice(&buf[..count]);
            self.dev.write_block(self.block_id, &data)?;

            self.offset += count;
            if self.offset >= BLOCK_SIZE {
                self.block_id += 1;
                self.offset -= BLOCK_SIZE;
            }
            count
        };
        Ok(write_size)
    }

    /// Read a single block starting from the specified offset.
    pub fn read_offset(&mut self, offset: usize) -> [u8; BLOCK_SIZE] {
        let block_id = offset / BLOCK_SIZE;
        let mut block_data = [0u8; BLOCK_SIZE];
        self.dev.read_block(block_id, &mut block_data).unwrap();
        block_data
    }

    /// Write single block starting from the specified offset.
    pub fn write_offset(&mut self, offset: usize, buf: &[u8]) -> DevResult<usize> {
        assert!(
            buf.len() == BLOCK_SIZE,
            "Buffer length must be equal to BLOCK_SIZE"
        );
        assert!(offset % BLOCK_SIZE == 0);
        let block_id = offset / BLOCK_SIZE;
        self.dev.write_block(block_id, buf).unwrap();
        Ok(buf.len())
    }
}

impl KernelDevOp for Disk {
    type DevType = Disk;
    /// 读取硬盘数据到指定buf
    fn read(dev: &mut Disk, mut buf: &mut [u8]) -> Result<usize, i32> {
        debug!("READ block device buf={}", buf.len());
        let mut read_len = 0;
        while !buf.is_empty() {
            match dev.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    read_len += n;
                }
                Err(_e) => return Err(-1),
            }
        }
        debug!("READ rt len={}", read_len);
        Ok(read_len)
    }
    /// 写入数据到硬盘
    fn write(dev: &mut Self::DevType, mut buf: &[u8]) -> Result<usize, i32> {
        debug!("WRITE block device buf={}", buf.len());
        let mut write_len = 0;
        while !buf.is_empty() {
            match dev.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_e) => return Err(-1),
            }
        }
        debug!("WRITE rt len={}", write_len);
        Ok(write_len)
    }
    fn flush(_dev: &mut Self::DevType) -> Result<usize, i32> {
        Ok(0)
    }
    fn seek(dev: &mut Disk, off: i64, whence: i32) -> Result<i64, i32> {
        let size = dev.size();
        debug!(
            "SEEK block device size:{}, pos:{}, offset={}, whence={}",
            size,
            &dev.position(),
            off,
            whence
        );
        let new_pos = match whence as u32 {
            lwext4_rust::bindings::SEEK_SET => Some(off),
            lwext4_rust::bindings::SEEK_CUR => dev
                .position()
                .checked_add_signed(off as isize)
                .map(|v| v as i64),
            lwext4_rust::bindings::SEEK_END => size.checked_add_signed(off as isize).map(|v| v as i64),
            _ => {
                error!("invalid seek() whence: {}", whence);
                Some(off)
            }
        }
        .ok_or(-1)?;

        if new_pos as usize > size {
            warn!("Seek beyond the end of the block device");
        }
        dev.set_position(new_pos as usize);
        Ok(new_pos)
    }
}