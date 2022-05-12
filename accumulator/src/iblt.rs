#[cfg(not(feature = "disable_validation"))]
use std::time::Instant;
#[cfg(not(feature = "disable_validation"))]
use std::collections::{HashSet, HashMap};
#[cfg(not(feature = "disable_validation"))]
use std::num::Wrapping;

use bincode;
#[cfg(not(feature = "disable_validation"))]
use num_traits::Zero;
use serde::{Serialize, Deserialize};
use bloom_sd::InvBloomLookupTable;
use crate::Accumulator;
use digest::Digest;
#[cfg(not(feature = "disable_validation"))]
use itertools::Itertools;

#[cfg(not(feature = "disable_validation"))]
#[link(name = "glpk", kind = "dylib")]
extern "C" {
    fn solve_ilp_glpk(
        n_buckets: usize,
        iblt: *const usize,
        n_hashes: usize,
        n_packets: usize,
        pkt_hashes: *const u32,
        n_dropped: usize,
        dropped: *mut usize,
    ) -> i32;
}

// IBLT parameters
const DEFAULT_BITS_PER_ENTRY: usize = 8;
const DEFAULT_CELLS_MULTIPLIER: usize = 10;
const DEFAULT_NUM_HASHES: u32 = 2;

/// The counting bloom filter (IBLT) accumulator stores a IBLT of all processed
/// packets in addition to the digest.
///
/// On validation, the accumulator calculates the IBLT of the given list of
/// elements and subtracts the processed IBLT. The resulting difference IBLT
/// represents all lost elements. If there is a subset of given elements that
/// produces the same IBLT, we can say with high probability the log is good.
/// The count may be stored modulo some number.
#[derive(Serialize, Deserialize)]
pub struct IBLTAccumulator {
    digest: Digest,
    iblt: InvBloomLookupTable,
}

