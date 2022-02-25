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

#[link(name = "gmp", kind = "dylib")]
extern "C" {
    fn compute_polynomial_coefficients_wrapper(
        coeffs: *mut i64,
        power_sums: *const i64,
        n_values: usize,
    );

    fn find_integer_monic_polynomial_roots_wrapper(
        roots: *mut i64,
        coeffs: *mut i64,
        degree: usize,
    );
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

fn calculate_power_sums(elems: &Vec<u32>, n_values: usize) -> Vec<i64> {
    let mut power_sums: Vec<i64> = vec![0; n_values];
    for &elem in elems {
        let mut value = 1;
        for i in 0..power_sums.len() {
            value = mul_and_mod(value, elem as i64, LARGE_PRIME);
            power_sums[i] = (power_sums[i] + value) % LARGE_PRIME;
        }
    }
    power_sums
}

fn calculate_difference(lhs: Vec<i64>, rhs: &Vec<i64>) -> Vec<i64> {
    (0..lhs.len())
        .map(|i| lhs[i] + LARGE_PRIME - rhs[i])
        .map(|power_sum| power_sum % LARGE_PRIME)
        .collect()
}

fn compute_polynomial_coefficients(power_sums_diff: Vec<i64>) -> Vec<i64> {
    let n_values = power_sums_diff.len();
    let mut coeffs: Vec<i64> = vec![0; n_values];
    unsafe {
        compute_polynomial_coefficients_wrapper(
            coeffs.as_mut_ptr(),
            power_sums_diff.as_ptr(),
            n_values,
        );
    }
    coeffs
}

fn find_integer_monic_polynomial_roots(mut coeffs: Vec<i64>) -> Vec<i64> {
    let mut roots: Vec<i64> = vec![0; coeffs.len()];
    unsafe {
        find_integer_monic_polynomial_roots_wrapper(
            roots.as_mut_ptr(),
            coeffs.as_mut_ptr(),
            roots.len(),
        );
    }
    roots
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
        // The number of power sum equations we need is equal to
        // the number of lost elements. Validation cannot be performed
        // if this number exceeds the threshold.
        let n_values = elems.len() - self.total();
        let threshold = self.power_sums.len();
        if n_values > threshold {
            panic!("number of lost elements exceeds threshold");
        }

        // Calculate the power sums of the given list of elements.
        // Find the difference with the power sums of the processed elements.
        // Solve the system of equations.
        let power_sums = calculate_power_sums(elems, n_values);
        let power_sums_diff = calculate_difference(power_sums, &self.power_sums);
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let roots = find_integer_monic_polynomial_roots(coeffs);

        // Check that a solution exists and that the solution is a subset of
        // the element list.
        let mut elem_count: HashMap<u32, usize> = HashMap::new();
        for &elem in elems {
            let count = elem_count.entry(elem).or_insert(0);
            *count += 1;
        }
        for root in roots {
            let root = u32::try_from(root);
            if root.is_err() {
                return false;
            }
            let count = elem_count.entry(root.unwrap()).or_insert(0);
            if *count == 0 {
                return false;
            }
            *count -= 1;
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mul_and_mod() {
        assert_eq!(mul_and_mod(2, 3, 10), 6);
        assert_eq!(mul_and_mod(2, 4, 10), 8);
        assert_eq!(mul_and_mod(2, 3, 5), 1);
        assert_eq!(mul_and_mod(2, 4, 5), 3);
    }

    #[test]
    fn test_calculate_power_sums() {
        assert_eq!(calculate_power_sums(&vec![2, 3, 5], 2), vec![10, 38]);
        assert_eq!(calculate_power_sums(&vec![2, 3, 5], 3), vec![10, 38, 160]);
        let one_large_num = calculate_power_sums(&vec![4294967295], 3);
        assert_eq!(one_large_num, vec![4294967295, 8947848534, 17567609286]);
        let two_large_nums = calculate_power_sums(&vec![4294967295, 2294967295], 3);
        assert_eq!(two_large_nums, vec![6589934590, 32873368637, 30483778854]);
    }

    #[test]
    fn test_calculate_difference() {
        let diff = calculate_difference(vec![2, 3, 4], &vec![1, 2, 3]);
        assert_eq!(diff, vec![1, 1, 1]);
        let diff = calculate_difference(vec![2, 3, 4], &vec![1, 2, 3, 4]);
        assert_eq!(diff, vec![1, 1, 1]);
        let overflow_diff = calculate_difference(vec![1], &vec![2]);
        assert_eq!(overflow_diff, vec![51539607550]);
    }

    #[test]
    fn test_compute_polynomial_coefficients_small_numbers() {
        let x = vec![2, 3, 5];
        let power_sums_diff = calculate_power_sums(&x, 3);
        assert_eq!(power_sums_diff, vec![10, 38, 160]);
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        assert_eq!(coeffs, vec![-10, 31, -30]);
    }

    #[test]
    fn test_compute_polynomial_coefficients_large_numbers() {
        let x = vec![4294966796, 3987231002];
        let power_sums_diff = calculate_power_sums(&x, 2);
        assert_eq!(power_sums_diff, vec![8282197798, 20796235250]);
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let e1 = (x[0] as i64) + (x[1] as i64) % LARGE_PRIME;
        let e2 = mul_and_mod(x[0] as i64, x[1] as i64, LARGE_PRIME);
        assert_eq!(coeffs, vec![-e1, e2]);
    }

    #[test]
    fn test_find_integer_monic_polynomial_roots_small_numbers() {
        let x = vec![2, 3, 5];
        let power_sums_diff = calculate_power_sums(&x, x.len());
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let mut roots = find_integer_monic_polynomial_roots(coeffs);
        roots.sort();
        assert_eq!(roots, x.into_iter().map(|x| x as i64).collect::<Vec<_>>());
    }

    #[ignore]
    #[test]
    fn test_find_integer_monic_polynomial_roots_large_numbers() {
        let x = vec![4294966796, 3987231002];
        let power_sums_diff = calculate_power_sums(&x, x.len());
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let mut roots = find_integer_monic_polynomial_roots(coeffs);
        roots.sort();
        assert_eq!(roots, x.into_iter().map(|x| x as i64).collect::<Vec<_>>());
    }
}
