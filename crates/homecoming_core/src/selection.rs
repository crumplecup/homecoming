//! `Selection`: a shave-time policy over what `Extent` already made a
//! candidate.

use crate::Fragment;

/// Decides which of a [`crate::Scope`] implementor's `boundary()` entries
/// end up in a `scope_with()` result.
///
/// A filter layered on top of [`crate::Locality`], not a replacement for
/// it: an entry survives only if `includes` answers `true` *and* its
/// assigned `Locality::contribute` answers `Some` — the buffet metaphor
/// extended one step further than [`crate::Extent`]'s. `Extent` decides
/// which dishes exist on the table at all; `Locality` decides how a dish
/// would be plated if served; `Selection` is the customer's order off
/// that menu, external and pluggable rather than fixed per item, since a
/// platter isn't necessarily one walked session's path — it can be any
/// deliberately curated selection ("just the `+ - * /` keys," which no
/// single session ever walks but is still a valid, edge-closed, rooted
/// subset).
pub trait Selection<F: Fragment> {
    /// Whether `item` is included in a `scope_with()` result.
    fn includes(&self, item: &F) -> bool;
}
