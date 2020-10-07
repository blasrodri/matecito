use std::ptr::NonNull;

#[derive(Debug)]
pub(crate) struct DoublyLinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    elements: usize,
}

#[derive(Debug)]
pub(crate) struct Node<T> {
    pub elem: T,
    pub next: Option<NonNull<Node<T>>>,
    pub prev: Option<NonNull<Node<T>>>,
}

impl<T> DoublyLinkedList<T> {
    pub(crate) fn new() -> Self {
        Self {
            head: None,
            tail: None,
            elements: 0,
        }
    }

    pub(crate) fn delete(&mut self, non_null_node: NonNull<Node<T>>) -> Option<T> {
        let mut non_null_node = non_null_node; // we need to make it mutable
        let node = unsafe { non_null_node.as_mut() };

        let prev_node = match node.prev {
            None => None,
            Some(mut pnode) => NonNull::new(unsafe { pnode.as_mut() }),
        };

        let next_node = match node.next {
            None => None,
            Some(mut pnode) => NonNull::new(unsafe { pnode.as_mut() }),
        };

        match next_node {
            // the current node was the head. This is because we expect that there is always
            // a previous node. The only case where this happen is when we're working with the head.
            None => {
                self.tail = prev_node;
            }
            Some(mut nnode_opt) => {
                let next_node = unsafe { nnode_opt.as_mut() };
                next_node.prev = prev_node;
            }
        }

        match prev_node {
            // the current node was the head. This is because we expect that there is always
            // a previous node. The only case where this happen is when we're working with the head.
            None => {
                self.head = next_node;
            }
            Some(nnode_opt) => {
                let prev_node = unsafe { nnode_opt.as_ptr().as_mut().unwrap() };
                prev_node.next = next_node;
            }
        }

        // Free node's memory
        let node = unsafe { Box::from_raw(node) };

        self.elements -= 1;
        Some(node.elem)
    }

    pub(crate) fn push_back(&mut self, elem: T) -> NonNull<Node<T>> {
        // a. if head is none, then head becomes the node
        //    also tail becomes then node
        // b. head becomes the node.
        //    The previous head.prev points to the node.
        //    head.next points to the previous head.

        let node = Node {
            elem,
            next: None,
            prev: None,
        };
        let boxed_node = Box::into_raw(Box::new(node));
        let node = NonNull::new(boxed_node);

        let raw_old_head = self.head;

        // scenario (a)
        if self.head.is_none() {
            self.head = node;
            self.tail = node;
            self.elements += 1;
            return self.head.unwrap();
        }

        // scenario (b)
        self.head = node;

        match raw_old_head {
            None => (),
            Some(mut non_null_node) => unsafe {
                non_null_node.as_mut().prev = node;
                self.head.unwrap().as_mut().next = raw_old_head;
            },
        };
        self.elements += 1;
        return self.head.unwrap();
    }

    pub(crate) fn pop_front(&mut self) -> Option<T> {
        // no nodes in the tail
        if self.tail.is_none() {
            return None;
        }
        // there is at least one node.
        // return the old tail.
        let old_tail = self.tail;

        self.tail = match old_tail {
            None => return None,
            Some(mut non_null_node) => unsafe { non_null_node.as_mut().prev },
        };

        if self.tail.is_none() {
            self.head = None;
        }

        let old_tail = unsafe { Box::from_raw(old_tail.unwrap().as_ptr()) };
        self.elements -= 1;
        Some(old_tail.elem)
    }

    #[allow(dead_code)]
    pub(crate) fn pop_back(&mut self) -> Option<T> {
        // no nodes in the tail
        if self.head.is_none() {
            return None;
        }
        // there is at least one node.
        // return the old head.
        let old_head = self.head;

        self.head = match old_head {
            None => return None,
            Some(mut non_null_node) => unsafe { non_null_node.as_mut().next },
        };

        if self.head.is_none() {
            self.tail = None;
        }

        let result = unsafe { Box::from_raw(old_head.unwrap().as_ptr()) };
        self.elements -= 1;
        Some(result.elem)
    }

    #[allow(dead_code)]
    pub(crate) fn peek_front(&self) -> Option<&NonNull<Node<T>>> {
        if self.tail.is_none() {
            return None;
        }
        self.tail.as_ref()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.elements == 0
    }

