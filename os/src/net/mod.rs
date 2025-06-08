pub mod addr;
pub mod dev;
pub mod ffi;
mod net_async;
mod port;
pub mod socket;
pub mod tcp;
mod udp;
mod unix;

pub use dev::*;
pub use ffi::*;
use net_async::*;
pub use port::*;
pub use socket::*;
pub use tcp::*;

use crate::fs::FileTrait;
use alloc::sync::Arc;
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
