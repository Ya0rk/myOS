use core::{
    borrow::Borrow,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use alloc::sync::Arc;
use hashbrown::HashMap;
use spin::RwLock;

// TODO: 当前仅仅是一个 hashmap 后续应当优化为 LRU 有淘汰机制
/// 一个 cache 模板供给内存使用
pub struct Cache<T: Eq + Hash + Clone, U: Clone + CacheStatus> {
    pub map: RwLock<HashMap<T, U>>,
}

impl<T: Eq + Hash + Clone, U: Clone + CacheStatus> Cache<T, U> {
    /// create a new cache
    /// now _capacity is useless
    pub fn new(_capacity: usize) -> Self {
        Self {
            map: RwLock::new(HashMap::<T, U>::new()), //HashMap::<T, U>::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.read().len()
    }
    pub fn insert(&self, key: &T, value: U) -> Option<U> {
        self.write().insert(key.clone(), value)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<U>
    where
        Q: ?Sized + Hash + Eq,
        T: Borrow<Q>,
    {
        let map = self.read();
        match map.get(key) {
            Some(value) if value.is_valid() => Some(value.clone()),
            Some(_) => {
                drop(map);
                self.write().remove(key);
                None
            }
            None => None,
        }
    }

    pub fn contains(&self, key: &T) -> bool {
        self.read().contains_key(key)
    }
    pub fn remove<Q>(&self, key: &Q) -> Option<U>
    where
        Q: ?Sized + Hash + Eq,
        T: Borrow<Q>,
    {
        self.write().remove(key)
    }

    /// unavailable
    pub fn peek(&self) -> Option<U> {
        todo!()
    }
    /// unavailable
    pub fn pop(&mut self) -> Option<U> {
        todo!()
    }
    /// unavailable
    pub fn capacity(&self) -> usize {
        todo!()
    }
    /// unavailable
    pub fn resize(&mut self, capacity: usize) {
        todo!()
    }
}

pub trait CacheStatus {
    fn is_valid(&self) -> bool;
}
//
impl<T: Eq + Hash + Clone, U: Clone + CacheStatus> Deref for Cache<T, U> {
    type Target = RwLock<HashMap<T, U>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<T: Eq + Hash + Clone, U: Clone + CacheStatus> DerefMut for Cache<T, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
