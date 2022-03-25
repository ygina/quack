//! Incremental additive multiset hash.
//!
//! Reference: https://people.csail.mit.edu/devadas/pubs/mhashes.pdf
use rand;
use rand::Rng;

/// Incremental additive multiset hash.
pub struct AdditiveMsetHash {
    hash: i64,
    count: i64,
    nonce: u32,
}

const L: i64 = 51539607551;  // TODO: parameter
const N: i64 = 51539607551;  // TODO: parameter
fn hash_fn(bit: u32, val: u32) -> i64 {
    // TODO: non-identity hash function?
    (val as i64) | ((bit as i64) << 32)
}

impl AdditiveMsetHash {
    pub fn new() -> Self {
        let nonce = rand::thread_rng().gen();
        Self {
            hash: hash_fn(0, nonce),
            count: 0,
            nonce,
        }
    }

    /// Adds an element to the digest.
    pub fn add(&mut self, elem: u32) {
        let hash = hash_fn(1, elem);
        self.hash = (self.hash + hash) % N;
        // TODO: do we need mod L? maybe we can just assume no overflow
        self.count = (self.count + 1) % L;
    }

    /// Adds multiple elements to the digest.
    pub fn add_all(&mut self, elems: &Vec<u32>) {
        for &elem in elems {
            let hash = hash_fn(1, elem);
            self.hash = (self.hash + hash) % N;
        }
        self.count = (self.count + (elems.len() as i64)) % L;
    }

    /// Returns the digest hash.
    pub fn value(&self) -> i64 {
        self.hash
    }

    /// Checks if two additive multiset hashes are equal.
    pub fn equals(&self, other: &Self) -> bool {
        if self.count != other.count {
            return false;
        }
        // switched hash_fn sides from the equality check in the paper
        // to ensure the LHS and RHS are positive.
        let lhs = (self.hash + hash_fn(0, other.nonce)) % N;
        let rhs = (other.hash + hash_fn(0, self.nonce)) % N;
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
