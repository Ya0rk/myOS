use alloc::ffi::CString;
use alloc::slice;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hal::config::{align_down_by_page, PAGE_MASK, PAGE_SIZE};
use crate::task::take_ktrap_ret;
use crate::utils::{Errno, SysResult};
use core::arch::asm;
use core::ffi::CStr;
use core::mem::transmute;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::str::FromStr;

use super::VirtAddr;

pub fn try_load_page(addr: VirtAddr) -> SysResult<()> {
    #[cfg(target_arch = "riscv64")]
    unsafe fn try_load_page_inner(addr: usize) {
        
        
        asm!(
            "mv t0, a0",
            "lb t0, 0(t0)",
            in("a0") addr,
            out("t0") _,
        );
    } 

    #[cfg(target_arch = "loongarch64")]
    unsafe fn try_load_page_inner(addr: usize) {
        
        asm!(
            "or $t0, $a0, $zero",
            "ld.b $t0, $t0, 0",
            in("$a0") addr,
            out("$t0") _,
        )
    }

    take_ktrap_ret();
    unsafe {
        try_load_page_inner(addr.0);
    }


    /// if None, which means no page fault is happened, Ok is expected
    /// if Some(Ok()), which means page fault is handled successfully, Ok is expected
    /// if Some(Err()), which means page fault failed because of no privilege or even no mapped area, Err is expected
    take_ktrap_ret().map_or(Ok(()), |ret| ret)
}

pub fn try_store_page(addr: VirtAddr) -> SysResult<()> {
    #[cfg(target_arch = "riscv64")]
    unsafe fn try_store_page_inner(addr: usize) {
        
        asm!(
            "mv t0, a0",
            "lb t1, 0(t0)",
            "sb t1, 0(t0)",
            in("a0") addr,
            out("t0") _,
            out("t1") _,
        )
    }

    #[cfg(target_arch = "loongarch64")]
    unsafe fn try_store_page_inner(addr: usize) {
        asm!(
            "or $t0, $a0, $zero",
            "ld.b $t1, $t0, 0",
            "st.b $t1, $t0, 0",
            in("$a0") addr,
            out("$t0") _,
            out("$t1") _,
        )
    }

    take_ktrap_ret();
    unsafe {
        try_store_page_inner(addr.0);
    }

    take_ktrap_ret().map_or(Ok(()), |ret| ret)
}




pub fn check_readable(start_va: VirtAddr, len: usize) -> SysResult<()> {
    for va_page in (start_va.align_down()..(start_va + len)).step_by(PAGE_SIZE) {
        try_load_page(va_page)?;
    }
    Ok(())
}
pub fn check_writable(start_va: VirtAddr, len: usize) -> SysResult<()> {
    for va_page in (start_va.align_down()..(start_va + len)).step_by(PAGE_SIZE) {
        try_store_page(va_page)?;
    }
    Ok(())
}

// 不使用zerocopy，减小代码理解难度
pub fn user_ref<T: Sized>(addr: VirtAddr) -> SysResult<Option<&'static T>> {
    unsafe {
        if addr.0 == 0 {
            return Ok(None);
        }
        let len = size_of::<T>();
        check_readable(addr, len)?;
        let ptr = addr.as_ptr() as *const T;
        Ok(Some(&*ptr))
    }
}
pub fn user_ref_mut<T: Sized>(addr: VirtAddr) -> SysResult<Option<&'static mut T>> {
    unsafe {
        if addr.0 == 0 {
            return Ok(None);
        }
        let len = size_of::<T>();
        check_readable(addr, len)?;
        let ptr = addr.as_ptr() as *mut T;
        Ok(Some(&mut *ptr))
    }
}


pub unsafe fn user_ptr<T: Sized>(addr: VirtAddr) -> SysResult<Option<*const T>> {
    if addr.0 == 0 {
        return Ok(None);
    }
    let len = size_of::<T>();
    check_readable(addr, len)?;
    let ptr = addr.as_ptr() as *const T;
    Ok(Some(ptr))
}
pub unsafe fn user_mut_ptr<T: Sized>(addr: VirtAddr) -> SysResult<Option<*mut T>> {
    if addr.0 == 0 {
        return Ok(None);
    }
    let len = size_of::<T>();
    check_readable(addr, len)?;
    let ptr = addr.as_ptr() as *mut T;
    Ok(Some(ptr))
}

pub fn user_slice<T: Sized>(addr: VirtAddr, len: usize) -> SysResult<Option<&'static [T]>> {
    unsafe {
        if addr.0 == 0 {
            return Ok(None);
        }
        check_readable(addr, len)?;
        let bytes = from_raw_parts(addr.as_ptr(), len);
        Ok(Some(transmute(bytes)))
    }
}

pub fn user_slice_mut<T: Sized>(addr: VirtAddr, len: usize) -> SysResult<Option<&'static mut [T]>> {
    unsafe {
        if addr.0 == 0 {
            return Ok(None);
        }
        check_writable(addr, len)?;
        let bytes = from_raw_parts_mut(addr.as_ptr(), len);
        Ok(Some(transmute(bytes)))
    }
}

pub fn user_cstr(addr: VirtAddr) -> SysResult<Option<String>> {
    unsafe {
        if addr.0 == 0 {
            return Ok(None);
        }
        // TODO: len设置太大，可能导致跨页
        let len = 256;
        // 跨页处理
        let mut cur_len = len.min(PAGE_SIZE - (addr.0 & PAGE_MASK));
        loop {
            check_readable(addr, cur_len)?;
            let bytes = from_raw_parts(addr.as_ptr(), cur_len);
            if let Ok(cstr) = CStr::from_bytes_until_nul(bytes) {
                return Ok(Some(String::from(cstr.to_str().unwrap())));
            } else if cur_len == len {
                return Err(Errno::ENAMETOOLONG);
            } else {
                cur_len = len.min(cur_len + PAGE_SIZE);
            }
        }
        // Ok(Some(transmute(bytes)))
        // Ok(Some(CString::from_raw(addr.as_ptr()).into_string().unwrap()))
    }
}

pub fn user_cstr_array(addr: VirtAddr) -> SysResult<Option<Vec<String>>> {
    unsafe {
        if addr.0 == 0 {
            return Ok(None);
        }
        let mut cstr_array = Vec::<String>::new();
        let len = 256;
        // check_readable(addr, len)?;
        let mut ptr = user_ptr::<usize>(addr)?.unwrap();
        loop {
            if let Some(cstr) = user_cstr((*ptr).into())?
                && !cstr.is_empty()
            {
                cstr_array.push(cstr);
            } else {
                break;
            }
            ptr = ptr.add(1);
        }
        Ok(Some(cstr_array))
    }
}
