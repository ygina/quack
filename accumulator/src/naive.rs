#[cfg(not(feature = "disable_validation"))]
use std::time::Instant;
use bincode;
#[cfg(not(feature = "disable_validation"))]
use itertools::Itertools;
use num_bigint::BigUint;
use serde::{Serialize, Deserialize};

use crate::Accumulator;
use digest::Digest;

/// The naive accumulator stores no auxiliary data structures outside
/// of the digest.
///
/// On validation, the accumulator tries every possible subset of the
/// given list of elements that is the size of the number of processed
/// elements. The log is valid if and only if any of the digests computed
/// from these subsets are equal to the existing digest. This approach
/// is exponential in the number of processed elements.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NaiveAccumulator {
    digest: Digest,
}

impl NaiveAccumulator {
    pub fn new() -> Self {
        Self {
            digest: Digest::new(),
        }
    }
}

impl Accumulator for NaiveAccumulator {
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    fn process(&mut self, elem: &BigUint) {
        self.digest.add(elem);
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
        let start = Instant::now();
        for (i, combination) in (0..elems.len())
                .combinations(self.total()).enumerate() {
            let mut digest = Digest::new();
            // We could amortize digest calculation using the previous digest,
            // but it's still exponential in the number of subsets
            for index in combination {
                digest.add(&elems[index]);
            }
            if digest.equals(&self.digest) {
                return true;
            }
            if i % 1000 == 0 && i != 0 {
                debug!("tried {} combinations: {:?}", i, Instant::now() - start);
            }
        }
        false
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
        let acc1 = NaiveAccumulator::new();
        let acc2 = NaiveAccumulator::new();
        assert_ne!(acc1, acc2, "different digest nonce");
    }

    #[test]
    fn empty_serialization() {
        let acc1 = NaiveAccumulator::new();
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: NaiveAccumulator = bincode::deserialize(&bytes).unwrap();
        assert_eq!(acc1, acc2);
    }

    #[test]
    fn serialization_with_data() {
        let mut acc1 = NaiveAccumulator::new();
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: NaiveAccumulator = bincode::deserialize(&bytes).unwrap();
        acc1.process_batch(&gen_elems(10));
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc3: NaiveAccumulator = bincode::deserialize(&bytes).unwrap();
        assert_ne!(acc1, acc2);
        assert_eq!(acc1, acc3);
    }
}
