use core::{marker::PhantomData, ptr::NonNull};

use alloc::boxed::Box;

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
