use crate::utils::SysResult;
use alloc::{string::String, sync::Arc, vec::Vec};
use flat_device_tree::{node::FdtNode, standard_nodes::Compatible, Fdt};


// unused
pub struct PhysDevice {}

pub trait PhysDriver {
    // fn probe(fdt: &Fdt) -> Option<Arc<Self>>;
    // fn path(&self) -> &str;
    fn irq_number(&self) -> Option<usize>;
    // TODO
}

pub trait PhysDriverProbe : PhysDriver {
    fn probe(fdt: &Fdt) -> Option<Arc<Self>>;
}



// pub struct DeviceManager{
//     // device_list: Vec<Device>,
// }