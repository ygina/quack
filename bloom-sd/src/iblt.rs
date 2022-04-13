use std::collections::HashSet;
use std::hash::Hasher;
use std::num::Wrapping;

use rand;
use rand::Rng;
use num_bigint::BigUint;
use serde::{Serialize, Deserialize};
use djb_hash::{HasherU32, x33a_u32::*};
use siphasher::sip128::SipHasher13;

use crate::valuevec::ValueVec;
use crate::hashing::HashIter;
use crate::SipHasher13Def;

#[derive(Serialize, Deserialize)]
pub struct InvBloomLookupTable {
    counters: ValueVec,
    // sum of djb_hashed data with wraparound overflow
    data: Vec<u32>,
    num_entries: u64,
    num_hashes: u32,
    #[serde(with = "SipHasher13Def")]
    hash_builder_one: SipHasher13,
    #[serde(with = "SipHasher13Def")]
    hash_builder_two: SipHasher13,
}

/// Maps an element in the lookup table to a u32.
pub fn elem_to_u32(elem: &BigUint) -> u32 {
    let mut hasher = X33aU32::new();
    hasher.write(&elem.to_bytes_be());
    hasher.finish_u32()
}

impl InvBloomLookupTable {
    /// Creates a InvBloomLookupTable that uses `bits_per_entry` bits for
    /// each entry and expects to hold `expected_num_items`. The filter
    /// will be sized to have a false positive rate of the value specified
    /// in `rate`.
    pub fn with_rate(
        bits_per_entry: usize,
        _rate: f32,
        expected_num_items: u32,
    ) -> Self {
        // TODO: determine number of entries and hashes from IBLT paper
        // let num_entries = bloom::bloom::needed_bits(rate, expected_num_items);
        // let num_hashes = bloom::bloom::optimal_num_hashes(
        //     bits_per_entry,
        //     expected_num_items,
        // );
        let num_hashes = 2;
        // 4 is a multiplier that is experimentally found to reduce the number
        // of false positives (stating the router is malicious when it is not)
        let num_entries = 4 * expected_num_items as usize;
        let mut rng = rand::thread_rng();
        InvBloomLookupTable {
            data: vec![0; num_entries],
            counters: ValueVec::new(bits_per_entry, num_entries),
            num_entries: num_entries as u64,
            num_hashes,
            hash_builder_one: SipHasher13::new_with_keys(rng.gen(), rng.gen()),
            hash_builder_two: SipHasher13::new_with_keys(rng.gen(), rng.gen()),
        }
    }

    /// Clones the InvBloomLookupTable where all counters are 0.
    pub fn empty_clone(&self) -> Self {
        let bits_per_entry = self.counters.bits_per_val();
        Self {
            data: vec![0; self.num_entries as usize],
            counters: ValueVec::new(bits_per_entry, self.num_entries as usize),
            num_entries: self.num_entries,
            num_hashes: self.num_hashes,
            hash_builder_one: self.hash_builder_one.clone(),
            hash_builder_two: self.hash_builder_two.clone(),
        }
    }

    pub fn data(&self) -> &Vec<u32> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u32> {
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
    pub fn insert(&mut self, item: &BigUint) -> bool {
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
            self.data[idx] = (Wrapping(self.data[idx]) + Wrapping(item_u32)).0;
        }
        min > 0
    }

