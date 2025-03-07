use super::HashType;
use super::symbol::{CodedSymbol, Direction};
use super::mapping::RandomMapping;

/// SymbolMapping is a mapping from a source symbol to a coded symbol. The
/// symbols are identified by their indices in CodingWindow.
#[derive(Debug)]
struct SymbolMapping {
    source_idx: usize,
    coded_idx: usize,
}

/// MappingHeap implements a priority queue of SymbolMappings. The priority is
/// the coded_idx of a SymbolMapping. A smaller value means higher priority.
/// The first item of the queue is always the item with the highest priority.
/// The fixHead and fixTail methods should be called after the first or the
/// last item is modified (or inserted, in the case of the tail), respectively.
/// The implementation is a partial copy of container/heap in Go 1.21.
type MappingHeap = Vec<SymbolMapping>;

/// fixHead reestablishes the heap invariant when the first item is modified.
fn fix_head(m: &mut MappingHeap) {
    let mut curr = 0;
    loop {
        let mut child = curr * 2 + 1;
        if child >= m.len() {
            // no left child
            break;
        }
        let rc = child + 1;
        if rc < m.len() && m[rc].coded_idx < m[child].coded_idx {
            child = rc;
        }
        if m[curr].coded_idx <= m[child].coded_idx {
            break;
        }
        m.swap(curr, child);
        curr = child;
    }
}

/// fixTail reestablishes the heap invariant when the last item is modified or
/// just inserted.
fn fix_tail(m: &mut MappingHeap) {
    let mut curr = m.len() - 1;
    loop {
        let parent = (curr - 1) / 2;
        if curr == parent || m[parent].coded_idx <= m[curr].coded_idx {
            break;
        }
        m.swap(parent, curr);
        curr = parent;
    }
}

/// CodingWindow is a collection of source symbols and their mappings to coded
/// symbols.
#[derive(Default, Debug)]
pub struct CodingWindow {
    /// source symbol hashes
    pub symbols: Vec<HashType>,
    /// mapping generators of the source symbols
    mappings: Vec<RandomMapping>,
    /// priority queue of source symbols by the next coded symbols they are
    /// mapped to
    queue: MappingHeap,
    /// index of the next coded symbol to be generated
    next_idx: usize,
}


impl CodingWindow {
    /// applyWindow maps the source symbols to the next coded symbol they should
    /// be mapped to, given as cw. The parameter direction controls how the
    /// counter of cw should be modified.
    pub fn apply_window(&mut self, mut cw: CodedSymbol, direction: Direction) -> CodedSymbol {
        if self.queue.len() == 0 {
            self.next_idx += 1;
            return cw;
        }
        while self.queue[0].coded_idx == self.next_idx {
            cw = cw.apply(self.symbols[self.queue[0].source_idx], direction);
            // generate the next mapping
            let next_map = self.mappings[self.queue[0].source_idx].next_index();
            self.queue[0].coded_idx = next_map as usize;
            fix_head(&mut self.queue);
        }
        self.next_idx += 1;
        cw
    }

    /// addHashWithMapping inserts a HashType and the current state of its
    /// mapping generator to the codingWindow.
    pub fn add_hash_with_mapping(&mut self, t: HashType, m: RandomMapping) {
        self.symbols.push(t);
        self.mappings.push(m);
        self.queue.push(SymbolMapping {
            source_idx: self.symbols.len() - 1,
            coded_idx: m.last_index as usize,
        });
        fix_tail(&mut self.queue);
    }
}
