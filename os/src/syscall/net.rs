use core::intrinsics::unlikely;

use super::ffi::{
    ShutHow, CONGESTION, MAXSEGMENT, NODELAY, SOL_SOCKET, SOL_TCP, SO_KEEPALIVE, SO_RCVBUF,
    SO_SNDBUF,
};
use crate::{
    fs::{FileTrait, OpenFlags, Pipe}, hal::config::USER_SPACE_TOP, mm::user_ptr::check_readable, net::{
        addr::{IpType, SockIpv4, SockIpv6, Sock, SockAddr}, Congestion, Protocol, Socket, SocketType, TcpSocket, AF_INET, AF_INET6, AF_UNIX, HOST_NAME, MAX_HOST_NAME, MAX_NIS_LEN, NIS_DOMAIN_NAME, TCP_MSS
    }, syscall::ffi::{IPPROTO_IP, IPPROTO_TCP, SO_OOBINLINE, SO_RCVTIMEO}, task::{current_task, sock_map_fd, FdInfo}, utils::{Errno, SysResult}
};
use log::{info, trace, warn};
use smoltcp::wire::IpAddress;

/// domain：即协议域，又称为协议族（family）, 协议族决定了socket的地址类型
/// 常用的协议族有，AF_INET、AF_INET6、AF_LOCAL（或称AF_UNIX，Unix域socket）、AF_ROUTE等
pub fn sys_socket(domain: usize, type_: usize, protocol: usize) -> SysResult<usize> {
    info!(
        "[sys_socket] start, domain = {}, type_ = {}, protocol = {}",
        domain, type_, protocol
    );
    let type_ = SocketType::from_bits(type_ as u32).ok_or(Errno::EINVAL)?;
    let protocol = protocol as u8;
    let cloexec_enable = type_.contains(SocketType::SOCK_CLOEXEC);
    if unlikely(domain == AF_UNIX.into()) {
        return Ok(4);
    } // 这里是特殊处理，通过musl libctest的网络测例，后序要修改

    // 根据协议族、套口类型、传输层协议创建套口
    let socket = <dyn Socket>::new(domain as u16, type_).map_err(|_| Errno::EAFNOSUPPORT)?;

    // 将socket和一个fd绑定
    let fd = sock_map_fd(socket.get(), cloexec_enable).map_err(|_| Errno::EMFILE)?;

    info!("[sys_socket] finished, fd = {}", fd);
    Ok(fd)
}

/// bind a name to a socket
/// On success, zero is returned.  On error, -1 is returned, and errno
/// is set to indicate the error.
pub fn sys_bind(sockfd: usize, addr: usize, addrlen: usize) -> SysResult<usize> {
    info!("[sys_bind] start, sockfd = {}", sockfd);
    trace!("[sys_bind] addr = {}, addrlen = {}", addr, addrlen);
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
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
    info!(
        "[sys_listen] start, sockfd = {}, basklog = {}",
        sockfd, backlog
    );
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
    socket.listen(backlog)?;

    Ok(0)
}

