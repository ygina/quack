use std::collections::HashSet;
use std::hash::Hasher;
use std::num::Wrapping;

use rand;
use serde::{Serialize, Deserialize};
use djb_hash::{HasherU32, x33a_u32::*};
use siphasher::sip128::SipHasher13;

use crate::valuevec::ValueVec;
use crate::hashing::HashIter;
use crate::SipHasher13Def;

const DJB_HASH_SIZE: usize = 32;

#[derive(Serialize, Deserialize)]
pub struct InvBloomLookupTable {
    counters: ValueVec,
    // sum of djb_hashed data with wraparound overflow
    data: ValueVec,
    num_entries: u64,
    num_hashes: u32,
    seed: u64,
    #[serde(with = "SipHasher13Def")]
    hash_builder_one: SipHasher13,
    #[serde(with = "SipHasher13Def")]
    hash_builder_two: SipHasher13,
}

/// Maps an element in the lookup table to a u32.
pub fn elem_to_u32(elem: &[u8]) -> u32 {
    let mut hasher = X33aU32::new();
    hasher.write(&elem);
    hasher.finish_u32()
}

impl InvBloomLookupTable {
    /// Creates a InvBloomLookupTable that uses `bits_per_entry` bits for each
    /// entry, `num_entries` number of entries, and `num_hashes` number of hash
    /// functions.
    ///
    /// The recommended parameters are 10x entries the number of expected items,
    /// and 2 hash functions. These were experimentally found to provide the
    /// best tradeoff between space and false positive rates (stating the
    /// router is malicious when it is not).
    pub fn new(
        bits_per_entry: usize,
        num_entries: usize,
        num_hashes: u32,
    ) -> Self {
        use rand::RngCore;
        let seed = rand::rngs::OsRng.next_u64();
        Self::new_with_seed(seed, bits_per_entry, num_entries, num_hashes)
    }

    /// Like `new()`, but seeds the hash builders.
    pub fn new_with_seed(
        seed: u64,
        bits_per_entry: usize,
        num_entries: usize,
        num_hashes: u32,
    ) -> Self {
        use rand::{SeedableRng, rngs::SmallRng, Rng};
        let mut rng = SmallRng::seed_from_u64(seed);
        InvBloomLookupTable {
            data: ValueVec::new(DJB_HASH_SIZE, num_entries),
            counters: ValueVec::new(bits_per_entry, num_entries),
            num_entries: num_entries as u64,
            num_hashes,
            seed,
            hash_builder_one: SipHasher13::new_with_keys(rng.gen(), rng.gen()),
            hash_builder_two: SipHasher13::new_with_keys(rng.gen(), rng.gen()),
        }
    }

    /// Clones the InvBloomLookupTable where all counters are 0.
    pub fn empty_clone(&self) -> Self {
        let bits_per_entry = self.counters.bits_per_val();
        Self {
            data: ValueVec::new(DJB_HASH_SIZE, self.num_entries as usize),
            counters: ValueVec::new(bits_per_entry, self.num_entries as usize),
            num_entries: self.num_entries,
            num_hashes: self.num_hashes,
            seed: self.seed,
            hash_builder_one: self.hash_builder_one.clone(),
            hash_builder_two: self.hash_builder_two.clone(),
        }
    }

