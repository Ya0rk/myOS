pub const AF_UNIX: u16 = 1;
pub const AF_INET: u16 = 2;
pub const AF_INET6: u16 = 10;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct SocketType: u32 {
        /// 关闭套接字时，是否关闭文件描述符
        const SOCK_CLOEXEC  = 02000000;
        /// 关闭套接字时，是否设置为非阻塞
        const SOCK_NONBLOCK = 04000;
        /// 数据流套接字，用于Tcp
        const SOCK_STREAM = 1;
        /// 数据报套接字，用于Udp
        const SOCK_DGRAM  = 2;
    }
}