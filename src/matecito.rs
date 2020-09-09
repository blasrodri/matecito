use crate::errors::MatecitoResult;

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

use serde::Serialize;
use twox_hash::{RandomXxHashBuilder64, XxHash64};

pub struct Matecito {
    m: HashMap<u64, (Vec<u8>, usize), RandomXxHashBuilder64>,
    lru: VecDeque<u64>,
    seed: u64,
    max_size: usize, // amount of bytes?
}

impl<'a> Matecito {
    pub fn new(seed: u64, max_size: usize) -> Self {
        let m: HashMap<_, _, RandomXxHashBuilder64> = Default::default();
        Self {
            m,
            lru: Default::default(),
            seed,
            max_size,
        }
    }

    pub fn put<K: Hash, V: Serialize>(&mut self, key: K, value: V) -> MatecitoResult<usize> {
        let value = bincode::serialize(&value).unwrap();
        let mut s = new_hasher(self.seed);
        key.hash(&mut s);
        let key = s.finish();
        let former_idx = self.m.get(&key).map(|x| x.1);
        match self.insert_in_lru(former_idx, key) {
            MatecitoResult::Ok(inserted_at) => {
                self.m.insert(key, (value, inserted_at));
                MatecitoResult::Ok(inserted_at)
            }
            err => err,
        }
    }

    pub fn get<T: Hash>(&self, key: &'a T) -> Option<&[u8]> {
        let mut s = new_hasher(self.seed);
        key.hash(&mut s);
        let key = s.finish();
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

fn new_hasher(seed: u64) -> impl Hasher {
    XxHash64::with_seed(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert_and_find_in_cache() {
        let mut matecito = Matecito::new(111, 2);
        assert_eq!(MatecitoResult::Ok(0 as usize), matecito.put(&123, b"123"));
        assert_eq!(MatecitoResult::Ok(0 as usize), matecito.put(&456, b"456"));

        assert_eq!(Some(&b"456"[..]), matecito.get(&456));
        assert_eq!(Some(&b"123"[..]), matecito.get(&123));
    }
}
