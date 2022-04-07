#[macro_use]
extern crate lazy_static;

mod xor;
mod mset;

/// Type alias to easily switch between digest types in crates that
/// use this library. Both the XOR digest and additive mset hash digest
/// use the same interface, but we don't actually implement a trait
/// to not complicate the equals() function.
pub type Digest = AdditiveMsetHash;

pub use xor::XorDigest;
pub use mset::AdditiveMsetHash;

use num_bigint::BigUint;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "BigUint")]
struct BigUintDef {
    #[serde(getter = "BigUint::to_bytes_be")]
    bytes_be: Vec<u8>,
}

// Provide a conversion to construct the remote type.
impl From<BigUintDef> for BigUint {
    fn from(def: BigUintDef) -> BigUint {
        BigUint::from_bytes_be(&def.bytes_be)
    }
}
