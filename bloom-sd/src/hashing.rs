use std::hash::{Hash, Hasher};
use siphasher::sip128::SipHasher13;

pub struct HashIter {
    h1: u64,
    h2: u64,
    i: u32,
    count: u32,
}

impl Iterator for HashIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.i == self.count {
            return None;
        }
        let r = match self.i {
            0 => { self.h1 }
            1 => { self.h2 }
            _ => {
                let p1 = self.h1.wrapping_add(self.i as u64);
                p1.wrapping_mul(self.h2)
            }
        };
        self.i+=1;
        Some(r)
    }
}

impl HashIter {
    pub fn from<T: Hash>(
        item: T,
        count: u32,
        build_hasher_one: &SipHasher13,
        build_hasher_two: &SipHasher13,
    ) -> HashIter {
        let mut hasher_one = build_hasher_one.clone();
        let mut hasher_two = build_hasher_two.clone();
        item.hash(&mut hasher_one);
        item.hash(&mut hasher_two);
        let h1 = hasher_one.finish();
        let h2 = hasher_two.finish();
        HashIter {
            h1: h1,
            h2: h2,
            i: 0,
            count: count,
        }
    }
}
