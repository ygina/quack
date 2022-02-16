/// Digest that is the bit-XOR of a collection of 32-bit numbers.
///
/// This is not a multiset hash, and cannot distinguish between different
/// even or odd quantities of the same element.
#[derive(PartialEq, Eq, )]
pub struct XorDigest {
    hash: u32,
}

impl XorDigest {
    pub fn new() -> Self {
        Self { hash: 0 }
    }

    /// Adds an element to the digest.
    pub fn add(&mut self, elem: u32) {
        self.hash ^= elem;
    }

    /// Adds multiple elements to the digest.
    pub fn add_all(&mut self, elems: &Vec<u32>) {
        for &elem in elems {
            self.add(elem)
        }
    }

    /// Returns the digest value.
    pub fn value(&self) -> u32 {
        self.hash
    }
}

#[cfg(test)]
mod tests {
    use rand;
    use rand::Rng;
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

        let mut digest_a = XorDigest::new();
        let mut digest_b = XorDigest::new();
        digest_a.add_all(&set_a);
        digest_b.add_all(&set_b);
        assert_ne!(digest_a.value(), digest_b.value());
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

        let mut digest_a = XorDigest::new();
        let mut digest_b = XorDigest::new();
        digest_a.add_all(&set_a);
        digest_b.add_all(&set_b);
        assert_eq!(digest_a.value(), digest_b.value());
    }
}
