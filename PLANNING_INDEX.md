# Planning Documents

This file tracks all planning documents for the homecoming project.

## Current Active Plans

### Homecoming Trait and Fragment Design

**Document:** [HOMECOMING_PLAN.md](HOMECOMING_PLAN.md)

**Status:** 🔲 Planning — dependencies resolved, no trait implementation yet

**Description:** `homecoming` defines `Homecoming`, the trait for
high-fidelity capture and replay of Rust source code, plus the composition
machinery needed to assemble captured fragments back into real programs.
Split off from `amenable`'s `amenable_code` scaffold because source-code
capture is orthogonal to `amenable`'s proof-role trait family. `amenable` is
expected to depend on this crate once the design stabilizes, with several
of its traits (`Witness` at minimum) implementing `Homecoming`.

**Architecture principle:** Capture and replay, not generation — every
`Homecoming` implementor reproduces the exact code that produced a value,
verified by round-trip checks, never a best-guess reconstruction.
