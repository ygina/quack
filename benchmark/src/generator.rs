use std::collections::HashSet;
use rand::{self, SeedableRng, Rng, RngCore};
use rand_chacha::ChaCha12Rng;

const NBYTES: usize = 16;
pub const MALICIOUS_ELEM: [u8; NBYTES] = [0; NBYTES];

fn gen_elem(rng: &mut Box<dyn RngCore>) -> Vec<u8> {
    (0..NBYTES).map(|_| rng.gen::<u8>()).collect()
}

pub struct SeedGenerator {
    rng: Option<ChaCha12Rng>,
}

impl SeedGenerator {
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            rng: seed.map(|seed| ChaCha12Rng::seed_from_u64(seed)),
        }
    }

    pub fn next(&mut self) -> Option<u64> {
        self.rng.as_mut().map(|rng| rng.gen())
    }
}

pub trait LoadGeneratorInner {
    fn log(&self) -> &Vec<Vec<u8>>;
    fn num_logged(&self) -> usize;
    fn num_dropped(&self) -> usize;
}

pub struct LoadGeneratorProb {
    /// The logged packets. All elements are in the range [0, MALICIOUS_ELEM).
    log: Vec<Vec<u8>>,
    /// Probability that a logged packet is dropped.
    p_dropped: f32,
    /// The index of the malicious packet, if the router is malicious
    /// (MALICIOUS_ELEM is sent in place of the packet logged at this index).
    malicious_i: Option<usize>,

    /// The number of logged elements.
    num_logged: usize,
    /// The number of dropped elements.
    num_dropped: usize,
    /// The current index in the log.
    index: usize,
    /// Random number generator.
    rng: Box<dyn RngCore>,
}

pub struct LoadGeneratorExact {
    /// The logged packets. All elements are in the range [0, MALICIOUS_ELEM).
    log: Vec<Vec<u8>>,
    /// Indexes of the packets that are dropped.
    dropped_is: HashSet<usize>,
    /// The index of the malicious packet, if the router is malicious
    /// (MALICIOUS_ELEM is sent in place of the packet logged at this index).
    malicious_i: Option<usize>,

    /// The number of logged elements.
    num_logged: usize,
    /// The number of dropped elements.
    num_dropped: usize,
    /// The current index in the log.
    index: usize,
}

impl LoadGeneratorProb {
    /// Create a load generator for 32-bit integers.
    ///
    /// The router logs `num_logged` packets, where `p_dropped` is the
    /// proportion of packets that are dropped en route to the ISP.
    /// If the router is `malicious`, it will send a single packet with
    /// MALICIOUS_ELEM as the value, instead of the value that is logged.
    /// The index of this packet is randomly chosen, and will always be
    /// sent even if p_dropped is 1. The iterator of the load generator
    /// will generate all packets the ISP actually receives.
    pub fn new(
        seed: Option<u64>,
        num_logged: usize,
        p_dropped: f32,
        malicious: bool,
    ) -> Self {
        let mut rng: Box<dyn RngCore> = if let Some(seed) = seed {
            Box::new(ChaCha12Rng::seed_from_u64(seed))
        } else {
            Box::new(rand::thread_rng())
        };
        let log: Vec<Vec<u8>> = (0..num_logged).map(|_| loop {
            let elem = gen_elem(&mut rng);
            if elem != MALICIOUS_ELEM {
                break elem;
            }
        }).collect();
        let malicious_i: Option<usize> = if malicious {
            Some(rng.gen_range(0..num_logged))
        } else {
            None
        };

        Self {
            log,
            p_dropped,
            malicious_i,
            num_logged,
            num_dropped: 0,
            index: 0,
            rng,
        }
    }
}

impl LoadGeneratorInner for LoadGeneratorProb {
    fn log(&self) -> &Vec<Vec<u8>> {
        &self.log
    }

    fn num_logged(&self) -> usize {
        self.num_logged
    }

    fn num_dropped(&self) -> usize {
        self.num_dropped
    }
}

