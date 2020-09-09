use crate::errors::MatecitoResult;

use std::collections::{HashMap, VecDeque};

use serde::Serialize;
use twox_hash::RandomXxHashBuilder64;

pub struct Matecito {
    m: HashMap<u64, (Vec<u8>, usize), RandomXxHashBuilder64>,
    lru: VecDeque<u64>,
    max_size: usize, // amount of bytes?
}

impl<'a> Matecito {
    pub fn new(max_size: usize) -> Self {
        let m: HashMap<_, _, RandomXxHashBuilder64> = Default::default();
        Self {
            m,
            lru: Default::default(),
            max_size,
        }
    }

    pub fn put<V: Serialize>(&mut self, key: u64, value: V) -> MatecitoResult<usize> {
        let value = bincode::serialize(&value).unwrap();
        let former_idx = self.m.get(&key).map(|x| x.1);
        match self.insert_in_lru(former_idx, key) {
            MatecitoResult::Ok(inserted_at) => {
                self.m.insert(key, (value, inserted_at));
                MatecitoResult::Ok(inserted_at)
            }
            err => err,
        }
    }

    pub fn get(&self, key: u64) -> Option<&[u8]> {
        self.m.get(&key).map(|x| x.0.as_slice())
    }

    fn insert_in_lru(&mut self, former_idx: Option<usize>, key: u64) -> MatecitoResult<usize> {
        match former_idx {
            None => (),
            Some(idx) => {
                self.lru.remove(idx);
            }
        }
        if self.lru.len() == self.max_size {
            let key = self.lru.pop_front().unwrap();
            self.m.remove(&key).unwrap();
        }
        self.lru.push_back(key);
        MatecitoResult::Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert_and_find_in_cache() {
        let mut matecito = Matecito::new(2);
        assert_eq!(MatecitoResult::Ok(0 as usize), matecito.put(123, b"123"));
        assert_eq!(MatecitoResult::Ok(0 as usize), matecito.put(456, b"456"));

        assert_eq!(Some(&b"456"[..]), matecito.get(456));
        assert_eq!(Some(&b"123"[..]), matecito.get(123));

        assert_eq!(None, matecito.get(01010));

        assert_eq!(MatecitoResult::Ok(0 as usize), matecito.put(789, b"789"));
        assert_eq!(Some(&b"789"[..]), matecito.get(789));
        // 123 is gone, since the cache is full
        assert_eq!(None, matecito.get(123));
    }
}
