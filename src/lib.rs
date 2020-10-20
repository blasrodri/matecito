pub(crate) mod bloom_filter;
pub(crate) mod cache;
pub(crate) mod matecito_internal;

use std::sync::Arc;

/// Matecito is an experimental concurrent cache. Its main purpose is to
/// give a thread safe interface to use an in-memory storage for some
/// expensive computations.
///
/// # Example
///
/// ```no_run
/// use matecito::Matecito;
///
/// // Initialize the cache with space for 1024 objects.
/// let m = Matecito::<u64, String>::new(2usize.pow(10));
/// let m1 = m.clone();
/// std::thread::spawn(move || {
///     m1.put(123, "asd".to_string());
///     m1.put(01010101, "321".to_string());
/// });
/// // We need to give the cache a sec to populate the values.
/// std::thread::sleep(std::time::Duration::from_millis(1));   
/// assert_eq!(Some("asd".to_string()), m.get(123));
///
/// ```

pub struct Matecito<K, T>(Arc<cache::Cache<K, T>>);

impl<K: Clone + Ord + std::hash::Hash, T: std::fmt::Debug + Clone> Matecito<K, T> {
    // num_elements should be a power of two
    pub fn new(num_elements: usize) -> Self {
        let put_threshold = 1000;
        Self(Arc::new(cache::Cache::new(num_elements, put_threshold)))
    }

    pub fn put(&self, key: K, value: T) {
        self.0.put(key, value)
    }

    pub fn get(&self, key: K) -> Option<T> {
        self.0.get(key)
    }
}

impl<K, T> Clone for Matecito<K, T> {
    fn clone(&self) -> Self {
        let cache = self.0.clone();
        Self(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simple() {
        let m = Matecito::<u64, String>::new(2usize.pow(10));
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