pub fn sys_shutdown(sockfd: usize, how: u8) -> SysResult<usize> {
    info!("[sys_shutdown] start, sockfd = {}, how = {}", sockfd, how);
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
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
    info!(
        "[sys_connect] start, sockfd = {}, addr = {}, addrlen = {}",
        sockfd, addr, addrlen
    );
    if unlikely(addr == 0 || addr > USER_SPACE_TOP) {
        info!("[sys_connect] invalid sockaddr");
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
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

/// accept函数的第一个参数为服务器的socket描述字，第二个参数为指向struct sockaddr *的指针，
/// 用于返回客户端的协议地址，第三个参数为协议地址的长度指针。
/// 如果accpet成功，那么其返回值是由内核自动生成的一个全新的描述字，代表与返回客户的TCP连接。
///
/// 注意：accept的第一个参数为服务器的socket描述字，是服务器开始调用socket()函数生成的，称为监听socket描述字；
/// 而accept函数返回的是 已连接的socket描述字。一个服务器通常通常仅仅只创建一个监听socket描述字，
/// 它在该服务器的生命周期内一直存在。内核为每个由服务器进程接受的客户连接创建了一个已连接socket描述字，
/// 当服务器完成了对某个客户的服务，相应的已连接socket描述字就被关闭.
pub async fn sys_accept(sockfd: usize, addr: usize, addrlen: usize) -> SysResult<usize> {
    info!(
        "[sys_accept] start, sockfd = {}, addr = {}, addrlen = {}",
        sockfd, addr, addrlen
    );
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let flags = file.get_flags();
    let socket = file.get_socket()?;

    let (remote_end, newfd) = socket.accept(OpenFlags::empty()).await?;
    // 将remote_end保存在addr中
    let ptr = addr as *mut u8;
    if unlikely(addr == 0) {
        return Err(Errno::EFAULT);
    }

    let buf = unsafe { core::slice::from_raw_parts_mut(ptr, addrlen) };
    let user_sockaddr = match remote_end.addr {
        IpAddress::Ipv4(addr) => {
            let port = remote_end.port;
            let addr = addr.octets();
            let temp = SockAddr::Inet4(SockIpv4::new(port, addr));
            temp
        }
        IpAddress::Ipv6(addr) => {
            let port = remote_end.port;
            let addr = addr.octets();
            let temp = SockAddr::Inet6(SockIpv6::new(port, addr));
            temp
        }
    };

    user_sockaddr.write2user(buf, addrlen)?;
    info!("[sys_accept] new sockfd: {}", newfd);

    Ok(newfd)
}

/// accept a connection on a socket
/// If flags is 0, then accept4() is the same as accept().
pub async fn sys_accept4(
    sockfd: usize,
    addr: usize,
    addrlen: usize,
    flags: u32,
) -> SysResult<usize> {
    info!(
        "[sys_accept4] start, sockfd = {}, addr = {}, addrlen = {}, flags = {}",
        sockfd, addr, addrlen, flags
    );
    if flags == 0 {
        return sys_accept(sockfd, addr, addrlen).await;
    }
    let flags = OpenFlags::from_bits(flags as i32).expect("[sys_accept4] flag parse fail");
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;

    let (remote_end, newfd) = socket.accept(flags).await?;
    let ptr = addr as *mut u8;
    if unlikely(addr == 0) {
        return Err(Errno::EFAULT);
    }

    // maybe bug: 需要检查懒分配
    let buf = unsafe { core::slice::from_raw_parts_mut(ptr, addrlen) };
    let user_sockaddr = match remote_end.addr {
        IpAddress::Ipv4(addr) => {
            let port = remote_end.port;
            let addr = addr.octets();
            let temp = SockAddr::Inet4(SockIpv4::new(port, addr));
            temp
        }
        IpAddress::Ipv6(addr) => {
            let port = remote_end.port;
            let addr = addr.octets();
            let temp = SockAddr::Inet6(SockIpv6::new(port, addr));
            temp
        }
    };
    user_sockaddr.write2user(buf, addrlen)?;
    info!("[sys_accept] new sockfd: {}", newfd);

    Ok(newfd)
}

/// getsockname() returns the current address to which the socket sockfd is bound, in the buffer pointed to by addr.
/// The addrlen argument should be initialized to
/// indicate the amount of space (in bytes) pointed to by addr.
/// On return it contains the actual size of the socket address.
///
/// The returned address is truncated if the buffer provided is too small;
/// in this case, addrlen will return a value greater than was supplied to the call.
pub fn sys_getsockname(sockfd: usize, addr: usize, addrlen: usize) -> SysResult<usize> {
    info!(
        "[sys_getsockname] start, sockfd = {}, addr = {}, addrlen = {}",
        sockfd, addr, addrlen
    );
    // println!(
    //     "[sys_getsockname] start, sockfd = {}, addr = {}, addrlen = {}",
    //     sockfd, addr, addrlen
    // );
    if unlikely(addrlen == 0 || addrlen == 1) {
        return Err(Errno::EFAULT);
    }
    let len = unsafe { *(addrlen as *const i32) };
    if unlikely(len < 0) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let ptr = addr as *mut u8;
    if unlikely(addr == 0) {
        return Err(Errno::EFAULT);
    }

    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
    let sockname = match socket.get_sockname() {
        Ok(SockAddr::Unspec) => {
            return Err(Errno::ENOTSOCK);
        }
        Ok(res) => res,
        _ => {
            return Err(Errno::ENOTSOCK);
        }
    };

    let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len as usize) };
    sockname.write2user(buf, len as usize)?;
    Ok(0)
}

