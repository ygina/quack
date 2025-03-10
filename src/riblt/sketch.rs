//! The IBLT quACK implementation is adapted from the paper Practical Rateless
//! Set Reconciliation by Lei Yang, Yossi Gilad, and Mohammad Alizadeh, which
//! appeared in ACM SIGCOMM 2024. A modified version of the original RIBLT
//! implementation by the authors of that papaer and a comparison to a power
//! sum quACK implementation in Go is available here:
//! https://github.com/ygina/subset-reconciliation/.
use super::HashType;
use super::symbol::{CodedSymbol, ADD, REMOVE};
use super::mapping::RandomMapping;
use super::decoder::Decoder;

use serde::{Deserialize, Serialize};
use crate::Quack;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBLTQuackU32 {
    sketch: Vec<CodedSymbol>,
    last_value: Option<HashType>,
    count: u32,
}

impl Quack for IBLTQuackU32 {
    type Element = HashType;

    fn new(num_symbols: usize) -> Self {
        Self {
            count: 0,
            last_value: None,
            sketch: vec![CodedSymbol::default(); num_symbols],
        }
    }

    fn threshold(&self) -> usize {
        self.sketch.len()
    }

    fn count(&self) -> u32 {
        self.count
    }

    fn last_value(&self) -> Option<Self::Element> {
        self.last_value
    }

    fn insert(&mut self, t: HashType) {
        self.count = self.count.wrapping_add(1);
        self.last_value = Some(t);
        let mut m = RandomMapping::new(t);
        while (m.last_index as usize) < self.sketch.len() {
            let idx = m.last_index as usize;
            self.sketch[idx].apply(t, ADD);
            m.next_index();
        }
    }

    /// Remove an element in the quACK. Does not validate that the element
    /// had actually been inserted in the quACK.
    fn remove(&mut self, t: HashType) {
        self.count = self.count.wrapping_sub(1);
        let mut m = RandomMapping::new(t);
        while (m.last_index as usize) < self.sketch.len() {
            let idx = m.last_index as usize;
            self.sketch[idx].apply(t, REMOVE);
            m.next_index();
        }
    }

    /// Subtracts another power sum quACK from this power sum quACK.
    fn sub_assign(&mut self, s2: &Self) {
        if self.sketch.len() != s2.sketch.len() {
            panic!("subtracting sketches of different sizes");
        }
        if self.count < s2.count {
            panic!("too many packets in rhs quack");
        }
        self.count = self.count.wrapping_sub(s2.count);

        for i in 0..self.sketch.len() {
            let x = s2.sketch[i];
            self.sketch[i].apply(x.hash, x.count.wrapping_neg());
        }
    }

    fn sub(self, s2: &Self) -> Self {
        let mut s1 = self.clone();
        s1.sub_assign(s2);
        s1
    }
}

impl IBLTQuackU32 {
    pub fn decode(&self) -> Option<Vec<HashType>> {
        let mut dec = Decoder::default();
        for &c in &self.sketch {
            dec.add_coded_symbol(c);
        }
        dec.try_decode();
        if dec.decoded() {
            Some(dec.remote())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_fixed_encode_and_decode() {
        let sizes = vec![10, 20, 40, 100, 200, u8::MAX.into()];
        for size in sizes {
            let nlocal = size;
            let ncommon = size;

            let mut next_id = 0;
            let mut slocal = IBLTQuackU32::new(nlocal * 3);
            let mut sremote = IBLTQuackU32::new(nlocal * 3);
            for _ in 0..nlocal {
                next_id += 1;
                slocal.insert(next_id);
            }
            for _ in 0..ncommon {
                next_id += 1;
                slocal.insert(next_id);
                sremote.insert(next_id);
            }

            // Decode
            slocal.sub_assign(&sremote);
            let res = slocal.decode();
            // let (fwd, rev, succ) = (res.remote(), res.local(), res.decoded());
            assert!(res.is_some(), "(size={}) failed to decode at all", size);
            let res = res.unwrap();
            // assert_eq!(rev.len(), 0, "(size={}) failed to detect subset", size);
            assert_eq!(res.len(), nlocal,
                "(size={}) missing symbols: {} local", size, res.len());
        }
    }
}
