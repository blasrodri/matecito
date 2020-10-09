use crate::errors::MatecitoResult;
use crate::matecito_internal::MatecitoInternal;
use parking_lot::Mutex;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;

const NUM_SHARDS: usize = 256;
pub(crate) struct Cache<K, T> {
    hash_builder: twox_hash::RandomXxHashBuilder64,
    sharded_matecitos: Arc<Vec<Arc<Mutex<MatecitoInternal<K, T>>>>>,
}

impl<K: Clone + std::hash::Hash + Ord, T: std::fmt::Debug + Clone> Cache<K, T> {
    pub fn new(max_size: usize) -> Self {
        let mut v = Vec::with_capacity(NUM_SHARDS);
        let hash_builder = twox_hash::RandomXxHashBuilder64::default();
        for _ in 0..NUM_SHARDS {
            v.push(Arc::new(Mutex::new(MatecitoInternal::<K, T>::new(
                max_size / NUM_SHARDS,
            ))))
        }
        let sharded_matecitos = Arc::new(v);
        Self {
            hash_builder,
            sharded_matecitos,
        }
    }

    pub(crate) fn put(&self, key: K, value: T) -> MatecitoResult<K> {
        // TODO: Check whether it makes sense to put it or not... smart stats missing.
        let slot = self.get_shard(key.clone());
        let mut matecito = (*self.sharded_matecitos)[slot].lock();
        matecito.put(key, value)
    }

    pub(crate) fn get(&self, key: K) -> Option<T> {
        // TODO: Bloom filter missing.
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
        Self {
            sharded_matecitos,
            hash_builder,
        }
    }
}

unsafe impl<K, T> Send for Cache<K, T> {}
unsafe impl<K, T> Sync for Cache<K, T> {}
