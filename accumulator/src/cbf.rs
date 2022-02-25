use std::time::Instant;
use bloom_sd::CountingBloomFilter;
use crate::Accumulator;
use digest::XorDigest;

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

        // k constants, the size of the CBF
        let v: Vec<usize> = (0..(cbf.num_entries() as usize))
            .map(|i| cbf.counters().get(i))
            .map(|count| count.try_into().unwrap())
            .collect();
        // n equations, the total number of elements, in k variables,
        // where the coefficients sum to the number of hashes.
        // We can omit an equation if none of the indexes are set in
        // the difference CBF.
        let eqs: Vec<Vec<usize>> = elems
            .iter()
            .filter(|elem| cbf.contains(&elem))
            .map(|elem| cbf.indexes(&elem))
            .collect();

        unimplemented!("solve ILP with {} eqs in {} variables",
                       eqs.len(), v.len(),);
    }
}
