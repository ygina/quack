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

use crate::Quack;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn sub(&self, rhs: &Self) -> Self {
        let threshold = std::cmp::min(self.threshold(), rhs.threshold());
        let sketch = self.sketch.iter().zip(rhs.sketch.iter())
            .take(threshold)
            .map(|(lhs, rhs)| lhs.clone().apply(rhs.hash, rhs.count.wrapping_neg()))
            .collect();
        Self {
            sketch,
            last_value: None,
            count: self.count.wrapping_sub(rhs.count),
        }
    }
}

impl IBLTQuackU32 {
    pub fn decode(self) -> Option<Vec<HashType>> {
        let mut dec = Decoder::new(self.sketch);
        dec.try_decode();
        if dec.decoded() {
            Some(dec.remote())
        } else {
            None
        }
    }

    pub fn serialize(&self, buf: &mut [u8]) -> usize {
        buf[0..4].copy_from_slice(&self.count.to_le_bytes());
        buf[4..8].copy_from_slice(&self.last_value.unwrap().to_le_bytes());
        let n = std::mem::size_of::<CodedSymbol>() * self.sketch.len();
        let src = self.sketch.as_ptr() as *const u8;
        let dst = (&mut buf[8..]).as_mut_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, n);
        }
        n + 8
    }

    pub fn serialize_with_hint(&self, buf: &mut [u8], num_missing: usize) -> usize {
        buf[0..4].copy_from_slice(&self.count.to_le_bytes());
        buf[4..8].copy_from_slice(&self.last_value.unwrap().to_le_bytes());
        let num_symbols = std::cmp::min(
            self.sketch.len(),
            if num_missing == 1 {
                num_missing
            } else {
                4 * num_missing
            },
        );
        let n = std::mem::size_of::<CodedSymbol>() * num_symbols;
        let src = self.sketch.as_ptr() as *const u8;
        let dst = (&mut buf[8..]).as_mut_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, n);
        }
        n + 8
    }

    pub fn deserialize(buf: &[u8]) -> Self {
        let n = (buf.len() - 8) / std::mem::size_of::<CodedSymbol>();
        let mut sketch = Vec::with_capacity(n);
        let src = (&buf[8..]).as_ptr() as *const CodedSymbol;
        let dst = sketch.as_mut_ptr();
        unsafe {
            sketch.set_len(n);
            std::ptr::copy_nonoverlapping(src, dst, n);
        }

        Self {
            count: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            last_value: Some(u32::from_le_bytes(buf[4..8].try_into().unwrap())),
            sketch,
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

    #[test]
    fn test_serialize_and_deserialize() {
        let mut buf = [0u8; 1500];
        let mut q1 = IBLTQuackU32::new(10);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        let len = q1.serialize(&mut buf);
        assert_eq!(len, 8+5*10);
        let q2 = IBLTQuackU32::deserialize(&buf[..len]);
        assert_eq!(q1.count(), q2.count());
        assert_eq!(q1.last_value(), q2.last_value());
        assert_eq!(q1.sketch, q2.sketch);
    }

    #[test]
    fn test_serialize_with_hint() {
        let mut buf = [0u8; 1500];
        let mut q1 = IBLTQuackU32::new(10);
        q1.insert(1);
        q1.insert(2);

        // Test number of symbols based on number missing
        assert_eq!(q1.serialize_with_hint(&mut buf, 0), 8+5*0);
        assert_eq!(q1.serialize_with_hint(&mut buf, 1), 8+5*1);
        assert_eq!(q1.serialize_with_hint(&mut buf, 2), 8+5*2*4);
        assert_eq!(q1.serialize_with_hint(&mut buf, 3), 8+5*10);

        // Serialize, deserialize, and decode
        let num_missing = 2;
        let len = q1.serialize_with_hint(&mut buf, num_missing);
        let q2 = IBLTQuackU32::deserialize(&buf[..len]);
        assert_eq!(q1.count(), q2.count());
        assert_eq!(q1.last_value(), q2.last_value());
        assert_eq!(q1.sketch.len(), 10);
        assert_eq!(q2.sketch.len(), 2*4);
    }
}
