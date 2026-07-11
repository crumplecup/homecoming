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
pub trait Locality {
    /// Render this dependency's contribution, or `None` if it contributes
    /// nothing to the scoped result at all.
    fn contribute(&self, dependency: &Fragment) -> Option<Fragment>;
}

/// Reproduce a dependency's fragment in full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Inline;

impl Locality for Inline {
    fn contribute(&self, dependency: &Fragment) -> Option<Fragment> {
        Some(dependency.clone())
    }
}

/// Name a dependency by a path instead of reproducing its definition.
#[derive(Debug, Clone)]
pub struct Reference {
    path: syn::Path,
}

impl Reference {
    /// Reference a dependency by the given path, without reproducing its
    /// definition.
    pub fn new(path: syn::Path) -> Self {
        Self { path }
    }
}

impl Locality for Reference {
    fn contribute(&self, _dependency: &Fragment) -> Option<Fragment> {
        let expr = syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: self.path.clone(),
        });
        Some(Fragment::leaf(expr))
    }
}

/// Shave a dependency away entirely — not reproduced, not referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Omit;

impl Locality for Omit {
    fn contribute(&self, _dependency: &Fragment) -> Option<Fragment> {
        None
    }
}
