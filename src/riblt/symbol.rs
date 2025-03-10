use super::HashType;

use serde::{Deserialize, Serialize};

pub type Direction = u8;
pub const ADD: Direction = 1;
pub const REMOVE: Direction = u8::MAX;

/// CodedSymbol is a coded symbol produced by a Rateless IBLT encoder.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CodedSymbol {
    pub hash: HashType,
    pub count: u8,
}

impl CodedSymbol {
    pub fn apply(&mut self, hash: HashType, direction: Direction) -> CodedSymbol {
        self.hash ^= hash;
        self.count = self.count.wrapping_add(direction);
        *self
    }
}
