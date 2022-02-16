mod cbf;
mod naive;
mod power_sum;

pub use cbf::CBFAccumulator;
pub use naive::NaiveAccumulator;
pub use power_sum::PowerSumAccumulator;

pub trait Accumulator {
    /// Process a single element.
    fn process(&mut self, elem: u32);
    /// Process a batch of elements.
    fn process_batch(&mut self, elems: &Vec<u32>);
    /// The total number of processed elements.
    fn total(&self) -> usize;
    /// Validate the accumulator against a list of elements.
    ///
    /// The accumulator is valid if the elements that the accumulator has
    /// processed are a subset of the provided list of elements.
    fn validate(&self, elems: &Vec<u32>) -> bool;
}
