# HOMECOMING_PLAN.md

## Goal

`homecoming` is a small crate defining `Homecoming`, the trait for
high-fidelity capture and replay of Rust source code, plus the composition
machinery needed to assemble captured fragments back into real programs.

This started as `amenable_code`, a scaffolded slot inside the `amenable`
workspace (`github.com/crumplecup/amenable`). It has been split off into its
own repository on further thought: `Code` (now `Homecoming`) is orthogonal
to `amenable`'s constitutional proof-role family — it doesn't need
`amenable`'s traits to exist, and `amenable`'s traits will need it, not the
other way around. Keeping it independent means it can be depended on by
anything that wants exact source capture, not only formal-verification
frameworks, and it avoids entangling `amenable`'s dependency-light core with
`syn`/`quote`/`proc-macro2`/`petgraph`.

`amenable` is expected to take `homecoming` as a dependency once this
design stabilizes, with several of its traits (at minimum `Witness`,
possibly `Evidence`) expected to also implement `Homecoming` — see
`AMENABLE_PLAN.md` in the `amenable` repository for the consuming side of
that relationship.

## Status

Early scaffold. `syn`/`quote`/`proc-macro2`/`petgraph` dependencies are
resolved and pinned per the version-pinning convention. No trait, no
`Fragment` type, and no std-lib leaf implementations exist yet — this
document is the design record from the `amenable_code` incubation period,
carried over so the design isn't lost, not a description of working code.

## `Homecoming` is capture and replay, not generation

The framing matters: `Homecoming` is not about synthesizing plausible-looking
source for an opaque runtime value after the fact — that is an open-ended,
judgment-call task that invites shortcuts (the motivating failure case was
`elicitation`'s `ToCodeLiteral` trait falling back to `default()` for an
uninspectable field — code that type-checks and compiles while having no
relationship to the value it claims to reproduce). Everything in a Rust
program starts out as code. The job is to preserve that original source
faithfully as a value flows through the program, not to reverse-engineer a
substitute once the trail has gone cold.

## Why this is part of formal verification, not just agent tooling

The original motivating use case was an agent assembling a real, compiling
Rust program by making tool calls that each execute for real, then asking
for the same trace replayed through a different interpreter that emits
source instead of performing the operation. That use case stands, but it is
not the deepest reason this trait matters. A solver's verdict is only
meaningful relative to the exact statement it checked. The code a proof ran
over — the receipt establishing what, precisely, was verified — has to be
*exact* to be of any value at all. An approximate or reconstructed rendering
of "what the code probably looked like" breaks the chain of custody between
what was verified and what ships, silently, in a way that looks like
assurance while providing none.

## The `Homecoming` trait itself

`Homecoming` is a leaf capability, implemented directly on the value it
describes — `self` already *is* the value; `code()` is the alternate
channel for getting its source instead of using it directly:

```rust
pub trait Homecoming {
    fn code(&self) -> Fragment;
}
```

## Fragment: `syn`-typed, `petgraph`-backed, not raw `TokenStream`

`Fragment` is not `proc_macro2::TokenStream`. A `TokenStream` is a flat-ish
token sequence with grouping but no grammar — it cannot distinguish an
`Expr` from an `Item`, so two fragments "clipping together" is not
statically checkable. `syn`'s typed AST is the actual tree Rust source
already is: a `syn::ExprCall`'s arguments are literally `Vec<syn::Expr>`, so
composition is type-directed. `quote::ToTokens` (which `syn` types already
implement) is the boundary where a fragment finally drops to `TokenStream`,
only at emission.

