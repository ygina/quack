use super::HashType;

/// randomMapping generates a sequence of indices indicating the coded symbols
/// that a source symbol should be mapped to. The generator is deterministic,
/// dependent only on its initial PRNG state. When seeded with a uniformly
/// random initial PRNG state, index i will be present in the generated
/// sequence with probability 1/(1+i/2), for any non-negative i.
#[derive(Debug, Copy, Clone)]
pub struct RandomMapping {
    /// PRNG state
    prng: u8,
    /// The last index the symbol was mapped to
    pub last_index: u16,
}

impl RandomMapping {
    pub fn new(prng: HashType) -> Self {
        Self { prng: prng as u8, last_index: 0 }
    }

    /// nextIndex returns the next index in the sequence.
    pub fn next_index(&mut self) -> u16 {
        // Update the PRNG. TODO: prove that the following update rule gives us
        // high quality randomness, assuming the multiplier is coprime to 2^64.
        let r = (self.prng as u16).wrapping_mul(0x58b5);
        self.prng = r as u8;
        // Calculate the difference from the current index (s.lastIdx) to the
        // next index. See the paper for details. We use the approximated form
        //   diff = (1.5+i)((1-u)^(-1/2)-1)
        // where i is the current index, i.e., lastIdx; u is a number uniformly
        // sampled from [0, 1). We apply the following optimization. Notice
        // that our u actually comes from sampling a random uint64 r, and then
        // dividing it by maxUint64, i.e., 1<<64. So we can replace (1-u)^
        // (-1/2) with 1<<32 / sqrt(r).
        let addend = ((self.last_index as f32 + 1.5) *
            ((1u16 << 8) as f32 / ((r+1) as f32).sqrt() - 1.0)).ceil() as u16;
        self.last_index = self.last_index.checked_add(addend).unwrap_or(u16::MAX);
        self.last_index
    }
}
