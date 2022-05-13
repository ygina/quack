#[cfg(not(feature = "disable_validation"))]
use std::collections::{HashSet, HashMap};
#[cfg(not(feature = "disable_validation"))]
use std::time::Instant;

use bincode;
use serde::{Serialize, Deserialize};
#[cfg(not(feature = "disable_validation"))]
use tokio::task;
#[cfg(not(feature = "disable_validation"))]
use tokio::runtime::Builder;
use crate::{Accumulator, ValidationResult};
use digest::{AmhHash, Digest};
#[cfg(not(feature = "disable_validation"))]
use itertools::Itertools;

/// I picked some random prime number in the range [2^32, 2^64] from
/// https://en.wikipedia.org/wiki/List_of_prime_numbers.
/// This one is a Thabit prime, which is not of significance.
const LARGE_PRIME: i64 =  4294967029;
const LARGE_PRIME_U32: u32 =  4294967029;
const LARGE_PRIME_U64: u64 =  4294967029;
const DJB_MASK: u32 = (1 << 31) - 1;

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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerSumAccumulator {
    digest: Digest,
    power_sums: Vec<u32>,
}

#[cfg(not(feature = "disable_validation"))]
#[link(name = "pari", kind = "dylib")]
extern "C" {
    fn find_integer_monic_polynomial_roots_libpari(
        roots: *mut u32,
        coeffs: *const u32,
        field: u32,
        degree: usize,
    ) -> i32;
}

fn add_and_mod(a: u32, b: u32) -> u32 {
    (((a as u64) + (b as u64)) % LARGE_PRIME_U64) as u32
}

fn mul_and_mod(a: u32, b: u32) -> u32 {
    (((a as u64) * (b as u64)) % LARGE_PRIME_U64) as u32
}

// modular division
#[cfg(not(feature = "disable_validation"))]
fn div_and_mod(a: u32, b: u32) -> u32 {
    // divide `a` and `b` by the GCD of `a` and `modulo`
    let mut a = a as i64;
    let mut b = b as i64;
    let gcd = {
        let (mut x, mut y) = if a < b {
            (a, b)
        } else {
            (b, a)
        };
        loop {
            let remainder = y - x * (y / x);
            if remainder == 0 {
                break x;
            }
            y = x;
            x = remainder;
        }
    };
    a /= gcd;
    b /= gcd;
    a %= LARGE_PRIME;
    if b == 1 {
        return a as u32;
    }

    // find the modular multiplicative inverse of b mod modulo
    // ax + by = gcd(a, b)
    let mmi = {
        let (mut old_r, mut r) = (b, LARGE_PRIME);
        let (mut old_x, mut x) = (1, 0);
        let (mut old_y, mut y) = (0, 1);
        while r != 0 {
            let quotient = old_r / r;
            (old_r, r) = (r, old_r - quotient * r);
            (old_x, x) = (x, old_x - quotient * x);
            (old_y, y) = (y, old_y - quotient * y);
        }
        let mut mmi = old_x;
        while mmi < 0 {
            mmi += LARGE_PRIME;
        }
        mmi % LARGE_PRIME
    };

    // return the divided `a` value multiplied by the MMI in the field
    mul_and_mod(a as u32, mmi as u32)
}

#[cfg(not(feature = "disable_validation"))]
async fn calculate_power_sums(elems: &Vec<u32>, num_psums: usize) -> Vec<u32> {
    let ncpus = num_cpus::get();
    let elems_per_thread = elems.len() / ncpus;
    debug!("found {} cpus", ncpus);
    let mut joins = vec![];
    for i in 0..ncpus {
        let lower = i * elems_per_thread;
        let upper = if i == ncpus - 1 {
            elems.len()
        } else {
            (i + 1) * elems_per_thread
        };
        let elems = elems[lower..upper].to_vec();  // TODO: avoid clone
        joins.push(task::spawn(async move {
            let mut power_sums: Vec<u32> = vec![0; num_psums];
            for i in 0..elems.len() {
                let mut value: u32 = 1;
                for j in 0..power_sums.len() {
                    value = mul_and_mod(value, elems[i]);
                    power_sums[j] = add_and_mod(power_sums[j], value);
                }
            }
            power_sums
        }));
    }

    // merge results
    let mut power_sums: Vec<u32> = vec![0; num_psums];
    for join in joins {
        let result = join.await.unwrap();
        for i in 0..num_psums {
            power_sums[i] = add_and_mod(power_sums[i], result[i]);
        }
    }
    power_sums
}