impl LoadGeneratorExact {
    /// Create a load generator for 32-bit integers.
    ///
    /// The router logs `num_logged` packets, where `p_dropped` is the
    /// proportion of packets that are dropped en route to the ISP.
    /// If the router is `malicious`, it will send a single packet with
    /// MALICIOUS_ELEM as the value, instead of the value that is logged.
    /// The index of this packet is randomly chosen, and will always be
    /// sent even if p_dropped is 1. The iterator of the load generator
    /// will generate all packets the ISP actually receives.
    pub fn new(
        seed: Option<u64>,
        num_logged: usize,
        n_dropped: usize,
        malicious: bool,
    ) -> Self {
        let mut rng: Box<dyn RngCore> = if let Some(seed) = seed {
            Box::new(ChaCha12Rng::seed_from_u64(seed))
        } else {
            Box::new(rand::thread_rng())
        };
        let log: Vec<Vec<u8>> = (0..num_logged).map(|_| loop {
            let elem = gen_elem(&mut rng);
            if elem != MALICIOUS_ELEM {
                break elem;
            }
        }).collect();
        let malicious_i: Option<usize> = if malicious {
            assert!(n_dropped + 1 <= num_logged);
            Some(rng.gen_range(0..num_logged))
        } else {
            assert!(n_dropped <= num_logged);
            None
        };
        let mut dropped_is = HashSet::new();
        while dropped_is.len() < n_dropped {
            let dropped_i = rng.gen_range(0..num_logged);
            if malicious_i == Some(dropped_i) {
                continue;
            }
            dropped_is.insert(dropped_i);
        }

        Self {
            log,
            dropped_is,
            malicious_i,
            num_logged,
            num_dropped: 0,
            index: 0,
        }
    }
}

impl LoadGeneratorInner for LoadGeneratorExact {
    fn log(&self) -> &Vec<Vec<u8>> {
        &self.log
    }

    fn num_logged(&self) -> usize {
        self.num_logged
    }

    fn num_dropped(&self) -> usize {
        self.num_dropped
    }
}

impl Iterator for LoadGeneratorProb {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Only allowed to iterate through the elements once since we
            // don't store the indexes of dropped packets (we could)
            if self.index >= self.log.len() {
                return None;
            }
            // Update the index
            self.index += 1;
            // Send MALICIOUS_ELEM if we are on the malicious index
            if let Some(malicious_i) = self.malicious_i {
                if malicious_i == self.index - 1 {
                    return Some(MALICIOUS_ELEM.to_vec());
                }
            }
            // Continue until the packet is not dropped
            let dropped = self.rng.gen::<f32>() < self.p_dropped;
            if dropped {
                self.num_dropped += 1;
            } else {
                return Some(self.log[self.index - 1].clone());
            }
        }
    }
}

impl Iterator for LoadGeneratorExact {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Only allowed to iterate through the elements once since we
            // don't store the indexes of dropped packets (we could)
            if self.index >= self.log.len() {
                return None;
            }
            // Update the index
            self.index += 1;
            // Send MALICIOUS_ELEM if we are on the malicious index
            if let Some(malicious_i) = self.malicious_i {
                if malicious_i == self.index - 1 {
                    return Some(MALICIOUS_ELEM.to_vec());
                }
            }
            // Continue until the packet is not dropped
            if self.dropped_is.contains(&(self.index - 1)) {
                self.num_dropped += 1;
            } else {
                return Some(self.log[self.index - 1].clone());
            }
        }
    }
}

pub trait LoadGenerator: Iterator + LoadGeneratorInner {}

