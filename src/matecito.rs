use crate::errors::MatecitoResult;
use crate::linked_list::{DoublyLinkedList, Node};

use std::collections::HashMap;
use std::ptr::NonNull;

use twox_hash::RandomXxHashBuilder64;

type NonNullNode<T> = NonNull<Node<T>>;
pub struct Matecito<T> {
    m: HashMap<u64, NonNullNode<(u64, T)>, RandomXxHashBuilder64>,
    dll: DoublyLinkedList<(u64, T)>,
    max_size: usize, // amount of bytes?
}

impl<'a, T: std::fmt::Debug + Clone> Matecito<T> {
    pub fn new(max_size: usize) -> Self {
        let m: HashMap<_, _, RandomXxHashBuilder64> = Default::default();
        Self {
            m,
            dll: DoublyLinkedList::new(),
            max_size,
        }
    }

    pub fn put(&mut self, key: u64, value: T) -> MatecitoResult<u64> {
        // TODO: handle the case where the dll is FULL!
        if self.max_size == self.dll.num_elements() {
            // we might want to do something with it.
            let _ = self.evict_node();
        }

        let node = self.dll.push_back((key, value));
        dbg!(&self.dll);

        self.m.insert(key, node);
        MatecitoResult::Ok(key)
    }

    pub fn get(&mut self, key: u64) -> Option<T> {
        if self.m.get(&key).is_none() {
            return None;
        }
        // We've confirmed that there is such key... so we can unwrap.
        dbg!(&self.dll);
        dbg!(&self.m);
        let node = self.m.get(&key).unwrap();

        let result = self.dll.delete(*node);
        self.dll.push_back(result.clone().unwrap()); // TODO: get rid of this Clone!!
        result.map(|(_, v)| v)
    }

    fn evict_node(&mut self) -> Option<T> {
        let opt_items = self.dll.pop_front();
        match opt_items {
            None => unreachable!("there were no items... strange"),
            Some((key, item)) => {
                self.m.remove(&key);
                Some(item)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert_and_find_in_cache() {
        let mut matecito = Matecito::<i32>::new(2);
        assert_eq!(MatecitoResult::Ok(123), matecito.put(123, 123));
        assert_eq!(MatecitoResult::Ok(456), matecito.put(456, 456));

        assert_eq!(Some(456), matecito.get(456));
        assert_eq!(Some(123), matecito.get(123));

        assert_eq!(None, matecito.get(01010));

        assert_eq!(MatecitoResult::Ok(789), matecito.put(789, 789_000));
        assert_eq!(Some(789_000), matecito.get(789));
        // 456 is gone, since the cache is full
        assert_eq!(None, matecito.get(456));

        assert_eq!(MatecitoResult::Ok(456), matecito.put(456, 456));
        assert_eq!(None, matecito.get(456));

        // 123 is gone, since the cache is full
        assert_eq!(None, matecito.get(123));
    }
}
