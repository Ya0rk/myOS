use core::intrinsics::unlikely;

use log::{error, info, warn};
use lwext4_rust::bindings::EINVAL;

use crate::task::current_task;
use crate::{
    hal::config::{align_up_by_page, is_aligned_to_page, PAGE_MASK, PAGE_SIZE},
    ipc::{
        shm::{self, ShmAtFlags, ShmGetFlags, ShmObject, ShmidDs, SHARED_MEMORY_MANAGER},
        IPCKey, IPCPerm, IPCPermMode,
    },
    mm::{
        memory_space::{
            vm_area::{MapPerm, VmArea},
            MemorySpace, MmapFlags, MmapProt,
        },
        user_ptr::user_ref_mut,
        VirtAddr,
    },
    utils::{Errno, SysResult},
};

use super::ffi::ShmOp;

pub fn sys_brk(new_brk: *const u8) -> SysResult<usize> {
    info!("[sys_brk] new_brk: {:#x}", new_brk as usize);

    // #[cfg(feature = "test")]
    // {
    //     if new_brk as usize == 0x1234_5678 {
    //         // do_kernel_test();

    //         use crate::do_test;
    //         do_test!(ucheck_test);
    //         return Ok(0);
    //     }
    //     else {
    //         info!("[sys_brk] NO TEST");
    //     }
    // }

    let task = current_task().unwrap();
    Ok(task
        .with_mut_memory_space(|m| m.reset_heap_break((new_brk as usize).into()))
        .0 as usize)
}

pub fn sys_mmap(
    addr: *const u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: usize,
    offset: usize,
) -> SysResult<usize> {
    info!("[sys_mmap] fd: {}, offset: {}", fd, offset);
    let addr = addr as usize;
    let flags = MmapFlags::from_bits_truncate(flags);
    let prot = MmapProt::from_bits_truncate(prot);
    let perm = MapPerm::from(prot);
    info!("[sys_mmap] addr:{addr:#x}, length:{length:#x}, prot:{prot:?}, flags:{flags:?}, fd:{fd}, offset:{offset:#x}");

    if unlikely(length == 0 || (addr & PAGE_MASK != 0) || (offset & PAGE_MASK != 0)) {
        info!("aaaaa");
        return Err(Errno::EINVAL);
    }

    // 将length对齐PAGESIZE
    let padding = match (PAGE_SIZE - length % PAGE_SIZE) {
        res if res == PAGE_SIZE => 0,
        res if res != PAGE_SIZE => res,
        _ => {
            unreachable!()
        }
    };
    let length = length + padding;

    let task = current_task().unwrap();
    if flags.contains(MmapFlags::MAP_FIXED) {
        // at specific addr
        task.with_mut_memory_space(|m| m.unmap(addr.into()..(addr + length).into()));
    }

    if flags.contains(MmapFlags::MAP_ANONYMOUS) {
        // 匿名映射
        // TODO: merge branches
        let start_va = task
            .with_mut_memory_space(|m| match flags.intersection(MmapFlags::MAP_TYPE_MASK) {
                MmapFlags::MAP_SHARED => m.alloc_mmap_anon(addr.into(), length, perm, flags),
                MmapFlags::MAP_PRIVATE => m.alloc_mmap_anon(addr.into(), length, perm, flags),
                _ => {
                    unimplemented!()
                }
            })
            .unwrap();
        info!("[sys_mmap] ret {:#x}", start_va.0);
        return Ok(start_va.0);
    } else {
        let fd = task.get_fd(fd)?;
        if fd.is_none() {
            return Err(Errno::EBADF);
        }
        if let Err(e) = fd.check_mmap_valid(flags, prot) {
            return Err(e);
        }
        let file = fd.file.unwrap();
        let start_va = task
            .with_mut_memory_space(|m| {
                m.alloc_mmap_area_lazily(addr.into(), length, perm, flags, file, offset)
            })
            .unwrap();
        info!("[sys_mmap] ret {:#x}", start_va.0);
        return Ok(start_va.0);
    }
    Err(Errno::EBADCALL)
}

