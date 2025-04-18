use super::HashType;
use super::symbol::{CodedSymbol, Direction, REMOVE};
use super::mapping::RandomMapping;
use log::error;

// Decoder computes the symmetric difference between two sets A, B. The Decoder
// knows B (the local set) and expects coded symbols for A (the remote set).
#[derive(Default)]
pub struct Decoder {
    /// coded symbols received so far
    cs: Vec<CodedSymbol>,
    /// decoded ientifiers
    remote: Vec<u32>,
    /// indices of coded symbols that can be decoded, i.e., degree equal to -1
    /// or 1 or degree equal to 0 and sum of hash equal to 0
    decodable: Vec<usize>,
    /// number of coded symbols that are decoded
    decoded: usize,
}

impl Decoder {
    /// Initialize with all the coded symbols
    pub fn new(cs: Vec<CodedSymbol>) -> Self {
        Self {
            decodable: cs
                .iter()
                .enumerate()
                .filter(|(_, c)| (c.count == 0 && c.hash == 0) || c.count == 1)
                .map(|(i, _)| i)
                .collect(),
            cs,
            remote: vec![],
            decoded: 0,
        }
    }

    /// Decoded returns true if and only if every existing coded symbols d
    /// received so far have been decoded.
    pub fn decoded(&self) -> bool {
        self.decoded == self.cs.len()
    }

    /// The list of source symbols that are present in A but not in B.
    pub fn remote(self) -> Vec<HashType> {
        self.remote
    }

    /// AddCodedSymbol passes the next coded symbol in A's sequence to the
    /// Decoder. Coded symbols must be passed in the same ordering as they are
    /// generated by A's Encoder.
    pub fn _add_coded_symbol(&mut self, c: CodedSymbol) {
        // insert the new coded c
        self.cs.push(c);
        // check if the coded c is decodable, and insert into decodable list if so
        if c.count == 1 {
            self.decodable.push(self.cs.len() - 1);
        } else if c.count == 0 && c.hash == 0 {
            self.decodable.push(self.cs.len() - 1);
        }
    }

    fn apply_new_symbol(&mut self, t: HashType, direction: Direction) -> RandomMapping {
        let mut m = RandomMapping::new(t);
        while (m.last_index as usize) < self.cs.len() {
            let cidx = m.last_index as usize;
            self.cs[cidx] = self.cs[cidx].apply(t, direction);
            // Check if the coded symbol is now decodable. We do not want to insert
            // a decodable symbol into the list if we already did, otherwise we
            // will visit the same coded symbol twice. To see how we achieve that,
            // notice the following invariant: if a coded symbol becomes decodable
            // with degree D (obviously -1 <= D <=1), it will stay that way, except
            // for that it's degree may become 0. For example, a decodable symbol
            // of degree -1 may not later become undecodable, or become decodable
            // but of degree 1. This is because each peeling removes a source
            // symbol from the coded symbol. So, if a coded symbol already contains
            // only 1 or 0 source symbol (the definition of decodable), the most we
            // can do is to peel off the only remaining source symbol.
            //
            // Meanwhile, notice that if a decodable symbol is of degree 0, then
            // there must be a point in the past when it was of degree 1 or -1 and
            // decodable, at which time we would have inserted it into the
            // decodable list. So, we do not insert degree-0 symbols to avoid
            // duplicates. On the other hand, it is fine that we insert all
            // degree-1 or -1 decodable symbols, because we only see them in such
            // state once.
            if self.cs[cidx].count == 1 {
                self.decodable.push(cidx);
            }
            m.next_index();
        }
        m
    }

    /// TryDecode tries to decode all coded symbols received so far.
    pub fn try_decode(&mut self) {
        let mut didx = 0;
        while didx < self.decodable.len() {
            let cidx = self.decodable[didx];
            let c = self.cs[cidx];
            // We do not need to compare Hash and Symbol.Hash() below, because
            // we have checked it before inserting into the decodable list. Per
            // the invariant mentioned in the comments in applyNewSymbol, a
            // decodable symbol does not turn undecodable, so there is no worry
            // that additional source symbols have been peeled off a coded
            // symbol after it was inserted into the decodable list and before
            // we visit them here.
            if c.count == 1 {
                // allocate a symbol and then XOR with the sum, so that we are
                // guaranted to copy the sum whether or not the symbol
                // interface is implemented as a pointer
                let ns = c.hash;
                self.apply_new_symbol(ns, REMOVE);
                self.remote.push(ns);
                self.decoded += 1;
            } else if c.count == 0 {
                self.decoded += 1;
            } else {
                // a decodable symbol does not turn undecodable, so its degree
                // must be -1, 0, or 1
                error!("invalid degree for decodable coded symbol {} {:?} {}/{}",
                    c.count, self.cs, self.decoded, self.cs.len());
                return;
            }
            didx += 1;
        }
        self.decodable.clear();
    }

    // /// Reset clears self. It is more efficient to call Reset to reuse an existing
    // /// Decoder than creating a new one.
    // fn reset(&mut self) {
    //     // if len(d.cs) != 0 {
    //     //     self.cs = self.cs[:0]
    //     // }
    //     // if len(d.decodable) != 0 {
    //     //     self.decodable = self.decodable[:0]
    //     // }
    //     // self.remote.reset()
    //     // self.decoded = 0
    // }
}
