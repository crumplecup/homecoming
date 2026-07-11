# Homecoming

> High-fidelity capture, isolation, and replay of Rust source code.

[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

---

## What is Homecoming?

Given a value, `Code` produces the exact code that produced it — not an
approximation of what that code probably looked like, but the actual
source, captured and replayed with fidelity.

`Scope` goes further: anyone can hand back the whole source of a program,
but that isn't useful on its own. Given one specific point of interest —
a state, a value, an operation — `Scope` isolates the minimal slice of
code that actually contributes to it, the way a tracing span isolates a
log region without needing a pre-declared taxonomy of what kind of span it
is.

The design is deliberately two-tiered, and doesn't target arbitrary Rust at
either tier. Any type implementing `Scope` gets shaving for free at the
core tier — no `amenable` dependency required, structural validity only.
A separate bridge crate, built directly on `amenable`'s real
`StateMachine`/`Exchange` traits (proof tokens included, not a lookalike),
makes the stronger claim that a shaved subset preserves the proof
obligations the original upheld. Both tiers work the same way implementing
`Iterator`'s required methods unlocks a toolkit of default ones — conform
to a shape, and the rest falls out for free.

That matters for more than convenience. A formal verifier's proof is only
meaningful relative to the exact code it checked. This crate is the
receipt establishing that connection: what a proof ran over, tied exactly
to what ships, scoped to just the relevant part. An approximate or
whole-program receipt is worse than a precise one — it either looks like
assurance while providing none, or buries the relevant part in everything
else.

Neither trait's soundness is trusted by construction. Both are checkable
claims, verified by round-tripping: emit a value's (or a slice's) code,
reconstruct it, and confirm it matches the original.

See [`homecoming.md`](homecoming.md) for the trait design and
[`HOMECOMING_PLAN.md`](HOMECOMING_PLAN.md) for the full architectural
rationale and phased implementation plan.

## Status

Early design, no implementation yet. Dependencies (`syn`, `quote`,
`proc-macro2`, `petgraph`) are resolved. The crate is a workspace —
`homecoming_core` (the `Code`/`Scope`/`Locality`/`Selection`/`Source`/
`Binding` traits and `Fragment`, no `amenable` dependency), a bridge crate
(name not yet settled, depending on both `homecoming_core` and
`amenable_core`), `homecoming_derive` (a future derive macro, not
started), `homecoming` (the top-level facade) — mirroring `amenable`'s own
shape.

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