impl LoadGenerator for LoadGeneratorProb {}
impl LoadGenerator for LoadGeneratorExact {}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: Option<u64> = Some(1234);
    const NUM_LOGGED: usize = 10000;

    #[test]
    fn no_elements_are_dropped() {
        let mut g = LoadGeneratorProb::new(SEED, NUM_LOGGED, 0.0, false);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert_eq!(g.num_dropped, 0);
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
    }

    #[test]
    fn all_elements_are_dropped() {
        let mut g = LoadGeneratorProb::new(SEED, NUM_LOGGED, 1.0, false);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert_eq!(g.num_dropped, NUM_LOGGED);
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
    }

    #[test]
    fn some_elements_are_dropped() {
        let mut g = LoadGeneratorProb::new(SEED, NUM_LOGGED, 0.5, false);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert!(g.num_dropped > 0); //with high probability
        assert!(g.num_dropped < NUM_LOGGED); //with high probability
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
    }

    #[test]
    fn malicious_element_is_generated_but_not_logged() {
        let mut g = LoadGeneratorProb::new(SEED, NUM_LOGGED, 0.5, true);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert!(g.num_dropped > 0); //with high probability
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
        assert!(!g.log.contains(&MALICIOUS_ELEM.to_vec()));
        assert!(processed.contains(&MALICIOUS_ELEM.to_vec()));
    }

    #[test]
    fn malicious_element_is_not_dropped() {
        let mut g = LoadGeneratorProb::new(SEED, NUM_LOGGED, 1.0, true);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.num_dropped, NUM_LOGGED - 1);
        assert_eq!(processed.len(), 1);
    }

    #[test]
    fn seed_load_generator() {
        let mut g1 = LoadGeneratorProb::new(SEED, NUM_LOGGED, 0.0, false);
        let mut g2 = LoadGeneratorProb::new(SEED, NUM_LOGGED, 0.0, false);
        let mut g3 = LoadGeneratorProb::new(None, NUM_LOGGED, 0.0, false);
        let (mut p1, mut p2, mut p3) = (vec![], vec![], vec![]);
        while let Some(e) = g1.next() {
            p1.push(e);
        }
        while let Some(e) = g2.next() {
            p2.push(e);
        }
        while let Some(e) = g3.next() {
            p3.push(e);
        }
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_seed_generator() {
        let mut g = SeedGenerator::new(None);
        assert!(g.next().is_none());
        assert!(g.next().is_none());
        assert!(g.next().is_none());
        let mut g = SeedGenerator::new(Some(1));
        let s1 = g.next();
        let s2 = g.next();
        assert!(s1.is_some());
        assert!(s2.is_some());
        assert_ne!(s1, s2);
    }

    #[test]
    fn no_elements_are_dropped_exact() {
        let mut g = LoadGeneratorExact::new(SEED, NUM_LOGGED, 0, false);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert_eq!(g.num_dropped, 0);
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
    }

    #[test]
    fn all_elements_are_dropped_exact() {
        let mut g = LoadGeneratorExact::new(SEED, NUM_LOGGED, NUM_LOGGED,
           false);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert_eq!(g.num_dropped, NUM_LOGGED);
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
    }

    #[test]
    fn some_elements_are_dropped_exact() {
        let mut g = LoadGeneratorExact::new(SEED, NUM_LOGGED, 100, false);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert_eq!(g.num_dropped, 100);
        assert_eq!(processed.len(), g.num_logged - g.num_dropped);
    }

    #[test]
    fn malicious_element_is_generated_but_not_logged_exact() {
        let mut g = LoadGeneratorExact::new(SEED, NUM_LOGGED, 100, true);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.log.len(), g.num_logged);
        assert_eq!(g.num_logged, NUM_LOGGED);
        assert_eq!(g.num_dropped, 100);
        assert!(!g.log.contains(&MALICIOUS_ELEM.to_vec()));
        assert!(processed.contains(&MALICIOUS_ELEM.to_vec()));
    }

    #[test]
    fn malicious_element_is_not_dropped_exact() {
        let mut g = LoadGeneratorExact::new(SEED, 101, 100, true);
        let mut processed = vec![];
        while let Some(elem) = g.next() {
            processed.push(elem);
        }
        assert_eq!(g.num_dropped, 100);
        assert_eq!(processed.len(), 1);
    }

    #[test]
    fn seed_load_generator_exact() {
        let mut g1 = LoadGeneratorExact::new(SEED, NUM_LOGGED, 0, false);
        let mut g2 = LoadGeneratorExact::new(SEED, NUM_LOGGED, 0, false);
        let mut g3 = LoadGeneratorExact::new(None, NUM_LOGGED, 0, false);
        let (mut p1, mut p2, mut p3) = (vec![], vec![], vec![]);
        while let Some(e) = g1.next() {
            p1.push(e);
        }
        while let Some(e) = g2.next() {
            p2.push(e);
        }
        while let Some(e) = g3.next() {
            p3.push(e);
        }
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }
}
