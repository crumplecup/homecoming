//! High-fidelity capture, isolation, and replay of Rust source code.
//!
//! `homecoming` is the top-level facade over the `homecoming_*` crate
//! family: it re-exports the `Code` trait and its supporting types from
//! `homecoming_core`, alongside the `#[derive(...)]` macro from
//! `homecoming_derive` once that crate has content, so most users depend on
//! this one crate rather than assembling the family themselves.
//!
//! Crates that are themselves part of the family (`homecoming_derive`)
//! depend on `homecoming_core` directly, never on this facade, to avoid a
//! circular dependency.
//!
//! See `HOMECOMING_PLAN.md` and `homecoming.md` in the repository root for
//! the full design rationale.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