    /// Removes an item, panics if the item does not exist.
    pub fn remove(&mut self, item: &BigUint) {
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
            self.data[idx] = (Wrapping(self.data[idx]) - Wrapping(item_u32)).0;
        }
    }

    /// Checks if the item has been inserted into this InvBloomLookupTable.
    /// This function can return false positives, but not false negatives.
    pub fn contains(&self, item: &BigUint) -> bool {
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
    pub fn indexes(&self, item: &BigUint) -> Vec<usize> {
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
    /// unless the IBLT uses an accumulator function that is not an XOR.
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
                let item = self.data[i].clone();
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
    use num_bigint::ToBigUint;
    use num_traits::Zero;

    fn init_iblt() -> InvBloomLookupTable {
        InvBloomLookupTable::with_rate(8, 0.01, 10)
    }

    fn vvsum(vec: &ValueVec) -> usize {
        let num_entries = vec.len() / vec.bits_per_val();
        (0..num_entries).map(|i| vec.get(i)).sum::<u32>() as usize
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
        iblt1.insert(&1234_u32.to_biguint().unwrap());
        let bytes = bincode::serialize(&iblt1).unwrap();
        let iblt2 = bincode::deserialize(&bytes).unwrap();
        assert!(iblt1.equals(&iblt2));
    }

    #[test]
    fn init_iblt_with_rate() {
        let iblt = init_iblt();
        assert_eq!(iblt.num_entries(), 10*4);
        assert_eq!(iblt.num_hashes(), 2);
        assert_eq!(vvsum(iblt.counters()), 0);
        assert_eq!(iblt.data().iter().sum::<BigUint>(), BigUint::zero());
        assert_eq!(iblt.data().len(), iblt.num_entries() as usize);
    }

    #[test]
    fn test_equals() {
        let mut iblt1 = init_iblt();
        let iblt2 = init_iblt();
        assert!(!iblt1.equals(&iblt2), "different random state");
        let iblt3 = iblt1.empty_clone();
        assert!(iblt1.equals(&iblt3), "empty clone duplicates random state");
        iblt1.insert(&1234_u32.to_biguint().unwrap());
        let iblt4 = iblt1.empty_clone();
        assert!(!iblt1.equals(&iblt4), "empty clone removes data");
        assert!(iblt1.equals(&iblt1), "reflexive equality");
        assert!(iblt2.equals(&iblt2), "reflexive equality");
    }

    #[test]
    fn test_insert() {
        let mut iblt = init_iblt();
        let elem = 1234_u32.to_biguint().unwrap();
        let indexes = iblt.indexes(&elem);
        for &idx in &indexes {
            assert_eq!(iblt.counters().get(idx), 0);
            assert_eq!(iblt.data()[idx], 0);
        }
        assert!(!iblt.insert(&elem), "element did not exist already");
        assert_eq!(vvsum(iblt.counters()), 1 * iblt.num_hashes() as usize);
        for &idx in &indexes {
            assert_ne!(iblt.counters().get(idx), 0);
            assert_ne!(iblt.data()[idx], 0);
        }
        assert!(iblt.insert(&elem), "added element twice");
        assert_eq!(vvsum(iblt.counters()), 2 * iblt.num_hashes() as usize);
        for &idx in &indexes {
            assert_ne!(iblt.counters().get(idx), 0);
            assert_ne!(iblt.data()[idx], 0);
        }
    }

    #[test]
    fn test_empty_clone() {
        let mut iblt1 = init_iblt();
        iblt1.insert(&1234_u32.to_biguint().unwrap());
        iblt1.insert(&5678_u32.to_biguint().unwrap());
        let iblt2 = iblt1.empty_clone();
        assert!(vvsum(iblt1.counters()) > 0);
        assert_eq!(vvsum(iblt2.counters()), 0);
        assert!(iblt1.data().iter().sum::<u32>() > 0);
        assert_eq!(iblt2.data().iter().sum::<u32>(), 0);
        assert_eq!(
            iblt1.indexes(&1234_u32.to_biguint().unwrap()),
            iblt2.indexes(&1234_u32.to_biguint().unwrap()));
    }

    #[test]
    fn counter_overflow() {
        // test should not panic because we handle counter overflows now
        let mut iblt = InvBloomLookupTable::with_rate(1, 0.01, 10);
        iblt.insert(&1234_u32.to_biguint().unwrap());
        assert_ne!(vvsum(iblt.counters()), 0);
        iblt.insert(&1234_u32.to_biguint().unwrap());
        assert_eq!(vvsum(iblt.counters()), 0);
    }
}
