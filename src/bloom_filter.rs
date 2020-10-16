use std::hash::Hasher;
use twox_hash::XxHash64;

pub(crate) struct BloomFilter {
    hash_functions: Vec<XxHash64>,
    counters: Vec<u64>,
    array_size: u64,
    // approximate value
    max_count: u64,
}

impl BloomFilter {
    pub fn new(array_size: u64) -> Self {
        let hash_functions = Vec::new();
        let counters = Vec::with_capacity(array_size as usize);
        // Generate independent
        Self {
            hash_functions,
            counters,
            array_size,
            max_count: 0,
        }
    }

    fn get_indices<K: std::hash::Hash>(&self, key: &K) -> Vec<u64> {
        self.hash_functions
            .iter()
            .map(|h| {
                let mut hh = h.clone();
                key.hash(&mut hh);
                h.finish() % self.array_size
            })
            .collect()
    }

    pub fn increment<K: std::hash::Hash>(&mut self, key: K) -> () {
        let indices = self.get_indices(&key);
        indices.iter().for_each(|idx| {
            self.counters[*idx as usize] += 1;
            if self.counters[*idx as usize] > self.max_count {
                self.max_count = self.counters[*idx as usize];
            }
        })
    }

    pub fn decrement<K: std::hash::Hash>(&mut self, key: K) -> () {
        let indices = self.get_indices(&key);
        indices
            .iter()
            .for_each(|idx| self.counters[*idx as usize] -= 1)
    }

    fn halve(&mut self) {
        for el in self.counters.iter_mut() {
            *el = *el / 2;
        }
    }
}
