//! `Code`, `Scope`, and `Extent` for types bound to `amenable`'s real
//! `StateMachine`/`Exchange` traits.
//!
//! Depends on both `homecoming_core` and `amenable_core` — the dependency
//! `homecoming_core` itself deliberately avoids, so consumers who only want
//! source capture and structural subsetting never pay for `amenable`.
//! `StateMachine`/`Exchange` bounds let `Extent` reason about which code is
//! live directly from proven transition structure, not only from `Scope`'s
//! generic edge-closed/rooted graph closure.
//!
//! No content yet — hand-implemented against `amenable_kani`'s real
//! `Stoplight` first, in this crate's own tests, before any generalized
//! trait or blanket impl is designed. See `HOMECOMING_PLAN.md`'s Phase 6.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
