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
//! See `HOMECOMING_PLAN.md` and `homecoming.md` in the repository root for
//! the full design rationale.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod code;
mod extent;
mod fragment;
mod ir;
mod locality;
mod scope;

pub use code::Code;
pub use extent::Extent;
pub use fragment::Fragment;
pub use ir::Ir;
pub use locality::{Inline, Locality, Omit, Reference};
pub use scope::Scope;