    pub(crate) fn num_elements(&self) -> usize {
        self.elements
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        while !self.is_empty() {
            self.pop_front();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_front_and_pop_front() {
        let mut dll = DoublyLinkedList::new();
        assert_eq!(None, dll.pop_front());
        let range = std::ops::Range {
            start: 10,
            end: 100,
        };

        for num in 0..range.len() {
            dll.push_back(num);
        }

        for num in 0..range.len() {
            assert_eq!(Some(num), dll.pop_front());
        }
        assert_eq!(None, dll.pop_front());
    }

    #[test]
    fn push_front_and_pop_back() {
        let mut dll = DoublyLinkedList::new();
        assert_eq!(None, dll.pop_back());
        let range = std::ops::Range {
            start: 10,
            end: 100,
        };
        for num in range.clone() {
            dll.push_back(num);
        }

        for num in range.clone().rev().clone() {
            assert_eq!(Some(num), dll.pop_back());
        }
        assert_eq!(None, dll.pop_back());
    }

    #[test]
    fn populate_and_delete() {
        let mut dll = DoublyLinkedList::new();
        let range = std::ops::Range { start: 0, end: 3 };

        let mut elements = vec![];
        for num in range.clone() {
            let node = dll.push_back(num as i32);
            elements.push(node);
        }

        dll.delete(elements[1]);
        dll.delete(elements[0]);
        assert_eq!(Some(2), dll.pop_front());
        assert!(dll.is_empty());
        assert_eq!(None, dll.pop_front());
    }

    #[test]
    fn num_elements() {
        let mut dll = DoublyLinkedList::new();
        let range = std::ops::Range { start: 0, end: 3 };

        // with delete
        let mut num_elements = range.len();
        let mut elements = vec![];
        for num in range.clone() {
            let node = dll.push_back(num as i32);
            elements.push(node);
        }
        for num in range.clone() {
            dll.delete(elements[num]);
            num_elements -= 1;
            assert_eq!(num_elements, dll.num_elements());
        }

        // pop_front
        let mut num_elements = range.len();
        for num in range.clone() {
            dll.push_back(num as i32);
        }
        for _ in range.clone() {
            dll.pop_front();
            num_elements -= 1;
            assert_eq!(num_elements, dll.num_elements());
        }

        // // pop_back
        let mut num_elements = range.len();
        for num in range.clone() {
            dll.push_back(num as i32);
        }
        for _ in range.clone() {
            dll.pop_back();
            num_elements -= 1;
            assert_eq!(num_elements, dll.num_elements());
        }
    }

    #[test]
    fn edge_case_push_after_emptied_delete_and_empty_again() {
        let mut dll = DoublyLinkedList::new();
        let n1 = dll.push_back(1);
        let n2 = dll.push_back(2);
        let n3 = dll.push_back(3);

        dll.delete(n1);
        assert_eq!(2, dll.num_elements());

        dll.delete(n2);
        assert_eq!(1, dll.num_elements());

        dll.delete(n3);
        assert_eq!(0, dll.num_elements());
        assert!(dll.is_empty());

        let n1 = dll.push_back(1);
        assert_eq!(1, dll.num_elements());
        dll.delete(n1);
        assert_eq!(0, dll.num_elements());
        assert!(dll.is_empty());
    }

    #[test]
    fn edge_case_push_after_emptied_pop_front_and_empty_again() {
        let mut dll = DoublyLinkedList::new();
        dll.push_back(1);
        dll.push_back(2);
        dll.push_back(3);

        dll.pop_front();
        assert_eq!(2, dll.num_elements());

        dll.pop_front();
        assert_eq!(1, dll.num_elements());

        dll.pop_front();
        assert_eq!(0, dll.num_elements());
        assert!(dll.is_empty());

        dll.push_back(1);
        assert_eq!(1, dll.num_elements());
        dll.pop_front();
        assert_eq!(0, dll.num_elements());
        assert!(dll.is_empty());
    }

    #[test]
    fn edge_case_push_after_emptied_pop_back_and_empty_again() {
        let mut dll = DoublyLinkedList::new();
        dll.push_back(1);
        dll.push_back(2);
        dll.push_back(3);

        dll.pop_back();
        assert_eq!(2, dll.num_elements());

        dll.pop_back();
        assert_eq!(1, dll.num_elements());

        dll.pop_back();
        assert_eq!(0, dll.num_elements());
        assert!(dll.is_empty());

        dll.push_back(1);
        assert_eq!(1, dll.num_elements());
        dll.pop_back();
        assert_eq!(0, dll.num_elements());
        assert!(dll.is_empty());
    }
}
