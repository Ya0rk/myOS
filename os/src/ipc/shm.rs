use alloc::{sync::Weak, sync::Arc, vec::Vec};
use hashbrown::HashMap;
use crate::{mm::page::Page, sync::SpinNoIrqLock};
use spin::RwLock;
use virtio_drivers::device::console::Size;

use crate::sync::timer::{get_time_ns, get_time_s};

use super::{IPCKey, IPCPerm};





#[repr(C)]
#[derive(Clone, Debug)]
pub struct ShmidDs {
    pub shm_perm: IPCPerm,
    pub shm_segsz: usize,
    pub shm_atime: usize,
    pub shm_dtime: usize,
    pub shm_ctime: usize,
    pub shm_cpid: usize,
    pub shm_lpid: usize,
    pub shm_nattch: usize,
    // pub shm_cbytes: usize,
    // pub shm_pid: usize,
}

impl ShmidDs {
    pub fn new(shm_perm: IPCPerm, shm_segsz: usize, shm_cpid: usize) -> Self {
        Self {
            shm_perm,
            shm_segsz,
            shm_atime: 0,
            shm_dtime: 0,
            shm_ctime: get_time_s(),
            shm_cpid,
            shm_lpid: 0,
            shm_nattch: 0,
        }
    }
    
}


bitflags! {
    /// other bits is not generally used in operating systems
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ShmGetFlags: i32 {
        const IPC_CREAT = 0o1000;
        const IPC_EXCL = 0o2000;
        const IPC_NOWAIT = 0o4000;
    }
}

bitflags! {
    
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ShmAtFlags: i32 {
        /// Attach the segment for read-only access.If this flag is not specified,
        /// the segment is attached for read and write access, and the process
        /// must have read and write permission for the segment.
        const SHM_RDONLY = 0o10000;
        /// round attach address to SHMLBA boundary
        const SHM_RND = 0o20000;
        /// take-over region on attach
        const SHM_REMAP = 0o40000;
        /// Allow the contents of the segment to be executed.
        const SHM_EXEC = 0o100000;
    }
}

/// todo
#[derive(Clone, Debug)]
pub struct ShmObject {
    pub shmid_ds: ShmidDs,
    pub pages: Vec<Weak<Page>>,
}

impl ShmObject {
    pub fn new(ipc_perm: IPCPerm, size: usize, pid: usize) -> Self {
        Self {
            shmid_ds: ShmidDs::new(ipc_perm, size, pid),
            pages: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.shmid_ds.shm_segsz
    }
    pub fn ipc_key(&self) -> i32 {
        self.shmid_ds.shm_perm.key.0
    }
    
    pub fn attach_one(&mut self, lpid: usize) {
        self.shmid_ds.shm_nattch += 1;
        self.shmid_ds.shm_lpid = lpid;
        self.shmid_ds.shm_atime = get_time_s();
    }

    pub fn detach_one(&mut self, lpid: usize) -> bool {
        self.shmid_ds.shm_nattch -= 1;
        self.shmid_ds.shm_lpid = lpid;
        self.shmid_ds.shm_dtime = get_time_s();
        
        /// return true if last detach , otherwise false
        self.shmid_ds.shm_nattch == 0
    }
}

type SharedMemoryManager = RwLock<HashMap<i32, ShmObject>>;

lazy_static! {
    pub static ref SHARED_MEMORY_MANAGER: SharedMemoryManager = RwLock::new(HashMap::new());
}