//! `Locality`: how one dependency contributes to a scoped fragment.

use crate::Fragment;

/// Decides how one [`crate::Scope::boundary`] entry contributes to a
/// scoped fragment — reproduced in full, referenced by name, omitted
/// entirely, or something else.
///
/// This is a trait rather than a closed enum on purpose: a fixed set of
/// variants would need to grow every time a new rendering strategy showed
/// up. As a trait, new localities are just new implementors — see
/// `HOMECOMING_PLAN.md`'s "`Code`, `Scope`, `Locality`" for the design
/// rationale, including why `Omit` fell out of this shape for free.
///
/// Generic over the concrete [`Fragment`] type, since `Locality` has to
/// work for whatever representation the paired `Code`/`Scope` implementor
/// uses, not just this crate's own [`crate::Ir`].
pub trait Locality<F: Fragment> {
    /// Render this dependency's contribution, or `None` if it contributes
    /// nothing to the scoped result at all.
    fn contribute(&self, dependency: &F) -> Option<F>;
}

/// Reproduce a dependency's fragment in full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Inline;

impl<F: Fragment> Locality<F> for Inline {
    fn contribute(&self, dependency: &F) -> Option<F> {
        Some(dependency.clone())
    }
}

/// Name a dependency by a caller-supplied replacement fragment instead of
/// reproducing its definition.
///
/// The replacement is supplied already built, rather than `Reference`
/// building it from a path itself: `Fragment` carries no construction
/// capability (deliberately — the interface only promises `ToTokens` +
/// `Clone`), so whoever holds a concrete `F` and knows how to build an
/// `F`-shaped reference has to do that construction, not `Locality`.
#[derive(Debug, Clone)]
pub struct Reference<F> {
    replacement: F,
}

impl<F> Reference<F> {
    /// Reference a dependency by the given replacement fragment, without
    /// reproducing its definition.
    pub fn new(replacement: F) -> Self {
        Self { replacement }
    }
}

impl<F: Fragment> Locality<F> for Reference<F> {
    fn contribute(&self, _dependency: &F) -> Option<F> {
        Some(self.replacement.clone())
    }
}

/// Shave a dependency away entirely — not reproduced, not referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Omit;

impl<F: Fragment> Locality<F> for Omit {
    fn contribute(&self, _dependency: &F) -> Option<F> {
        None
    }
}
