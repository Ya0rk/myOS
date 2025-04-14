use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use bitvec_rs::BitVec;
use hashbrown::HashSet;
use log::info;
use smoltcp::iface::SocketSet;
use crate::{sync::SpinNoIrqLock, utils::{Errno, SysResult, RNG}};
use super::{addr::Sock, PORT_RANGE};

lazy_static! {
    pub static ref PORT_MANAGER: SpinNoIrqLock<PortManager> = SpinNoIrqLock::new(PortManager::new());
    pub static ref SOCKET_SET: SpinNoIrqLock<SocketSet<'static>> = SpinNoIrqLock::new(SocketSet::new(Vec::new()));
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
            start: 32768,
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
    fn dealloc(&mut self, domain: Sock, port: u16) {
        info!("[port dealloc] port: {}", port);
        assert!(port >= self.start && port <= self.end, "port {} is out of range", port);
        self.recycled.push_back(port);
        match domain {
            Sock::Tcp => {
                self.tcp_used_ports.set(port as usize, false);
            }
            Sock::Udp => {
                self.udp_used_ports.set(port  as usize, false);
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

#[derive(Clone)]
pub struct Port {
    pub port: u16,
    pub domain: Sock,
}

impl Port {
    pub fn new(domain: Sock, port: u16) -> Self {
        Port { port, domain }
    }
}

impl From<Port> for u16 {
    fn from(value: Port) -> Self {
        value.port
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        PORT_MANAGER.lock().dealloc(self.domain, self.port);
    }
}

pub fn alloc_port(domain: Sock) -> SysResult<Port> {
    let port = PORT_MANAGER.lock().alloc(domain)?;
    Ok(Port { port, domain })
}