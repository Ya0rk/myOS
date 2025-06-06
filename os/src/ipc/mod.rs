use core::u32;

use alloc::collections::btree_set::BTreeSet;
use hashbrown::HashMap;
use spin::RwLock;

use crate::sync::SpinNoIrqLock;

pub mod shm;
// pub mod sem;
// pub mod msg;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct IPCPerm {
    pub key: IPCKey,
    pub uid: u32,
    pub gid: u32,
    pub cuid: u32,
    pub cgid: u32,
    pub mode: IPCPermMode,
    pub seq: u32,
}

impl IPCPerm {

    pub fn new(key: IPCKey, mode: IPCPermMode) -> Self {
        Self {
            key,
            uid: 0,
            gid: 0,
            cuid: 0,
            cgid: 0,
            mode,
            seq: 0,
        }
    }

    pub fn check_perm(&self, mode: IPCPermMode) -> bool {
        // self.mode.contains(mode)
        true
    }
    
}


bitflags! {
    #[derive(Debug,Clone,Copy)]
    pub struct IPCPermMode: u32 {
        const S_IRUSR = 0o400;
        const S_IWUSR = 0o200;
        /// should not be used
        const S_IXUSR = 0o100;
        const S_IRGRP = 0o040;
        const S_IWGRP = 0o020;
        /// should not be used
        const S_IXGRP = 0o010;
        const S_IROTH = 0o004;
        const S_IWOTH = 0o002;
        /// should not be used
        const S_IXOTH = 0o001;

    }

}
impl IPCPermMode {
    
    pub fn from_str(s: &str) -> IPCPermMode {
        let bytes = s.as_bytes();
        let mut mode = IPCPermMode::empty();
        mode.set(Self::S_IRUSR, bytes[0] == b'r');
        mode.set(Self::S_IWUSR, bytes[1] == b'w');
        mode.set(Self::S_IRGRP, bytes[3] == b'r');
        mode.set(Self::S_IWGRP, bytes[4] == b'w');
        mode.set(Self::S_IROTH, bytes[6] == b'r');
        mode.set(Self::S_IWOTH, bytes[7] == b'w');
        mode
    }
}



#[repr(C)]
#[derive(Clone, Debug)]
pub struct IPCKey(pub i32);
pub struct IPCKeyAllocator {
    current: i32,
    recycled: BTreeSet<i32>
}

impl IPCKeyAllocator {
    pub fn new() -> Self {
        IPCKeyAllocator {
            current: 0,
            recycled: BTreeSet::new()
        }
    }
    pub fn alloc(&mut self) -> IPCKey {
        if let Some(key) = self.recycled.pop_first() {
            IPCKey(key)
        } else {
            self.current += 1;
            IPCKey(self.current)
        }
    }

    pub fn dealloc(&mut self, key: i32) {
        self.recycled.insert(key);
    }
}


lazy_static!{
    pub static ref IPC_KEY_ALLOCATOR: SpinNoIrqLock<IPCKeyAllocator> = SpinNoIrqLock::new(IPCKeyAllocator::new());

}


impl IPCKey {
    pub fn new_alloc() -> IPCKey {
        IPC_KEY_ALLOCATOR.lock().alloc()
    }

    pub fn from_user(user_key: i32) -> IPCKey {
        const IPC_PRIVATE: i32 = 0;
        if (user_key == IPC_PRIVATE) {
            Self::new_alloc()
        } else {
            IPCKey(user_key)
        }
    }
    
}

impl Drop for IPCKey {
    fn drop(&mut self) {
        IPC_KEY_ALLOCATOR.lock().dealloc(self.0);
    }
}