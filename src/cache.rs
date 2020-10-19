use crate::bloom_filter::BloomFilter;
use crate::matecito_internal::MatecitoInternal;

use parking_lot::Mutex;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;

const NUM_SHARDS: usize = 256;
pub(crate) struct Cache<K, T> {
    hash_builder: twox_hash::RandomXxHashBuilder64,
    sharded_matecitos: Arc<Vec<Arc<Mutex<MatecitoInternal<K, T>>>>>,
    bloom_filter: Arc<Mutex<BloomFilter>>,
    put_threshold: usize,
}

impl<K: Clone + std::hash::Hash + Ord, T: std::fmt::Debug + Clone> Cache<K, T> {
    pub fn new(max_size: usize, put_threshold: usize) -> Self {
        let mut v = Vec::with_capacity(NUM_SHARDS);
        let hash_builder = twox_hash::RandomXxHashBuilder64::default();
        for _ in 0..NUM_SHARDS {
            v.push(Arc::new(Mutex::new(MatecitoInternal::<K, T>::new(
                max_size / NUM_SHARDS,
            ))))
        }
        let sharded_matecitos = Arc::new(v);
        let bloom_filter = BloomFilter::new(max_size as u64, 1000);
        Self {
            hash_builder,
            sharded_matecitos,
            bloom_filter: Arc::new(Mutex::new(bloom_filter)),
            put_threshold,
        }
    }

    pub(crate) fn put(&self, key: K, value: T) {
        // TODO: Check whether it makes sense to put it or not... smart stats missing.
        let (count, max_count) = {
            let mut bloom_filter = (*self.bloom_filter).lock();
            bloom_filter.increment(&key);
            (bloom_filter.count_present(&key), bloom_filter.max_count)
        };
        if count > self.put_threshold as u64 {
            let slot = self.get_shard(key.clone());
            let mut matecito = (*self.sharded_matecitos)[slot].lock();
            if let Some(another_key) = matecito.put(&key, value) {
                let mut bloom_filter = (*self.bloom_filter).lock();
                bloom_filter.decrement(another_key);
            }
            if max_count > (self.put_threshold * 10) as u64 {
                let mut bloom_filter = (*self.bloom_filter).lock();
                bloom_filter.halve();
            }
        }
    }

    pub(crate) fn get(&self, key: K) -> Option<T> {
        let mut bloom_filter = (*self.bloom_filter).lock();
        bloom_filter.increment(&key);

        let slot = self.get_shard(key.clone());
        let mut matecito = (*self.sharded_matecitos)[slot].lock();
        matecito.get(key).cloned()
    }

    #[inline]
    fn get_shard(&self, key: K) -> usize {
        let mut state = self.hash_builder.build_hasher();
        key.hash(&mut state);
        state.finish() as usize % NUM_SHARDS
    }
}

impl<K, T> Clone for Cache<K, T> {
    fn clone(&self) -> Self {
        let sharded_matecitos = self.sharded_matecitos.clone();
        let hash_builder = self.hash_builder.clone();
        let bloom_filter = self.bloom_filter.clone();
        let put_threshold = self.put_threshold;
        Self {
            sharded_matecitos,
            hash_builder,
            bloom_filter,
            put_threshold,
        }
    }
}

unsafe impl<K, T> Send for Cache<K, T> {}
unsafe impl<K, T> Sync for Cache<K, T> {}
