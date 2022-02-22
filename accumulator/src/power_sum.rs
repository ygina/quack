use std::collections::HashMap;
use crate::Accumulator;
use digest::XorDigest;

/// I picked some random prime number in the range [2^32, 2^64] from
/// https://en.wikipedia.org/wiki/List_of_prime_numbers.
/// This one is a Thabit prime, which is not of significance.
const LARGE_PRIME: i64 = 51539607551;

/// The power sum accumulator stores the power sums of all processed elements
/// up to the threshold number of lost elements.
///
/// On validation, the accumulator computes the power sums for the given
/// list of elements, and subtracts the corresponding power sums of the
/// processed elements. The result is a system of polynomial equations for
/// the power sums of the lost elements. We solve this system to find the
/// values of the lost elements, and determine if these are a subset of the
/// given list. If it is, the log is valid. If it is not, or there is no
/// solution, then the log is invalid.
///
/// Note that validation cannot be  performed if the number of lost elements
/// exceeds the threshold. All calculations are done in a finite field, modulo
/// some 2^32 < large prime < 2^64 (the range of possible elements).
pub struct PowerSumAccumulator {
    digest: XorDigest,
    num_elems: usize,
    power_sums: Vec<i64>,
}

/// https://www.geeksforgeeks.org/multiply-large-integers-under-large-modulo/
fn mul_and_mod(mut a: i64, mut b: i64, modulo: i64) -> i64 {
    let mut res = 0;
    while b > 0 {
        if (b & 1) == 1 {
            res = (res + a) % modulo;
        }
        a = (2 * a) % modulo;
        b >>= 1; // b = b / 2
    }
    res
}

impl PowerSumAccumulator {
    pub fn new(threshold: usize) -> Self {
        Self {
            digest: XorDigest::new(),
            num_elems: 0,
            power_sums: (0..threshold).map(|_| 0).collect(),
        }
    }
}

impl Accumulator for PowerSumAccumulator {
    fn process(&mut self, elem: u32) {
        self.digest.add(elem);
        self.num_elems += 1;
        let mut value: i64 = 1;
        for i in 0..self.power_sums.len() {
            value = mul_and_mod(value, elem as i64, LARGE_PRIME);
            self.power_sums[i] = (self.power_sums[i] + value) % LARGE_PRIME;
        }
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
        // Validation cannot be performed if the number of lost elements
        // exceeds the threshold.
        let threshold = self.power_sums.len();
        if elems.len() - self.total() > threshold {
            panic!("number of lost elements exceeds threshold");
        }

        // Calculate the power sums of the given list of elements
        let power_sums = {
            let mut power_sums: Vec<i64> =
                (0..threshold).map(|_| 0).collect();
            for &elem in elems {
                let mut value = 1;
                for i in 0..power_sums.len() {
                    value = mul_and_mod(value, elem as i64, LARGE_PRIME);
                    power_sums[i] = (power_sums[i] + value) % LARGE_PRIME;
                }
            }
            power_sums
        };

        // Find the difference with the power sums of the processed elements
        let power_sums_diff: Vec<i64> = (0..power_sums.len())
            .map(|i| self.power_sums[i] + LARGE_PRIME - power_sums[i])
            .map(|power_sum| power_sum % LARGE_PRIME)
            .collect();

        // Solve the system of equations, and check that a solution exists
        // and that the solution is a subset of the element list.
        let mut elem_count: HashMap<u32, usize> = HashMap::new();
        for &elem in elems {
            let count = elem_count.entry(elem).or_insert(0);
            *count += 1;
        }

        unimplemented!("solve the system of equations");
        // let solutions: Vec<u32> = vec![];
        // for solution in solutions {
        //     let count = elem_count.entry(solution).or_insert(0);
        //     if *count == 0 {
        //         return false;
        //     }
        //     *count -= 1;
        // }
        // true
    }
}