/// Calculate an IBLT from the logged elements, and subtract the IBLT of the
/// received elements from this newly-constructed IBLT.
/// - `n_dropped`: expected number of dropped elements
/// - `logged_elems`: the list of logged elements
/// - `received_iblt`: the IBLT of the receiving accumulator
#[cfg(not(feature = "disable_validation"))]
fn calculate_difference_iblt(
    n_dropped: usize,
    logged_elems: &Vec<Vec<u8>>,
    received_iblt: &InvBloomLookupTable,
) -> Option<InvBloomLookupTable> {
    let mut iblt = received_iblt.empty_clone();
    for elem in logged_elems {
        iblt.insert(elem);
    }
    let mut iblt_sum = 0;
    let wraparound_mask = (1 << (iblt.counters().bits_per_val() as u32)) - 1;
    for i in 0..(iblt.num_entries() as usize) {
        let logged_count = iblt.counters().get(i);
        let received_count = received_iblt.counters().get(i);
        // Handle counter overflows i.e. if the Bloom filter
        // stores the count modulo some number instead of the exact count.
        // This number is derived from the bits per entry.
        let difference_count =
            (Wrapping(logged_count) - Wrapping(received_count)).0
            & wraparound_mask;
        iblt.counters_mut().set(i, difference_count);
        iblt_sum += difference_count;

        // Additionally set the data value
        let logged_data = iblt.data()[i];
        let received_data = received_iblt.data()[i];
        let difference_data =
            (Wrapping(logged_data) - Wrapping(received_data)).0;
        if difference_count == 0 && !difference_data.is_zero() {
            return None;
        }
        iblt.data_mut()[i] = difference_data;
    }

    // If the number of dropped packets multiplied by the number of hashes is
    // equal to the sum of all entries in the IBLT, proceed with the ILP check.
    if (n_dropped as u32) * iblt.num_hashes() == iblt_sum {
        return Some(iblt);
    }

    // Otherwise there was wraparound, which either occurs if a counter has a
    // a real negative entry (meaning there was a malicious packet), or if the
    // difference is some larger number modulo the max value of an IBLT entry.
    // We can say that if the number of dropped packets does not exceed this
    // max value, then wraparound definitely should not have occurred.
    // Otherwise, we simply do not handle wraparound, as the user should choose
    // a threshold that allows the number of dropped packets in a cell to rarely
    // exceed the threshold (unless dropped packets have high multiplicity?).
    if (n_dropped as u32) <= wraparound_mask {
        debug!("malicious wraparound detected");
    } else {
        warn!("not handling potentially benign wraparound, may need to select
            a bigger threshold");
    }
    None
}

/// Checks whether there is a subset of elements with the DJB hashes of the
/// dropped elements that produce the same digest.
/// - `elems`: the list of logged elements
/// - `removed_u32`: the set of DJB hashes of removed elements from the IBLT.
///    Elements are necessarily unique or they would have hashed to the same
///    slot in the IBLT.
#[cfg(not(feature = "disable_validation"))]
fn check_digest_from_removed_set(
    expected_digest: &Digest,
    elems: Vec<&Vec<u8>>,
    removed: HashSet<u32>,
) -> bool {
    // Create a map from DJB hash to elements that hash to that value. If the
    // DJB hash is not in the removed set, then the packet was not dropped, so
    // add it to the digest. Otherwise, it might have been dropped.
    let mut digest = Digest::new();
    let mut collisions_map: HashMap<u32, Vec<&Vec<u8>>> = HashMap::new();
    for elem in elems {
        let elem_u32 = bloom_sd::elem_to_u32(&elem);
        if removed.contains(&elem_u32) {
            collisions_map.entry(elem_u32).or_insert(vec![]).push(elem);
        } else {
            digest.add(elem);
        }
    }

    // If not every element in the removed set has a preimage, we are missing
    // an element from the log.
    if removed.len() != collisions_map.len() {
        return false;
    }

    // Remove any entries from the collisions map with only one preimage value.
    // Those packets were necessarily dropped since there is no other mapping.
    // Map remaining entries to possible combinations with one element removed.
    let combinations = collisions_map.into_iter()
        .filter(|(_, collisions)| collisions.len() != 1)
        .map(|(_, collisions)| (collisions.len() - 1, collisions))
        .map(|(n, collisions)| collisions.into_iter().combinations(n))
        .collect::<Vec<_>>();
    if combinations.len() == 0 {
        debug!("no collisions, checking digest");
        assert_eq!(digest.count, expected_digest.count);
        return digest.equals(&expected_digest);
    }
    debug!("handling collisions for {} removed elems", combinations.len());

    // Try every combination of remaining elements with one removed per slot,
    // and if any of them produce a matching digest, accept.
    let t1 = Instant::now();
    for (n_digests, combination) in
            combinations.into_iter().multi_cartesian_product().enumerate() {
        let mut digest = digest.clone();
        for elem in combination.into_iter().flat_map(|val| val) {
            digest.add(&elem);
        }
        assert_eq!(digest.count, expected_digest.count);
        if digest.equals(&expected_digest) {
            debug!(
                "found matching digest after checking {} digests: {:?}",
                n_digests,
                Instant::now() - t1,
            );
            return true;
        }
    }
    false
}

/// Returns the indexes of the dropped elements in `elems` that satisfy the
/// counters in the IBLT. Does not check the data fields in the IBLT, which may
/// not be accurate if there is more than one solution these constraints.
/// - `n_dropped`: expected number of dropped elements less the number of
///    elements already removed from the IBLT
/// - `elems`: the list of logged elements
/// - `iblt`: the difference IBLT
#[cfg(not(feature = "disable_validation"))]
fn solve_ilp_for_iblt(
    n_dropped_remaining: usize,
    elems: &Vec<Vec<u8>>,
    iblt: InvBloomLookupTable,
) -> Option<HashSet<usize>> {
    // Number of equations = # of remaining candidate elements in `elems_i`.
    // Number of variables = number of cells in the IBLT.
    let mut elems_i: Vec<usize> = vec![];
    let pkt_hashes: Vec<u32> = elems
        .iter()
        .enumerate()
        .filter(|(_, elem)| iblt.contains(&elem))
        .flat_map(|(i, elem)| {
            elems_i.push(i);
            iblt.indexes(&elem)
        })
        .map(|hash| hash as u32)
        .collect();
    let counters: Vec<usize> = (0..(iblt.num_entries() as usize))
        .map(|i| iblt.counters().get(i))
        .map(|count| count.try_into().unwrap())
        .collect();
    assert!(n_dropped_remaining <= elems_i.len());
    debug!("setup system of {} eqs in {} vars (expect sols to sum to {})",
        elems_i.len(),
        counters.len(),
        n_dropped_remaining);

    // Solve the ILP with GLPK. The result is the indices of the dropped
    // packets in the `elems_i` vector. The number of solutions
    // does not depend entirely on the number of equations and variables.
    // Instead, if there are fewer (linearly independent) equations than
    // the sum of the counters divided by the number of hashes, then there
    // is no solution. If there are more, there may be multiple solutions.
    let mut dropped: Vec<usize> = vec![0; n_dropped_remaining];
    let err = unsafe {
        solve_ilp_glpk(
            counters.len(),
            counters.as_ptr(),
            iblt.num_hashes() as usize,
            elems_i.len(),
            pkt_hashes.as_ptr(),
            n_dropped_remaining,
            dropped.as_mut_ptr(),
        )
    };
    if err != 0 {
        warn!("ILP solving error: {}", err);
        return None;
    }
    Some(dropped
        .into_iter()
        .map(|dropped_i| elems_i[dropped_i])
        .collect::<HashSet<_>>())
}

impl IBLTAccumulator {
    pub fn new_with_params(
        threshold: usize,
        bits_per_entry: usize,
        cells_multiplier: usize,
        num_hashes: u32,
        seed: Option<u64>,
    ) -> Self {
        let iblt = if let Some(seed) = seed {
            InvBloomLookupTable::new_with_seed(
                seed,
                bits_per_entry,
                cells_multiplier * threshold,
                num_hashes,
            )
        } else {
            InvBloomLookupTable::new(
                bits_per_entry,
                cells_multiplier * threshold,
                num_hashes,
            )
        };
        Self {
            digest: Digest::new(),
            iblt,
        }
    }

    pub fn new(threshold: usize, seed: Option<u64>) -> Self {
        Self::new_with_params(
            threshold,
            DEFAULT_BITS_PER_ENTRY,
            DEFAULT_CELLS_MULTIPLIER,
            DEFAULT_NUM_HASHES,
            seed,
        )
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.digest == other.digest
            && self.iblt.equals(&other.iblt)
    }
}

impl Accumulator for IBLTAccumulator {
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn reset(&mut self) {
        self.digest = Digest::new();
        self.iblt = self.iblt.empty_clone();
    }

    fn process(&mut self, elem: &[u8]) {
        self.digest.add(elem);
        self.iblt.insert(elem);
    }

    fn process_batch(&mut self, elems: &Vec<Vec<u8>>) {
        for elem in elems {
            self.process(elem);
        }
    }

    fn total(&self) -> usize {
        self.digest.count as usize
    }

    #[cfg(feature = "disable_validation")]
    fn validate(&self, _elems: &Vec<Vec<u8>>) -> bool {
        panic!("validation not enabled")
    }

    #[cfg(not(feature = "disable_validation"))]
    fn validate(&self, elems: &Vec<Vec<u8>>) -> bool {
        let t1 = Instant::now();
        if elems.len() < self.total() {
            warn!("more elements received than logged");
            return false;
        }

        // If no elements are missing, just recalculate the digest.
        let n_dropped = elems.len() - self.total();
        if n_dropped == 0 {
            let mut digest = Digest::new();
            for elem in elems {
                digest.add(elem);
            }
            return digest.equals(&self.digest);
        }

        let mut iblt = {
            let iblt = calculate_difference_iblt(n_dropped, elems, &self.iblt);
            if let Some(iblt) = iblt {
                iblt
            } else {
                return false;
            }
        };
        let t2 = Instant::now();
        debug!("calculated the difference iblt: {:?}", t2 - t1);

        // Remove any elements that are definitely dropped based on counters
        // in the IBLT that are set to 1. Then find the remaining list of
        // candidate dropped elements by based on any whose indexes are still
        // not 0. If elements are not unique, the ILP can find _a_ solution.
        let removed = iblt.eliminate_elems();
        let t3 = Instant::now();
        debug!("eliminated {}/{} elements using the iblt: {:?}",
            removed.len(), n_dropped, t3 - t2);

        // The remaining maybe dropped elements should make up any non-zero
        // entries in the IBLT. Since we checked that the number of dropped
        // elements is at most the size of the original set, if we removed the
        // number of dropped elements, then the IBLT necessarily only has zero
        // entries. This means solving an ILP is unnecessary but we still
        // check that the digest matches in case the router constructed a
        // preimage collision.
        if removed.len() == n_dropped {
            debug!("all iblt elements removed");
            return check_digest_from_removed_set(
                &self.digest,
                elems.iter().collect(),
                removed,
            );
        }

        // Then there are still some remaining candidate dropped elements,
        // and the IBLT is not empty. Solve an ILP to determine which elements
        // could make up the counters in the IBLT.
        assert!(n_dropped > removed.len());
        let n_dropped_remaining = n_dropped - removed.len();
        let dropped_is = if let Some(dropped_is) = solve_ilp_for_iblt(
            n_dropped_remaining,
            elems,
            iblt,
        ) {
            dropped_is
        } else {
            return false;
        };
        let t4 = Instant::now();
        debug!("solved ILP: {:?}", t4 - t3);

        // Right now we have:
        // * `removed` - the djb hash of elems that were definitely dropped
        // * `dropped_is` - the indexes of the elems the ILP believes were
        //    dropped in the `elems` vec.
        debug!("checking combinations for removed IBLT elems");
        let elems = elems.iter()
            .enumerate()
            .filter(|(i, _)| !dropped_is.contains(&i))
            .map(|(_, elem)| elem)
            .collect::<Vec<_>>();
        return check_digest_from_removed_set(&self.digest, elems, removed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode;
    use bloom_sd::ValueVec;

    const NBYTES: usize = 16;

    fn gen_elems_with_seed(n: usize, seed: u64) -> Vec<Vec<u8>> {
        use rand::{SeedableRng, Rng};
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..n).map(|_| (0..NBYTES).map(|_| rng.gen::<u8>()).collect()).collect()
    }

    #[test]
    fn test_not_equals() {
        let acc1 = IBLTAccumulator::new(100, None);
        let acc2 = IBLTAccumulator::new(100, None);
        assert!(!acc1.equals(&acc2), "different digest nonce");
    }

    #[test]
    fn empty_serialization() {
        let acc1 = IBLTAccumulator::new(1000, None);
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: IBLTAccumulator = bincode::deserialize(&bytes).unwrap();
        assert!(acc1.equals(&acc2));
    }

    #[test]
    fn serialization_with_data() {
        let mut acc1 = IBLTAccumulator::new(1000, None);
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: IBLTAccumulator = bincode::deserialize(&bytes).unwrap();
        acc1.process_batch(&gen_elems_with_seed(10, 111));
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc3: IBLTAccumulator = bincode::deserialize(&bytes).unwrap();
        assert!(!acc1.equals(&acc2));
        assert!(acc1.equals(&acc3));
    }

    fn vvsum(vec: &ValueVec) -> usize {
        let num_entries = vec.len() / vec.bits_per_val();
        (0..num_entries).map(|i| vec.get(i)).sum::<u32>() as usize
    }

    #[test]
    fn test_calculate_difference_iblt_inverse() {
        let n_logged = 100;
        let n_dropped = 0;
        let log = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut iblt = InvBloomLookupTable::new_with_seed(111, 4, 10, 3);
        for elem in &log {
            iblt.insert(&elem);
        }
        let diff = {
            let diff = calculate_difference_iblt(n_dropped, &log, &iblt);
            assert!(diff.is_some());
            diff.unwrap()
        };
        assert_eq!(vvsum(diff.counters()), 0);
    }

    #[test]
    fn test_calculate_difference_iblt() {
        let n_logged = 100;
        let n_dropped = 60;
        let log = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        // Insert only the first 40 elements into the IBLT.
        let bpe = 4;
        let mut d1 = InvBloomLookupTable::new_with_seed(111, bpe, 60, 3);
        let mut d2 = InvBloomLookupTable::new_with_seed(111, bpe, 60, 3);
        for i in 0..n_logged {
            d1.insert(&log[i]);
        }
        for i in 0..(n_logged - n_dropped) {
            d2.insert(&log[i]);
        }

        // Calculate the difference.
        let diff = {
            let diff = calculate_difference_iblt(n_dropped, &log, &d2);
            assert!(diff.is_some());
            diff.unwrap()
        };

        // Check that every case with and without wraparound is tested.
        let (mut counter_no_wrap, mut counter_wrap) = (1 << 31, 1 << 31);
        let (mut data_no_wrap, mut data_wrap) = (1 << 31, 1 << 31);
        for i in 0..(d1.num_entries() as usize) {
            if d1.counters().get(i) >= d2.counters().get(i) {
                counter_no_wrap = i;
            } else {
                counter_wrap = i;
            }
            if d1.data()[i] >= d2.data()[i] {
                data_no_wrap = i;
            } else {
                data_wrap = i;
            }
        }
        assert!(counter_no_wrap < (d1.num_entries() as usize));
        assert!(counter_wrap < (d1.num_entries() as usize));
        assert!(data_no_wrap < (d1.num_entries() as usize));
        assert!(data_wrap < (d1.num_entries() as usize));

        // Check that the counters and data values were subtracted correctly.
        assert_eq!(
            diff.counters().get(counter_no_wrap),
            d1.counters().get(counter_no_wrap) - d2.counters().get(counter_no_wrap));
        assert_eq!(
            diff.counters().get(counter_wrap),
            ((1 << bpe) - 1) - d2.counters().get(counter_wrap) + d1.counters().get(counter_wrap) + 1);
        assert_eq!(
            diff.data()[data_no_wrap],
            d1.data()[data_no_wrap] - d2.data()[data_no_wrap]);
        assert_eq!(
            diff.data()[data_wrap],  // u32
            u32::max_value() - d2.data()[data_wrap] + d1.data()[data_wrap] + 1);
    }

    #[test]
    fn test_calculate_difference_iblt_with_wraparound_from_low_threshold() {
        let n_logged = 100;
        let n_dropped = 60;
        let log = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut d1 = InvBloomLookupTable::new_with_seed(111, 4, 6, 3);
        let mut d2 = InvBloomLookupTable::new_with_seed(111, 4, 6, 3);
        for i in 0..n_logged {
            d1.insert(&log[i]);
        }
        for i in 0..(n_logged - n_dropped) {
            d2.insert(&log[i]);
        }
        assert!(calculate_difference_iblt(n_dropped, &log, &d2).is_none());
    }

    #[test]
    fn test_calculate_difference_iblt_with_malicious_wraparound() {
        let n_logged = 100;
        let n_dropped = 60;
        let log_start_i = 40;
        let log = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut d1 = InvBloomLookupTable::new_with_seed(111, 4, 60, 3);
        let mut d2 = InvBloomLookupTable::new_with_seed(111, 4, 60, 3);
        for i in log_start_i..n_logged {
            d1.insert(&log[i]);
        }
        for i in 0..(n_logged - n_dropped) {
            d2.insert(&log[i]);
        }
        assert!(calculate_difference_iblt(
            n_dropped, &log[log_start_i..].to_vec(), &d2).is_none());
    }

    #[test]
    fn test_check_digest_no_drop() {
        let n_logged = 100;
        let elems = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut d = Digest::new();
        for e in &elems {
            d.add(e);
        }
        // Succeeds because no elements are dropped
        let elems_ref = elems.iter().collect::<Vec<_>>();
        assert!(check_digest_from_removed_set(&d, elems_ref, HashSet::new()));
    }

    #[test]
    fn test_check_digest_removed_elem_does_not_exist() {
        let n_logged = 100;
        let elems = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let hashes: HashSet<_> = elems.iter()
            .map(|e| bloom_sd::elem_to_u32(e)).collect();
        assert_eq!(
            elems.len(), hashes.len(),
            "DJB hashes are unique in this test");
        let mut d = Digest::new();
        for e in &elems {
            d.add(e);
        }
        let removed = {
            let mut set = HashSet::new();
            let removed_hash: u32 = 111;
            assert!(!hashes.contains(&removed_hash), "check no collision");
            set.insert(removed_hash);
            set
        };
        // Fails because a dropped element is not in the original log
        let elems_ref = elems.iter().collect::<Vec<_>>();
        assert!(!check_digest_from_removed_set(&d, elems_ref, removed));
    }

    #[test]
    fn test_check_digest_subset_no_collisions() {
        let n_logged = 100;
        let n_dropped = 20;
        let elems = (0..(n_logged as u32))
            .map(|i| i.to_be_bytes().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let hashes: HashSet<_> = elems.iter()
            .map(|e| bloom_sd::elem_to_u32(e)).collect();
        assert_eq!(
            elems.len(), hashes.len(),
            "DJB hashes are unique in this test");
        let mut d = Digest::new();
        // "Drop" the first `n_dropped` elements
        for i in n_dropped..n_logged {
            d.add(&elems[i]);
        }
        let removed = (0..n_dropped)
            .map(|i| bloom_sd::elem_to_u32(&elems[i]))
            .collect::<HashSet<_>>();
        assert_eq!(removed.len(), n_dropped, "DJB hashes should be unique in \
            the remove set (the property is also enforced because the elems \
            eliminated from the IBLT must be unique).");
        let elems_ref = elems.iter().collect::<Vec<_>>();
        assert!(check_digest_from_removed_set(&d, elems_ref, removed));
    }

    #[test]
    fn test_check_digest_subset_with_collisions() {
        let n_logged = 10000;
        let n_dropped = 40;
        // With this seed, there is a collision at indexes 223 and 6875
        let (drop_i, drop_j) = (223, 6875);
        let elems = gen_elems_with_seed(n_logged, 112);
        assert_eq!(
            bloom_sd::elem_to_u32(&elems[drop_i]),
            bloom_sd::elem_to_u32(&elems[drop_j]));

        let mut d = Digest::new();
        // "Drop" the first `n_dropped` elems and the elem at index `drop_i`
        for i in n_dropped..n_logged {
            if i == drop_i {
                continue;
            }
            d.add(&elems[i]);
        }
        let mut removed = (0..n_dropped)
            .map(|i| bloom_sd::elem_to_u32(&elems[i]))
            .collect::<HashSet<_>>();
        removed.insert(bloom_sd::elem_to_u32(&elems[drop_i]));
        assert_eq!(removed.len(), n_dropped + 1, "DJB hashes should be unique \
            in the remove set (the property is also enforced because the elems \
            eliminated from the IBLT must be unique).");
        let elems_ref = elems.iter().collect::<Vec<_>>();
        assert!(check_digest_from_removed_set(&d, elems_ref, removed));
    }

    #[test]
    fn test_solve_ilp_for_iblt_success_whp() {
        let n_logged = 1000;
        let n_dropped = 100;
        let elems = gen_elems_with_seed(n_logged, 123);

        // Set up the IBLT with the dropped elements and eliminate.
        let mut iblt = InvBloomLookupTable::new_with_seed(1234, 8, 200, 2);
        for i in 0..n_dropped {
            iblt.insert(&elems[i]);
        }
        let mut removed = iblt.eliminate_elems();
        let n_dropped_remaining = n_dropped - removed.len();
        assert_ne!(n_dropped_remaining, 0, "this test requires the ILP");
        let result = solve_ilp_for_iblt(n_dropped_remaining, &elems, iblt);
        assert!(result.is_some(), "no error when solving ILP");
        let result = result.unwrap();
        assert_eq!(result.len(), n_dropped_remaining);

        // Check that the results are the first `n_dropped` elements,
        // at least with high probability.
        let mut dropped_is = (0..n_dropped).collect::<HashSet<usize>>();
        for dropped_i in result {
            assert!(dropped_is.remove(&dropped_i), "{}", dropped_i);
        }
        for dropped_i in dropped_is {
            assert!(removed.remove(&bloom_sd::elem_to_u32(&elems[dropped_i])),
                "{}", dropped_i);
        }
    }

    #[test]
    fn test_solve_ilp_for_iblt_failure_whp() {
        let n_logged = 1000;
        let n_dropped = 100;
        let elems = gen_elems_with_seed(n_logged, 123);

        // Set up the IBLT with the dropped elements and eliminate.
        let mut iblt = InvBloomLookupTable::new_with_seed(1234, 8, 150, 2);
        for i in 0..n_dropped {
            iblt.insert(&elems[i]);
        }
        let removed = iblt.eliminate_elems();
        let n_dropped_remaining = n_dropped - removed.len();
        assert_ne!(n_dropped_remaining, 0, "this test requires the ILP");
        let result = solve_ilp_for_iblt(n_dropped_remaining, &elems, iblt);
        assert!(result.is_some(), "no error when solving ILP");
        let result = result.unwrap();
        assert_eq!(result.len(), n_dropped_remaining);

        // Check that the results are the first `n_dropped` elements,
        // at least with high probability.
        // A slightly smaller IBLT results in a non-robust ILP solver.
        let mut dropped_is = (0..n_dropped).collect::<HashSet<usize>>();
        let mut d_expected = true;
        for dropped_i in result {
            d_expected = d_expected && dropped_is.remove(&dropped_i);
        }
        assert!(!d_expected);
    }
}
