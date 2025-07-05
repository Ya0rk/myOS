use alloc::sync::Arc;
use core::hash::Hash;
use hashbrown::HashMap;
use spin::RwLock;
struct LruNode<T: Copy> {
    inner: T,
    next: Option<Arc<LruNode<T>>>,
    prev: Option<Arc<LruNode<T>>>,
}

impl<T: Copy> LruNode<T> {
    fn new(inner: T, next: Arc<LruNode<T>>, prev: Arc<LruNode<T>>) -> Self {
        Self {
            inner: inner.clone(),
            next: Some(next),
            prev: Some(prev),
        }
    }
    fn new_bare(inner: T) -> Self {
        Self {
            inner: inner.clone(),
            next: None,
            prev: None,
        }
    }
    fn set_next(&mut self, next: Arc<LruNode<T>>) {
        self.next = Some(next);
    }
    fn set_prev(&mut self, prev: Arc<LruNode<T>>) {
        self.prev = Some(prev);
    }
    fn value(&self) -> T {
        self.inner.clone()
    }
}

// struct LruHead<T: Copy> {
//     next: Arc<LruNode<T>>,
// }
//
// struct LruTail<T: Copy> {
//     prev: Arc<LruNode<T>>,
// }

/// LRU 本体，注意到当前没有实现细颗粒度的锁
/// 在使用的时候应当用一把大锁包裹住
/// 未来可以以细颗粒度的锁作为优化方向
struct LRU<T: Copy, K: Copy + Hash + Eq> {
    head: Arc<LruNode<T>>,
    tail: Arc<LruNode<T>>,
    map: HashMap<K, Arc<LruNode<T>>>,
    size: usize,
    cnt: usize,
}

impl<T: Copy, K: Copy + Hash + Eq> LRU<T, K> {
    /// 在当前的设计中需要传入一个无意义的值作为head 和 tail
    fn new(size: usize, blah: T) -> Self {
        let mut head = Arc::new(LruNode::new_bare(blah.clone()));
        let mut tail = Arc::new(LruNode::new_bare(blah.clone()));
        head.set_prev(tail.clone());
        head.set_next(tail.clone());
        let cnt = 0;
        let map: HashMap<K, Arc<LruNode<T>>> = HashMap::new();
        Self {
            head,
            tail,
            map,
            size,
            cnt,
        }
    }
    fn get(&self, key: K) -> Option<T> {
        let node = if let Some(node) = self.map.get(&key) {
            node
        } else {
            return None;
        };
        Some(node.value())
    }
}
