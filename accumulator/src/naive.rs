use itertools::Itertools;
use crate::Accumulator;
use digest::XorDigest;

/// The naive accumulator stores no auxiliary data structures outside
/// of the digest.
///
/// On validation, the accumulator tries every possible subset of the
/// given list of elements that is the size of the number of processed
/// elements. The log is valid if and only if any of the digests computed
/// from these subsets are equal to the existing digest. This approach
/// is exponential in the number of processed elements.
pub struct NaiveAccumulator {
    digest: XorDigest,
    num_elems: usize,
}

impl NaiveAccumulator {
    pub fn new() -> Self {
        Self {
            digest: XorDigest::new(),
            num_elems: 0,
        }
    }
}

impl Accumulator for NaiveAccumulator {
    fn process(&mut self, elem: u32) {
        self.digest.add(elem);
        self.num_elems += 1;
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
        for combination in (0..elems.len()).combinations(self.num_elems) {
            let mut digest = XorDigest::new();
            // We could amortize digest calculation using the previous digest,
            // but it's still exponential in the number of subsets
            for index in combination {
                digest.add(elems[index]);
            }
            if digest == self.digest {
                return true;
            }
        }
        false
    }
}
