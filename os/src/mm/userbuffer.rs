use core::ops::{Index, IndexMut};
use alloc::vec::Vec;
/// u8数组切片，使内核可以访问用户空间
pub struct UserBuffer {
    ///U8 vec
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    ///Create a `UserBuffer` by parameter
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    ///`UserBuffer`中所有切片总长度
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
    /// 将userbuffer填满0
    pub fn clear(&mut self) -> usize {
        for buffer in self.buffers.iter_mut() {
            buffer.fill(0); // 使用 fill 方法快速填充 0
        }
        self.len()
    }
    // 将一个buffer的数据写入UserBuffer，返回写入长度
    pub fn write(&mut self, buff: &[u8]) -> usize {
        let len = self.len().min(buff.len());
        let mut current = 0;
        for sub_buff in self.buffers.iter_mut() {
            if current >= len {
                break;
            }
            let copy_len = sub_buff.len().min(len - current);
            let (src, _) = buff.split_at(current + copy_len);
            let dst = &mut sub_buff[..copy_len];
            dst.copy_from_slice(&src[current..]);
            current += copy_len;
        }
        current
    }

    /// 从指定offset写入数据
    pub fn write_at(&mut self, offset: usize, buff: &[u8]) -> isize {
        let len = buff.len();
        if offset + len > self.len() {
            return -1; // 返回错误码
        }
    
        let mut head = 0; // offset of slice in UBuffer
        let mut current = 0; // current offset of buff
    
        for sub_buff in self.buffers.iter_mut() {
            let sblen = sub_buff.len();
            if head + sblen <= offset {
                head += sblen;
                continue;
            }
    
            let start = if head < offset { offset - head } else { 0 };
            let end = (start + len - current).min(sblen);
    
            if start >= sblen {
                head += sblen;
                continue;
            }
    
            sub_buff[start..end].copy_from_slice(&buff[current..current + (end - start)]);
            current += end - start;
    
            if current == len {
                return len as isize;
            }
    
            head += sblen;
        }
    
        0
    }
}

impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}
/// Iterator of `UserBuffer`
pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}


// 实现 Index trait，允许不可变索引访问
impl Index<usize> for UserBuffer {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffers[index]
    }
}

// 实现 IndexMut trait，允许可变索引访问
impl IndexMut<usize> for UserBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffers[index]
    }
}