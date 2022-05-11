mod hashing;
mod valuevec;
mod cbf;
mod iblt;

pub use cbf::CountingBloomFilter;
pub use iblt::InvBloomLookupTable;
pub use iblt::elem_to_u32;
pub use valuevec::ValueVec;

use bit_vec::BitVec;
use siphasher::sip128::SipHasher13;
use serde::{Serialize, Deserialize};

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
