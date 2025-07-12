use super::{addr::Sock, PORT_RANGE};
use crate::{
    sync::SpinNoIrqLock,
    utils::{Errno, SysResult, RNG},
};
use alloc::collections::vec_deque::VecDeque;
use alloc::vec;
use bitvec_rs::BitVec;
use hashbrown::HashSet;
use log::info;
use smoltcp::iface::{SocketHandle, SocketSet};
use smoltcp::wire::IpEndpoint;

lazy_static! {
    pub static ref PORT_MANAGER: SpinNoIrqLock<PortManager> =
        SpinNoIrqLock::new(PortManager::new());
    pub static ref SOCKET_SET: SpinNoIrqLock<SocketSet<'static>> =
        SpinNoIrqLock::new(SocketSet::new(vec![]));
}

pub struct PortManager {
    pub start: u16,
    pub end: u16,
    pub recycled: VecDeque<u16>,
    pub tcp_used_ports: BitVec,
    pub udp_used_ports: BitVec,
}

impl PortManager {
    pub fn new() -> Self {
        PortManager {
            start: 49152,
            end: 65535,
            recycled: VecDeque::new(),
            tcp_used_ports: BitVec::from_elem(65536, false),
            udp_used_ports: BitVec::from_elem(65536, false),
        }
    }
    fn alloc(&mut self, domain: Sock) -> SysResult<u16> {
        if let Some(port) = self.recycled.pop_front() {
            info!("[port alloc] recycled port: {}", port);
            self.mark_used(domain, port);
            return Ok(port);
        }

        let chance = self.end - self.start - self.recycled.len() as u16;
        for _ in 0..chance {
            let random_port = self.start + (RNG.lock().next() % PORT_RANGE as u32) as u16;
            if self.try_mark_used(&domain, random_port) {
                return Ok(random_port);
            }
        }

        for port in self.start..=self.end {
            if self.try_mark_used(&domain, port) {
                return Ok(port);
            }
        }
        Err(Errno::EADDRINUSE)
    }
    pub fn dealloc(&mut self, domain: Sock, port: u16) {
        info!("[port dealloc] port: {}", port);
        assert!(
            port >= self.start && port <= self.end,
            "port {} is out of range",
            port
        );
        self.recycled.push_back(port);
        match domain {
            Sock::Tcp => {
                self.tcp_used_ports.set(port as usize, false);
            }
            Sock::Udp => {
                self.udp_used_ports.set(port as usize, false);
            }
            _ => {}
        }
    }
    fn mark_used(&mut self, domain: Sock, port: u16) {
        match domain {
            Sock::Tcp => {
                self.tcp_used_ports.set(port as usize, true);
            }
            Sock::Udp => {
                self.udp_used_ports.set(port as usize, true);
            }
            _ => {}
        }
    }
    fn try_mark_used(&mut self, domain: &Sock, port: u16) -> bool {
        match domain {
            Sock::Tcp => {
                if !self.tcp_used_ports[port as usize] {
                    self.tcp_used_ports.set(port as usize, true);
                    return true;
                }
            }
            Sock::Udp => {
                if !self.udp_used_ports[port as usize] {
                    self.udp_used_ports.set(port as usize, true);
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

/// 检查传入的endpoint的port，分配port，并返回
pub fn do_port_aloc(endpoint: &mut IpEndpoint, port_type: Sock) -> SysResult<u16> {
    let p: u16;
    if endpoint.port == 0 {
        p = PORT_MANAGER.lock().alloc(port_type)?;
        endpoint.port = p;
    } else {
        let mut port_manager = PORT_MANAGER.lock();
        p = endpoint.port;
        // 标记已使用
        if !port_manager.try_mark_used(&port_type, p) {
            drop(port_manager);
            info!("[do_port] port = {} is in use", p);
            return Err(Errno::EADDRINUSE);
        }
        drop(port_manager);
    }
    return Ok(p);
}