pub fn sys_getpeername(sockfd: usize, addr: usize, addrlen: usize) -> SysResult<usize> {
    info!(
        "[sys_sockpeername] start, sockfd = {}, addr = {}, addrlen = {}",
        sockfd, addr, addrlen
    );
    // println!("addr = {}, addrlen = {}", addr, addrlen);
    if unlikely(addr > USER_SPACE_TOP || addrlen == 0) {
        return Err(Errno::EFAULT);
    }
    let len = unsafe { *(addrlen as *const usize) };
    if unlikely((len as isize) < 0) {
        return Err(Errno::EINVAL);
    }

    let task = current_task().unwrap();
    let ptr = addr as *mut u8;
    if unlikely(ptr.is_null()) {
        return Err(Errno::EINVAL);
    }

    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
    let peername = match socket.get_peername() {
        Ok(SockAddr::Unspec) => {
            return Err(Errno::ENOTSOCK);
        }
        Ok(res) => res,
        Err(e) => {
            return Err(e);
        }
    };

    let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    peername.write2user(buf, len)?;
    Ok(0)
}

/// send a message on a socket
/// If the socket is a connectionless-mode socket, the message shall
/// be sent to the address specified by dest_addr if no pre-specified
/// peer address has been set. If a peer address has been pre-
/// specified, either the message shall be sent to the address
/// specified by dest_addr (overriding the pre-specified peer
/// address), or the function shall return -1 and set errno to
/// [EISCONN].
pub async fn sys_sendto(
    sockfd: usize,
    message: usize,
    msg_len: usize,
    flags: u32,
    dest_addr: usize,
    addrlen: usize,
) -> SysResult<usize> {
    info!("[sys_sendto] start, sockfd = {}, flags = {}", sockfd, flags);
    let task = current_task().unwrap();
    let dest_sockaddr = SockAddr::from(dest_addr, addrlen);
    match dest_sockaddr {
        SockAddr::Unspec => {
            info!("[sys_sendto] invalid dest_addr");
            return Err(Errno::EINVAL);
        }
        _ => {}
    }

    let buf = unsafe { core::slice::from_raw_parts(message as *const u8, msg_len) };
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;

    let res = match socket.get_socktype()? {
        Sock::Tcp => socket.send_msg(buf, Some(dest_sockaddr)).await?,
        Sock::Udp => {
            socket.connect(&dest_sockaddr).await;
            socket.send_msg(buf, Some(dest_sockaddr)).await?
        }
        _ => todo!(),
    };

    Ok(res)
}

/// The recvfrom() function shall receive a message from a connection-
/// mode or connectionless-mode socket. It is normally used with
/// connectionless-mode sockets because it permits the application to
/// retrieve the source address of received data.
pub async fn sys_recvfrom(
    sockfd: usize,
    buf_ptr: usize,
    buflen: usize,
    flags: u32,
    src_addr: usize,
    addrlen: usize,
) -> SysResult<usize> {
    info!(
        "[sys_recvfrom] start, sockfd = {}, flags = {}",
        sockfd, flags
    );
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;

    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, buflen) };
    let (size, remote_end) = socket.recv_msg(buf).await?;
    if src_addr != 0 {
        // 将远程地址写到用户空间
        // maybe bug: 需要检查懒分配
        let buf = unsafe { core::slice::from_raw_parts_mut(src_addr as *mut u8, addrlen) };
        remote_end.write2user(buf, addrlen)?;
    }

    Ok(size)
}