#[cfg(not(feature = "disable_validation"))]
fn calculate_difference(lhs: Vec<u32>, rhs: &Vec<u32>) -> Vec<u32> {
    (0..std::cmp::min(lhs.len(), rhs.len()))
        .map(|i| add_and_mod(lhs[i], LARGE_PRIME_U32 - rhs[i]))
        .collect()
}

// https://en.wikipedia.org/wiki/Newton%27s_identities
//   e0 = 1
//   e1 = e0*p0
// 2*e2 = e1*p0 - e0*p1
// 3*e3 = e2*p0 - e1*p1 + e0*p2
// 4*e4 = e3*p0 - e2*p1 + e1*p2 - e0*p3
// ...
// Returns the coefficients as positive numbers in the field GF(LARGE_PRIME).
#[cfg(not(feature = "disable_validation"))]
fn compute_polynomial_coefficients(p: Vec<u32>) -> Vec<u32> {
    let n = p.len();
    if n == 0 {
        return vec![];
    }
    let mut e: Vec<i64> = vec![1];
    for i in 0..n {
        let mut sum: i64 = 0;
        for j in 0..(i+1) {
            if j & 1 == 0 {
                sum += mul_and_mod(e[i-j] as u32, p[j]) as i64;
            } else {
                sum -= mul_and_mod(e[i-j] as u32, p[j]) as i64;
            }
        }
        while sum < 0 {
            sum += LARGE_PRIME;
        }
        e.push(div_and_mod((sum % LARGE_PRIME) as u32, i as u32 + 1) as i64);
    }
    for i in 0..(n+1) {
        if i & 1 != 0 {
            e[i] *= -1;
            e[i] += LARGE_PRIME;
        }
    }
    // includes the leading coefficient
    e.into_iter().map(|x| x as u32).collect()
}

