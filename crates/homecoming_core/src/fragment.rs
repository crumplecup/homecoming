//! `Fragment`: the interface a captured piece of Rust source must satisfy.

/// A captured piece of Rust source.
///
/// This is the interface `Code`/`Scope`/`Locality` are written against, not
/// a concrete representation. [`crate::Ir`], this crate's own
/// `petgraph`-backed implementation, is one way to satisfy it — used for
/// this crate's own std-lib `Code` impls — but nothing requires every
/// implementor to share its internal shape.
///
/// Explicit impl only, no blanket coverage: a type earns the capability by
/// declaring it, the same discipline `amenable`'s constitutional traits
/// use rather than granting it for free.
pub trait Fragment: quote::ToTokens + Clone {}
