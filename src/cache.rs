use crate::errors::MatecitoResult;
use crate::matecito::Matecito;
use parking_lot::Mutex;
use std::sync::Arc;

const NUM_SHARDS: usize = 256;
pub struct Cache<T> {
    sharded_matecitos: Arc<Vec<Arc<Mutex<Matecito<T>>>>>,
}

impl<T: std::fmt::Debug + Clone> Cache<T> {
    pub fn new(max_size: usize) -> Self {
        let mut v = Vec::with_capacity(NUM_SHARDS);
        for _ in 0..NUM_SHARDS {
            v.push(Arc::new(Mutex::new(Matecito::<T>::new(
                max_size / NUM_SHARDS,
            ))))
        }
        let sharded_matecitos = Arc::new(v);
        Self { sharded_matecitos }
    }

    pub fn put(&self, key: u64, value: T) -> MatecitoResult<u64> {
        // TODO: Check whether it makes sense to put it or not... smart stats missing.
        let slot = self.get_shard(key);
        let mut matecito = (*self.sharded_matecitos)[slot].lock();
        matecito.put(key, value)
    }

    pub fn get(&self, key: u64) -> Option<T> {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple() {
        let m = Arc::new(Cache::<String>::new(2usize.pow(10)));
        let m1 = m.clone();
        std::thread::spawn(move || {
            m1.put(123, "asd".to_string());
            m1.put(01010101, "321".to_string());
        });

        std::thread::sleep(std::time::Duration::from_millis(1));

        let m2 = m.clone();
        let result = std::thread::spawn(move || m2.get(123)).join();
        assert_eq!(Some("asd".to_string()), result.unwrap());

        let m3 = m.clone();
        let result = std::thread::spawn(move || m3.get(01010101)).join();
        assert_eq!(Some("321".to_string()), result.unwrap());

        let m4 = m.clone();
        let result = std::thread::spawn(move || m4.get(0xf00)).join();
        assert_eq!(None, result.unwrap());
    }
}
