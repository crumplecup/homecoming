# Planning Documents

This file tracks all planning documents for the homecoming project.

## Current Active Plans

### Two-Tier Code/Scope Design: Core Shaving Plus an `amenable` Bridge

**Document:** [HOMECOMING_PLAN.md](HOMECOMING_PLAN.md)

**Status:** 🔲 Planning — workspace restructured to match `amenable`'s
shape, no trait implementation yet

**Description:** `homecoming_core` defines `Code`, the trait for
high-fidelity capture and replay of Rust source code, and `Scope`/
`Locality`/`Selection`/`Source`/`Binding`, the traits for isolating and
configurably shaving a minimal slice of code rather than capturing whole
programs. This core tier has no dependency on `amenable` and makes no
claim beyond structural validity (edge-closed, rooted). A separate bridge
crate (name not yet settled) depends on both `homecoming_core` and
`amenable_core`, using `amenable`'s real `StateMachine`/`Exchange` traits
directly — not a homecoming-native lookalike — to make the stronger claim
that a shaved subset preserves the proof obligations the original upheld.
`amenable` is expected to depend on `homecoming` once the design
stabilizes, with `Witness` at minimum implementing `Code`.

**Architecture principles:**

- Capture and replay, not generation — every `Code` implementor reproduces
  the exact code that produced a value, verified by round-trip checks,
  never a best-guess reconstruction. A shaved subset program is a
  reassembly of real captured pieces, never a fabrication.
- Isolation, not just capture — `Scope` extracts the minimal slice of code
  relevant to one point of interest, the way a tracing span isolates a log
  region, rather than handing back an entire program.
- No closed taxonomies for open-ended design questions — `Locality` is a
  trait, not an `Inline`/`Reference` enum, so new rendering strategies
  (like `Omit`, discovered via the shaving use case) are new implementors,
  not breaking changes.
- Deliberately scoped, not general — targets types with a `Scope` impl
  (core tier) or a state-machine shape (bridge tier), not arbitrary
  unstructured Rust.
- Composable configuration, not fixed modes — `Selection` (which components
  are included) and `Source` (which values are frozen vs. left open) are
  independent, pluggable pieces; any combination is a valid configuration,
  not just the ones that originally motivated the design.
- Reuse real traits, not lookalikes, when the substance matters — `amenable
  ::StateMachine` is minimal enough to depend on directly; `Exchange`'s
  proof-token machinery is the entire reason the bridge tier exists, so a
  copy of its shape without that machinery would be pointless.
