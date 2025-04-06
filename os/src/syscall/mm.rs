use crate::{config::is_aligned_to_page, mm::memory_space::{vm_area::{MapPerm, VmArea}, MemorySpace, MmapFlags, MmapProt}, utils::{Errno, SysResult}};
use crate::task::{current_task};

pub fn sys_brk(new_brk: *const u8) -> SysResult<usize> {
    let task = current_task().unwrap();
    Ok(task.with_mut_memory_space(|m| { m.reset_heap_break((new_brk as usize).into()) }).0 as usize)
}

