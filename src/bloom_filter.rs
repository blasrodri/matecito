use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

pub(crate) struct BloomFilter {
    hash_functions: Vec<XxHash64>,
    counters: Vec<u64>,
    array_size: u64,
    // approximate value
    pub max_count: u64,
}

impl BloomFilter {
    pub fn new(num_elements: u64, array_size: u64) -> Self {
        let num_hash_functions =
            BloomFilter::optimal_hash_functions(num_elements, array_size).round();
        let mut hash_functions = Vec::new();
        for i in 0..(num_hash_functions as usize) {
            hash_functions.push(XxHash64::with_seed(i as u64))
        }
        let counters = vec![0; array_size as usize];
        // Generate independent
        Self {
            hash_functions,
            counters,
            array_size,
            max_count: 0,
        }
    }

    #[inline]
    fn optimal_hash_functions(_n: u64, _m: u64) -> f64 {
        7f64 // TODO: find out what's the best way to define this for counteer bloom filters.
    }

    fn get_indices<K: std::hash::Hash>(&self, key: &K) -> Vec<u64> {
        self.hash_functions
            .iter()
            .map(|h| {
                let mut hh = h.clone();
                key.hash(&mut hh);
                hh.finish() % self.array_size
            })
            .collect()
    }

    pub fn count_present<K: Hash>(&self, key: &K) -> u64 {
        let indices = self.get_indices(key);
        indices
            .iter()
            .map(|idx| {
                let idx = *idx as usize;
                self.counters[idx]
            })
            .min()
            .unwrap()
    }

    pub fn increment<K: Hash>(&mut self, key: &K) -> () {
        let indices = self.get_indices(key);
        indices.iter().for_each(|idx| {
            self.counters[*idx as usize] += 1;
            if self.counters[*idx as usize] > self.max_count {
                self.max_count = self.counters[*idx as usize];
            }
        })
    }

    pub fn decrement<K: Hash>(&mut self, key: K) -> () {
        let indices = self.get_indices(&key);
        indices
            .iter()
            .for_each(|idx| self.counters[*idx as usize] -= 1)
    }

    pub fn halve(&mut self) {
        for el in self.counters.iter_mut() {
            *el = *el / 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_bloom_filter() {
        BloomFilter::new(10_000, 1000);
    }

    #[test]
    fn init_bloom_filter_insert_and_check_presence() {
        let mut bf = BloomFilter::new(10_000, 100);
        let key = b"asd";
        bf.increment(key);
        assert_eq!(0, bf.count_present(&b"123"));
        assert_eq!(1, bf.count_present(&b"asd"));
    }

    #[test]
    fn init_bloom_filter_insert_decrement_and_check_presence() {
        let mut bf = BloomFilter::new(10_000, 100);
        let key = b"asd";
        bf.increment(key);
        assert_eq!(0, bf.count_present(&b"123"));
        assert_eq!(1, bf.count_present(&b"asd"));

        bf.decrement(key);
        assert_eq!(0, bf.count_present(&b"asd"));
        assert_eq!(0, bf.count_present(&b"123"));
    }

    #[test]
    fn init_bloom_increment_ten_times_then_halve() {
        let mut bf = BloomFilter::new(10_000, 100);
        let key = b"asd";

        for _ in 0..10 {
            bf.increment(key);
        }
        bf.halve();
        assert_eq!(5, bf.count_present(&b"asd"));
        assert_eq!(0, bf.count_present(&b"123"));
        bf.halve();
        assert_eq!(2, bf.count_present(&b"asd"));
        bf.halve();
        assert_eq!(1, bf.count_present(&b"asd"));
        bf.halve();
        assert_eq!(0, bf.count_present(&b"asd"));
    }
}
