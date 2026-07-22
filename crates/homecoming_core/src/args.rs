//! `Args`: a value obtainable from a [`crate::Source`], without saying
//! where it came from or how it will be rendered as code.

/// A value handed back by a [`crate::Source`] query.
///
/// Deliberately says nothing about `Fragment`/`Code` — `Source` deals in
/// raw values, not code. Whoever calls `value` is responsible for turning
/// the result into a `Fragment`, via whatever `Code` impl the raw value's
/// own type already has.
pub trait Args {
    /// The raw value type this `Args` wraps.
    type Value;

    /// The value itself.
    fn value(&self) -> Self::Value;
}