pub fn sys_munmap(addr: *const u8, length: usize) -> SysResult<usize> {
    info!(
        "[sys_munmap] addr:{:#x}, length:{:#x}",
        addr as usize, length
    );
    let addr = addr as usize;
    if unlikely(!is_aligned_to_page(addr)) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let length_aligned = align_up_by_page(length);
    task.with_mut_memory_space(|m| m.unmap(addr.into()..(addr + length_aligned).into()));
    Ok(0)
}

pub fn sys_mprotect(addr: *const u8, length: usize, prot: i32) -> SysResult<usize> {
    let addr = addr as usize;
    if unlikely(!is_aligned_to_page(addr)) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let length_aligned = align_up_by_page(length);
    task.with_mut_memory_space(|m| {
        m.mprotect(
            addr.into()..(addr + length_aligned).into(),
            MmapProt::from_bits(prot).ok_or(Errno::EINVAL)?.into(),
        )
    })
    .map(|_| 0)
}

pub fn sys_mremap() -> SysResult<usize> {
    info!("[sys_mremap] start");
    Ok(0)
}

pub fn sys_membarrier() -> SysResult<usize> {
    info!("[sys_membarrier] start");
    Ok(0)
}

pub fn sys_shmget(key: isize, size: usize, shmflg: i32) -> SysResult<usize> {
    // info!("[sys_shmget] start");
    let shmflag = ShmGetFlags::from_bits_truncate(shmflg);
    info!(
        "[sys_shmget] key: {}, size: {}, shmflag: {:?}",
        key, size, shmflag
    );
    let key32 = key as i32;
    let size = align_up_by_page(size);
    let task = current_task().unwrap();
    if let Some(shmobj) = SHARED_MEMORY_MANAGER.read().get(&key32) {
        if unlikely(shmflag.contains(ShmGetFlags::IPC_EXCL)) {
            return Err(Errno::EEXIST);
        }
        if unlikely(size > shmobj.size()) {
            return Err(Errno::EINVAL);
        }
        // not used
        if unlikely(!shmobj.shmid_ds.shm_perm.check_perm(IPCPermMode::empty())) {
            return Err(Errno::EPERM);
        }
        /// todo: check perm
        info!("[sys_shmget] existed, ret = {}", key32);
        return Ok(key32 as usize);
    }
    if unlikely(!shmflag.contains(ShmGetFlags::IPC_CREAT)) {
        return Err(Errno::ENOENT);
    }
    if unlikely(size == 0) {
        return Err(Errno::EINVAL);
    }
    let mode = IPCPermMode::from_bits_truncate(shmflg as u32 & 0o777);
    let shmobj = ShmObject::new(
        IPCPerm::new(IPCKey::from_user(key32), mode),
        size,
        task.get_pid(),
    );
    let ret = shmobj.ipc_key();
    SHARED_MEMORY_MANAGER.write().insert(ret, shmobj);
    info!("[sys_shmget] new, ret = {}", ret);
    Ok(ret as usize)
    // let shmid = task.shm_list.lock().alloc(key, size, shmflag).ok_or(Errno::ENOMEM)?;
}

