use crate::sync::SpinNoIrqLock;
use alloc::{collections::btree_map::BTreeMap, format, string::String, vec::Vec};

lazy_static! {
    pub static ref DOMAINNAME: SpinNoIrqLock<Domainname> =
        SpinNoIrqLock::new(Domainname::new("(domain)"));
}

lazy_static! {
    pub static ref PIPE_MAX_SIZE: SpinNoIrqLock<Domainname> =
        SpinNoIrqLock::new(Domainname::new("1024"));
}
// pub struct Domainname([u8; 256]);
pub struct Domainname {
    data: [u8; 64],
    size: usize,
}

impl Domainname {
    fn new(content: &str) -> Self {
        let mut res = Self {
            data: [0; 64],
            size: 0,
        };
        let data = content.as_bytes();
        let size = data.len();
        res.data[0..size].copy_from_slice(data);
        res.size = size;
        res
    }
    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.data[0..buf.len()].copy_from_slice(buf);
        self.size = buf.len();
        // *self.0 = String::from_utf8(buf.to_vec()).ok().unwrap();
        buf.len()
    }
    pub fn read(&self) -> &[u8] {
        &self.data
    }
    pub fn len(&self) -> usize {
        self.size
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
