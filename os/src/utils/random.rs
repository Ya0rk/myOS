#![allow(unused)]

use spin::Mutex;

use crate::mm::UserBuffer;

const LCG_MULTIPLIER: u64 = 6364136223846793005;
const LCG_INCREMENT: u64 = 1;
const LCG_MODULUS: u64 = 1 << 63;

pub struct LcgRng {
    state: u64,
}

impl LcgRng {
    // 使用当前时间作为种子初始化 RNG
    const fn new() -> Self {
        let seed = 42;

        LcgRng { state: seed }
    }

    // 生成下一个随机数
    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
        (self.state >> 32) as u32
    }

    /// 用随机字节填充缓冲区
    pub fn fill_buf(&mut self, mut buf: UserBuffer) -> usize {
        let mut offset = 0;

        while offset < buf.len() {
            let rand = self.next();
            let rand_bytes = rand.to_le_bytes();
            let chunk_size = (buf.len() - offset).min(4);

            buf.write_at(offset, &rand_bytes[..chunk_size]);
            offset += chunk_size;
        }
        buf.len()
    }
}

pub static RNG: Mutex<LcgRng> = Mutex::new(LcgRng::new());