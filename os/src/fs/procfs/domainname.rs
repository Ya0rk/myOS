use crate::sync::SpinNoIrqLock;
use alloc::{collections::btree_map::BTreeMap, format, string::String, vec::Vec};
lazy_static! {
    pub static ref DOMAINNAME: SpinNoIrqLock<Domainname> = SpinNoIrqLock::new(Domainname::new());
}

lazy_static! {
    pub static ref PIPE_MAX_SIZE: SpinNoIrqLock<Domainname> = SpinNoIrqLock::new(Domainname::new());
}
pub struct Domainname(SpinNoIrqLock<String>);

impl Domainname {
    fn new() -> Self {
        Self(SpinNoIrqLock::new(String::from("(none)")))
    }
    pub fn write(&self, buf: &[u8]) -> usize {
        *self.0.lock() = String::from_utf8(buf.to_vec()).ok().unwrap();
        buf.len()
    }
}

//
// pub const SupervisorExternal: u32 = 8;
// pub const SupervisorTimer: u32 = 5;
//
// lazy_static!{
//     pub static ref IRQTABLE: SpinNoIrqLock<IrqTable> = SpinNoIrqLock::new(IrqTable::new());
// }
//
// pub struct IrqTable(BTreeMap<u32, u32>);
//
// impl IrqTable {
//     fn new() -> Self {
//         Self(BTreeMap::new())
//     }
//
//     pub fn inc(&mut self, irq_num: u32) {
//         self.0.entry(irq_num).and_modify(|e| *e += 1).or_insert(1);
//     }
//
//     pub fn cnt(&self, irq_num: u32) -> u32 {
//         *self.0.get(&irq_num).unwrap_or(&0)
//     }
//
//     pub fn tostring(&self) -> String {
//         return format!(
//             r"{timer_irq}:     {timer_cnt}
// {external_irq}:      {external_cnt}
// ",
//             timer_irq = SupervisorTimer,
//             timer_cnt = self.cnt(SupervisorTimer),
//             external_irq = SupervisorExternal,
//             external_cnt = self.cnt(SupervisorExternal)
//         );
//     }
// }
