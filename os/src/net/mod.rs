mod udp;
mod unix;
mod manager;
mod net_async;
pub mod tcp;
pub mod ffi;
pub mod socket;
pub mod addr;
pub mod dev;

use net_async::*;
pub use tcp::*;
pub use socket::*;
pub use ffi::*;
pub use manager::*;
pub use dev::*;

use alloc::sync::Arc;
use crate::fs::FileTrait;
pub enum SockClass {
    Tcp(Arc<tcp::TcpSocket>),
    Udp(Arc<udp::UdpSocket>),
    Unix(Arc<unix::UnixSocket>),
    Unspec(), 
}

impl SockClass {
    pub fn get(&self) -> Arc<dyn FileTrait> {
        match self {
            SockClass::Tcp(tcp) => tcp.clone(),
            SockClass::Udp(udp) => udp.clone(),
            SockClass::Unix(unix) => unix.clone(),
            SockClass::Unspec() => unreachable!(),
        }
    }
}