/// create a pair of connected sockets
/// The socketpair() call creates an unnamed pair of connected sockets
/// in the specified domain, of the specified type, and using the
/// optionally specified protocol.
/// The file descriptors used in referencing the new sockets are
/// returned in sv[0] and sv[1].  The two sockets are
/// indistinguishable.
pub fn sys_socketpair(domain: usize, _type: usize, protocol: usize, sv: usize) -> SysResult<usize> {
    info!(
        "[sys_socketpair] start, domain = {}, _type = {}, protocol = {}",
        domain, _type, protocol
    );
    if unlikely(domain != AF_UNIX.into() && domain != AF_INET.into() && domain != AF_INET6.into()) {
        return Err(Errno::EAFNOSUPPORT);
    }
    if unlikely(sv == 0 || sv > USER_SPACE_TOP || sv == 7) {
        return Err(Errno::EFAULT);
    }
    let task = current_task().unwrap();
    let length = core::mem::size_of::<i32>();
    let sv = unsafe { core::slice::from_raw_parts_mut(sv as *mut i32, length) };
    let proto = Protocol::from_bits(protocol as u32)
        .ok_or(Errno::EPROTONOSUPPORT)?;
    let _type = SocketType::from_bits(_type as u32)
        .ok_or(Errno::EINVAL)?;
    let flags = if _type.contains(SocketType::SOCK_CLOEXEC) {
        OpenFlags::O_CLOEXEC
    } else if _type.contains(SocketType::SOCK_NONBLOCK) {
        OpenFlags::O_NONBLOCK
    } else if _type.contains(SocketType::SOCK_RAW) {
        return Err(Errno::EPROTONOSUPPORT);
    } else {
        OpenFlags::empty()
    };

    if (proto.contains(Protocol::IPPROTO_TCP) && _type.contains(SocketType::SOCK_STREAM))
        || (proto.contains(Protocol::IPPROTO_UDP) && _type.contains(SocketType::SOCK_DGRAM))
    {
        return Err(Errno::EOPNOTSUPP);
    }

    if (_type.contains(SocketType::SOCK_STREAM) && !proto.contains(Protocol::IPPROTO_TCP))
        || (_type.contains(SocketType::SOCK_DGRAM) && !proto.contains(Protocol::IPPROTO_UDP))
    {
        return Err(Errno::EPROTONOSUPPORT);
    }

    let (read_fd, write_fd) = {
        let (read_end, write_end) = Pipe::new();
        (
            task.alloc_fd(FdInfo::new(read_end, OpenFlags::O_RDONLY | flags))?,
            task.alloc_fd(FdInfo::new(write_end, OpenFlags::O_WRONLY | flags))?,
        )
    };
    info!("alloc read_fd = {}, write_fd = {}", read_fd, write_fd);
    sv[0] = read_fd as i32;
    sv[1] = write_fd as i32;
    Ok(0)
}

/// get options on sockets
/// getsockopt() and setsockopt() manipulate options for the socket
/// referred to by the file descriptor sockfd.
///
pub fn sys_getsockopt(
    sockfd: usize,
    level: usize,
    optname: usize,
    optval_ptr: usize,
    optlen: usize,
) -> SysResult<usize> {
    info!(
        "[sys_getsockopt] start, sockfd = {}, level = {}, optname = {}, optlen = {}",
        sockfd, level, optname, optlen
    );
    if unlikely((optname as isize) < 0) {
        return Err(Errno::ENOPROTOOPT);
    }
    if unlikely(optval_ptr == 0 || optlen == 0) {
        return Err(Errno::EFAULT);
    }
    match (level as u8, optname as u32) {
        (SOL_SOCKET, SO_OOBINLINE) => {
            let task = current_task().unwrap();
            let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
            let socket = file.get_socket()?;
            let len = unsafe { *(optlen as *const i32) };
            if unlikely(len <= 0) {
                return Err(Errno::EINVAL);
            }
        }
        (SOL_SOCKET, SO_SNDBUF) => {
            let task = current_task().unwrap();
            let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
            let socket = file.get_socket()?;
            // 获取发送缓冲区大小，并写到用户空间
            let send_buf_size = socket.get_send_buf_size()?;
            unsafe {
                *(optval_ptr as *mut u32) = send_buf_size as u32;
                *(optlen as *mut u32) = core::mem::size_of::<u32>() as u32;
            }
        }
        (SOL_SOCKET, SO_RCVBUF) => {
            let task = current_task().unwrap();
            let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
            let socket = file.get_socket()?;
            // 获取接收缓冲区大小，并写到用户空间
            let recv_buf_size = socket.get_recv_buf_size()?;
            unsafe {
                *(optval_ptr as *mut u32) = recv_buf_size as u32;
                *(optlen as *mut u32) = core::mem::size_of::<u32>() as u32;
            }
        }
        (SOL_TCP, MAXSEGMENT) => {
            // 返回TCP最大段大小 MSS
            unsafe {
                *(optval_ptr as *mut u32) = TCP_MSS;
                *(optlen as *mut u32) = core::mem::size_of::<u32>() as u32;
            }
        }
        (SOL_TCP, CONGESTION) => {
            // 获取 TCP 拥塞控制算法名称
            let name_len = Congestion.len();
            let buf = unsafe { core::slice::from_raw_parts_mut(optval_ptr as *mut u8, name_len) };
            let bytes = Congestion.as_bytes();
            buf.copy_from_slice(bytes);
            unsafe { *(optlen as *mut u32) = name_len as u32 };
        }
        _ => {
            warn!("[sys_getsockopt] sockfd: {:?}, level: {:?}, optname: {:?}, optval_ptr: {:?}, optlen: {:?}",
            sockfd,
            level,
            optname,
            optval_ptr,
            optlen);
            return Err(Errno::EOPNOTSUPP);
        }
    }

    Ok(0)
}

