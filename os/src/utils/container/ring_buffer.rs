use core::intrinsics::unlikely;

use alloc::{vec, vec::Vec};

use crate::utils::SysResult;

use num_traits::PrimInt;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RingBufferState {
    Empty,
    Normal,
    Full
}



pub struct RingBuffer<T>
    where T: PrimInt + Copy
{
    data: Vec<T>,
    head: usize,
    tail: usize,
    state: RingBufferState
}

impl<T> RingBuffer<T>
    where T: PrimInt + Copy
{
    pub fn new(len: usize) -> Self {
        Self {
            data: vec![T::zero(); len],
            head: 0,
            tail: 0,
            state: RingBufferState::Empty
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state == RingBufferState::Empty
    }

    pub fn is_full(&self) -> bool {
        self.state == RingBufferState::Full
    }

    pub fn num_items(&self) -> usize {
        if self.head < self.tail {
            self.tail - self.head
        } else {
            self.data.len() - self.head + self.tail
        }
    }
    /// must ensure a valid offset
    fn offset_head(&mut self, offset: usize) {
        self.head = self.head + offset;
        if self.head >= self.data.len() {
            self.head -= self.data.len();
        }
        if self.head == self.tail {
            self.state = RingBufferState::Empty;
        }
        else {
            self.state = RingBufferState::Normal;
        }
    }
    /// must ensure a valid offset
    fn offset_tail(&mut self, offset: usize) {
        self.tail = self.tail + offset;
        if self.tail >= self.data.len() {
            self.tail -= self.data.len();
        }
        if self.head == self.tail {
            self.state = RingBufferState::Full;
        }
        else {
            self.state = RingBufferState::Normal;
        }
    }

    pub fn push(&mut self, item: T) -> Option<usize> {
        match self.state {
            RingBufferState::Empty => {
                // head == tail
                self.data[self.tail] = item;
                self.offset_tail(1);
                Some(1)
            }
            RingBufferState::Normal => {
                self.data[self.tail] = item;
                self.offset_tail(1);
                Some(1)
            }
            RingBufferState::Full => {
                None
            }
            
        }
    }



    pub fn pop(&mut self) -> Option<T> {
        match self.state {
            RingBufferState::Empty => {
                None
            }
            RingBufferState::Normal => {
                let item = self.data[self.head].clone();
                self.offset_head(1);
                Some(item)
            }
            RingBufferState::Full => {
                let item = self.data[self.head].clone();
                self.offset_head(1);
                Some(item)
            }
        }
    }

    pub fn read(&mut self, buf: &mut [T]) -> usize {
        if buf.is_empty() {
            return 0;
        }
        match self.state {
            RingBufferState::Empty => 0,
            _ => {
                let data_len = self.num_items();
                let max_read_len = data_len.min(buf.len());
                let max_data_len = self.data.len();
                if self.head < self.tail {
                    buf[..max_read_len].copy_from_slice(&self.data[self.head..self.head + max_read_len]);
                    self.offset_head(max_read_len);
                }
                else {
                    let read_len_1 = max_read_len.min(max_data_len - self.head);
                    buf[..read_len_1].copy_from_slice(&self.data[self.head..self.head + read_len_1]);
                    self.offset_head(read_len_1);
                    let read_len_2 = max_read_len - read_len_1;
                    if self.head == 0 {
                        buf[read_len_1..read_len_1 + read_len_2].copy_from_slice(&self.data[0..read_len_2]);
                        self.offset_head(read_len_2);
                    }
                }
                max_read_len
            }
        }
    }


    pub fn write(&mut self, buf: &[T]) -> usize {
        // if buf.is_empty() {
        //     return 0;
        // }
        // match self.state {
        //     RingBufferState::Full => 0,
        //     _ => {
        //         let data_len = self.num_items();
        //         let max_write_len = data_len.min(buf.len());
        //         let max_data_len = self.data.len();
        //         if self.head < self.tail {
        //             self.data[self.tail..self.tail + max_write_len].copy_from_slice(&buf[..max_write_len]);
        //             self.offset_tail(max_write_len);
        //         }
        //         else {
        //             let write_len_1 = max_write_len.min(max_data_len - self.tail);
        //             self.data[self.tail..self.tail + write_len_1].copy_from_slice(&buf[..write_len_1]);
        //             self.offset_tail(write_len_1);
        //             let write_len_2 = max_write_len - write_len_1;
        //             if self.tail == 0 {
        //                 self.data[0..write_len_2].copy_from_slice(&buf[write_len_1..write_len_1 + write_len_2]);
        //                 self.offset_tail(write_len_2);
        //             }
        //         }
        //         max_write_len
        //     }
        // }
        todo!("not implemented")
    }

}

pub type LineBuffer = RingBuffer<u8>;
pub type CharBuffer = RingBuffer<u8>;

