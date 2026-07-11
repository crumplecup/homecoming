//! Derive macro generating `Code` impls.
//!
//! Not yet implemented. The plan (see `HOMECOMING_PLAN.md`) is to hand-write
//! `impl Code for` a range of std-lib and state-machine examples first, and
//! extract the derive from the patterns proven out by that hand-written
//! work — not to design the macro before the representation it generates
//! code for has been validated.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
