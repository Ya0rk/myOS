use crate::{config::KB, fs::OpenFlags};

pub const AF_UNIX: u16 = 1;
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 10;
pub const META_SIZE: usize = 1 * KB;
pub const BUFF_SIZE: usize = 512 * KB;
pub const PORT_RANGE: usize = 65535 - 32768;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct SocketType: u32 {
        /// 关闭套接字时，是否关闭文件描述符
        const SOCK_CLOEXEC  = 1 << 19;
        /// 关闭套接字时，是否设置为非阻塞
        const SOCK_NONBLOCK = 1 << 11;
        /// 数据流套接字，用于Tcp
        const SOCK_STREAM = 1;
        /// 数据报套接字，用于Udp
        const SOCK_DGRAM  = 2;
    }
}