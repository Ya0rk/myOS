use alloc::{string::String, sync::Arc, vec};
use log::info;
use smoltcp::{iface::SocketHandle, socket::udp::{self, PacketMetadata}, storage::PacketBuffer, wire::{IpAddress, IpEndpoint}};
use crate::{fs::{FileMeta, OpenFlags, RenameFlags}, sync::SpinNoIrqLock, syscall::ShutHow, utils::{Errno, SysResult}};
use super::{addr::{IpType, Ipv4, Ipv6, Sock, SockAddr}, alloc_port, Port, SockMeta, Socket, AF_INET, BUFF_SIZE, META_SIZE, NET_DEV, PORT_MANAGER, SOCKET_SET};
use alloc::boxed::Box;
use crate::fs::FileTrait;
use crate::mm::UserBuffer;
use async_trait::async_trait;

/// UDP 是一种无连接的报文套接字
pub struct UdpSocket {
    pub handle: SocketHandle,
    pub flags: OpenFlags,
    pub sockmeta: SpinNoIrqLock<SockMeta>,
}

impl UdpSocket {
    pub fn new(iptype: IpType) -> Self {
        let socket = Self::new_socket();
        let handle = SOCKET_SET.lock().add(socket);
        let sockmeta = SpinNoIrqLock::new(
            SockMeta::new(
                Sock::Udp,
                iptype,
                BUFF_SIZE,
                BUFF_SIZE,
            )
        );
        // TODO(YJJ): maybe bug
        // NET_DEV.lock().poll();
        Self {
            handle,
            flags: OpenFlags::O_RDWR,
            sockmeta,
        }
    }

    fn new_socket() -> udp::Socket<'static> {
        let recv_buf = PacketBuffer::new(
            vec![PacketMetadata::EMPTY; META_SIZE],
            vec![0; BUFF_SIZE], 
        );
        let send_buf = PacketBuffer::new(
            vec![PacketMetadata::EMPTY; META_SIZE],
            vec![0; BUFF_SIZE], 
        );
        udp::Socket::new(
            recv_buf,
            send_buf,
        )
    }

    /// 这里不只是要检查地址，还要本地是否绑定local end，没有的话就bind一个
    fn check_addr(&self, mut endpoint: IpEndpoint) -> SysResult<()> {
        let mut sockmeta = self.sockmeta.lock();
        match sockmeta.domain {
            Sock::Udp => {
                if endpoint.addr.is_unspecified() {
                    match sockmeta.iptype {
                        IpType::Ipv4 => endpoint.addr = IpAddress::v4(127, 0, 0, 1),
                        IpType::Ipv6 => endpoint.addr = IpAddress::v6(0, 0, 0, 0, 0, 0, 0, 1),
                    }
                }
            }
            _ => return Err(Errno::EAFNOSUPPORT),
        }
        
        if sockmeta.local_end.is_none() {
            match sockmeta.iptype {
                IpType::Ipv4 => {
                    let addr = SockAddr::Inet4(Ipv4 {
                        family: AF_INET,
                        port: 0,
                        addr: [127, 0, 0, 1],
                        zero: [0u8; 8],
                    });
                    self.bind(&addr);
                }
                IpType::Ipv6 => todo!(),
            };
        }

        Ok(())
    }
}

#[async_trait]
impl Socket for UdpSocket {
    fn bind(&self, addr: &SockAddr) -> SysResult<()> {
        // NET_DEV.lock().poll();
        let mut endpoint = IpEndpoint::try_from(addr.clone()).map_err(|_| Errno::EINVAL)?;
        let mut sockmeta = self.sockmeta.lock();
        if sockmeta.port.is_some() {
            return Err(Errno::EADDRINUSE);
        }
        let mut p: Port;
        let mut port_manager = PORT_MANAGER.lock();

        if endpoint.port == 0 {
            p = alloc_port(Sock::Udp)?;
            endpoint.port = p.port;
        } else {
            if port_manager.udp_used_ports.get(endpoint.port as usize).unwrap() {
                return Err(Errno::EADDRINUSE);
            }
            p = Port::new(Sock::Udp, endpoint.port);
            port_manager.udp_used_ports.set(endpoint.port as usize, true);
        }
        drop(port_manager);
        sockmeta.port = Some(p);

        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        match socket.bind(endpoint) {
            Ok(_) => {
                info!("[bind] bind to port: {}", endpoint.port);
                sockmeta.local_end = Some(endpoint);
            }
            Err(_) => {
                return Err(Errno::EADDRINUSE);
            }
        }

        Ok(())
    }
    fn listen(&self, _backlog: usize) -> SysResult<()> {
        Err(Errno::EOPNOTSUPP)
    }
    async fn accept(&self, flags: OpenFlags) -> SysResult<(IpEndpoint, usize)> {
        unimplemented!()
    }
    async fn connect(&self, addr: &SockAddr) -> SysResult<()> {
        /// 与TCP不同，UDP的connect函数不会引发三次握手，而是将目标IP和端口记录下来
        let remote_endpoint = IpEndpoint::try_from(addr.clone()).map_err(|_| Errno::EINVAL)?;
        self.check_addr(remote_endpoint);
        self.sockmeta.lock().remote_end = Some(remote_endpoint);
        Ok(())
    }
    fn set_recv_buf_size(&self, size: usize) -> SysResult<()> {
        self.sockmeta.lock().recv_buf_size = size;
        Ok(())
    }
    fn set_send_buf_size(&self, size: usize) -> SysResult<()> {
        self.sockmeta.lock().send_buf_size = size;
        Ok(())
    }
    fn shutdown(&self, how: ShutHow) -> SysResult<()> {
        let mut binding = SOCKET_SET.lock();
        let socket = binding.get_mut::<udp::Socket>(self.handle);
        socket.close();
        NET_DEV.lock().poll();
        Ok(())
    }
}

#[async_trait]
impl FileTrait for UdpSocket {
    fn get_socket(self: Arc<Self>) -> Arc<dyn Socket> {
        self
    }
    fn get_inode(&self) -> Arc<dyn crate::fs::InodeTrait> {
        unimplemented!()
    }
    fn readable(&self) -> bool {
        unimplemented!()
    }
    fn writable(&self) -> bool {
        unimplemented!()
    }
    fn executable(&self) -> bool {
        false
    }
    async fn read(&self, _buf: UserBuffer) -> SysResult<usize> {
        unimplemented!()
    }
    async fn write(&self, _buf: UserBuffer) -> SysResult<usize> {
        unimplemented!()
    }
    fn get_name(&self) -> SysResult<String> {
        unimplemented!()
    }
    fn rename(&mut self, _new_path: String, _flags: RenameFlags) -> SysResult<usize> {
        unimplemented!()
    }
    fn fstat(&self, _stat: &mut crate::fs::Kstat) -> SysResult {
        unimplemented!()
    }
    fn is_dir(&self) -> bool {
        false
    }
    async fn get_page_at(&self, _offset: usize) -> Option<Arc<crate::mm::page::Page>> {
        unimplemented!()
    }
}