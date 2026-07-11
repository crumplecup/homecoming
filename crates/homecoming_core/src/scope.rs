//! `Scope`: isolating a minimal slice of code, not the whole program.

use crate::{Code, Locality};

/// Isolates the minimal slice of code that contributes to one value or
/// operation, rather than everything transitively reachable from it.
///
/// The guiding intuition is a tracing span: a span doesn't require a
/// pre-declared taxonomy of "request scope" versus "database-call scope"
/// — any code that wants to be traceable enters its own span, and
/// isolating everything that happened during that span works identically
/// no matter how deep it's nested. `Scope` works the same way: no fixed
/// taxonomy of scope kinds, just a boundary each implementor reports for
/// itself.
///
/// `scope()` has no default body. Combining `code()` with `boundary()`'s
/// contributions into one final [`crate::Fragment`] is a composition
/// operation, and `Fragment`'s interface (`ToTokens` + `Clone`) carries no
/// composition capability — deliberately, since how contributions ought
/// to combine is exactly the open question the lateralizing composition
/// traits are meant to answer (`HOMECOMING_PLAN.md`, "Lateralizing
/// composition traits"). Until that exists, each implementor writes its
/// own `scope()` using whatever its concrete `Fragment` type actually
/// offers.
pub trait Scope: Code {
    /// Everything this fragment depends on to compile standalone, each
    /// paired with the [`Locality`] that decides how it renders when
    /// scoped.
    fn boundary(&self)
    -> impl Iterator<Item = (Self::Fragment, Box<dyn Locality<Self::Fragment>>)>;

    /// The isolated slice: this item's own code, plus whatever its
    /// boundary's localities decide to contribute. The drop-out — code
    /// isolated from its surrounding program the way a span isolates a
    /// log region from the rest of a trace.
    fn scope(&self) -> Self::Fragment;
}
