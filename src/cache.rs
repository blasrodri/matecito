use crate::errors::MatecitoResult;
use crate::matecito_internal::MatecitoInternal;
use parking_lot::Mutex;
use std::sync::Arc;

const NUM_SHARDS: usize = 256;
pub(crate) struct Cache<T> {
    sharded_matecitos: Arc<Vec<Arc<Mutex<MatecitoInternal<T>>>>>,
}

impl<T: std::fmt::Debug + Clone> Cache<T> {
    pub fn new(max_size: usize) -> Self {
        let mut v = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            v.push(Arc::new(Mutex::new(MatecitoInternal::<T>::new(
                max_size / NUM_SHARDS,
            ))))
        }
        let sharded_matecitos = Arc::new(v);
        Self { sharded_matecitos }
    }

    pub(crate) fn put(&self, key: u64, value: T) -> MatecitoResult<u64> {
        // TODO: Check whether it makes sense to put it or not... smart stats missing.
        let slot = self.get_shard(key);
        let mut matecito = (*self.sharded_matecitos)[slot].lock();
        matecito.put(key, value)
    }

    pub(crate) fn get(&self, key: u64) -> Option<T> {
        // TODO: Bloom filter missing.
        let slot = self.get_shard(key);
        let mut matecito = (*self.sharded_matecitos)[slot].lock();
        matecito.get(key).cloned()
    }

    #[inline]
    fn get_shard(&self, key: u64) -> usize {
        key as usize % NUM_SHARDS
    }
}

impl<T> Clone for Cache<T> {
    fn clone(&self) -> Self {
        let sharded_matecitos = self.sharded_matecitos.clone();
        Self { sharded_matecitos }
    }
}

unsafe impl<T> Send for Cache<T> {}
unsafe impl<T> Sync for Cache<T> {}
