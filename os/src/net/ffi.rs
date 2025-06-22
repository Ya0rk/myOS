use crate::{fs::OpenFlags, hal::config::KB};

pub const AF_UNIX: u16 = 1;
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 10;
pub const META_SIZE: usize = 1 * KB;
pub const BUFF_SIZE: usize = 512 * KB; // 用于设置接受和发送缓冲区大小
pub const PORT_RANGE: usize = 65535 - 32768;
pub const MAX_BUFFER_SIZE: u32 = 128 * KB as u32; // 只用于当下
pub const TCP_MSS_DEFAULT: u32 = 32 * KB as u32;
pub const TCP_MSS: u32 = match TCP_MSS_DEFAULT > MAX_BUFFER_SIZE {
    true => MAX_BUFFER_SIZE,
    false => TCP_MSS_DEFAULT,
};
pub const Congestion: &str = "reno"; // TCP 拥塞控制算法名称
pub const MAX_HOST_NAME: usize = 64;// 本机domin name长度最大值，不包含0结尾
pub static mut HOST_NAME: [u8; 65] = [0; 65];
// the NIS domain name of the host system
pub static mut NIS_DOMAIN_NAME: [u8; 65] = [0; 65];
pub const MAX_NIS_LEN: usize = 64;


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
        const SOCK_RAW    = 3;
    }

    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct Protocol: u32 {
        /// TCP 协议
        const IPPROTO_TCP = 6;
        /// UDP 协议
        const IPPROTO_UDP = 17;
        /// ICMP 协议
        const IPPROTO_ICMP = 1;
        /// IP 协议族
        const IPPROTO_IP   = 0;
    }
}
