use smoltcp::{
    iface::{Config, Interface}, 
    phy::{Loopback, Medium, Tracer}, 
    time::Instant, 
    wire::{EthernetAddress, IpAddress, IpCidr}
};
use crate::sync::{once::LateInit, timer::get_time_ms, SpinNoIrqLock};

use super::SOCKET_SET;

pub static NET_DEV: LateInit<SpinNoIrqLock<NetDev>> = LateInit::new();

pub fn init_net_dev() {
    NET_DEV.init(SpinNoIrqLock::new(NetDev::new()));
}

pub enum NetDevType {
    Loopback(Loopback),
    Unspec,
}

pub struct NetDev {
    pub device: NetDevType,
    pub iface: Interface,
}

impl NetDev {
    pub fn new() -> Self {
        let mut loopback = Loopback::new(Medium::Ethernet);
        let config = Config::new(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]).into());
        let instant = Instant::from_millis(get_time_ms() as i64);
        let mut iface = Interface::new(
            config,
            &mut loopback,
            instant,
        );
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(IpAddress::v4(127, 0, 0, 1), 24))
                .unwrap();
        });
        let gateway = IpAddress::v4(127, 0, 0, 1);
        let device = NetDevType::Loopback(loopback);
        let mut res = Self { device, iface };
        res.set_gateway(gateway);
        res
    }

    fn set_gateway(&mut self, gateway: IpAddress) {
        match gateway {
            IpAddress::Ipv4(ip) => {
                self.iface.routes_mut().add_default_ipv4_route(ip).unwrap();
            }
            IpAddress::Ipv6(_) => unimplemented!(),
        }
    }

    /// 用于接受和发送数据包
    pub fn poll(&mut self) {
        let instant = Instant::from_millis(get_time_ms() as i64);
        let mut socket = SOCKET_SET.lock();
        let device = match self.device {
            NetDevType::Loopback(ref mut dev) => dev,
            NetDevType::Unspec => panic!("Device not initialized"),
        };
        self.iface.poll(instant, device, &mut socket);
    }
}