use log::info;
use crate::{
    hal::config::{align_up_by_page, is_aligned_to_page, PAGE_MASK},
    mm::{memory_space::{vm_area::{MapPerm, VmArea}, MemorySpace, MmapFlags, MmapProt}, MapPermission},
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
    let flags = MmapFlags::from_bits_truncate(flags);
    let prot = MmapProt::from_bits_truncate(prot);
    info!("[sys_mmap] addr:{addr:#x}, length:{length:#x}, prot:{prot:?}, flags:{flags:?}, fd:{fd}, offset:{offset:#x}");
    let mut perm = MapPerm::U;
    if (prot.contains(MmapProt::PROT_READ))     { perm.insert(MapPerm::R); }
    if (prot.contains(MmapProt::PROT_WRITE))    { perm.insert(MapPerm::W); }
    if (prot.contains(MmapProt::PROT_EXEC))     { perm.insert(MapPerm::X); }

    if length == 0 || (addr & PAGE_MASK != 0) || (offset & PAGE_MASK != 0) {
        return Err(Errno::EINVAL);
    }
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
        Ok(start_va.0)
    } else {
        if let Ok(fd) = task.fd_table.lock().get_fd(fd) {
            if let Err(e) = fd.check_mmap_valid(flags, prot) {
                return Err(e);
            } 
            let file = fd.file.unwrap();
            let start_va = task.with_mut_memory_space(|m| {
                m.alloc_mmap_area_lazily(addr.into(), length, perm, flags, file, offset)
            }).unwrap();
            Ok(start_va.0)
        }
        else {
            Err(Errno::EBADF)
        }
        
    }
    // Ok(0)
}

pub fn sys_munmap(addr: *const u8, length: usize) -> SysResult<usize> {
    let addr = addr as usize;
    if (!is_aligned_to_page(addr)) {
        return Err(Errno::EINVAL);
    }
    let task = current_task().unwrap();
    let length_aligned = align_up_by_page(length);
    task.with_mut_memory_space(|m| {
        m.unmap(addr.into()..(addr+length).into())
    });
    Ok(0)
}