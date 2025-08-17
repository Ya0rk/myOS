use crate::utils::SysResult;
use alloc::{string::String, sync::Arc, vec::Vec};
use flat_device_tree::{node::FdtNode, standard_nodes::Compatible, Fdt};


// unused
// TODO：设计之初是作为fdt node、transport的统一抽象，但transport模型有点复杂，搁置
pub struct PhysDevice {}

pub trait PhysDriver {
    // fn probe(fdt: &Fdt) -> Option<Arc<Self>>;
    // fn path(&self) -> &str;
    fn irq_number(&self) -> Option<usize>;
    // TODO
}

pub trait PhysDriverProbe<'b, 'a> : PhysDriver {
    // TODO: 这里更好的做法是传入PhysDevice作为参数，在一次设备树遍历中分别对节点和驱动进行匹配，搁置
    fn probe(fdt: &'b Fdt<'a>) -> Option<Arc<Self>>;
}



// pub struct DeviceManager{
//     // device_list: Vec<Device>,
// }