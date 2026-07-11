# Homecoming

> High-fidelity capture and replay of Rust source code.

[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

---

## What is Homecoming?

Given a value, `homecoming` produces the exact code that produced it — not
an approximation of what that code probably looked like, but the actual
source, captured and replayed with fidelity.

That matters for more than convenience. A formal verifier's proof is only
meaningful relative to the exact code it checked. `Homecoming` is the
receipt establishing that connection: what a proof ran over, tied exactly to
what ships. An approximate receipt is worse than none — it looks like
assurance while providing none.

`Homecoming`'s own soundness isn't trusted by construction. It's a checkable
claim, verified by round-tripping: emit a value's code, reconstruct the
value from it, and confirm it matches the original.

See [`homecoming.md`](homecoming.md) for the trait design and
[`HOMECOMING_PLAN.md`](HOMECOMING_PLAN.md) for the full architectural
rationale and phased implementation plan.

## Status

Early design. Dependencies (`syn`, `quote`, `proc-macro2`, `petgraph`) are
resolved; the `Homecoming` trait, `Fragment` type, and std-lib leaf
implementations are not yet written.

## Relationship to `amenable`

`homecoming` split off from
[`amenable`](https://github.com/crumplecup/amenable)'s `amenable_code`
scaffold, on the reasoning that source-code capture is orthogonal to
`amenable`'s constitutional proof-role trait family — it doesn't depend on
`amenable`, and `amenable` is expected to depend on it once this design
stabilizes.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
