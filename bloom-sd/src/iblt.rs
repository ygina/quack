use std::hash::BuildHasher;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use bloom::valuevec::ValueVec;
use crate::hashing::HashIter;

/// The image of the packet hash.
type Packet = u32;

/// TODO: Fatally, assumes inserted elements are unique.
/// Elements are u32s.
pub struct InvBloomLookupTable<R = RandomState, S = RandomState> {
    counters: ValueVec,
    xors: Vec<Packet>,
    num_entries: u64,
    num_hashes: u32,
    hash_builder_one: R,
    hash_builder_two: S,
}

impl InvBloomLookupTable<RandomState, RandomState> {
    /// Creates a InvBloomLookupTable that uses `bits_per_entry` bits for
    /// each entry and expects to hold `expected_num_items`. The filter
    /// will be sized to have a false positive rate of the value specified
    /// in `rate`.
    pub fn with_rate(
        bits_per_entry: usize,
        rate: f32,
        expected_num_items: u32,
    ) -> Self {
        // TODO: determine number of entries and hashes from IBLT paper
        let num_entries = bloom::bloom::needed_bits(rate, expected_num_items);
        let num_hashes = bloom::bloom::optimal_num_hashes(
            bits_per_entry,
            expected_num_items,
        );
        InvBloomLookupTable {
            xors: vec![0; num_entries],
            counters: ValueVec::new(bits_per_entry, num_entries),
            num_entries: num_entries as u64,
            num_hashes,
            hash_builder_one: RandomState::new(),
            hash_builder_two: RandomState::new(),
        }
    }

    /// Clones the InvBloomLookupTable where all counters are 0.
    pub fn empty_clone(&self) -> Self {
        let bits_per_entry = self.counters.bits_per_val();
        Self {
            xors: vec![0; self.num_entries as usize],
            counters: ValueVec::new(bits_per_entry, self.num_entries as usize),
            num_entries: self.num_entries,
            num_hashes: self.num_hashes,
            hash_builder_one: self.hash_builder_one.clone(),
            hash_builder_two: self.hash_builder_two.clone(),
        }
    }

    pub fn xors(&self) -> &Vec<u32> {
        &self.xors
    }

    pub fn xors_mut(&mut self) -> &mut Vec<u32> {
        &mut self.xors
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

impl<R,S> InvBloomLookupTable<R,S> where R: BuildHasher, S: BuildHasher {
    /// Inserts an item, returns true if the item was already in the filter
    /// any number of times.
    pub fn insert(&mut self, item: &Packet) -> bool {
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
                self.xors[idx] ^= item;
            } else {
                panic!("counting bloom filter counter overflow");
            }
        }
        min > 0
    }

    /// Removes an item, panics if the item does not exist.
    pub fn remove(&mut self, item: &Packet) {
        for h in HashIter::from(item,
                                self.num_hashes,
                                &self.hash_builder_one,
                                &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur == 0 {
                panic!("item is not in the iblt");
            }
            self.counters.set(idx, cur - 1);
            self.xors[idx] ^= item;
        }
    }

    /// Checks if the item has been inserted into this InvBloomLookupTable.
    /// This function can return false positives, but not false negatives.
    pub fn contains(&self, item: &Packet) -> bool {
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
    pub fn indexes(&self, item: &Packet) -> Vec<usize> {
        HashIter::from(item,
                       self.num_hashes,
                       &self.hash_builder_one,
                       &self.hash_builder_two)
            .into_iter()
            .map(|h| (h % self.num_entries) as usize)
            .collect()
    }

    /// Enumerates as many items as possible in the IBLT and removes them.
    /// Returns the removed items. Note removed elements must be unique
    /// unless the IBLT uses an accumulator function that is not an XOR.
    pub fn eliminate_elems(&mut self) -> HashSet<Packet> {
        // Loop through all the counters of the IBLT until there are no
        // remaining cells with count 1. This is O(num_counters*max_count).
        let mut removed_set: HashSet<Packet> = HashSet::new();
        loop {
            let mut removed = false;
            for i in 0..(self.num_entries as usize) {
                if self.counters.get(i) != 1 {
                    continue;
                }
                let item = self.xors[i];
                self.remove(&item);
                assert!(removed_set.insert(item));
                removed = true;
            }
            if !removed {
                return removed_set;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_iblt() -> InvBloomLookupTable {
        InvBloomLookupTable::with_rate(8, 0.01, 10)
    }

    fn vvsum(vec: &ValueVec) -> usize {
        let num_entries = vec.len() / vec.bits_per_val();
        (0..num_entries).map(|i| vec.get(i)).sum::<u32>() as usize
    }

    #[test]
    fn init_iblt_with_rate() {
        let iblt = init_iblt();
        assert_eq!(iblt.num_entries(), 96);
        assert_eq!(iblt.num_hashes(), 2);
        assert_eq!(vvsum(iblt.counters()), 0);
        assert_eq!(iblt.xors().iter().sum::<u32>(), 0);
        assert_eq!(iblt.xors().len(), iblt.num_entries() as usize);
    }

    #[test]
    fn test_insert() {
        let mut iblt = init_iblt();
        let elem = 1234;
        let indexes = iblt.indexes(&elem);
        for &idx in &indexes {
            assert_eq!(iblt.counters().get(idx), 0);
            assert_eq!(iblt.xors()[idx], 0);
        }
        assert!(!iblt.insert(&elem), "element did not exist already");
        assert_eq!(vvsum(iblt.counters()), 1 * iblt.num_hashes() as usize);
        for &idx in &indexes {
            assert_ne!(iblt.counters().get(idx), 0);
            assert_ne!(iblt.xors()[idx], 0);
        }
        assert!(iblt.insert(&elem), "added element twice: violates spec?");
        assert_eq!(vvsum(iblt.counters()), 2 * iblt.num_hashes() as usize);
        for &idx in &indexes {
            assert_ne!(iblt.counters().get(idx), 0);
            assert_eq!(iblt.xors()[idx], 0);
        }
    }

    #[test]
    fn test_empty_clone() {
        let mut iblt1 = init_iblt();
        iblt1.insert(&1234);
        iblt1.insert(&5678);
        let iblt2 = iblt1.empty_clone();
        assert!(vvsum(iblt1.counters()) > 0);
        assert_eq!(vvsum(iblt2.counters()), 0);
        assert!(iblt1.xors().iter().sum::<u32>() > 0);
        assert_eq!(iblt2.xors().iter().sum::<u32>(), 0);
        assert_eq!(iblt1.indexes(&1234), iblt2.indexes(&1234));
    }

    #[test]
    #[should_panic]
    fn counter_overflow() {
        let mut iblt = InvBloomLookupTable::with_rate(1, 0.01, 10);
        iblt.insert(&1234);
        iblt.insert(&1234);
    }
}
