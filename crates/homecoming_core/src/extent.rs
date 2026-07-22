//! `Extent`: the user's grammar for which methods and types are
//! meaningful, isolatable units at all.

use crate::Code;

/// Names a point in `Self`'s dependency graph as a meaningful, isolatable
/// unit, and answers queries against it.
///
/// "Live" here is the compiler's sense of liveness — a definition is live
/// if it is used to compute a result reachable from a given point, dead
/// otherwise — not a runtime-tracing sense. `anchor` does not track
/// anything as it happens; it reuses whatever the implementor's `Code`/
/// `Scope` impls already compute for the named point, the same graph
/// `Scope::boundary` already describes. A name that was never declared
/// answers `None`, not an empty fragment standing in for "nothing here" —
/// the same discipline [`crate::Locality::contribute`] uses for
/// [`crate::Omit`].
///
/// Naming and shaving are different questions answered at different
/// times: an anchor stays valid even when a particular [`crate::Scope`]
/// configuration would shave it out of a rendered `scope()` — `Extent`
/// only answers "is this a nameable unit," never "is this included in a
/// given shaved result." That second question belongs to `Selection`.
pub trait Extent: Code {
    /// The live code for a named, isolatable unit, or `None` if `name`
    /// was never declared for this type.
    fn anchor(&self, name: &str) -> Option<Self::Fragment>;
}
