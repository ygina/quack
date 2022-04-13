//! Incremental additive multiset hash.
//!
//! Reference: https://people.csail.mit.edu/devadas/pubs/mhashes.pdf
use rand;
use rand::Rng;
use num_bigint::BigUint;
use serde::{Serialize, Deserialize};
use sha3::{Digest, Sha3_256};

pub const NBYTES_HASH: usize = 32;
pub const NBYTES_NONCE: usize = 16;
type AmhHash = [u8; NBYTES_HASH];
type AmhNonce = [u8; NBYTES_NONCE];

/// Incremental additive multiset hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdditiveMsetHash {
    hash: AmhHash,
    pub count: u32,
    nonce: AmhNonce,
}

fn hash_fn(bit: u8, val: &[u8]) -> AmhHash {
    let mut hash: AmhHash = Default::default();
    let mut hasher = Sha3_256::new();
    hasher.update([bit]);
    hasher.update(val);
    let bytes = hasher.finalize();
    assert!(bytes.len() <= NBYTES_HASH);
    hash.copy_from_slice(&bytes[..]);
    hash
}

/// add two hashes then modulo 2^256
fn add_hashes(a: &AmhHash, b: &AmhHash) -> AmhHash {
    let a = BigUint::from_bytes_le(a);
    let b = BigUint::from_bytes_le(b);
    let c = (a + b).to_bytes_le();
    let mut hash: AmhHash = Default::default();
    let len = std::cmp::min(NBYTES_HASH, c.len());
    hash[..len].copy_from_slice(&c[..len]);
    hash
}

impl AdditiveMsetHash {
    pub fn new_with_seed(nonce: AmhNonce) -> Self {
        Self {
            hash: hash_fn(0, &nonce),
            count: 0,
            nonce,
        }
    }

    pub fn new() -> Self {
        let nonce: AmhNonce = rand::thread_rng().gen();
        Self::new_with_seed(nonce)
    }

    /// Adds an element to the digest.
    pub fn add(&mut self, elem: &BigUint) {
        let hash = hash_fn(1, &elem.to_bytes_be());
        self.hash = add_hashes(&self.hash, &hash);
        // assume no overflow
        self.count += 1;
    }

    /// Adds multiple elements to the digest.
    pub fn add_all(&mut self, elems: &Vec<BigUint>) {
        for elem in elems {
            let hash = hash_fn(1, &elem.to_bytes_be());
            self.hash = add_hashes(&self.hash, &hash);
        }
        self.count += elems.len() as u32;
    }

    /// Returns the digest hash.
    pub fn value(&self) -> &AmhHash {
        &self.hash
    }

    /// Checks if two additive multiset hashes are equal.
    pub fn equals(&self, other: &Self) -> bool {
        if self.count != other.count {
            // println!("count does not match {} != {}", self.count, other.count);
            return false;
        }
        // switched hash_fn sides from the equality check in the paper
        // to ensure the LHS and RHS are positive.
        let lhs = add_hashes(&self.hash, &hash_fn(0, &other.nonce));
        let rhs = add_hashes(&other.hash, &hash_fn(0, &self.nonce));
        lhs == rhs
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::ToBigUint;
    use rand::seq::SliceRandom;
    use super::*;

    fn gen_elements(n: usize) -> Vec<BigUint> {
        let mut rng = rand::thread_rng();
        (0..n).map(|_| rng.gen::<u128>().to_biguint().unwrap()).collect()
    }

    #[test]
    fn default_digests_are_equal() {
        let digest_a = AdditiveMsetHash::new();
        let digest_b = AdditiveMsetHash::new();
        assert!(digest_a.equals(&digest_b));
        assert!(digest_b.equals(&digest_a));
    }

    #[test]
    fn different_elements_produce_different_digests() {
        let set_a = gen_elements(10); //random 10 values
        let set_b = gen_elements(10); //different random 10 values
        assert_ne!(set_a, set_b);

        let mut digest_a = AdditiveMsetHash::new();
        let mut digest_b = AdditiveMsetHash::new();
        digest_a.add_all(&set_a);
        digest_b.add_all(&set_b);
        assert!(!digest_a.equals(&digest_b));
    }

    #[test]
    fn equality_works_with_different_nonces() {
        let set = gen_elements(10); //random 10 values
        let mut digest_a = AdditiveMsetHash::new();
        let mut digest_b = AdditiveMsetHash::new();
        digest_a.add_all(&set);
        digest_b.add_all(&set);
        assert_ne!(digest_a.value(), digest_b.value(), "hashes are different");
        assert!(digest_a.equals(&digest_b), "digests are equivalent");
    }

    #[test]
    fn set_and_multiset_collision() {
        let set = gen_elements(10); //random 10 values
        let mut digest_a = AdditiveMsetHash::new();
        let mut digest_b = AdditiveMsetHash::new();
        digest_a.add_all(&set);
        digest_b.add_all(&set);
        digest_b.add(&set[0]);
        assert!(!digest_a.equals(&digest_b), "set with same elements does not equal multiset");
        digest_a.add(&set[0]);
        assert!(digest_a.equals(&digest_b));
    }

    #[test]
    fn element_order_does_not_matter_random() {
        let set_a = gen_elements(10); //random 10 values
        let set_b = {
            let mut set = set_a.clone(); //randomly shuffle a
            set.shuffle(&mut rand::thread_rng());
            set
        };
        assert_ne!(set_a, set_b);

        let mut digest_a = AdditiveMsetHash::new();
        let mut digest_b = AdditiveMsetHash::new();
        digest_a.add_all(&set_a);
        digest_b.add_all(&set_b);
        assert!(digest_a.equals(&digest_b));
    }

    #[test]
    fn deterministic_hash_fn() {
        assert_eq!(
            hash_fn(0, b"151314873930905896330907615404940191038"),
            hash_fn(0, b"151314873930905896330907615404940191038"));
        assert_eq!(
            hash_fn(1, b"151314873930905896330907615404940191038"),
            hash_fn(1, b"151314873930905896330907615404940191038"));
    }
}
