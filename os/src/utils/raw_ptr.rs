use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct RawPtr<T> {
    ptr: *mut T,
}

impl<T> RawPtr<T> {
    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.ptr }
    }

    pub fn from_ref(t: &T) -> Self {
        Self {
            ptr: unsafe {
                t as *const T as *mut T
            }
        }
    }
}

unsafe impl<T> Send for RawPtr<T> {}
unsafe impl<T> Sync for RawPtr<T> {}

impl<T> Deref for RawPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for RawPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}