use log::info;

use crate::{
    hal::config::{align_up_by_page, is_aligned_to_page, PAGE_MASK, PAGE_SIZE}, 
    mm::{memory_space::{vm_area::{MapPerm, VmArea}, MemorySpace, MmapFlags, MmapProt}, 
    MapPermission}, 
    utils::{Errno, SysResult}
};
use crate::task::{current_task};

pub fn sys_brk(new_brk: *const u8) -> SysResult<usize> {
    let task = current_task().unwrap();
    Ok(task.with_mut_memory_space(|m| { m.reset_heap_break((new_brk as usize).into()) }).0 as usize)
}

pub fn sys_mmap(
    addr: *const u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: usize,
    offset: usize,
) -> SysResult<usize> {
    let addr = addr as usize;
    if length == 0 || (addr & PAGE_MASK != 0) || (offset & PAGE_MASK != 0) {
        info!("aaaaa");
        return Err(Errno::EINVAL);
    }
    let flags = MmapFlags::from_bits_truncate(flags);
    let prot = MmapProt::from_bits_truncate(prot);
    info!("[sys_mmap] addr:{addr:#x}, length:{length:#x}, prot:{prot:?}, flags:{flags:?}, fd:{fd}, offset:{offset:#x}");
    let perm = MapPerm::from(prot);
    
    // 将length对齐PAGESIZE
    let padding = match (PAGE_SIZE - length % PAGE_SIZE) {
        res if res == PAGE_SIZE => 0,
        res if res != PAGE_SIZE => res,
        _ => {unreachable!()},
    };
    let length = length + padding;
    
    let task = current_task().unwrap();
    if flags.contains(MmapFlags::MAP_FIXED) {
        // at specific addr
        task.with_mut_memory_space(|m| { m.unmap(addr.into()..(addr + length).into()) });
    }

    if flags.contains(MmapFlags::MAP_ANONYMOUS) { // 匿名映射
        let start_va = task.with_mut_memory_space(|m| {
            match flags.intersection(MmapFlags::MAP_TYPE_MASK) {
                MmapFlags::MAP_SHARED => {
                    m.alloc_mmap_shared_anonymous(addr.into(), length, perm, flags)
                }
                MmapFlags::MAP_PRIVATE => {
                    m.alloc_mmap_anonymous(addr.into(), length, perm, flags)
                }
                _ => {
                    unimplemented!()
                }
            }
        }).unwrap();
        return Ok(start_va.0);
    } else {
        let fd = task.get_fd(fd)?;
        if fd.is_none() { return Err(Errno::EBADF); }
        if let Err(e) = fd.check_mmap_valid(flags, prot) {
            return Err(e);
        } 
        let file = fd.file.unwrap();
        let start_va = task.with_mut_memory_space(|m| {
            m.alloc_mmap_area_lazily(addr.into(), length, perm, flags, file, offset)
        }).unwrap();
        return Ok(start_va.0);
    }
    Err(Errno::EBADCALL)
}

pub fn sys_munmap(addr: *const u8, length: usize) -> SysResult<usize> {
    let addr = addr as usize;
    if (!is_aligned_to_page(addr)) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let length_aligned = align_up_by_page(length);
    task.with_mut_memory_space(|m| {
        m.unmap(addr.into()..(addr+length_aligned).into())
    });
    Ok(0)
}

pub fn sys_mprotect(addr: *const u8, length: usize, prot: i32) -> SysResult<usize>{
    let addr = addr as usize;
    if (!is_aligned_to_page(addr)) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let length_aligned = align_up_by_page(length);
    task.with_mut_memory_space(|m| {
        m.mprotect(
            addr.into()..(addr + length_aligned).into(),
            MmapProt::from_bits(prot).
                ok_or(Errno::EINVAL)?
                .into(),
        )
    }).map(|_| 0)
}

pub fn sys_mremap() -> SysResult<usize> {
    info!("[sys_mremap] start");
    Ok(0)
}

pub fn sys_membarrier() -> SysResult<usize> {
    info!("[sys_membarrier] start");
    Ok(0)
}