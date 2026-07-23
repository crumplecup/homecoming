//! `Code`, `Scope`, and `Locality`: high-fidelity capture, isolation, and
//! replay of Rust source code.
//!
//! `Code` gives back the exact source that produced a value, in place of
//! the value itself. Anyone can hand back the whole source of a program,
//! though — `Scope` is for a narrower, harder job: given one specific
//! point of interest, isolate the minimal slice of code that actually
//! contributes to it, not everything transitively reachable from it,
//! guided by `Locality`'s decision for each dependency about whether it
//! must be inlined to compile standalone or can merely be referenced.
//!
//! `Fragment` is the interface these traits are written against, not a
//! concrete type — [`Ir`] is this crate's own `petgraph`-backed
//! implementation of it, used for this crate's own std-lib `Code` impls,
//! but callers with a different representation in mind can satisfy
//! `Fragment` on their own terms.
//!
//! `Extent` layers a naming grammar on top of `Scope`: a named anchor into
//! the dependency graph a `Scope` implementor already builds, so a caller
//! can query "the live code for `divide`" by name instead of hand-writing
//! a `Scope` impl for every cut point.
//!
//! `Selection` layers a shave-time policy on top of `Locality`: an
//! external, pluggable filter over `boundary()`'s entries, queried through
//! `Scope::scope_with`, rather than a fixed choice baked into a `Scope`
//! implementor's own `boundary()`.
//!
//! `Source` and `Binding` answer a finer-grained question than `Locality`/
//! `Selection` do — not "is this boundary entry included," but "is this
//! one value, within an included entry, frozen or left open." `Source`
//! decides whether a value is available at all, in terms of a raw
//! [`Args`] value with no `Fragment`/`Code` involved; `Binding` decides
//! separately how that answer gets rendered (`Bound`, a frozen literal;
//! `Free`, an open slot).
//!
//! The `binary_expr`/`call_expr`/`path_expr`/... family of direct `syn`
//! AST construction helpers `Code`/`Scope`/`Extent` implementors need is
//! promoted here (see `build`'s own module docs) once the same handful of
//! builders had been hand-written nearly identically in several separate
//! examples.
//!
//! See `HOMECOMING_PLAN.md` and `homecoming.md` in the repository root for
//! the full design rationale.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod args;
mod binding;
mod build;
mod code;
mod extent;
mod fragment;
mod ir;
mod locality;
mod scope;
mod selection;
mod source;

pub use args::Args;
pub use binding::{Binding, Bound, Free};
pub use build::{
    array_expr, binary_expr, block_expr, call_expr, closure_expr, ident, ident_pat, let_stmt,
    match_arm, match_expr, method_call_expr, path, path_expr, struct_expr, tuple_expr, tuple_pat,
    tuple_struct_pat, type_path, typed_pat, unwrap_infallible_expr, wildcard_pat,
};
pub use code::Code;
pub use extent::Extent;
pub use fragment::Fragment;
pub use ir::Ir;
pub use locality::{Inline, Locality, Omit, Reference};
pub use scope::Scope;
pub use selection::Selection;
pub use source::Source;
