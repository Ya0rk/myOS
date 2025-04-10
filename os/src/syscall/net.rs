use crate::{
    net::{addr::DomainType, Socket, SocketType}, 
    task::sock_map_fd, 
    utils::{Errno, SysResult}
};


/// domain：即协议域，又称为协议族（family）, 协议族决定了socket的地址类型
/// 常用的协议族有，AF_INET、AF_INET6、AF_LOCAL（或称AF_UNIX，Unix域socket）、AF_ROUTE等
pub fn sys_socket(domain: usize, type_: usize, protocol: usize) -> SysResult<usize> {
    let domain = match DomainType::from(domain as u16) {
        valid_domain => valid_domain,
        DomainType::Unspec => return Err(Errno::EAFNOSUPPORT),
    };
    let type_ = SocketType::from_bits(type_ as u32).ok_or(Errno::EINVAL)?;
    let protocol = protocol as u8;
    let cloexec_enable = type_.contains(SocketType::SOCK_CLOEXEC);

    // 根据协议族、套口类型、传输层协议创建套口
    let socket = <dyn Socket>::new(domain, type_)
        .map_err(|_| Errno::EAFNOSUPPORT)?;

    // 将socket和一个fd绑定
    let fd = sock_map_fd(socket.get(), cloexec_enable)
        .map_err(|_| Errno::EMFILE)?;

    Ok(fd)
}