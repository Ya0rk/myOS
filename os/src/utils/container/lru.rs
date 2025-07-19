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
        panic!("finished linked test")
    }
}
