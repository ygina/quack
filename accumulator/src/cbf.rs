use crate::Accumulator;

pub struct CBFAccumulator {
}

impl CBFAccumulator {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Accumulator for CBFAccumulator {
    fn process(&mut self, elem: u32) {
        unimplemented!()
    }

    fn process_batch(&mut self, elems: &Vec<u32>) {
        unimplemented!()
    }

    fn total(&self) -> usize {
        unimplemented!()
    }

    fn validate(&self, elems: &Vec<u32>) -> bool {
        unimplemented!()
    }
}
