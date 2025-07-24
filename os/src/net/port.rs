use super::addr::Sock;
use crate::{
    sync::SpinNoIrqLock,
    utils::{Errno, SysResult, RNG},
};
use alloc::{collections::{btree_map::BTreeMap, vec_deque::VecDeque}, vec::Vec};
use alloc::vec;
use bitvec_rs::BitVec;
use hashbrown::{HashMap, HashSet};
use log::info;
use smoltcp::iface::{SocketHandle, SocketSet};
use smoltcp::wire::IpEndpoint;

lazy_static! {
    pub static ref PORT_MANAGER: SpinNoIrqLock<PortManager> =
        SpinNoIrqLock::new(PortManager::new());
    pub static ref PORT_FD_MANAMER: SpinNoIrqLock<PortFdMap> = 
        SpinNoIrqLock::new(PortFdMap::new());
    pub static ref SOCKET_SET: SpinNoIrqLock<SocketSet<'static>> =
        SpinNoIrqLock::new(SocketSet::new(vec![]));
}

pub const PORT_START: u16 = 49152;
pub const PORT_END: u16 = 65535;
pub const PORT_RANGE: u32 = PORT_END as u32 - PORT_START as u32 + 1;

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
            start: PORT_START,
            end: PORT_END,
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
        // assert!(
        //     port >= self.start && port <= self.end,
        //     "port {} is out of range",
        //     port
        // );
        if port >= PORT_START {
            self.recycled.push_back(port);
        }
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

#[derive(Debug, Clone)]
pub struct PortFdMap(HashMap<usize, BTreeMap<u16, Vec<usize>>>);  // pid -> port -> [fd]

impl PortFdMap {
    /// 创建一个新的空 PortFdMap
    pub fn new() -> Self {
        Self(HashMap::new())  // 修正：原来是 BTreeMap::new()，现在改为 HashMap::new()
    }

    /// 插入一个 port 和对应的 fd 到指定 PID 的映射
    /// 如果 port 已存在，则将 fd 添加到现有列表中
    pub fn insert(&mut self, pid: usize, port: u16, fd: usize) {
        // 获取或创建该 PID 的 BTreeMap
        let port_fd_map = self.0.entry(pid).or_insert_with(BTreeMap::new);
        
        // 检查该 port 是否已存在该 fd（避免重复插入）
        if let Some(fds) = port_fd_map.get_mut(&port) {
            if !fds.contains(&fd) {  // 如果 fd 不存在才插入
                fds.push(fd);
            }
        } else {
            // 如果 port 不存在，直接插入新的 fd 列表
            port_fd_map.insert(port, vec![fd]);
        }
    }

    /// 删除指定 PID 的 port 及其所有关联的 fds
    /// 返回被删除的 fds（如果有）
    pub fn remove(&mut self, pid: usize, port: u16) -> Option<Vec<usize>> {
        if let Some(port_fd_map) = self.0.get_mut(&pid) {
            port_fd_map.remove(&port)
        } else {
            None
        }
    }

    /// 检查指定 PID 的 port 是否存在
    pub fn contains(&self, pid: usize, port: u16) -> bool {
        if let Some(port_fd_map) = self.0.get(&pid) {
            port_fd_map.contains_key(&port)
        } else {
            false
        }
    }

    /// 获取指定 PID 的 port 对应的 fds（如果存在）
    pub fn get(&self, pid: usize, port: u16) -> Option<&Vec<usize>> {
        if let Some(port_fd_map) = self.0.get(&pid) {
            port_fd_map.get(&port)
        } else {
            None
        }
    }

    /// 检查指定的 PID、port 和 fd 组合是否存在
    pub fn contains_fd(&self, pid: usize, port: u16, fd: usize) -> bool {
        if let Some(port_fd_map) = self.0.get(&pid) {
            if let Some(fds) = port_fd_map.get(&port) {
                fds.contains(&fd)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 删除指定的 PID、port 和 fd 组合
    /// 如果删除后 port 的 fds 为空，则删除整个 port
    /// 返回是否成功删除
    pub fn remove_fd(&mut self, pid: usize, port: u16, fd: usize) -> bool {
        if let Some(port_fd_map) = self.0.get_mut(&pid) {
            if let Some(fds) = port_fd_map.get_mut(&port) {
                if let Some(pos) = fds.iter().position(|&f| f == fd) {
                    fds.remove(pos);
                    // 如果 fds 为空，则删除整个 port
                    if fds.is_empty() {
                        port_fd_map.remove(&port);
                    }
                    return true;
                }
            }
        }
        false
    }

    /// 获取所有管理的 PID（可选）
    pub fn pids(&self) -> impl Iterator<Item = &usize> {
        self.0.keys()
    }

    /// 获取指定 PID 的所有 port 映射（可选）
    pub fn ports(&self, pid: usize) -> Option<&BTreeMap<u16, Vec<usize>>> {
        self.0.get(&pid)
    }

    /// 移除指定 PID 及其所有关联的 port -> [fd] 映射
    /// 返回被移除的 port -> [fd] 映射（如果存在）
    pub fn remove_pid(&mut self, pid: usize) -> Option<BTreeMap<u16, Vec<usize>>> {
        self.0.remove(&pid)
    }

    pub fn remove_all_fds_by_pid_and_fd(&mut self, pid: usize, fd: usize) {
        if let Some(port_fd_map) = self.0.get_mut(&pid) {
            // 收集所有需要检查的port，避免在迭代过程中修改BTreeMap
            let ports: Vec<u16> = port_fd_map.keys().cloned().collect();
            
            for port in ports {
                if let Some(fds) = port_fd_map.get_mut(&port) {
                    if let Some(pos) = fds.iter().position(|&f| f == fd) {
                        fds.remove(pos);
                        // 如果fd列表为空，则删除该port映射
                        if fds.is_empty() {
                            port_fd_map.remove(&port);
                        }
                    }
                }
            }
        }
    }

    pub fn insert_newfd_by_oldfd(&mut self, pid: usize, oldfd: usize, newfd: usize) {
        if let Some(port_fd_map) = self.0.get_mut(&pid) {
            // 遍历所有 port 查找包含 oldfd 的那个
            for fds in port_fd_map.values_mut() {
                if fds.contains(&oldfd) {
                    // 检查 newfd 是否已存在
                    if !fds.contains(&newfd) {
                        // 在 fds 列表末尾添加 newfd
                        fds.push(newfd);
                    }
                    return;  // 找到并处理后立即返回
                }
            }
        }
        // 如果没找到 oldfd 或 pid 不存在，则静默失败(不执行任何操作)
    }

}