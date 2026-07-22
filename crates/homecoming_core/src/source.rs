//! `Source`: whether a value is available for a given slot, decoupled from
//! how that value gets rendered as code.

use crate::Args;

/// Answers whether a value is available for a named slot — the raw-value
/// half of the shifting-scope spectrum, paired with [`crate::Binding`]'s
/// decision about how that answer gets rendered.
///
/// `Source` doesn't mention `Fragment` or `Code` at all: it hands back
/// something satisfying [`Args`], and turning that into a `Fragment` is
/// left to whichever `Code` impl the raw value's own type already has.
pub trait Source {
    /// The concrete `Args` type this `Source`'s queries hand back.
    type Args: Args;

    /// The value for `slot`, without consuming anything — a peek, safe to
    /// call more than once with the same answer.
    fn value_for(&self, slot: &str) -> Option<Self::Args>;

    /// The value for `slot`, permitted to consume or mutate whatever
    /// backs it. Defaults to a non-consuming peek — most `Source`s (a
    /// fixed, already-captured record) have nothing to drain and never
    /// override this.
    fn value_mut_for(&mut self, slot: &str) -> Option<Self::Args> {
        self.value_for(slot)
    }
}
