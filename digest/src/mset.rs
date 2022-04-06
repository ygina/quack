//! Incremental additive multiset hash.
//!
//! Reference: https://people.csail.mit.edu/devadas/pubs/mhashes.pdf
use rand;
use rand::Rng;
use num_bigint::{BigUint, ToBigUint};
use sha3::{Digest, Sha3_256};

/// Incremental additive multiset hash.
pub struct AdditiveMsetHash {
    hash: BigUint,
    count: u32,
    nonce: u32,
}

fn hash_fn(bit: &[u8; 1], val: u32) -> BigUint {
    let mut hasher = Sha3_256::new();
    hasher.update(bit);
    hasher.update(val.to_be_bytes());
    let bytes = &hasher.finalize()[..];
    BigUint::from_bytes_be(bytes)
}

lazy_static! {
    static ref ONE: BigUint = 1_u32.to_biguint().unwrap();
    static ref MOD: BigUint = 2_u64.to_biguint().unwrap().pow(256);
}

/// add two hashes then modulo 2^256
fn add_hashes(a: &BigUint, b: &BigUint) -> BigUint {
    (a + b).modpow(&ONE, &MOD)
}

impl AdditiveMsetHash {
    pub fn new() -> Self {
        let nonce = rand::thread_rng().gen();
        Self {
            hash: hash_fn(b"0", nonce),
            count: 0,
            nonce,
        }
    }

    /// Adds an element to the digest.
    pub fn add(&mut self, elem: u32) {
        let hash = hash_fn(b"1", elem);
        self.hash = add_hashes(&self.hash, &hash);
        // assume no overflow
        self.count += 1;
    }

    /// Adds multiple elements to the digest.
    pub fn add_all(&mut self, elems: &Vec<u32>) {
        for &elem in elems {
            let hash = hash_fn(b"1", elem);
            self.hash = add_hashes(&self.hash, &hash);
        }
        self.count += elems.len() as u32;
    }

    /// Returns the digest hash.
    pub fn value(&self) -> &BigUint {
        &self.hash
    }

    /// Checks if two additive multiset hashes are equal.
    pub fn equals(&self, other: &Self) -> bool {
        if self.count != other.count {
            return false;
        }
        // switched hash_fn sides from the equality check in the paper
        // to ensure the LHS and RHS are positive.
        let lhs = add_hashes(&self.hash, &hash_fn(b"0", other.nonce));
        let rhs = add_hashes(&other.hash, &hash_fn(b"0", self.nonce));
        lhs == rhs
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;
    use super::*;

    fn gen_elements(n: usize) -> Vec<u32> {
        let mut rng = rand::thread_rng();
        (0..n).map(|_| rng.gen()).collect()
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
        digest_b.add(set[0]);
        assert!(!digest_a.equals(&digest_b), "set with same elements does not equal multiset");
        digest_a.add(set[0]);
        assert!(digest_a.equals(&digest_b));
    }

    #[test]
    fn element_order_does_not_matter() {
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
}
