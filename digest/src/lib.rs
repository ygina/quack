mod xor;
mod mset;

/// Type alias to easily switch between digest types in crates that
/// use this library. Both the XOR digest and additive mset hash digest
/// use the same interface, but we don't actually implement a trait
/// to not complicate the equals() function.
pub type Digest = AdditiveMsetHash;

pub use xor::XorDigest;
pub use mset::AdditiveMsetHash;
pub use mset::AmhHash;
