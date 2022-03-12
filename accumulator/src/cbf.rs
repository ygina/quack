use std::time::Instant;
use bloom_sd::CountingBloomFilter;
use crate::Accumulator;
use digest::XorDigest;

#[link(name = "glpk", kind = "dylib")]
extern "C" {
    fn solve_ilp_glpk(
        n_buckets: usize,
        cbf: *const usize,
        n_hashes: usize,
        n_packets: usize,
        pkt_hashes: *const u32,
        n_dropped: usize,
        dropped: *mut usize,
    ) -> i32;
}

/// The counting bloom filter (CBF) accumulator stores a CBF of all processed
/// packets in addition to the digest.
///
/// On validation, the accumulator calculates the CBF of the given list of
/// elements and subtracts the processed CBF. The resulting difference CBF
/// represents all lost elements. If there is a subset of given elements that
/// produces the same CBF, we can say with high probability the log is good.
/// The count may be stored modulo some number.
pub struct CBFAccumulator {
    digest: XorDigest,
    num_elems: usize,
    cbf: CountingBloomFilter,
}

// TODO: CBF parameters
const BITS_PER_ENTRY: usize = 16;
const FALSE_POSITIVE_RATE: f32 = 0.0001;

impl CBFAccumulator {
    pub fn new(threshold: usize) -> Self {
        Self {
            digest: XorDigest::new(),
            num_elems: 0,
            cbf: CountingBloomFilter::with_rate(
                BITS_PER_ENTRY,
                FALSE_POSITIVE_RATE,
                threshold.try_into().unwrap(),
            ),
        }
    }
}

impl Accumulator for CBFAccumulator {
    fn process(&mut self, elem: u32) {
        self.digest.add(elem);
        self.num_elems += 1;
        self.cbf.insert(&elem);
    }

    fn process_batch(&mut self, elems: &Vec<u32>) {
        for &elem in elems {
            self.process(elem);
        }
    }

    fn total(&self) -> usize {
        self.num_elems
    }

    fn validate(&self, elems: &Vec<u32>) -> bool {
        let t1 = Instant::now();
        if elems.len() < self.total() {
            warn!("more elements received than logged");
            return false;
        }
        let mut cbf = self.cbf.empty_clone();
        for &elem in elems {
            cbf.insert(&elem);
        }
        for i in 0..(cbf.num_entries() as usize) {
            let processed_count = cbf.counters().get(i);
            let received_count = self.cbf.counters().get(i);
            // TODO: handle counter overflows i.e. if the Bloom filter
            // stores the count modulo some number instead of the exact count
            if processed_count < received_count {
                return false;
            }
            cbf.counters_mut().set(i, processed_count - received_count)
        }
        let t2 = Instant::now();
        debug!("calculated the difference cbf: {:?}", t2 - t1);

        // Handle the case where no packets are dropped. All counters in the
        // difference CBF should be equal to 0.
        // k constants, the size of the CBF
        let counters: Vec<usize> = (0..(cbf.num_entries() as usize))
            .map(|i| cbf.counters().get(i))
            .map(|count| count.try_into().unwrap())
            .collect();
        let n_dropped = elems.len() - self.total();
        if n_dropped == 0 {
            for &counter in &counters {
                if counter != 0 {
                    return false;
                }
            }
            return true;
        }

        // n equations, the total number of elements, in k variables,
        // where the coefficients sum to the number of hashes.
        // We can omit an equation if none of the indexes are set in
        // the difference CBF.
        let mut elems_i: Vec<usize> = vec![];
        let pkt_hashes: Vec<u32> = elems
            .iter()
            .enumerate()
            .filter(|(_, elem)| cbf.contains(&elem))
            .flat_map(|(i, elem)| {
                elems_i.push(i);
                cbf.indexes(&elem)
            })
            .map(|hash| hash as u32)
            .collect();
        let t3 = Instant::now();
        debug!("setup the system of equations: {:?}", t3 - t2);

        // Solve the ILP with GLPK. The result is the indices of the dropped
        // packets in the `elems_i` vector. This just shows that there is _a_
        // solution to the ILP, we don't know if it's the right one.
        let mut dropped: Vec<usize> = vec![0; n_dropped];
        let err = unsafe {
            solve_ilp_glpk(
                counters.len(),
                counters.as_ptr(),
                cbf.num_hashes() as usize,
                elems_i.len(),
                pkt_hashes.as_ptr(),
                n_dropped,
                dropped.as_mut_ptr(),
            )
        };
        let t4 = Instant::now();
        debug!("solved an ILP with {} equations in {} variables: {:?}",
            elems_i.len(), counters.len(), t4 - t3);
        if err == 0 {
            // TODO: do something with `dropped`?
            for dropped_i in dropped {
                assert!(dropped_i < elems_i.len());
            }
            true
        } else {
            warn!("ILP solving error: {}", err);
            false
        }
    }
}
