use matecito_dll::{DoublyLinkedList, Node};

use std::collections::HashMap;
use std::ptr::NonNull;

use twox_hash::RandomXxHashBuilder64;

pub(crate) type NonNullNode<T> = NonNull<Node<T>>;

pub(crate) struct MatecitoInternal<K, T> {
    m: HashMap<K, (T, NonNullNode<K>), RandomXxHashBuilder64>,
    dll: DoublyLinkedList<K>,
    max_size: usize, // threshold on the amount of elements we can store
}

impl<'a, K: Ord + Clone + std::hash::Hash, T: std::fmt::Debug> MatecitoInternal<K, T> {
    pub(crate) fn new(max_size: usize) -> Self {
        let m: HashMap<_, _, RandomXxHashBuilder64> = Default::default();
        Self {
            m,
            dll: DoublyLinkedList::new(),
            max_size,
        }
    }

    pub(crate) fn put(&mut self, key: &K, value: T) -> Option<K> {
        let mut result = None;
        if self.max_size == self.dll.num_elements() {
            result = self.evict_node();
        }

        let node = self.dll.push_back(key.clone());

        self.m.insert(key.clone(), (value, node));
        result
    }

    pub(crate) fn get(&mut self, key: K) -> Option<&T> {
        if self.m.get(&key).is_none() {
            return None;
        }
        // We've confirmed that there is such key... so we can unwrap.
        let (value, node) = self.m.get(&key).unwrap();

        let result = self.dll.delete(*node);
        self.dll.push_back(result.clone().unwrap());
        Some(value)
    }

    fn evict_node(&mut self) -> Option<K> {
        let opt_items = self.dll.pop_front();
        match opt_items {
            None => unreachable!("there were no items... strange"),
            Some(key) => {
                let (_, _) = self.m.remove(&key).unwrap();
                Some(key)
            }
        }
    }
    pub(crate) fn is_full(&self) -> bool {
        self.max_size == self.dll.num_elements()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert_and_find_in_cache() {
        let mut matecito = MatecitoInternal::<u64, i32>::new(2);
        matecito.put(&123, 123);
        matecito.put(&456, 456);

        assert_eq!(Some(&456), matecito.get(456));
        assert_eq!(Some(&123), matecito.get(123));

        assert_eq!(None, matecito.get(01010));

        matecito.put(&789, 789_000);
        assert_eq!(Some(&789_000), matecito.get(789));
        // 456 is gone, since the cache is full
        assert_eq!(None, matecito.get(456));

        matecito.put(&456, 456);
        assert_eq!(Some(&456), matecito.get(456));

        // 123 is gone, since the cache is full
        assert_eq!(None, matecito.get(123));
    }
}
