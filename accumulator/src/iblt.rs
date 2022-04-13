#[cfg(not(feature = "disable_validation"))]
use std::time::Instant;
#[cfg(not(feature = "disable_validation"))]
use std::collections::{HashSet, HashMap};
#[cfg(not(feature = "disable_validation"))]
use std::num::Wrapping;

use bincode;
use num_bigint::BigUint;
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

// TODO: IBLT parameters
const BITS_PER_ENTRY: usize = 8;
#[cfg(not(feature = "disable_validation"))]
const WRAPAROUND_MASK: u32 = (1 << BITS_PER_ENTRY) - 1;
const FALSE_POSITIVE_RATE: f32 = 0.0001;

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
    logged_elems: &Vec<BigUint>,
    received_iblt: &InvBloomLookupTable,
) -> Option<InvBloomLookupTable> {
    let mut iblt = received_iblt.empty_clone();
    for elem in logged_elems {
        iblt.insert(elem);
    }
    let mut iblt_sum = 0;
    for i in 0..(iblt.num_entries() as usize) {
        let logged_count = iblt.counters().get(i);
        let received_count = received_iblt.counters().get(i);
        // Handle counter overflows i.e. if the Bloom filter
        // stores the count modulo some number instead of the exact count.
        // This number is derived from the bits per entry.
        let difference_count =
            (Wrapping(logged_count) - Wrapping(received_count)).0
            & WRAPAROUND_MASK;
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
    if (n_dropped as u32) <= WRAPAROUND_MASK {
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
    elems: Vec<&BigUint>,
    removed: HashSet<u32>,
) -> bool {
    // Create a map from DJB hash to elements that hash to that value. If the
    // DJB hash is not in the removed set, then the packet was not dropped, so
    // add it to the digest. Otherwise, it might have been dropped.
    let mut digest = Digest::new();
    let mut collisions_map: HashMap<u32, Vec<&BigUint>> = HashMap::new();
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
    for (n_digests, combination) in
            combinations.into_iter().multi_cartesian_product().enumerate() {
        let mut digest = digest.clone();
        for elem in combination.into_iter().flat_map(|val| val) {
            digest.add(&elem);
        }
        assert_eq!(digest.count, expected_digest.count);
        if digest.equals(&expected_digest) {
            debug!("found matching digest after checking {} digests", n_digests);
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
    elems: &Vec<BigUint>,
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
    info!("setup system of {} eqs in {} vars (expect sols to sum to {})",
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
    pub fn new(threshold: usize) -> Self {
        let iblt = InvBloomLookupTable::with_rate(
            BITS_PER_ENTRY,
            FALSE_POSITIVE_RATE,
            threshold.try_into().unwrap(),
        );
        debug!("{} entries and {} bits per entry",
            iblt.num_entries(), BITS_PER_ENTRY);
        Self {
            digest: Digest::new(),
            iblt,
        }
    }

    pub fn new_with_rate(threshold: usize, fp_rate: f32) -> Self {
        let iblt = InvBloomLookupTable::with_rate(
            BITS_PER_ENTRY,
            fp_rate,
            threshold.try_into().unwrap(),
        );
        let data_size = std::mem::size_of_val(&iblt.data()[0]);
        debug!("{} entries and {} bits per entry",
            iblt.num_entries(), BITS_PER_ENTRY);
        info!("size of iblt = {} bytes",
            (iblt.num_entries() as usize) * (BITS_PER_ENTRY + data_size) / 8);
        Self {
            digest: Digest::new(),
            iblt,
        }
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

    fn process(&mut self, elem: &BigUint) {
        self.digest.add(elem);
        self.iblt.insert(elem);
    }

    fn process_batch(&mut self, elems: &Vec<BigUint>) {
        for elem in elems {
            self.process(elem);
        }
    }

    fn total(&self) -> usize {
        self.digest.count as usize
    }

    #[cfg(feature = "disable_validation")]
    fn validate(&self, _elems: &Vec<BigUint>) -> bool {
        panic!("validation not enabled")
    }

    #[cfg(not(feature = "disable_validation"))]
    fn validate(&self, elems: &Vec<BigUint>) -> bool {
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
        info!("calculated the difference iblt: {:?}", t2 - t1);

        // Remove any elements that are definitely dropped based on counters
        // in the IBLT that are set to 1. Then find the remaining list of
        // candidate dropped elements by based on any whose indexes are still
        // not 0. If elements are not unique, the ILP can find _a_ solution.
        let removed = iblt.eliminate_elems();
        let t3 = Instant::now();
        info!("eliminated {}/{} elements using the iblt: {:?}",
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
        info!("solved ILP: {:?}", t4 - t3);

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
    use rand;
    use rand::Rng;
    use num_bigint::ToBigUint;

    fn gen_elems(n: usize) -> Vec<BigUint> {
        let mut rng = rand::thread_rng();
        (0..n).map(|_| rng.gen::<u128>().to_biguint().unwrap()).collect()
    }

    #[test]
    fn test_not_equals() {
        let acc1 = IBLTAccumulator::new(100);
        let acc2 = IBLTAccumulator::new(100);
        assert!(!acc1.equals(&acc2), "different digest nonce");
    }

    #[test]
    fn empty_serialization() {
        let acc1 = IBLTAccumulator::new(1000);
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: IBLTAccumulator = bincode::deserialize(&bytes).unwrap();
        assert!(acc1.equals(&acc2));
    }

    #[test]
    fn serialization_with_data() {
        let mut acc1 = IBLTAccumulator::new(1000);
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: IBLTAccumulator = bincode::deserialize(&bytes).unwrap();
        acc1.process_batch(&gen_elems(10));
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc3: IBLTAccumulator = bincode::deserialize(&bytes).unwrap();
        assert!(!acc1.equals(&acc2));
        assert!(acc1.equals(&acc3));
    }

    #[test]
    fn test_calculate_difference_iblt() {
        unimplemented!()
    }

    #[test]
    fn test_check_digest_from_removed_set() {
        unimplemented!()
    }

    #[test]
    fn test_solve_ilp_for_iblt() {
        unimplemented!()
    }
}