#[cfg(not(feature = "disable_validation"))]
fn find_integer_monic_polynomial_roots(
    coeffs: Vec<u32>,
) -> Result<Vec<u32>, String> {
    let mut roots: Vec<u32> = vec![0; coeffs.len() - 1];
    if unsafe {
        find_integer_monic_polynomial_roots_libpari(
            roots.as_mut_ptr(),
            coeffs.as_ptr(),
            LARGE_PRIME_U32,
            roots.len(),
        )
    } == 0 {
        Ok(roots)
    } else {
        Err("could not factor polynomial".to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct MiniPowerSumAccumulator {
    hash: AmhHash,       // [u8; HASH_SIZE]
    count: u16,          // expect ~1024 = 2^10
    seed: u64,           // seed for multiset hash, IBLT hash
    power_sums: Vec<u8>, // DJB_HASH_SIZE bits per power sum
}

impl PowerSumAccumulator {
    pub fn new(
        threshold: usize,
        seed: Option<u64>,
    ) -> Self {
        let digest = if let Some(seed) = seed {
            Digest::new_with_seed(seed.to_be_bytes())
        } else {
            Digest::new()
        };
        Self {
            digest,
            power_sums: (0..threshold).map(|_| 0).collect(),
        }
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> Self {
        assert_eq!(bloom_sd::DJB_HASH_SIZE % 8, 0);
        let bytes_per_psum = bloom_sd::DJB_HASH_SIZE / 8;
        let x: MiniPowerSumAccumulator = bincode::deserialize(bytes).unwrap();
        let num_psums = x.power_sums.len() / bytes_per_psum;
        Self {
            digest: Digest {
                hash: x.hash,
                count: x.count as u32,
                nonce: x.seed.to_be_bytes(),
            },
            power_sums: (0..num_psums)
                .map(|i| &x.power_sums[(i * bytes_per_psum)..((i+1) *
                   bytes_per_psum)])
                .map(|b| [b[0], b[1], b[2], b[3]])
                .map(|bytes| u32::from_be_bytes(bytes))
                .collect(),
        }
    }
}

impl Accumulator for PowerSumAccumulator {
    fn to_bytes(&self) -> Vec<u8> {
        assert_eq!(self.digest.count, (self.digest.count as u16) as u32);
        bincode::serialize(&MiniPowerSumAccumulator {
            hash: self.digest.hash,
            count: self.digest.count as u16,
            seed: u64::from_be_bytes(self.digest.nonce),
            power_sums: self.power_sums.iter()
                .flat_map(|psum| psum.to_be_bytes()).collect(),
        }).unwrap()
    }

    fn reset(&mut self) {
        self.digest = Digest::new();
        self.power_sums = vec![0; self.power_sums.len()];
    }

    fn process(&mut self, elem: &[u8]) {
        self.digest.add(elem);
        let mut value: u32 = 1;
        let elem_u32 = bloom_sd::elem_to_u32(elem) & DJB_MASK;
        for i in 0..self.power_sums.len() {
            value = mul_and_mod(value, elem_u32);
            self.power_sums[i] = add_and_mod(self.power_sums[i], value);
        }
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
    fn validate(&self, _elems: &Vec<Vec<u8>>) -> ValidationResult {
        panic!("validation not enabled")
    }

    #[cfg(not(feature = "disable_validation"))]
    fn validate(&self, elems: &Vec<Vec<u8>>) -> ValidationResult {
        if self.total() == 0 {
            warn!("no elements received, valid by default");
            return ValidationResult::Valid;
        }
        // The number of power sum equations we need is equal to
        // the number of lost elements. Validation cannot be performed
        // if this number exceeds the threshold.
        if elems.len() < self.total() {
            warn!("more elements received than logged");
            return ValidationResult::Invalid;
        }
        let n_values = elems.len() - self.total();
        let threshold = self.power_sums.len();
        if n_values > threshold {
            return ValidationResult::PsumExceedsThreshold;
        }

        // If no elements are missing, just recalculate the digest.
        if n_values == 0 {
            let mut digest = Digest::new();
            for elem in elems {
                digest.add(elem);
            }
            return if digest.equals(&self.digest) {
                ValidationResult::Valid
            } else {
                ValidationResult::Invalid
            };
        }

        // Calculate the power sums of the given list of elements.
        // Find the difference with the power sums of the processed elements.
        let t1 = Instant::now();
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        let elems_u32: Vec<u32> = elems.iter()
            .map(|elem| bloom_sd::elem_to_u32(elem) & DJB_MASK)
            .collect();
        let power_sums = rt.block_on(async {
            calculate_power_sums(&elems_u32, n_values).await
        }).into_iter().collect();
        let t2 = Instant::now();
        debug!("calculated power sums: {:?}", t2 - t1);
        let power_sums_diff = calculate_difference(power_sums, &self.power_sums
            .iter().map(|x| *x as u32).collect());
        let t3 = Instant::now();
        debug!("calculated power sum difference: {:?}", t3 - t2);

        // Solve the system of equations.
        let coeffs = compute_polynomial_coefficients(
            power_sums_diff[..n_values].to_vec());
        let t4 = Instant::now();
        debug!("computed polynomial coefficients: {:?}", t4 - t3);
        let roots = {
            let roots = find_integer_monic_polynomial_roots(coeffs);
            let t5 = Instant::now();
            debug!("found integer monic polynomial roots: {:?}", t5 - t4);
            match roots {
                Ok(roots) => roots,
                Err(_) => {
                    return ValidationResult::PsumErrorFindingRoots;
                },
            }
        };

        // This technique gives a single deterministic solution.
        // If the solutions are indeed packets in the element list, and
        // calculating the digest from the element list with those packets
        // removed yields the same digest, then verification succeeds.
        let t5 = Instant::now();
        // Map from u32 root to multiplicity.
        let dropped_counts: HashMap<u32, usize> = {
            let mut map = HashMap::new();
            for root in roots {
                let root = u32::try_from(root);
                if root.is_err() {
                    // root is not in the packet domain
                    return ValidationResult::Invalid;
                }
                let count = map.entry(root.unwrap()).or_insert(0);
                *count += 1;
            }
            map
        };

        let mut digest = Digest::new();
        let mut collisions: HashMap<u32, Vec<Vec<u8>>> = HashMap::new();
        for elem in elems {
            let elem_u32 = bloom_sd::elem_to_u32(elem) & DJB_MASK;
            if !dropped_counts.contains_key(&elem_u32) {
                // If an element in the log doesn't hash to a u32 root,
                // it wasn't dropped, so add it to the digest.
                digest.add(elem);
            } else {
                // Otherwise collect every element that maps to a u32 root.
                collisions.entry(elem_u32).or_insert(vec![]).push(elem.clone());
            }
        }
        let t6 = Instant::now();
        debug!("created dropped_counts and collisions maps: {:?}", t6 - t5);

        let mut combinations = vec![];
        let mut dropped = 0;
        for (elem_u32, &dropped_count) in dropped_counts.iter() {
            if let Some(elems) = collisions.get(&elem_u32) {
                if dropped_count == elems.len() {
                    // they are all dropped
                    dropped += dropped_count;
                } else if dropped_count > elems.len() {
                    error!("more elements dropped than exist candidates");
                    return ValidationResult::Invalid;
                } else {
                    let received_count = elems.len() - dropped_count;
                    if elems.iter().collect::<HashSet<_>>().len() == 1 {
                        // only one unique element so it was dropped
                        digest.add_all(&elems[..received_count].to_vec());
                        dropped += dropped_count;
                        continue;
                    }
                    warn!("{} elems for {} slots", elems.len(), dropped_count);
                    // Narrow down the combinations we need to try.
                    let mut map = HashMap::new();
                    for elem in elems {
                        let entry = map.entry(elem).or_insert(0);
                        if *entry < dropped_count {
                            *entry += 1;
                        } else {
                            // By Pigeonhole it couldn't have been dropped
                            digest.add(elem);
                        }
                    }
                    combinations.push(map.into_iter()
                        .flat_map(|(elem, count)| vec![elem.clone(); count])
                        .collect::<Vec<Vec<u8>>>()
                        .into_iter()
                        .combinations(received_count));
                    dropped += dropped_count;
                }
            } else {
                error!("dropped element does not exist in log: {}", elem_u32);
                return ValidationResult::Invalid;
            }
        }
        let t7 = Instant::now();
        debug!("prepared combos for resolving djb collisions: {:?}", t7 - t6);

        debug!("accounted for {} dropped elements", dropped);
        let mut n_digests = 0;
        for curr in combinations.into_iter().multi_cartesian_product() {
            // check curr, if good return it else return None
            let mut digest = digest.clone();
            for elem in curr.into_iter().flat_map(|val| val) {
                digest.add(&elem);
            }
            n_digests += 1;
            if digest.equals(&self.digest) {
                return ValidationResult::PsumCollisionsValid;
            }
        }
        let t8 = Instant::now();
        debug!("recalculated {} digests: {:?}", n_digests, t8 - t7);
        if digest.equals(&self.digest) {
            ValidationResult::Valid
        } else if n_digests == 0 {
            ValidationResult::Invalid
        } else {
            ValidationResult::PsumCollisionsInvalid
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bincode;
    use rand;
    use rand::Rng;

    const NBYTES: usize = 16;

    fn gen_elems(n: usize) -> Vec<Vec<u8>> {
        let mut rng = rand::thread_rng();
        (0..n).map(|_| (0..NBYTES).map(|_| rng.gen::<u8>()).collect()).collect()
    }

    #[test]
    fn test_not_equals() {
        let acc1 = PowerSumAccumulator::new(100, None);
        let acc2 = PowerSumAccumulator::new(100, None);
        assert_ne!(acc1, acc2, "different digest nonce");
    }

    #[test]
    fn bincode_empty_serialization() {
        let acc1 = PowerSumAccumulator::new(100, None);
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: PowerSumAccumulator = bincode::deserialize(&bytes).unwrap();
        assert_eq!(acc1, acc2);
    }

    #[test]
    fn bincode_serialization_with_data() {
        let mut acc1 = PowerSumAccumulator::new(100, None);
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc2: PowerSumAccumulator = bincode::deserialize(&bytes).unwrap();
        acc1.process_batch(&gen_elems(10));
        let bytes = bincode::serialize(&acc1).unwrap();
        let acc3: PowerSumAccumulator = bincode::deserialize(&bytes).unwrap();
        assert_ne!(acc1, acc2);
        assert_eq!(acc1, acc3);
    }

    #[test]
    fn empty_serialization() {
        let acc1 = PowerSumAccumulator::new(100, None);
        let acc2 = PowerSumAccumulator::from_bytes(&acc1.to_bytes());
        assert_eq!(acc1, acc2);
    }

    #[test]
    fn serialization_with_data() {
        let mut acc1 = PowerSumAccumulator::new(100, None);
        let acc2 = PowerSumAccumulator::from_bytes(&acc1.to_bytes());
        acc1.process_batch(&gen_elems(10));
        let acc3 = PowerSumAccumulator::from_bytes(&acc1.to_bytes());
        assert_ne!(acc1, acc2);
        assert_eq!(acc1, acc3);
    }

    #[test]
    fn test_mul_and_mod() {
        // 4294967029
        assert_eq!(mul_and_mod(429496702, 4), 1717986808, "no overflow");
        assert_eq!(mul_and_mod(429496702, 12), 858993395, "overflow");
    }

    #[test]
    fn test_div_and_mod() {
        assert_eq!(div_and_mod(1717986808, 429496702), 4);
        assert_eq!(div_and_mod(858993395, 429496702), 12);
    }

    #[tokio::test]
    async fn test_calculate_power_sums() {
        assert_eq!(calculate_power_sums(&vec![2, 3, 5], 2).await, vec![10, 38]);
        assert_eq!(calculate_power_sums(&vec![2, 3, 5], 3).await, vec![10, 38, 160]);
        let one_large_num = calculate_power_sums(&vec![294967295], 3).await;
        assert_eq!(one_large_num, vec![294967295, 2507781770, 2201765005]);
        let two_large_nums = calculate_power_sums(&vec![294967295, 2294967295], 3).await;
        assert_eq!(two_large_nums, vec![2589934590, 1563208361, 4070406309]);
    }

    #[test]
    fn test_calculate_difference() {
        let diff = calculate_difference(vec![2, 3, 4], &vec![1, 2, 3]);
        assert_eq!(diff, vec![1, 1, 1]);
        let diff = calculate_difference(vec![2, 3, 4], &vec![1, 2, 3, 4]);
        assert_eq!(diff, vec![1, 1, 1]);
        let overflow_diff = calculate_difference(vec![1], &vec![2]);
        assert_eq!(overflow_diff, vec![4294967028]);
    }

    #[tokio::test]
    async fn test_compute_polynomial_coefficients_small_numbers() {
        let x = vec![2, 3, 5];
        let power_sums_diff = calculate_power_sums(&x, 3).await;
        assert_eq!(power_sums_diff, vec![10, 38, 160]);
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        assert_eq!(coeffs, vec![1, LARGE_PRIME_U32-10, 31, LARGE_PRIME_U32-30]);
    }

    #[tokio::test]
    async fn test_compute_polynomial_coefficients_large_numbers() {
        let x = vec![4294966796, 3987231002];
        let power_sums_diff = calculate_power_sums(&x, 2).await;
        assert_eq!(power_sums_diff, vec![3987230769, 3419665331]);
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let e1 = add_and_mod(x[0], x[1]);
        let e2 = mul_and_mod(x[0], x[1]);
        assert_eq!(coeffs, vec![1, LARGE_PRIME_U32-e1, e2]);
    }

    #[tokio::test]
    async fn test_find_integer_monic_polynomial_roots_small_numbers() {
        let x = vec![2, 3, 5];
        let power_sums_diff = calculate_power_sums(&x, x.len()).await;
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let mut roots = {
            let roots = find_integer_monic_polynomial_roots(coeffs);
            assert!(roots.is_ok());
            roots.unwrap()
        };
        roots.sort();
        assert_eq!(roots, x.into_iter().map(|x| x).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn test_find_integer_monic_polynomial_roots_large_numbers() {
        let x = vec![3987231002, 4294966796];
        let power_sums_diff = calculate_power_sums(&x, x.len()).await;
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let mut roots = {
            let roots = find_integer_monic_polynomial_roots(coeffs);
            assert!(roots.is_ok());
            roots.unwrap()
        };
        roots.sort();
        assert_eq!(roots, x.into_iter().map(|x| x).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn test_find_integer_monic_polynomial_roots_multiplicity() {
        let x = vec![3987231002, 4294966796, 4294966796, 4294966796];
        let power_sums_diff = calculate_power_sums(&x, x.len()).await;
        let coeffs = compute_polynomial_coefficients(power_sums_diff);
        let mut roots = {
            let roots = find_integer_monic_polynomial_roots(coeffs);
            assert!(roots.is_ok());
            roots.unwrap()
        };
        roots.sort();
        assert_eq!(roots, x.into_iter().map(|x| x).collect::<Vec<_>>());
    }

    #[test]
    fn test_find_integer_monic_polynomial_roots_no_solution() {
        let coeffs = vec![1, 479202874, 1224376254, 3930719704];
        let roots = find_integer_monic_polynomial_roots(coeffs);
        assert!(roots.is_err());
    }
}
