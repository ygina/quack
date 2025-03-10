use super::HashType;

/// randomMapping generates a sequence of indices indicating the coded symbols
/// that a source symbol should be mapped to. The generator is deterministic,
/// dependent only on its initial PRNG state. When seeded with a uniformly
/// random initial PRNG state, index i will be present in the generated
/// sequence with probability 1/(1+i/2), for any non-negative i.
#[derive(Debug, Copy, Clone)]
pub struct RandomMapping {
    /// PRNG state
    prng: HashType,
    /// The last index the symbol was mapped to
    pub last_index: u64,
}

impl RandomMapping {
    pub fn new(prng: HashType) -> Self {
        Self { prng, last_index: 0 }
    }

    /// nextIndex returns the next index in the sequence.
    pub fn next_index(&mut self) -> u64 {
        // Update the PRNG. TODO: prove that the following update rule gives us
        // high quality randomness, assuming the multiplier is coprime to 2^64.
        let r = (self.prng as u64).wrapping_mul(0xda942042e4dd58b5);
        self.prng = r as HashType;
        // Calculate the difference from the current index (s.lastIdx) to the
        // next index. See the paper for details. We use the approximated form
        //   diff = (1.5+i)((1-u)^(-1/2)-1)
        // where i is the current index, i.e., lastIdx; u is a number uniformly
        // sampled from [0, 1). We apply the following optimization. Notice
        // that our u actually comes from sampling a random uint64 r, and then
        // dividing it by maxUint64, i.e., 1<<64. So we can replace (1-u)^
        // (-1/2) with 1<<32 / sqrt(r).
        self.last_index += ((self.last_index as f64 + 1.5) *
            ((1u64 << 32) as f64 / ((r+1) as f64).sqrt() - 1.0)).ceil() as u64;
        self.last_index
    }
}