    pub fn data(&self) -> &ValueVec {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ValueVec {
        &mut self.data
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

    pub fn equals(&self, other: &Self) -> bool {
        if self.num_entries != other.num_entries
            || self.num_hashes != other.num_hashes
            || self.hash_builder_one.keys() != other.hash_builder_one.keys()
            || self.hash_builder_two.keys() != other.hash_builder_two.keys()
            || self.data != other.data
        {
            return false;
        }
        let nbits = self.counters.len();
        if nbits != other.counters.len() {
            return false;
        }
        for i in 0..(nbits / self.counters.bits_per_val()) {
            if self.counters.get(i) != other.counters.get(i) {
                return false;
            }
        }
        true
    }

    /// Inserts an item, returns true if the item was already in the filter
    /// any number of times.
    pub fn insert(&mut self, item: &[u8]) -> bool {
        let mut min = u32::max_value();
        let item_u32 = elem_to_u32(item);
        for h in HashIter::from(item_u32,
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
                // TODO: write a test for wraparound
                self.counters.set(idx, 0);
            }
            self.data.set(
                idx, (Wrapping(self.data.get(idx)) + Wrapping(item_u32)).0);
        }
        min > 0
    }

    /// Removes an item, panics if the item does not exist.
    pub fn remove(&mut self, item: &[u8]) {
        let item_u32 = elem_to_u32(item);
        self.remove_u32(item_u32);
    }

    fn remove_u32(&mut self, item_u32: u32) {
        for h in HashIter::from(item_u32,
                            self.num_hashes,
                            &self.hash_builder_one,
                            &self.hash_builder_two) {
            let idx = (h % self.num_entries) as usize;
            let cur = self.counters.get(idx);
            if cur == 0 {
                // wraparound
                self.counters.set(idx, self.counters.max_value());
            } else {
                self.counters.set(idx, cur - 1);
            }
            self.data.set(
                idx, (Wrapping(self.data.get(idx)) - Wrapping(item_u32)).0);
        }
    }

    /// Checks if the item has been inserted into this InvBloomLookupTable.
    /// This function can return false positives, but not false negatives.
    pub fn contains(&self, item: &[u8]) -> bool {
        let item_u32 = elem_to_u32(item);
        for h in HashIter::from(item_u32,
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
    pub fn indexes(&self, item: &[u8]) -> Vec<usize> {
        let item_u32 = elem_to_u32(item);
        HashIter::from(item_u32,
                       self.num_hashes,
                       &self.hash_builder_one,
                       &self.hash_builder_two)
            .into_iter()
            .map(|h| (h % self.num_entries) as usize)
            .collect()
    }

    /// Enumerates as many items as possible in the IBLT and removes them.
    /// Returns the removed items. Note removed elements must be unique
    /// because the corresponding counters would be at least 2.
    /// The caller will need to map elements to u32.
    pub fn eliminate_elems(&mut self) -> HashSet<u32> {
        // Loop through all the counters of the IBLT until there are no
        // remaining cells with count 1. This is O(num_counters*max_count).
        let mut removed_set: HashSet<u32> = HashSet::new();
        loop {
            let mut removed = false;
            for i in 0..(self.num_entries as usize) {
                if self.counters.get(i) != 1 {
                    continue;
                }
                let item = self.data.get(i).clone();
                self.remove_u32(item);
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
    use bincode;

    fn init_iblt() -> InvBloomLookupTable {
        InvBloomLookupTable::new(8, 100, 2)
    }

    fn vvsum(vec: &ValueVec) -> usize {
        let num_entries = vec.len() / vec.bits_per_val();
        (0..num_entries).map(|i| vec.get(i)).sum::<u32>() as usize
    }

    fn data_is_nonzero(vec: &ValueVec) -> bool {
        let num_entries = vec.len() / vec.bits_per_val();
        for i in 0..num_entries {
            if vec.get(i) != 0 {
                return true;
            }
        }
        false
    }

    #[test]
    fn test_serialization_empty() {
        let iblt1 = init_iblt();
        let bytes = bincode::serialize(&iblt1).unwrap();
        let iblt2 = bincode::deserialize(&bytes).unwrap();
        assert!(iblt1.equals(&iblt2));
    }

    #[test]
    fn test_serialization_with_data() {
        let mut iblt1 = init_iblt();
        iblt1.insert(&1234_u32.to_be_bytes());
        let bytes = bincode::serialize(&iblt1).unwrap();
        let iblt2 = bincode::deserialize(&bytes).unwrap();
        assert!(iblt1.equals(&iblt2));
    }

    #[test]
    fn test_new_iblt() {
        let iblt = init_iblt();
        assert_eq!(iblt.num_entries(), 100);
        assert_eq!(iblt.num_hashes(), 2);
        assert_eq!(vvsum(iblt.counters()), 0);
        assert_eq!(vvsum(iblt.data()), 0);
    }

    #[test]
    fn test_new_iblt_with_seed() {
        let iblt1 = InvBloomLookupTable::new_with_seed(111, 8, 100, 2);
        let iblt2 = InvBloomLookupTable::new_with_seed(222, 8, 100, 2);
        let iblt3 = InvBloomLookupTable::new_with_seed(111, 8, 100, 2);
        assert!(!iblt1.equals(&iblt2));
        assert!(iblt1.equals(&iblt3));
    }

    #[test]
    fn test_equals() {
        let mut iblt1 = init_iblt();
        let iblt2 = init_iblt();
        assert!(!iblt1.equals(&iblt2), "different random state");
        let iblt3 = iblt1.empty_clone();
        assert!(iblt1.equals(&iblt3), "empty clone duplicates random state");
        iblt1.insert(&1234_u32.to_be_bytes());
        let iblt4 = iblt1.empty_clone();
        assert!(!iblt1.equals(&iblt4), "empty clone removes data");
        assert!(iblt1.equals(&iblt1), "reflexive equality");
        assert!(iblt2.equals(&iblt2), "reflexive equality");
    }

    #[test]
    fn test_insert_without_overflow() {
        let mut iblt = init_iblt();
        let elem = 1234_u32.to_be_bytes();
        let indexes = iblt.indexes(&elem);
        for &idx in &indexes {
            assert_eq!(iblt.counters().get(idx), 0);
            assert_eq!(iblt.data().get(idx), 0);
        }
        assert!(!iblt.insert(&elem), "element did not exist already");
        assert_eq!(vvsum(iblt.counters()), 1 * iblt.num_hashes() as usize);
        for &idx in &indexes {
            assert_ne!(iblt.counters().get(idx), 0);
            assert_ne!(iblt.data().get(idx), 0);
        }
        assert!(iblt.insert(&elem), "added element twice");
        assert_eq!(vvsum(iblt.counters()), 2 * iblt.num_hashes() as usize);
        for &idx in &indexes {
            assert_ne!(iblt.counters().get(idx), 0);
            assert_ne!(iblt.data().get(idx), 0);
        }
    }

    #[test]
    fn test_empty_clone() {
        let mut iblt1 = init_iblt();
        iblt1.insert(&1234_u32.to_be_bytes());
        iblt1.insert(&5678_u32.to_be_bytes());
        let iblt2 = iblt1.empty_clone();
        assert!(vvsum(iblt1.counters()) > 0);
        assert_eq!(vvsum(iblt2.counters()), 0);
        assert!(data_is_nonzero(iblt1.data()));
        assert_eq!(vvsum(iblt2.data()), 0);
        assert_eq!(
            iblt1.indexes(&1234_u32.to_be_bytes()),
            iblt2.indexes(&1234_u32.to_be_bytes()));
    }

    #[test]
    fn test_insert_with_counter_overflow() {
        let mut iblt = InvBloomLookupTable::new(1, 10, 1);  // 1 bit per entry
        let elem = 1234_u64.to_be_bytes();
        let elem_u32 = elem_to_u32(&elem);
        let i = iblt.indexes(&elem)[0];

        // counters and data are updated
        iblt.insert(&elem);
        assert_eq!(iblt.counters().get(i), 1);
        assert_eq!(iblt.data().get(i), elem_u32);

        // on overflow, counter is zero but data is nonzero
        iblt.insert(&elem);
        assert_eq!(iblt.counters().get(i), 0);
        assert_eq!(iblt.data().get(i), elem_u32 * 2);
    }

    #[test]
    fn test_insert_with_data_wraparound() {
        let mut iblt = InvBloomLookupTable::new(2, 10, 1);
        let elem = 9983_u32.to_be_bytes();
        let elem_u32 = elem_to_u32(&elem);
        assert_eq!(elem_u32, 2086475114, "DJB hash of 9983 is very big");
        let i = iblt.indexes(&elem)[0];

        // counters and data are updated
        iblt.insert(&elem);
        assert_eq!(iblt.counters().get(i), 1);
        assert_eq!(iblt.data().get(i), elem_u32);

        // on overflow, counter is zero but data is nonzero
        iblt.insert(&elem);
        iblt.insert(&elem);
        assert_eq!(iblt.counters().get(i), 3);
        assert!(iblt.data().get(i) < elem_u32);
    }

    #[test]
    fn test_eliminate_all_elems_without_duplicates() {
        let mut iblt = InvBloomLookupTable::new_with_seed(111, 8, 10, 2);
        let mut hashes = HashSet::new();
        let n: usize = 6;
        for i in 0..n {
            let elem = (i as u32).to_be_bytes();
            iblt.insert(&elem);
            hashes.insert(elem_to_u32(&elem));
        }
        assert_eq!(vvsum(iblt.counters()), n * (iblt.num_hashes() as usize));
        assert_eq!(hashes.len(), n, "djb hashes are unique in this test");

        // Return the original elements
        let elems = iblt.eliminate_elems();
        assert_eq!(elems.len(), n);
        assert_eq!(vvsum(iblt.counters()), 0);
        assert_eq!(vvsum(iblt.data()), 0);
        for i in 0..n {
            let elem = elem_to_u32(&(i as u32).to_be_bytes());
            assert!(hashes.remove(&elem));
        }
    }

    #[test]
    fn test_eliminate_all_elems_with_duplicates() {
        let mut iblt = InvBloomLookupTable::new_with_seed(111, 8, 10, 2);
        let mut hashes = HashSet::new();
        let n: usize = 8;
        for i in 0..n {
            let elem = (i as u32).to_be_bytes();
            iblt.insert(&elem);
            hashes.insert(elem_to_u32(&elem));
        }
        assert_eq!(vvsum(iblt.counters()), n * (iblt.num_hashes() as usize));
        assert_eq!(hashes.len(), n, "djb hashes are unique in this test");

        // Not all elements were eliminated
        let elems = iblt.eliminate_elems();
        assert!(elems.len() < n);
        assert_eq!(vvsum(iblt.counters()),
            (n - elems.len()) * (iblt.num_hashes() as usize));

        // Test that the sums were updated correctly?
        assert!(data_is_nonzero(iblt.data()));
    }
}

