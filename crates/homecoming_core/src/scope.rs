//! `Scope`: isolating a minimal slice of code, not the whole program.

use crate::{Code, Fragment, Locality};

/// Isolates the minimal slice of code that contributes to one value or
/// operation, rather than everything transitively reachable from it.
///
/// The guiding intuition is a tracing span: a span doesn't require a
/// pre-declared taxonomy of "request scope" versus "database-call scope"
/// — any code that wants to be traceable enters its own span, and
/// isolating everything that happened during that span works identically
/// no matter how deep it's nested. `Scope` works the same way: no fixed
/// taxonomy of scope kinds, just a boundary each implementor reports for
/// itself, and a default derivation that walks it.
pub trait Scope: Code {
    /// Everything this fragment depends on to compile standalone, each
    /// paired with the [`Locality`] that decides how it renders when
    /// scoped.
    fn boundary(&self) -> impl Iterator<Item = (Fragment, Box<dyn Locality>)>;

    /// The isolated slice: this item's own code, plus whatever its
    /// boundary's localities decide to contribute. The current default
    /// sequences contributions ahead of this item's own expression in a
    /// block — a placeholder composition shape pending the lateralizing
    /// composition traits (`HOMECOMING_PLAN.md` "Lateralizing composition
    /// traits"), not a final answer to how contributions ought to combine.
    fn scope(&self) -> Fragment {
        let mut stmts: Vec<syn::Stmt> = self
            .boundary()
            .filter_map(|(dependency, locality)| locality.contribute(&dependency))
            .map(|fragment| syn::Stmt::Expr(fragment.expr().clone(), Some(Default::default())))
            .collect();
        stmts.push(syn::Stmt::Expr(self.code().expr().clone(), None));

        let block = syn::Block {
            brace_token: Default::default(),
            stmts,
        };
        Fragment::leaf(syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block,
        }))
    }
}
