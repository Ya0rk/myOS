use alloc::collections::btree_map::BTreeMap;

use crate::mm::VirtAddr;

pub type ShmidTable = BTreeMap<VirtAddr, i32>;
