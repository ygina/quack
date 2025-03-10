use super::HashType;
use super::mapping::RandomMapping;

/// SymbolMapping is a mapping from a source symbol to a coded symbol. The
/// symbols are identified by their indices in CodingWindow.
#[derive(Debug)]
struct SymbolMapping {
    coded_idx: usize,
}

/// MappingHeap implements a priority queue of SymbolMappings. The priority is
/// the coded_idx of a SymbolMapping. A smaller value means higher priority.
/// The first item of the queue is always the item with the highest priority.
/// The fixHead and fixTail methods should be called after the first or the
/// last item is modified (or inserted, in the case of the tail), respectively.
/// The implementation is a partial copy of container/heap in Go 1.21.
type MappingHeap = Vec<SymbolMapping>;

/// fixTail reestablishes the heap invariant when the last item is modified or
/// just inserted.
fn fix_tail(m: &mut MappingHeap) {
    let mut curr = m.len() - 1;
    while curr != 0 {
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
}


impl CodingWindow {
    /// addHashWithMapping inserts a HashType and the current state of its
    /// mapping generator to the codingWindow.
    pub fn add_hash_with_mapping(&mut self, t: HashType, m: RandomMapping) {
        self.symbols.push(t);
        self.mappings.push(m);
        self.queue.push(SymbolMapping {
            coded_idx: m.last_index as usize,
        });
        fix_tail(&mut self.queue);
    }
}