pub fn sys_shmctl(shmid: isize, op: isize, buf: *const u8) -> SysResult<usize> {
    info!(
        "[sys_shmctl] shmid: {}, op: {}, buf_addr: {:#x}",
        shmid, op, buf as usize
    );
    let op = ShmOp::from(op);
    let shmid = shmid as i32;
    match op {
        ShmOp::IPC_STAT => {
            if buf as usize == 0 {
                return Err(Errno::EINVAL);
            }
            if let Some(shmobj) = SHARED_MEMORY_MANAGER.read().get(&shmid) {
                let buf = user_ref_mut::<ShmidDs>((buf as usize).into())?.unwrap();
                *buf = shmobj.shmid_ds.clone();
                Ok(0)
            } else {
                Err(Errno::EINVAL)
            }
        }
        ShmOp::IPC_RMID => {
            warn!("[sys_shmctl] IPC_RMID, unimplemented");
            Ok(0)
        }
        ShmOp::IPC_INFO => {
            warn!("[sys_shmctl] IPC_INFO, unimplemented");
            Ok(0)
        }
        ShmOp::IPC_SET => {
            warn!("[sys_shmctl] IPC_SET, unimplemented");
            Ok(0)
        }
        ShmOp::SHM_INFO => {
            warn!("[sys_shmctl] SHM_INFO, unimplemented");
            Ok(0)
        }
        ShmOp::SHM_STAT => {
            warn!("[sys_shmctl] SHM_STAT, unimplemented");
            Ok(0)
        }
        ShmOp::SHM_STAT_ANY => {
            warn!("[sys_shmctl] SHM_STAT_ANY, unimplemented");
            Ok(0)
        }
        _ => {
            error!("[sys_shmctl] unimplemented");
            Err(Errno::EINVAL)
        }
    }
}

pub fn sys_shmat(shmid: isize, shmaddr: *const u8, shmflg: i32) -> SysResult<usize> {
    // info!("[sys_shmat] start");
    let shmflag = ShmAtFlags::from_bits_truncate(shmflg);
    info!(
        "[sys_shmat] shmid: {}, shmaddr: {:#x}, shmflag: {:?}",
        shmid, shmaddr as usize, shmflag
    );
    let shmaddr = VirtAddr::from(shmaddr as usize);
    let shmid = shmid as i32;

    if !is_aligned_to_page(shmaddr.0) && !shmflag.contains(ShmAtFlags::SHM_RND) {
        info!("[sys_shmat] shmaddr is not aligned to page");
        return Err(Errno::EINVAL);
    }
    let mut map_perm = MapPerm::UR;
    map_perm.set(MapPerm::W, !shmflag.contains(ShmAtFlags::SHM_RDONLY));
    map_perm.set(MapPerm::X, shmflag.contains(ShmAtFlags::SHM_EXEC));
    if shmflag.contains(ShmAtFlags::SHM_REMAP) {
        warn!("[sys_shmat] SHM_REMAP, unimplemented");
    }
    if let Some(shmobj) = SHARED_MEMORY_MANAGER.write().get_mut(&shmid) {
        let task = current_task().unwrap();
        task.with_mut_shmid_table(|shmid_table| {
            shmid_table.insert(shmaddr, shmid);
            Ok(())
        })?;

        let ret = task.with_mut_memory_space(|m| {
            m.attach_shm(shmobj.size(), shmaddr, map_perm, &mut shmobj.pages)
        });

        shmobj.attach_one(task.get_pid());

        info!("[sys_shmat] ret = {:#x}", ret.0);
        Ok(ret.0)
    } else {
        Err(Errno::EINVAL)
    }
}

pub fn sys_shmdt(shmaddr: *const u8) -> SysResult<usize> {
    // info!("[sys_shmdt] start");
    info!("[sys_shmdt] shmaddr: {:#x}", shmaddr as usize);
    let shmaddr = VirtAddr::from(shmaddr as usize);
    let task = current_task().unwrap();
    let shmid =
        task.with_mut_shmid_table(|shmid_table| shmid_table.remove(&shmaddr).ok_or(Errno::EINVAL))?;

    if let Some(shmobj) = SHARED_MEMORY_MANAGER.write().get_mut(&shmid) {
        task.with_mut_memory_space(|m| m.detach_shm(shmaddr));
        shmobj.detach_one(task.get_pid());
        info!("[sys_shmdt] ret = {:#x}", shmaddr.0);
        Ok(shmaddr.0)
    } else {
        panic!("[sys_shmdt] global manager did not contain the shmid which is attached by this process");
    }
}
