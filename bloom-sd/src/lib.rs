mod hashing;
mod valuevec;
mod cbf;
// mod iblt;

pub use cbf::CountingBloomFilter;
// pub use iblt::InvBloomLookupTable;

use num_bigint::BigUint;
use bit_vec::BitVec;
use siphasher::sip128::SipHasher13;
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

#[derive(Serialize, Deserialize)]
#[serde(remote = "SipHasher13")]
struct SipHasher13Def {
    #[serde(getter = "SipHasher13::keys")]
    keys: (u64, u64),
}

// Provide a conversion to construct the remote type.
impl From<SipHasher13Def> for SipHasher13 {
    fn from(def: SipHasher13Def) -> SipHasher13 {
        SipHasher13::new_with_keys(def.keys.0, def.keys.1)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "BitVec")]
struct BitVecDef {
    #[serde(getter = "BitVec::to_bytes")]
    bytes: Vec<u8>,
}

// Provide a conversion to construct the remote type.
impl From<BitVecDef> for BitVec {
    fn from(def: BitVecDef) -> BitVec {
        BitVec::from_bytes(&def.bytes)
    }
}