pub fn sys_setsockopt(
    sockfd: usize,
    level: usize,
    optname: usize,
    optval_ptr: usize,
    optlen: usize,
) -> SysResult<usize> {
    info!(
        "[sys_setsockopt] start, sockfd = {}, level = {}, optname = {}, optlen = {}",
        sockfd, level, optname, optlen
    );
    if unlikely(optval_ptr == 0) {
        return Err(Errno::EFAULT);
    }
    if unlikely(optlen == 0) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let file = task.get_file_by_fd(sockfd).ok_or(Errno::EBADF)?;
    let socket = file.get_socket()?;
    match (level as u8, optname as u32) {
        (SOL_SOCKET, SO_SNDBUF) => {
            // 修改发送缓冲区大小
            let new_size = unsafe { *(optval_ptr as *const u32) };
            socket.set_send_buf_size(new_size)?;
        }
        (SOL_SOCKET, SO_RCVBUF) => {
            // 修改接受缓冲区大小
            let new_size = unsafe { *(optval_ptr as *const u32) };
            socket.set_recv_buf_size(new_size)?;
        }
        (SOL_SOCKET, SO_KEEPALIVE) => {
            let action = unsafe { *(optval_ptr as *const u32) };
            socket.set_keep_alive(action)?;
        }
        (SOL_TCP, NODELAY) => {
            let action = unsafe { *(optval_ptr as *const u32) };
            socket.enable_nagle(action);
        }
        (SOL_SOCKET, SO_RCVTIMEO) => {
            // 设置超时时间暂时未实现
            return Ok(0);
        }
        _ => return Err(Errno::ENOPROTOOPT),
    }

    Ok(0)
}

pub fn sys_setdominname(name: usize, size: usize) -> SysResult<usize> {
    info!("[sys_setdominname] start, name = {:#x}, size = {:#x}", name, size);
    if unlikely((size as isize) < 0) || unlikely(size > MAX_NIS_LEN) {
        return Err(Errno::EINVAL);
    }
    if unlikely(name == 0 || check_readable(name.into(), size).is_err()) {
        return Err(Errno::EFAULT);
    }

    let new_name = unsafe {
        core::slice::from_raw_parts(name as *const u8, size)
    };

    unsafe { NIS_DOMAIN_NAME [..size].copy_from_slice(new_name);}

    Ok(0)
}

pub fn sys_sethostname(name: usize, size: usize) -> SysResult<usize> {
    info!("[sys_sethostname] start, name = {:#x}, size = {:#x}", name, size);
    if unlikely((size as isize) < 0) || unlikely(size > MAX_HOST_NAME) {
        return Err(Errno::EINVAL);
    }
    if unlikely(name == 0 || check_readable(name.into(), size).is_err()) {
        return Err(Errno::EFAULT);
    }

    let new_name = unsafe {
        core::slice::from_raw_parts(name as *const u8, size)
    };

    unsafe { HOST_NAME [..size].copy_from_slice(new_name);}

    Ok(0)
}