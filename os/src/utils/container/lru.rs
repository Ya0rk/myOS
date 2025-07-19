use alloc::boxed::Box;
use core::{hash::Hash, marker::PhantomData, ptr::NonNull};
use hashbrown::HashMap;

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    prev: Link<T>,
    next: Link<T>,
    elem: T,
}

impl<T> Node<T> {
    fn new(elem: T) -> Self {
        Self {
            prev: None,
            next: None,
            elem: elem,
        }
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let mut new = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(elem))));
            if let Some(mut old) = self.front {
                old.as_mut().prev = Some(new);
                new.as_mut().next = Some(old);
            } else {
                self.back = Some(new);
            }
            self.front = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr());
                let result = boxed_node.elem;
                self.front = boxed_node.next;
                if let Some(mut new) = self.front {
                    new.as_mut().prev = None;
                } else {
                    self.back = None;
                }
                self.len -= 1;
                result
            })
        }
    }

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let mut new = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(elem))));
            if let Some(mut old) = self.back {
                old.as_mut().next = Some(new);
                new.as_mut().prev = Some(old);
            } else {
                self.front = Some(new);
            }
            self.back = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.back.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr());
                let result = boxed_node.elem;
                self.back = boxed_node.prev;
                if let Some(mut new) = self.back {
                    new.as_mut().next = None;
                } else {
                    self.front = None;
                }
                self.len -= 1;
                result
            })
        }
    }

    pub fn remove_by_ptr(&mut self, node: NonNull<Node<T>>) -> T {
        unsafe {
            let prev = (*node.as_ptr()).prev;
            let next = (*node.as_ptr()).next;

            if let Some(mut p) = prev {
                p.as_mut().next = next;
            } else {
                self.front = next;
            }

            if let Some(mut p) = next {
                p.as_mut().prev = prev;
            } else {
                self.back = prev;
            }

            let boxed_node = Box::from_raw(node.as_ptr());
            self.len -= 1;
            boxed_node.elem
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    pub fn test() {
        let mut list = LinkedList::new();

        // push_front 测试
        list.push_front(1);
        list.push_front(2);
        list.push_back(3);
        println!("len after push: {}", list.len); // 3

        // pop_front 测试
        println!("pop_front: {:?}", list.pop_front()); // Some(2)
        println!("pop_back: {:?}", list.pop_back()); // Some(3)
        println!("pop_front: {:?}", list.pop_front()); // Some(1)
        println!("pop_front: {:?}", list.pop_front()); // None

        println!("len after pop: {}", list.len); // 0

        Self::test_remove_by_ptr();
        panic!("finished linked test")
    }

    pub fn test_remove_by_ptr() {
        println!("\n--- Testing remove_by_ptr ---");

        // --- Scenario 1: Remove the only node ---
        let mut list_single = LinkedList::new();
        list_single.push_front(10); // List: [10]
        let node_10_ptr = list_single.front.unwrap(); // Get pointer to the only node

        println!("Test: Remove the only node (10)");
        assert_eq!(list_single.len(), 1);
        let removed_val = list_single.remove_by_ptr(node_10_ptr);
        assert_eq!(removed_val, 10);
        assert_eq!(list_single.len(), 0);
        assert!(list_single.front.is_none());
        assert!(list_single.back.is_none());
        println!("  - Successfully removed the only node");

        // --- Scenario 2: Remove head node (multiple elements) ---
        let mut list_multi_front = LinkedList::new();
        list_multi_front.push_back(1); // List: [1]
        list_multi_front.push_back(2); // List: [1, 2]
        list_multi_front.push_back(3); // List: [1, 2, 3]

        let head_ptr_multi_front = list_multi_front.front.unwrap();

        println!("Test: Remove head node (1)");
        assert_eq!(list_multi_front.len(), 3);
        let removed_val = list_multi_front.remove_by_ptr(head_ptr_multi_front);
        assert_eq!(removed_val, 1);
        assert_eq!(list_multi_front.len(), 2);
        unsafe {
            assert_eq!(list_multi_front.pop_front(), Some(2)); // New head should be 2
            assert_eq!(list_multi_front.pop_front(), Some(3)); // Then 3
        }
        println!("  - Successfully removed head node");

        // --- Scenario 3: Remove tail node (multiple elements) ---
        let mut list_multi_back = LinkedList::new();
        list_multi_back.push_front(30); // List: [30]
        list_multi_back.push_front(20); // List: [20, 30]
        list_multi_back.push_front(10); // List: [10, 20, 30]

        let tail_ptr_multi_back = list_multi_back.back.unwrap();

        println!("Test: Remove tail node (30)");
        assert_eq!(list_multi_back.len(), 3);
        let removed_val = list_multi_back.remove_by_ptr(tail_ptr_multi_back);
        assert_eq!(removed_val, 30);
        assert_eq!(list_multi_back.len(), 2);
        unsafe {
            assert_eq!(list_multi_back.pop_back(), Some(20)); // New tail should be 20
            assert_eq!(list_multi_back.pop_back(), Some(10)); // Then 10
        }
        println!("  - Successfully removed tail node");

        // --- Scenario 4: Remove middle node ---
        let mut list_middle = LinkedList::new();
        list_middle.push_back(100); // List: [100]
        list_middle.push_back(200); // List: [100, 200]
        list_middle.push_back(300); // List: [100, 200, 300]
        list_middle.push_back(400); // List: [100, 200, 300, 400]

        unsafe {
            let head_ptr = list_middle.front.unwrap(); // Points to 100
            let node_200_ptr = (*head_ptr.as_ptr()).next.unwrap(); // Points to 200

            println!("Test: Remove middle node (200)");
            assert_eq!(list_middle.len(), 4);
            let removed_val = list_middle.remove_by_ptr(node_200_ptr);
            assert_eq!(removed_val, 200);
            assert_eq!(list_middle.len(), 3);

            assert_eq!(list_middle.pop_front(), Some(100));
            assert_eq!(list_middle.pop_front(), Some(300));
            assert_eq!(list_middle.pop_front(), Some(400));
        }
        println!("  - Successfully removed middle node");

        println!("--- All remove_by_ptr tests passed! ---");
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub trait LruCache<K, V> {
    /// a cheap function to get the capacity of the cache
    fn capacity(&self) -> usize;

    /// a cheap function to get the length of the cache
    fn len(&self) -> usize;

    /// a cheap function to check if the key is in the cache
    fn contains(&self, key: &K) -> bool;

    /// get a reference to the value of the key
    ///
    /// this function will not update the lru list
    fn get(&self, key: &K) -> Option<&V>;

    /// access the key, and update the lru list
    ///
    /// NOTE: this function's signature is changed to `&mut self` to allow mutation
    fn access(&mut self, key: &K) -> bool;

    /// insert a key-value pair into the cache
    ///
    /// if the key is already in the cache, update the value and return the old value
    ///
    /// if the key is not in the cache, insert the new key-value pair
    ///
    /// if the cache is full, remove the least recently used item
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// remove the key-value pair from the cache
    fn remove(&mut self, key: &K) -> Option<V>;

    /// pop the least recently used item from the cache
    fn pop(&mut self) -> Option<V>;

    /// clear the cache
    fn clear(&mut self);
}

/// A simple LRU cache implementation
///
/// using a HashMap and a doubly linked list
pub struct Lru<K, V> {
    /// the linked list is used to store the key-value pairs
    ///
    /// the front of the list is the most recently used item
    ///
    /// the back of the list is the least recently used item
    ///
    /// we store `(K, V)` in the list to get the key when we pop from the back
    list: LinkedList<(K, V)>,

    /// the hash map is used to store the key and a pointer to the node in the linked list
    map: HashMap<K, NonNull<Node<(K, V)>>>,

    /// the capacity of the cache
    capacity: usize,
}

impl<K: Eq + Hash, V> Lru<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be greater than 0");
        Self {
            list: LinkedList::new(),
            map: HashMap::with_capacity(capacity),
            capacity,
        }
    }
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> for Lru<K, V> {
    fn capacity(&self) -> usize {
        self.capacity
    }

    fn len(&self) -> usize {
        self.list.len()
    }

    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.map
            .get(key)
            .map(|node| unsafe { &(*node.as_ptr()).elem.1 })
    }

    fn access(&mut self, key: &K) -> bool {
        if let Some(node_ptr) = self.map.get(key).copied() {
            // remove the node from the list
            let (key, value) = self.list.remove_by_ptr(node_ptr);
            // push it to the front
            self.list.push_front((key, value));
            // update the pointer in the map
            if let Some(front) = self.list.front {
                let key_ref = unsafe { &(*front.as_ptr()).elem.0 };
                *self.map.get_mut(key_ref).unwrap() = front;
            }
            true
        } else {
            false
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        // if the key is already in the cache, update the value
        if let Some(node_ptr) = self.map.get(&key).copied() {
            let (old_key, old_value) = self.list.remove_by_ptr(node_ptr);
            self.list.push_front((old_key, value));
            if let Some(front) = self.list.front {
                *self.map.get_mut(&key).unwrap() = front;
            }
            return Some(old_value);
        }

        // if the cache is full, remove the least recently used item
        if self.list.len() >= self.capacity {
            if let Some((key, _)) = self.list.pop_back() {
                self.map.remove(&key);
            }
        }

        // insert the new key-value pair
        self.list.push_front((key.clone(), value));
        if let Some(front) = self.list.front {
            self.map.insert(key, front);
        }

        None
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key).map(|node_ptr| {
            let (_, value) = self.list.remove_by_ptr(node_ptr);
            value
        })
    }

    fn pop(&mut self) -> Option<V> {
        self.list.pop_back().map(|(key, value)| {
            self.map.remove(&key);
            value
        })
    }

    fn clear(&mut self) {
        self.list.clear();
        self.map.clear();
    }
}

impl<K: Eq + Hash + Clone, V> Lru<K, V> {
    /// a test function to test the lru cache
    pub fn test_lru() {
        println!("\n--- Testing Lru ---");

        let mut lru = Lru::new(3);

        // Test insertion
        assert_eq!(lru.insert(1, "a"), None); // [1:a]
        assert_eq!(lru.insert(2, "b"), None); // [2:b, 1:a]
        assert_eq!(lru.insert(3, "c"), None); // [3:c, 2:b, 1:a]
        assert_eq!(lru.len(), 3);
        println!("Test insert: OK");

        // Test get
        assert_eq!(lru.get(&1), Some(&"a"));
        assert_eq!(lru.get(&2), Some(&"b"));
        assert_eq!(lru.get(&3), Some(&"c"));
        println!("Test get: OK");

        // Test insertion that causes eviction
        assert_eq!(lru.insert(4, "d"), None); // [4:d, 3:c, 2:b], evicts 1:a
        assert_eq!(lru.len(), 3);
        assert_eq!(lru.contains(&1), false);
        assert_eq!(lru.get(&4), Some(&"d"));
        println!("Test eviction: OK");

        // Test access
        lru.access(&2); // [2:b, 4:d, 3:c]
                        // Now inserting should evict 3
        assert_eq!(lru.insert(5, "e"), None); // [5:e, 2:b, 4:d], evicts 3:c
        assert_eq!(lru.contains(&3), false);
        assert_eq!(lru.get(&5), Some(&"e"));
        println!("Test access: OK");

        // Test update
        assert_eq!(lru.insert(2, "B"), Some("b")); // [2:B, 5:e, 4:d]
        assert_eq!(lru.get(&2), Some(&"B"));
        assert_eq!(lru.len(), 3);
        println!("Test update: OK");

        // Test remove
        assert_eq!(lru.remove(&5), Some("e")); // [2:B, 4:d]
        assert_eq!(lru.len(), 2);
        assert_eq!(lru.contains(&5), false);
        println!("Test remove: OK");

        // Test pop (LRU element)
        assert_eq!(lru.pop(), Some("d")); // [2:B], pops 4:d
        assert_eq!(lru.len(), 1);
        assert_eq!(lru.contains(&4), false);
        println!("Test pop: OK");

        // Test clear
        lru.clear();
        assert_eq!(lru.len(), 0);
        assert!(lru.map.is_empty());
        assert!(lru.list.front.is_none());
        println!("Test clear: OK");

        println!("--- All Lru tests passed! ---");
    }
}
