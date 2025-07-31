use crate::utils::SysResult;



pub trait PhysDevice {}

pub trait PhysDriver {
    fn probe(&self, device: &dyn PhysDevice) -> SysResult<()> ;
    // TODO
}

// pub struct DeviceManager{
//     // device_list: Vec<Device>,
// }