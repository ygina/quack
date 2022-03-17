#[macro_use]
extern crate log;

mod cbf;
mod iblt;
mod naive;
mod power_sum;

use std::collections::HashMap;
use digest::XorDigest;

pub use cbf::CBFAccumulator;
pub use iblt::IBLTAccumulator;
pub use naive::NaiveAccumulator;
pub use power_sum::PowerSumAccumulator;

pub trait Accumulator {
    /// Process a single element.
    fn process(&mut self, elem: u32);
    /// Process a batch of elements.
    fn process_batch(&mut self, elems: &Vec<u32>);
    /// The total number of processed elements.
    fn total(&self) -> usize;
    /// Validate the accumulator against a list of elements.
    ///
    /// The accumulator is valid if the elements that the accumulator has
    /// processed are a subset of the provided list of elements.
    fn validate(&self, elems: &Vec<u32>) -> bool;
}

fn check_digest(
    elems: &Vec<u32>,
    mut dropped_count: HashMap<u32, usize>,
    expected: &XorDigest,
) -> bool {
    let mut digest = XorDigest::new();
    for &elem in elems {
        if let Some(count) = dropped_count.remove(&elem) {
            if count > 0 {
                dropped_count.insert(elem, count - 1);
            }
        } else {
            digest.add(elem);
        }
    }
    &digest == expected
}

#[cfg(test)]
mod tests {
    use rand;
    use rand::Rng;
    use super::*;

    const MALICIOUS_ELEM: u32 = std::u32::MAX;

    fn base_accumulator_test(
        mut accumulator: Box<dyn Accumulator>,
        num_logged: usize,
        num_dropped: usize,
        malicious: bool,
    ) {
        let mut rng = rand::thread_rng();
        let elems: Vec<u32> = (0..num_logged)
            .map(|_| rng.gen_range(0..MALICIOUS_ELEM)).collect();
        // indexes may be repeated but it's close enough
        let dropped_is: Vec<usize> = (0..num_dropped)
            .map(|_| rng.gen_range(0..num_logged)).collect();
        let malicious_i: usize = rng.gen_range(0..num_logged);
        for i in 0..elems.len() {
            if malicious && malicious_i == i {
                accumulator.process(MALICIOUS_ELEM);
            } else if !dropped_is.contains(&i) {
                accumulator.process(elems[i]);
            }
        }
        let valid = accumulator.validate(&elems);
        assert_eq!(valid, !malicious);
    }

    #[test]
    fn naive_none_dropped() {
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 0, false);
    }

    #[test]
    fn naive_one_dropped() {
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 1, false);
    }

    #[test]
    fn naive_two_dropped() {
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 2, false);
    }

    #[test]
    fn naive_three_dropped() {
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 3, false);
    }

    #[test]
    fn naive_one_malicious_and_none_dropped() {
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 0, true);
    }

    #[test]
    fn naive_one_malicious_and_one_dropped() {
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 1, true);
    }

    #[test]
    fn naive_one_malicious_and_many_dropped() {
        // validation takes much longer to fail because many
        // combinations must be tried and they all fail
        let accumulator = NaiveAccumulator::new();
        base_accumulator_test(Box::new(accumulator), 100, 3, true);
    }

    #[test]
    fn power_sum_none_dropped() {
        let accumulator = PowerSumAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 0, false);
    }

    #[test]
    fn power_sum_one_dropped() {
        let accumulator = PowerSumAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 1, false);
    }

    #[test]
    fn power_sum_two_dropped() {
        let accumulator = PowerSumAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 2, false);
    }

    #[test]
    fn power_sum_many_dropped() {
        let accumulator = PowerSumAccumulator::new(1000);
        base_accumulator_test(Box::new(accumulator), 1000, 10, false);
    }

    #[test]
    fn power_sum_one_malicious_and_none_dropped() {
        let accumulator = PowerSumAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 0, true);
    }

    #[test]
    fn power_sum_one_malicious_and_one_dropped() {
        let accumulator = PowerSumAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 1, true);
    }

    #[test]
    fn power_sum_one_malicious_and_many_dropped() {
        // validation is much faster than the naive approach
        let accumulator = PowerSumAccumulator::new(1000);
        base_accumulator_test(Box::new(accumulator), 1000, 10, true);
    }

    #[test]
    fn cbf_none_dropped() {
        let accumulator = CBFAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 0, false);
    }

    #[test]
    fn cbf_one_dropped() {
        let accumulator = CBFAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 1, false);
    }

    #[test]
    fn cbf_two_dropped() {
        let accumulator = CBFAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 2, false);
    }

    #[test]
    fn cbf_many_dropped() {
        let accumulator = CBFAccumulator::new(1000);
        base_accumulator_test(Box::new(accumulator), 1000, 10, false);
    }

    #[test]
    fn cbf_one_malicious_and_none_dropped() {
        let accumulator = CBFAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 0, true);
    }

    #[test]
    fn cbf_one_malicious_and_one_dropped() {
        let accumulator = CBFAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 1, true);
    }

    #[test]
    fn cbf_one_malicious_and_many_dropped() {
        // validation is much faster compared to the naive approach,
        // so we increase the number of packets
        let accumulator = CBFAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 1000, 10, true);
    }

    #[test]
    fn iblt_none_dropped() {
        let accumulator = IBLTAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 0, false);
    }

    #[test]
    fn iblt_one_dropped() {
        let accumulator = IBLTAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 1, false);
    }

    #[test]
    fn iblt_many_dropped_without_ilp_solver() {
        let accumulator = IBLTAccumulator::new(1000);
        base_accumulator_test(Box::new(accumulator), 1000, 10, false);
    }

    #[test]
    fn iblt_many_dropped_with_ilp_solver() {
        let accumulator = IBLTAccumulator::new_with_rate(1000, 0.1);
        base_accumulator_test(Box::new(accumulator), 1000, 100, false);
    }

    #[test]
    fn iblt_one_malicious_and_none_dropped() {
        let accumulator = IBLTAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 0, true);
    }

    #[test]
    fn iblt_one_malicious_and_one_dropped() {
        let accumulator = IBLTAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 100, 1, true);
    }

    #[test]
    fn iblt_one_malicious_and_many_dropped() {
        let accumulator = IBLTAccumulator::new(100);
        base_accumulator_test(Box::new(accumulator), 1000, 10, true);
    }
}
