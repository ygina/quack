//! The IBLT quACK implementation is adapted from the paper Practical Rateless
//! Set Reconciliation by Lei Yang, Yossi Gilad, and Mohammad Alizadeh, which
//! appeared in ACM SIGCOMM 2024. A modified version of the original RIBLT
//! implementation by the authors of that papaer and a comparison to a power
//! sum quACK implementation in Go is available here:
//! https://github.com/ygina/subset-reconciliation/.
use super::HashType;

pub struct IBLTQuackU32 {
}

impl IBLTQuackU32 {
    /// Create a new IBLT quACK.
    pub fn new(num_symbols: usize) -> Self {
        Self {
        }
    }

    /// Insert an element in the quACK.
    pub fn insert(&mut self, value: HashType) {
        unimplemented!()
    }

    /// Remove an element in the quACK. Does not validate that the element
    /// had actually been inserted in the quACK.
    pub fn remove(&mut self, value: HashType) {
        unimplemented!()
    }

    /// Subtracts another power sum quACK from this power sum quACK.
    pub fn sub_assign(&mut self, rhs: &Self) {
        unimplemented!()
    }

    /// Similar to [sub_assign](trait.IBLTQuackU32.html#method.sub_assign)
    /// but returns the difference as a new quACK.
    pub fn sub(self, rhs: Self) -> Self {
        unimplemented!()
    }

    /// Decode the elements in the difference quACK, if it can be decoded.
    pub fn decode(&self) -> (Vec<HashType>, Vec<HashType>, bool) {
        unimplemented!()
    }
}
