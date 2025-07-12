use core::any::Any;
use alloc::sync::Arc;


pub trait Downcast: Any + Send + Sync {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any>;

    /// downcast_arc 用于将 Arc<dyn Trait> 动态转换为具体类型的 Arc<T>
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Option<Arc<T>> {
        let any_arc: Arc<dyn Any> = self.as_any(); // 转换为 dyn Any
        if any_arc.is::<T>() {
            let ptr = Arc::into_raw(any_arc) as *const T;
            Some(unsafe { Arc::from_raw(ptr) })
        } else {
            None
        }
    }
}