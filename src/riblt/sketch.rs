//! The IBLT quACK implementation is adapted from the paper Practical Rateless
//! Set Reconciliation by Lei Yang, Yossi Gilad, and Mohammad Alizadeh, which
//! appeared in ACM SIGCOMM 2024. A modified version of the original RIBLT
//! implementation by the authors of that papaer and a comparison to a power
//! sum quACK implementation in Go is available here:
//! https://github.com/ygina/subset-reconciliation/.
use super::HashType;
use super::symbol::CodedSymbol;
use super::mapping::RandomMapping;
use super::decoder::Decoder;

#[derive(Debug)]
pub struct IBLTQuackU32 {
    sketch: Vec<CodedSymbol>,
}

impl IBLTQuackU32 {
    /// Create a new IBLT quACK.
    pub fn new(num_symbols: usize) -> Self {
        Self {
            sketch: vec![CodedSymbol::default(); num_symbols],
        }
    }

    /// Insert an element in the quACK.
    pub fn insert(&mut self, t: HashType) {
        let mut m = RandomMapping::new(t);
        while (m.last_index as usize) < self.sketch.len() {
            let idx = m.last_index as usize;
            self.sketch[idx].count += 1;
            self.sketch[idx].hash ^= t;
            m.next_index();
        }
    }

    /// Remove an element in the quACK. Does not validate that the element
    /// had actually been inserted in the quACK.
    pub fn remove(&mut self, t: HashType) {
        let mut m = RandomMapping::new(t);
        while (m.last_index as usize) < self.sketch.len() {
            let idx = m.last_index as usize;
            self.sketch[idx].count -= 1;
            self.sketch[idx].hash ^= t;
            m.next_index();
        }
    }

    /// Subtracts another power sum quACK from this power sum quACK.
    pub fn sub_assign(&mut self, s2: &Self) {
        if self.sketch.len() != s2.sketch.len() {
            panic!("subtracting sketches of different sizes");
        }

        for i in 0..self.sketch.len() {
            self.sketch[i].count -= s2.sketch[i].count;
            self.sketch[i].hash ^= s2.sketch[i].hash;
        }
    }

    /// Decode the elements in the difference quACK, if it can be decoded.
    pub fn decode(&self) -> Decoder {
        let mut dec = Decoder::default();
        for &c in &self.sketch {
            dec.add_coded_symbol(c);
        }
        dec.try_decode();
        dec
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_fixed_encode_and_decode() {
        let sizes = vec![10, 20, 40, 100, 1000, 10000, 50000, 100000];
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
            let (fwd, rev, succ) = (res.remote(), res.local(), res.decoded());
            assert!(succ, "(size={}) failed to decode at all", size);
            assert_eq!(rev.len(), 0, "(size={}) failed to detect subset", size);
            assert_eq!(fwd.len(), nlocal,
                "(size={}) missing symbols: {} local", size, fwd.len());
        }
    }
}
