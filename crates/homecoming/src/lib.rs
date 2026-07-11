//! High-fidelity capture and replay of Rust source code fragments.
//!
//! Homecoming is about coming back to the source: given a value, produce the
//! exact code that produced it, not an approximation of what that code
//! probably looked like. See `HOMECOMING_PLAN.md` and `homecoming.md` in the
//! repository root for the full design rationale — including why this
//! matters for formal verification, not just for agents assembling programs
//! from tool calls.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
