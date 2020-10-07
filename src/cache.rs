use parking_lot::RwLock;

use crate::errors::MatecitoResult;
use crate::matecito::Matecito;

const NUM_SHARDS: usize = 256;
pub struct Cache<T> {
    sharded_matecitos: Vec<RwLock<Matecito<T>>>,
}

impl<T: std::fmt::Debug> Cache<T> {
    pub fn new(max_size: usize) -> Self {
        let mut sharded_matecitos = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            sharded_matecitos.push(RwLock::new(Matecito::new(max_size / NUM_SHARDS)))
        }
        Self { sharded_matecitos }
    }

    pub fn put(&mut self, key: u64, value: T) -> MatecitoResult<u64> {
        // TODO: Check whether it makes sense to put it or not... smart stats missing.
        let slot = self.get_shard(key);
        let matecito = self.sharded_matecitos[slot].get_mut();
        matecito.put(key, value)
    }

    pub fn get(&mut self, key: u64) -> Option<&T> {
        // TODO: Bloom filter missing.
        let slot = self.get_shard(key);
        let matecito = self.sharded_matecitos[slot].get_mut();
        matecito.get(key)
    }

    #[inline]
    fn get_shard(&self, key: u64) -> usize {
        key as usize % NUM_SHARDS
    }
}
