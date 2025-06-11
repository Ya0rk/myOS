use alloc::ffi::CString;
use alloc::slice;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hal::config::{PAGE_MASK, PAGE_SIZE};
use crate::utils::{Errno, SysResult};
use core::ffi::CStr;
use core::mem::transmute;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::str::FromStr;

use super::VirtAddr;

pub fn check_readable(addr: VirtAddr, len: usize) -> SysResult<()> {
    Ok(())
}
pub fn check_writable(addr: VirtAddr, len: usize) -> SysResult<()> {
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

unsafe fn _user_ptr<T: Sized>(addr: VirtAddr) -> SysResult<Option<*const T>> {
    if addr.0 == 0 {
        return Ok(None);
    }
    let len = size_of::<T>();
    check_readable(addr, len)?;
    let ptr = addr.as_ptr() as *const T;
    Ok(Some(ptr))
}
unsafe fn _user_ptr_mut<T: Sized>(addr: VirtAddr) -> SysResult<Option<*mut T>> {
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
        let mut ptr = _user_ptr::<usize>(addr)?.unwrap();
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
