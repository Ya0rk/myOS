use log::info;

use crate::{
    net::{addr::SockAddr, Socket, SocketType}, 
    task::{current_task, sock_map_fd}, 
    utils::{Errno, SysResult}
};

use super::ffi::ShutHow;


/// domain：即协议域，又称为协议族（family）, 协议族决定了socket的地址类型
/// 常用的协议族有，AF_INET、AF_INET6、AF_LOCAL（或称AF_UNIX，Unix域socket）、AF_ROUTE等
pub fn sys_socket(domain: usize, type_: usize, protocol: usize) -> SysResult<usize> {
    let type_ = SocketType::from_bits(type_ as u32).ok_or(Errno::EINVAL)?;
    let protocol = protocol as u8;
    let cloexec_enable = type_.contains(SocketType::SOCK_CLOEXEC);

    // 根据协议族、套口类型、传输层协议创建套口
    let socket = <dyn Socket>::new(domain as u16, type_)
        .map_err(|_| Errno::EAFNOSUPPORT)?;

    // 将socket和一个fd绑定
    let fd = sock_map_fd(socket.get(), cloexec_enable)
        .map_err(|_| Errno::EMFILE)?;

    Ok(fd)
}

/// bind a name to a socket
/// On success, zero is returned.  On error, -1 is returned, and errno
/// is set to indicate the error.
pub fn sys_bind(sockfd: usize, addr: usize, addrlen: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket();
    let sockaddr = SockAddr::from(addr, addrlen);
    match sockaddr {
        SockAddr::Unspec => {
            info!("[sys_bind] invalid sockaddr");
            return Err(Errno::EINVAL);
        }
        _ => {}
    }
    socket.bind(&sockaddr)?;

    Ok(0)
}

/// 监听来自客户端的tcp socket的连接请求
/// The sockfd argument is a file descriptor that refers to a socket
/// of type SOCK_STREAM or SOCK_SEQPACKET.
/// The backlog argument defines the maximum length to which the queue of pending connections for sockfd may grow.
pub fn sys_listen(sockfd: usize, backlog: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket();
    socket.listen(backlog)?;

    Ok(0)
}

pub fn sys_shutdown(sockfd: usize, how: u8) -> SysResult<usize> {
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket();
    let how = ShutHow::from_bits(how).ok_or(Errno::EINVAL)?;
    socket.shutdown(how)?;

    Ok(0)
}

/// tcp客户端连接到tcp服务器
/// The connect() system call connects the socket referred to by the
/// file descriptor sockfd to the address specified by addr.  The
/// addrlen argument specifies the size of addr.  The format of the
/// address in addr is determined by the address space of the socket
/// sockfd; see socket(2) for further details.
/// 
/// If the socket sockfd is of type SOCK_DGRAM, then addr is the
/// address to which datagrams are sent by default, and the only
/// address from which datagrams are received.  If the socket is of
/// type SOCK_STREAM or SOCK_SEQPACKET, this call attempts to make a
/// connection to the socket that is bound to the address specified by
/// addr.
pub async fn sys_connect(sockfd: usize, addr: usize, addrlen: usize) -> SysResult<usize> {
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket();
    let sockaddr = SockAddr::from(addr, addrlen);
    match sockaddr {
        SockAddr::Unspec => {
            info!("[sys_connect] invalid sockaddr");
            return Err(Errno::EINVAL);
        }
        _ => {}
    }
    socket.connect(&sockaddr).await?;

    Ok(0)
}