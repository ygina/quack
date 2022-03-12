//! Based on https://docs.rs/bloom/0.3.2/bloom/counting/struct.CountingBloomFilter.html
//! except with properties specific to the CBF accumulator of the subset digest.
use std::hash::{BuildHasher, Hash};
use std::collections::hash_map::RandomState;
use bloom::valuevec::ValueVec;
use crate::hashing::HashIter;

pub struct CountingBloomFilter<R = RandomState, S = RandomState> {
    counters: ValueVec,
    num_entries: u64,
    num_hashes: u32,
    hash_builder_one: R,
    hash_builder_two: S,
}

impl CountingBloomFilter<RandomState, RandomState> {
    /// Creates a CountingBloomFilter that uses `bits_per_entry` bits for
    /// each entry and expects to hold `expected_num_items`. The filter
    /// will be sized to have a false positive rate of the value specified
    /// in `rate`.
    pub fn with_rate(
        bits_per_entry: usize,
        rate: f32,
        expected_num_items: u32,
    ) -> Self {
        let num_entries = bloom::bloom::needed_bits(rate, expected_num_items);
        let num_hashes = bloom::bloom::optimal_num_hashes(
            bits_per_entry,
            expected_num_items,
        );
        CountingBloomFilter {
            counters: ValueVec::new(bits_per_entry, num_entries),
            num_entries: num_entries as u64,
            num_hashes,
            hash_builder_one: RandomState::new(),
            hash_builder_two: RandomState::new(),
        }
    }

    /// Clones the CountingBloomFilter where all counters are 0.
    pub fn empty_clone(&self) -> Self {
        let bits_per_entry = self.counters.bits_per_val();
        Self {
            counters: ValueVec::new(bits_per_entry, self.num_entries as usize),
            num_entries: self.num_entries,
            num_hashes: self.num_hashes,
            hash_builder_one: self.hash_builder_one.clone(),
            hash_builder_two: self.hash_builder_two.clone(),
        }
    }

    pub fn counters(&self) -> &ValueVec {
        &self.counters
    }

    pub fn counters_mut(&mut self) -> &mut ValueVec {
        &mut self.counters
    }

    pub fn num_entries(&self) -> u64 {
        self.num_entries
    }

    pub fn num_hashes(&self) -> u32 {
        self.num_hashes
    }
}

impl<R,S> CountingBloomFilter<R,S> where R: BuildHasher, S: BuildHasher {
    /// Inserts an item, returns true if the item was already in the filter
    /// any number of times.
    pub fn insert<T: Hash>(&mut self, item: &T) -> bool {
        let mut min = u32::max_value();
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur < min {
                min = cur;
            }
            if cur < self.counters.max_value() {
                self.counters.set(idx, cur + 1);
            } else {
                panic!("counting bloom filter counter overflow");
            }
        }
        min > 0
    }

    /// Checks if the item has been inserted into this CountingBloomFilter.
    /// This function can return false positives, but not false negatives.
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur == 0 {
                return false;
            }
        }
        true
    }

    /// Gets the indexes of the item in the vector.
    pub fn indexes<T: Hash>(&self, item: &T) -> Vec<usize> {
        HashIter::from(item,
                       self.num_hashes,
                       &self.hash_builder_one,
                       &self.hash_builder_two)
            .into_iter()
            .map(|h| (h % self.num_entries) as usize)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_cbf() -> CountingBloomFilter {
        CountingBloomFilter::with_rate(8, 0.01, 10)
    }

    fn vvsum(vec: &ValueVec) -> usize {
        let num_entries = vec.len() / vec.bits_per_val();
        (0..num_entries).map(|i| vec.get(i)).sum::<u32>() as usize
    }

    #[test]
    fn init_cbf_with_rate() {
        let cbf = init_cbf();
        assert_eq!(cbf.num_entries(), 96);
        assert_eq!(cbf.num_hashes(), 2);
        assert_eq!(vvsum(cbf.counters()), 0);
    }

    #[test]
    fn init_cbf_random_state() {
        let cbf1 = init_cbf();
        let cbf2 = init_cbf();
        assert_ne!(cbf1.indexes(&1234), cbf2.indexes(&1234));
    }

    #[test]
    fn test_insert() {
        let mut cbf = init_cbf();
        assert!(!cbf.insert(&1234), "element did not exist already");
        assert_eq!(vvsum(cbf.counters()), 1 * cbf.num_hashes() as usize);
        assert!(cbf.insert(&1234));
        assert!(cbf.insert(&1234));
        assert_eq!(vvsum(cbf.counters()), 3 * cbf.num_hashes() as usize);
        assert!(!cbf.insert(&5678));
        assert_eq!(vvsum(cbf.counters()), 4 * cbf.num_hashes() as usize);
    }

    #[test]
    fn test_contains() {
        let mut cbf = init_cbf();
        cbf.insert(&1234);
        cbf.insert(&1234);
        cbf.insert(&1234);
        cbf.insert(&5678);
        assert!(cbf.contains(&1234));
        assert!(cbf.contains(&5678));
        assert!(!cbf.contains(&3456)); // with high probability
    }

    #[test]
    fn test_indexes() {
        let mut cbf = init_cbf();
        cbf.insert(&1234);
        cbf.insert(&1234);
        cbf.insert(&1234);
        cbf.insert(&5678);
        let indexes = cbf.indexes(&1234);
        assert_eq!(indexes.len(), cbf.num_hashes() as usize);
        assert_eq!(indexes, cbf.indexes(&1234), "indexes are deterministic");
        for i in indexes {
            assert!(cbf.counters().get(i) >= 3);
        }
        for i in cbf.indexes(&5678) {
            assert!(cbf.counters().get(i) >= 1);
        }
    }

    #[test]
    fn test_empty_clone() {
        let mut cbf1 = init_cbf();
        cbf1.insert(&1234);
        cbf1.insert(&5678);
        let cbf2 = cbf1.empty_clone();
        assert!(vvsum(cbf1.counters()) > 0);
        assert_eq!(vvsum(cbf2.counters()), 0);
        assert_eq!(cbf1.indexes(&1234), cbf2.indexes(&1234));
    }

    #[test]
    #[should_panic]
    fn counter_overflow() {
        let mut cbf = CountingBloomFilter::with_rate(1, 0.01, 10);
        cbf.insert(&1234);
        cbf.insert(&1234);
    }
}
