#![feature(doc_cfg)]
#![feature(const_trait_impl)]

//! The _quACK_ is a data structure for being able to refer to and efficiently
//! acknowledge a set of opaque packets seen by a network intermediary. The
//! recommended quACK implementation is the 32-bit power sum quACK with no
//! features.
//!
//! In the quACK problem, a data sender transmits a multiset (meaning the same
//! element can be transmitted more than once) of elements `S` (these
//! correspond to packets). At any given time, a receiver (such as a proxy
//! server) has received a subset `R \subseteq S` of the sent elements. We
//! would like the receiver to communicate a small amount of information to the
//! sender, who then efficiently decodes the missing elements---the set
//! difference `S \ R`---knowing `S`. This small amount of information is called
//! the _quACK_ and the problem is: what is in a quACK and how do we decode it?

#[macro_use]
mod macros;

pub(crate) mod precompute;
pub use precompute::global_config_set_max_power_sum_threshold;

/// Efficient modular arithmetic and polynomial evaluation.
pub mod arithmetic {
    mod evaluator;
    mod modint;

    pub use evaluator::*;
    pub use modint::{ModularArithmetic, ModularInteger};

    cfg_montgomery! {
        mod montgomery;
        pub use montgomery::MontgomeryInteger;
    }
}

/// A quACK represented by a threshold number of power sums.
///
/// The power sum quACK is useful for decoding a set difference of elements
/// when the number of elements in the set difference is comparatively small
/// to the number of elements in either set. It is also efficient to insert
/// elements in the power sum quACK. The tradeoff is that it becomes impossible
/// to decode the quACK when the number of elements in the quACK exceeds a
/// pre-determined threshold. The number of bytes needed to transmit the quACK
/// over the wire is proportional to this threshold.
///
/// The underlying representation of a power sum quACK is a `threshold` number
/// of power sums. If `X` is the multiset of elements in the quACK, then the
/// `i`-th power sum is just the sum of `x^i` for all `x` in `X`.
pub trait Quack {
    /// The type of element that can be inserted in the quACK.
    type Element;

    /// Creates a new power sum quACK that can decode at most `threshold`
    /// number of elements.
    fn new(threshold: usize) -> Self
    where
        Self: Sized;

    /// The maximum number of elements that can be decoded by the quACK.
    fn threshold(&self) -> usize;

    /// The number of elements represented by the quACK.
    fn count(&self) -> u32;

    /// The last element inserted in the quACK, if known.
    ///
    /// If `None`, either there are no elements in the quACK, or a previous last
    /// element was removed and the actual last element is unknown.
    fn last_value(&self) -> Option<Self::Element>;

    /// Insert an element in the quACK.
    fn insert(&mut self, value: Self::Element);

    /// Remove an element in the quACK. Does not validate that the element
    /// had actually been inserted in the quACK.
    fn remove(&mut self, value: Self::Element);

    /// Subtracts another power sum quACK from this power sum quACK.
    ///
    /// The difference between a quACK with `x` elements and a quACK with `y`
    /// elements is a quACK with `x - y` elements. Assumes the elements in the
    /// second quACK are a subset of the elements in the first quACK. Assumes
    /// the two quACKs have the same threshold. If these conditions are met,
    /// then the `x - y` elements in the difference represent the set
    /// difference, and can be decoded from the quACK as long as this number of
    /// elements does not exceed the threshold.
    ///
    /// # Examples
    ///
    /// ```
    /// use quack::{Quack, PowerSumQuack, PowerSumQuackU32};
    /// use quack::arithmetic::{ModularInteger, ModularArithmetic};
    ///
    /// const THRESHOLD: usize = 20;
    ///
    /// fn main() {
    ///     // Insert some elements in the first quACK.
    ///     let mut quack1 = PowerSumQuackU32::new(THRESHOLD);
    ///     quack1.insert(1);
    ///     quack1.insert(2);
    ///     quack1.insert(3);
    ///     quack1.insert(4);
    ///     quack1.insert(5);
    ///
    ///     // Insert a subset of the same elements in the second quACK.
    ///     let mut quack2 = PowerSumQuackU32::new(THRESHOLD);
    ///     quack2.insert(2);
    ///     quack2.insert(5);
    ///
    ///     // Subtract the second quACK from the first and decode the elements.
    ///     quack1.sub_assign(&quack2);
    ///     let mut roots = quack1.decode_with_log(&[1, 2, 3, 4, 5]);
    ///     roots.sort();
    ///     assert_eq!(roots, vec![1, 3, 4]);
    /// }
    /// ```
    fn sub_assign(&mut self, rhs: &Self);

    /// Similar to [sub_assign](trait.PowerSumQuack.html#method.sub_assign)
    /// but returns the difference as a new quACK.
    fn sub(self, rhs: &Self) -> Self;
}

mod power_sum;
mod riblt;
pub use power_sum::{PowerSumQuack, PowerSumQuackU32};
pub use riblt::IBLTQuackU32;

cfg_strawmen! {
    mod strawmen;
    pub use strawmen::StrawmanAQuack;
    pub use strawmen::StrawmanBQuack;
}

cfg_montgomery! {
    mod montgomery;
    pub use power_sum::PowerSumQuackU64;
    pub use montgomery::MontgomeryQuack;
}

cfg_power_table! {
    mod power_table;
    pub use power_sum::PowerSumQuackU16;
    pub use power_table::PowerTableQuack;
}

mod ffi;