`Fragment` is concretely a node in a `petgraph` graph: a leaf node (from a
primitive's `Homecoming::code()`) wraps a `syn::Expr` with no children; a
composite node carries a composition-shape tag plus edges to its
constituent parts' nodes. This is the canonical, concrete representation,
not a placeholder for something else later — std-lib primitives (`bool`,
the integer types, `String`) are meant to be the first leaf-node
implementors, in this crate alongside `Homecoming` itself (any trait
implemented directly on foreign standard-library types must live in the
crate that defines it — Rust's orphan rule leaves no other option; this was
learned the hard way while building `amenable_std`, see `amenable`'s
`AMENABLE_PLAN.md`).

## Lateralizing composition traits: "A + B = C" as an explicit operation

A blanket `impl<T: Homecoming> Homecoming for Vec<T>` covers fixed Rust
composite shapes fine, but it cannot cover an agent *choosing*, at runtime,
which composition shape to apply to a pile of fragments it just produced —
fold them into a sequential block, thread them as call arguments, assemble
them as struct-literal fields, stack them as match arms. That choice needs
to be a first-class, visible, invokable step, not something buried in a
generic impl. This likely wants to be several lateralizing traits, one per
composition shape (a working sketch: `SequenceCode`, `ApplyCode`,
`ConstructCode`, `MatchCode`), each producing a new `Output: Homecoming` so
the result re-enters the system as a fresh, composable fragment.

Two forks are genuinely open and are meant to be resolved by implementing
the std-type leaf case first, not by pre-specifying the grammar:

- Binary composition ("A + B = C" literally, chained pairwise for bigger
  structures) versus native N-ary (`IntoIterator<Item = P>` from the start,
  since a block of statements or a struct's fields is not naturally binary).
- Whether every composition shape produces the same underlying `Fragment`
  node type (just tagged differently), making the lateralizing traits
  *builders* over one substrate rather than each having a genuinely distinct
  `Output` type — the `petgraph`-backed representation makes this the likely
  answer, but it should fall out of building `Homecoming for Vec<T>`, not be
  decided in the abstract.

## Round-trip as a proof obligation

`Homecoming`'s own soundness is not trusted by construction — it is a
checkable claim, verified two ways that do different jobs:

- **Simple form.** Emit a value's fragment via `Homecoming::code()`,
  reconstruct a value from that fragment, and confirm it equals the
  original. This is the baseline soundness obligation for any `Homecoming`
  implementor on its own. For a primitive this is close to trivial, but it
  is exactly the check that would have caught the `ToCodeLiteral`
  `default()`-fallback failure mode mechanically, with no review required.
- **Strong form.** Reconstruct a whole program from its emitted fragments,
  regenerate its proofs (via whatever proof-emission machinery the
  consuming crate provides — `amenable`'s `Witness`, most likely), and
  compare them to the original program's proofs. This is the real
  chain-of-custody guarantee: not "the code looks the same" but "a verifier
  run over the reconstructed code reaches the same conclusions a verifier
  run over the original code did." It cannot be fully exercised inside this
  crate alone, since `homecoming` deliberately does not depend on
  `amenable`'s `Witness` — the strong-form check belongs to whoever
  consumes both traits together, most likely `amenable` itself once it
  takes this crate as a dependency.

## Design Constraints

- No dependency on `amenable` or any proof-role trait family — `homecoming`
  stays consumable by anything that wants exact source capture, not only
  formal-verification frameworks.
- No unnecessary runtime dependencies beyond `syn`/`quote`/`proc-macro2`/
  `petgraph`, which are load-bearing for the core capability, not optional
  extras.
- No reconstruction-after-the-fact code paths — every `Homecoming` impl
  captures real source, verified by the round-trip obligation, never a
  best-guess substitute.

## Phased Implementation Plan

### Phase 1: Split from `amenable_code` — done

- [x] Create the standalone `homecoming` repository and workspace.
- [x] Carry over the resolved `syn`/`quote`/`proc-macro2`/`petgraph`
  dependencies and version pins.
- [x] Write this design record before any implementation, so the design
  discussion from the `amenable_code` incubation period isn't lost.

### Phase 2: `Fragment` and the std-lib leaf case

- [ ] Define `Fragment` as a `petgraph` graph with `syn`-typed node payloads
  (leaf nodes wrap `syn::Expr`; composite nodes carry a shape tag plus child
  edges).
- [ ] Implement `Homecoming` for a handful of std primitives (`bool`, an
  integer type, `String`) as the leaf case.
- [ ] Write the simple-form round-trip check for every leaf impl, and
  confirm it actually catches a deliberately fabricated fragment that does
  not reconstruct its claimed value.

### Phase 3: Composition

- [ ] Implement `Homecoming for Vec<T>` (or an equivalent first composite)
  and let its actual composition needs settle the lateralizing-trait arity
  and node-shape-count questions empirically.
- [ ] Name and implement the lateralizing composition traits that fall out
  of that exercise.

### Phase 4: `amenable` becomes a consumer

- [ ] `amenable` adds `homecoming` as a dependency.
- [ ] At least one `amenable` trait (`Witness` is the leading candidate)
  implements or requires `Homecoming`.
- [ ] Exercise the strong-form round-trip check against a real
  `Witness`-bearing example once `amenable`'s proof-emission machinery
  exists.

## Open Questions

- Is "A + B = C" composition literally binary, or does each lateralizing
  trait need to be N-ary from the start?
- Is one lateralizing trait per composition shape the right cut, or a
  smaller number?
- Does every composition shape produce the same underlying `Fragment` node
  type, or do genuinely different shapes need distinct `Output` types?
- What is the return shape for the round-trip check itself — a bare `bool`,
  a `Result` carrying a mismatch description, or something a reviewing
  agent can act on directly?
- Should `homecoming` itself become a workspace with multiple crates (as
  `amenable` did) once it has enough surface area to warrant a split, or
  does it stay a single crate?

## Success Condition

This plan succeeds when `Homecoming` produces an exact receipt of what code
generated a value, verified by round-trip checks rather than trusted by
convention, and when `amenable`'s proof-bearing traits can lean on it to
prove — not just assert — that the code a proof ran over is the code that
actually ships.
