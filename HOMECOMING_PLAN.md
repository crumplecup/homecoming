# HOMECOMING_PLAN.md

## Goal

`homecoming` is a small crate family defining `Code`, the trait for
high-fidelity capture, isolation, and replay of Rust source code, plus the
composition and scoping machinery needed to assemble fragments into real
programs and extract minimal slices back out of them.

The design deliberately does not target arbitrary Rust programs, and it is
deliberately two-tiered rather than one design applied uniformly.

**Core tier** — `Code`, `Scope`, `Locality` in `homecoming_core` — has no
dependency on `amenable` and works for any type. What it buys: a program
that could previously only *perform* an operation can now also *emit* the
source of that operation, and isolate a minimal, structurally valid slice
(edge-closed, rooted) of a larger program. No claim beyond structural
validity is made at this tier, because nothing about a bare `Code`
implementor says anything was ever proven.

**Bridge tier** — a separate crate, name not yet settled, depending on
*both* `homecoming_core` and `amenable_core` — targets programs shaped like
a state machine with exchange properties, using `amenable`'s actual
`StateMachine`/`Exchange` traits directly, proof-token machinery included,
not a homecoming-native lookalike. That is a deliberate choice, not an
oversight: `amenable_core::StateMachine` is minimal enough (`{ type State;
type Invariant; }`) that reinventing it bought nothing, but
`amenable_core::Exchange` carries real `Evidence`/`ProofToken` machinery
that a lookalike trait *couldn't* carry, because that machinery is the
whole reason the bridge tier is worth having: shaving a state machine that
is bound to real `Exchange` doesn't just preserve graph connectivity, it
preserves the actual proof obligations the original upheld, checkably, not
by convention. The exact mechanism — what method checks this, and how — is
not yet designed (see Open Questions).

The core tier alone is the success case for the earlier plan version's
"conform to the shape, unlock isolation for free" thesis; the bridge tier
is the stronger version of that thesis, available only where real proof
machinery already exists to make the stronger claim honest. General,
unstructured Rust stays out of scope for both tiers on purpose; it is a
much harder problem and not the one this crate exists to solve.

This started as `amenable_code`, a scaffolded slot inside the `amenable`
workspace (`github.com/crumplecup/amenable`). It has been split off into its
own repository on further thought: source-code capture is orthogonal to
`amenable`'s constitutional proof-role family — it doesn't need `amenable`'s
traits to exist, and `amenable`'s traits will need it, not the other way
around. Keeping it independent means it can be depended on by anything that
wants exact source capture, not only formal-verification frameworks, and it
avoids entangling `amenable`'s dependency-light core with
`syn`/`quote`/`proc-macro2`/`petgraph`.

`amenable` is expected to take `homecoming` as a dependency once this
design stabilizes, with several of its traits (at minimum `Witness`,
possibly `Evidence`) expected to also implement `Code` — see
`AMENABLE_PLAN.md` in the `amenable` repository for the consuming side of
that relationship.

## Status

Early design, no implementation yet. `syn`/`quote`/`proc-macro2`/`petgraph`
dependencies are resolved and pinned. The crate is a workspace mirroring
`amenable`'s shape, with one more member than previously planned:

- **`homecoming_core`** — the `Code`/`Scope`/`Locality` trait family,
  `Selection`/`Source`/`Binding`, and `Fragment`. No dependency on
  `amenable`. No content yet. No longer defines `Family`/`Sibling`/
  `Relation`/`Tradition` — that was the pre-bridge-tier design and is
  superseded; see "Goal" above.
- **A bridge crate** (name not yet settled) — depends on both
  `homecoming_core` and `amenable_core`; provides the `StateMachine`/
  `Exchange`-gated capability. Not yet scaffolded.
- **`homecoming_derive`** — a `#[derive(...)]` macro generating `Code`
  impls. Deliberately not started — see "Build order" below.
- **`homecoming`** — the top-level facade, re-exporting `homecoming_core`
  (and the bridge crate and `homecoming_derive` once they have content) for
  user convenience, the same sanctioned-exception pattern `amenable` uses
  for its own facade.

Naming is settled for the core trait family (`Code`, `Scope`, `Locality`,
`Selection`, `Source`, `Binding`) and for the convention that a trait's
primary method mirrors the trait's own name in lowercase (`Code::code()`,
`Scope::scope()`). Open for the bridge crate's name and for the derive
macro — `Homecoming` is the working name for the latter, matching the
crate, but whether the macro should mirror a trait name or diverge from it
(both patterns exist in this ecosystem — see `elicitation`/
`elicitation_derive`, where `#[derive(Elicit)]` does not match any single
trait name) is undecided until there's a macro to name.

## Capture and replay, not generation

The framing matters: this is not about synthesizing plausible-looking
source for an opaque runtime value after the fact — that is an open-ended,
judgment-call task that invites shortcuts (the motivating failure case was
`elicitation`'s `ToCodeLiteral` trait falling back to `default()` for an
uninspectable field — code that type-checks and compiles while having no
relationship to the value it claims to reproduce). Everything in a Rust
program starts out as code. The job is to preserve that original source
faithfully as a value flows through the program, not to reverse-engineer a
substitute once the trail has gone cold.

This applies just as much to isolated slices as to whole values. A "shaved"
program assembled from real, previously-captured states and relations is
still capture — every piece it contains really executed, really compiled,
really was part of the original. What would cross the line into generation
is fabricating a smaller program that merely *looks like* what a subset of
the original might have done. The composition traits (below) exist to
combine real captured pieces into new arrangements; they are not licensed
to invent pieces that were never actually there.

## Why this is part of formal verification, not just agent tooling

The original motivating use case was an agent assembling a real, compiling
Rust program by making tool calls that each execute for real, then asking
for the same trace replayed through a different interpreter that emits
source instead of performing the operation. That use case stands, but it is
not the deepest reason this trait family matters. A solver's verdict is
only meaningful relative to the exact statement it checked. The code a
proof ran over — the receipt establishing what, precisely, was verified —
has to be *exact* to be of any value at all. An approximate or
reconstructed rendering of "what the code probably looked like" breaks the
chain of custody between what was verified and what ships, silently, in a
way that looks like assurance while providing none.

## Isolation is the harder, more valuable half of the problem

Anyone can hand back the whole source of a program. That is not useful on
its own — reviewing a proof, or an agent's generated program, by reading
everything is the thing this crate exists to avoid. The valuable capability
is narrower: given one specific point of interest, isolate the *minimal*
slice of code that actually contributes to it, not everything transitively
reachable from it. This is program slicing, applied to Rust source.

A calculator program sharpens why this can't be solved by capture alone,
and also shows where the design landed. Unlike a small, fixed state machine
(a stoplight has three states and three transitions, full stop), a
calculator handles an unbounded number of runtime inputs — there is no
enumerable set of "all the code for all possible calculator sessions."
The resolution is not a separate tracking mechanism that records operations
as they execute. If the calculator is itself modeled as a state machine
(state = current value plus pending operator; transitions = digit entry,
operator selection, equals), then "did you subtract or divide first" is not
a fact that needs recording out-of-band — it is simply *which path was
walked* through the state graph. A specific calculation is a walk; the
sequence of edges taken along that walk is already the trace, with no
separate recorder needed. Isolating "the code for this one calculation" is
then a graph operation — extracting the subgraph along one walked path —
not a new kind of capability.

The first (rejected) instinct was to give scope a fixed taxonomy — Program,
Operation, Case, Value, as a closed set of levels a fragment could belong
to. That repeated the exact mistake `amenable`'s `Provenance` redesign
already corrected once: describing structure instead of capability. The
second (also rejected) instinct was that this needed to work for arbitrary
Rust programs at all — general program slicing over unstructured control
flow is a much harder problem than this crate needs to solve. The design
that stuck: scope the ambition to programs with state-machine shape, where
"isolate a slice" has a precise, well-studied meaning — subgraph
extraction — and `petgraph`, already `Fragment`'s backing representation,
does the hard part natively.

## `Code`, `Scope`, `Locality`

`Code` is the leaf capability, implemented directly on the value it
describes — `self` already *is* the value; `code()` is the alternate
channel for getting its source instead of using it directly:

```rust
pub trait Code {
    fn code(&self) -> Fragment;
}
```

Isolation is a second, composable capability, not a property of `Code`
itself. The guiding intuition is a tracing span: a span doesn't require a
pre-declared taxonomy of "request scope" versus "database-call scope" —
any code that wants to be traceable enters its own span, and isolating
"everything that happened during this span" works identically no matter how
deep it's nested or what kind of thing it represents. `Scope` is that same
idea applied to source instead of logs:

```rust
pub trait Scope: Code {
    /// Everything this fragment depends on to compile standalone, each
    /// paired with the Locality that decides how it renders when scoped.
    fn boundary(&self) -> impl Iterator<Item = (Fragment, Box<dyn Locality>)>;

    /// The isolated slice: this item's own code, plus whatever its
    /// boundary's Localities decide to contribute, recursively. The
    /// drop-out — code isolated from its surrounding program the way a
    /// span isolates a log region from the rest of a trace.
    fn scope(&self) -> Fragment {
        // default, built from code() + boundary(), delegating each
        // entry's rendering to its own Locality::contribute()
    }
}
```

`Locality` decides, for one boundary entry, how it contributes to a scoped
fragment — reproduced in full, referenced by name, omitted entirely, or
something else. This is deliberately a trait, not an enum of fixed
variants: a closed enum would force every future rendering strategy to
either be shoehorned into existing variants or require changing the enum
and every match on it. As a trait, new localities are just new
implementors:

```rust
pub trait Locality {
    fn contribute(&self, dependency: &dyn Code) -> Fragment;
}

pub struct Inline;
impl Locality for Inline {
    fn contribute(&self, dependency: &dyn Code) -> Fragment {
        dependency.code()
    }
}

pub struct Reference;
impl Locality for Reference {
    fn contribute(&self, dependency: &dyn Code) -> Fragment {
        // a name/path fragment, not the dependency's own definition
        todo!()
    }
}

pub struct Omit;
impl Locality for Omit {
    fn contribute(&self, dependency: &dyn Code) -> Fragment {
        // nothing — not even a reference. This dependency is shaved away.
        todo!()
    }
}
```

`Inline`, `Reference`, and `Omit` are the localities the design has concrete
motivation for so far — not the only ones that will ever exist. `Omit`
wasn't anticipated when `Locality` was designed as a trait instead of a
2-variant enum; it fell straight out of the first real domain (shaving a
state machine down to a subset) the design was tried against, with zero
changes needed to `Scope` or `Locality`'s own definition. That is the
extensibility argument for making it a trait, validated rather than
theoretical.

A known cost of this shape, not yet resolved: `Locality` being a trait
means `boundary()`'s items are heterogeneous, so the iterator has to return
`Box<dyn Locality>` rather than a bare `impl Locality` — a `dyn`-dispatch
cost `Provenance` never had to pay, since every `MetadataEntry` was the
same concrete type. Accepted for now in exchange for extensibility; revisit
if it turns out to matter for a real implementation.

## The core tier's success case: structural shaving via `Scope`

Any type implementing `Scope` — no bound beyond `Scope`/`Code` themselves,
core tier, no `amenable` involved — gets shaving for free, via `boundary()`
and the `Locality` each entry is assigned. "Shaving" a value down to a
smaller one is choosing `Inline` for the parts that belong to the desired
subset and `Omit` for the rest. What makes the result a valid "complete
logical entity" rather than a broken fragment is a real, checkable
condition, not a vibe:

- **Edge-closed.** No `Inline`d entry may reference something that was
  `Omit`ted — no dangling references, or the result doesn't compile.
- **Rooted.** At least one entry must be present that doesn't depend on an
  `Inline`d entry to justify its presence — a starting point the rest of
  the shaved result grows from, the same "states are roots" principle
  `amenable` already names (asserted, not derived — see `amenable`'s
  `AMENABLE_PLAN.md`, "States Are Roots, Transitions Are Relations").

Both conditions are graph-native — reachability and connectivity are
exactly what `petgraph` already computes, not bespoke logic this crate
needs to invent. This is the whole claim the core tier makes: the shaved
result is structurally sound. Nothing here says it preserves whatever the
*original* was proven to guarantee, because at the core tier nothing was
necessarily proven in the first place.

## The bridge tier's success case: proof-preserving shaving

This is where the stronger claim lives, and it requires the real
`amenable_core::StateMachine`/`amenable_core::Exchange` traits, proof
tokens included — not a lookalike. A type satisfying `amenable`'s actual
`Exchange` has transitions that were *proven* lawful, not merely present;
`Exchange`'s `Precondition`/`Postcondition` `Evidence` and `ProofToken`s
are exactly the machinery needed to check whether a shaved subset still
upholds what the original proved, rather than only whether it's still
graph-connected. A homecoming-native lookalike trait could copy the shape
but not this property, because there would be nothing to check against.

The bridge crate's blanket impl is expected to look like:

```rust
impl<T> Scope for T
where
    T: amenable_core::StateMachine + amenable_core::Exchange<
        <T as amenable_core::StateMachine>::State,
        <T as amenable_core::StateMachine>::State,
    >,
{
    fn boundary(&self) -> impl Iterator<Item = (Fragment, Box<dyn Locality>)> {
        // walk the StateMachine's states and Exchange's transitions
    }
}
```

Not yet designed: the concrete mechanism that checks "does this shaved
subset still uphold the proof obligations `Exchange` originally
established" — what gets called, what gets compared to what, and what it
means for that check to fail. This was asserted as the bridge tier's
payoff in conversation before it was designed as a signature; resolving it
concretely is bridge-crate work, not core-crate work, and comes after the
core tier is implemented and compiling (see Phased Implementation Plan).

## Shifting scope: `Selection`, `Source`, and `Binding`

Core-tier concept, not specific to either tier's shaving mechanism — any
`Scope` implementor's `boundary()` entries can be filtered by `Selection`
and value-bound by `Source`, whether the implementor is a bare `Code` value
or a `StateMachine`/`Exchange`-bound bridge-tier type.

Shaving a value down to a smaller one turned out not to be one decision
but two independent ones, and conflating them was the mistake in an
earlier pass at this. Think of a program like a buffet: an agent can
pull components off the table, in any order, to build a platter. "Shifting
scope" splits into which components land on the platter at all, and,
separately, whether each one's values are frozen or still adjustable —
two questions, not one.

**Which components are included** is still `Locality`'s job
(`Inline`/`Reference`/`Omit`), but the earlier framing pictured `boundary()`
assigning each entry a fixed `Locality` intrinsically, as if "isolate this
one walked path" were the only kind of request there'd ever be. The buffet
reframes it: a platter isn't necessarily one walked path, it can be an
arbitrary, deliberately curated selection — "just the `+ - * /` keys," say,
which no single session ever walks as a path, but which is a perfectly
valid, edge-closed, rooted subset all the same. What decides inclusion,
then, is an external, pluggable policy:

```rust
pub trait Selection {
    /// Should this Sibling or Relation be on the platter at all?
    fn includes(&self, item: &dyn Code) -> bool;
}
```

**Whether an included item's values are frozen or open** is a different
question, and it doesn't map onto `Locality` at all — an `Inline`d entry
can still have inputs that are either fixed to a specific observed value
or left as an open parameter for a future caller to supply.
The right mental model turned out to be Rust's own block scoping: a value's
lifetime is bounded by the scope it's declared in, and whether it survives
past that scope is a choice, not a fact about the value itself. Emission
time is one scope; the emitted, compiled program's own later runtime is a
different, separate one. A value crosses that boundary one of two ways —
copied across as a frozen literal (`Bound`), or not copied across at all,
with the emitted program instead declaring its own fresh, empty slot in its
own scope, to be filled by whoever calls *it*, later (`Free`). Whether the
bound value happened to come from a replayed file or the last live session
doesn't matter to this question — both are just data sitting in the
emission-time scope, available to be copied across the boundary if asked;
the `Source` doesn't need to distinguish where it got an answer, only
whether it has one:

```rust
pub trait Binding {
    fn contribute(&self, value: &dyn Code) -> Fragment;
}

pub struct Bound;
impl Binding for Bound {
    fn contribute(&self, value: &dyn Code) -> Fragment {
        value.code() // copied across the boundary as a frozen literal
    }
}

pub struct Free;
impl Binding for Free {
    fn contribute(&self, _value: &dyn Code) -> Fragment {
        // a fresh, open parameter slot in the emitted program's own scope
        todo!()
    }
}

pub trait Source {
    /// A concrete value for this slot, if the emission-time scope has one
    /// available. None means: no value here, this slot stays Free.
    fn value_for(&self, slot: SlotId) -> Option<Fragment>;
}
```

`Selection` and `Source` are independent, composable pieces — a Lego kit,
not a fixed set of "modes." Neither knows the other exists. Emission takes
both:

```rust
fn scope_with(&self, selection: &dyn Selection, source: &dyn Source) -> Fragment
```

The two motivating requests are just two different pairs of the same two
parts, not two special cases requiring their own machinery:

- *"The keys the user pressed this session, hardcoded forever"* —
  `Selection` = the path this session actually walked; `Source` = that
  session's (or a replayed file's) recorded values, answering `Some` for
  everything.
- *"A calculator with just `+ - * /`"* — `Selection` = an arithmetic
  capability filter; `Source` = a trivial source that always answers
  `None`, leaving every value free.

Any other pair — a capability filter with some values pre-bound and others
left open, say — is a new, valid point on the spectrum nobody had to
explicitly design for, which is the same payoff `Locality`'s extensibility
already demonstrated once with `Omit`.

Not yet resolved: exactly how `scope_with()` composes with the simpler,
parameterless `scope()` default from `Scope` — whether `scope()` becomes a
convenience wrapper over `scope_with()` with a trivial always-include
`Selection` and always-bind `Source`, or whether they stay genuinely
separate entry points. Left for Phase 4, alongside `Selection`/`Source`'s
own implementation.

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
primitive's `Code::code()`) wraps a `syn::Expr` with no children; a
composite node carries edges to the nodes it depends on, and
`Scope::boundary()`'s entries correspond to a node's outgoing edges. This
is the canonical, concrete representation, not a placeholder for something
later — any trait implemented directly on foreign standard-library types
must live in the crate that defines it (Rust's orphan rule leaves no other
option; learned the hard way while building `amenable_std`, see
`amenable`'s `AMENABLE_PLAN.md`), so `Code` impls for std primitives live
in `homecoming_core` alongside `Code` itself.

## Lateralizing composition traits: "A + B = C" as an explicit operation

`Scope`/`Locality` handle taking a fragment *apart* — isolating a minimal
slice out of a larger context. A separate, complementary concern is putting
fragments *together* — composing several `Code`-bearing parts into a new
whole. A blanket `impl<T: Code> Code for Vec<T>` covers fixed Rust
composite shapes fine, but it cannot cover an agent *choosing*, at runtime,
which composition shape to apply to a pile of fragments it just produced —
fold them into a sequential block, thread them as call arguments, assemble
them as struct-literal fields, stack them as match arms. That choice needs
to be a first-class, visible, invokable step, not something buried in a
generic impl. This likely wants to be several lateralizing traits, one per
composition shape (a working sketch: `SequenceCode`, `ApplyCode`,
`ConstructCode`, `MatchCode`), each producing a new `Output: Code` so the
result re-enters the system as a fresh, composable fragment.

This is secondary to the core-tier shaving success case, not a prerequisite
for it — deferred until there's a concrete composition need the success
case surfaces. Two forks are genuinely open and are meant to be resolved
empirically when that happens, not by pre-specifying the grammar:

- Binary composition ("A + B = C" literally, chained pairwise for bigger
  structures) versus native N-ary (`IntoIterator<Item = P>` from the start,
  since a block of statements or a struct's fields is not naturally binary).
- Whether every composition shape produces the same underlying `Fragment`
  node type (just tagged differently), making the lateralizing traits
  *builders* over one substrate rather than each having a genuinely distinct
  `Output` type.

## Round-trip as a proof obligation

`Code`'s own soundness is not trusted by construction — it is a checkable
claim, verified two ways that do different jobs:

- **Simple form.** Emit a value's fragment via `Code::code()`, reconstruct a
  value from that fragment, and confirm it equals the original. This is the
  baseline soundness obligation for any `Code` implementor on its own. For
  a primitive this is close to trivial, but it is exactly the check that
  would have caught the `ToCodeLiteral` `default()`-fallback failure mode
  mechanically, with no review required. `Scope::scope()` has the same
  obligation one level up: the isolated slice, compiled standalone, must
  behave identically to the equivalent code extracted by hand — and for a
  bridge-tier shaved result, "behave identically" means preserving the
  proof obligations the original's `Exchange` upheld, not just compiling.
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

- No dependency on `amenable` or any proof-role trait family from
  `homecoming_core` specifically — `Code`/`Scope`/`Locality` stay
  consumable by anything that wants exact source capture and structural
  subsetting, not only formal-verification frameworks. The `amenable`
  dependency is confined to the bridge crate, which is opt-in, not part of
  `homecoming_core` or the top-level `homecoming` facade's required graph.
- Where `amenable` traits are reused (the bridge crate's
  `StateMachine`/`Exchange`), reuse the real thing, proof tokens included —
  no homecoming-native lookalike traits that copy the shape without the
  substance, since the substance (real proof machinery) is the entire
  reason the bridge tier exists.
- No unnecessary runtime dependencies beyond `syn`/`quote`/`proc-macro2`/
  `petgraph`, which are load-bearing for the core capability, not optional
  extras.
- No reconstruction-after-the-fact code paths — every `Code` impl captures
  real source, verified by the round-trip obligation, never a best-guess
  substitute. A shaved result is a reassembly of real, previously captured
  pieces, never a fabrication of what a subset might have looked like.
- No closed taxonomies for open-ended design questions (scope kinds,
  locality kinds) — describe the capability needed, let implementors supply
  the cases, the same discipline `amenable`'s `Provenance` redesign
  established first.
- No targeting arbitrary, unstructured Rust programs — the design scopes
  deliberately to programs with a `Scope` implementation (core tier) or a
  state-machine shape (bridge tier), where isolation has a precise,
  checkable meaning, not a harder general program-slicing problem this
  crate doesn't need to solve to be useful.
- No fixed "modes" for configuring emission (a replay mode, an interactive
  mode) — `Selection` and `Source` are independent, composable pieces, and
  any combination of them is a valid configuration, not just the two that
  motivated the design.

## Phased Implementation Plan

### Phase 1: Split from `amenable_code` — done

- [x] Create the standalone `homecoming` repository.
- [x] Carry over the resolved `syn`/`quote`/`proc-macro2`/`petgraph`
  dependencies and version pins.
- [x] Write this design record before any implementation, so the design
  discussion from the `amenable_code` incubation period isn't lost.

### Phase 2: Workspace restructuring — done

- [x] Split into `homecoming_core` (traits), `homecoming` (facade),
  `homecoming_derive` (proc-macro, scaffolded but empty), mirroring
  `amenable`'s shape.
- [x] Settle the naming split: `Code` is the trait, `Homecoming` is the
  working name for the derive macro (not finalized).

### Phase 3: `Fragment` and minimal `Code` leaves

- [ ] Define `Fragment` as a `petgraph` graph with `syn`-typed node payloads
  (leaf nodes wrap `syn::Expr`; composite nodes carry a shape tag plus child
  edges).
- [ ] Implement `Code` for just enough values to build the Phase 4 success
  case (e.g. a small enum's variants) — not a broad std-lib sweep; that is
  deferred until something concretely needs it.
- [ ] Write the simple-form round-trip check for every leaf `Code` impl,
  and confirm it actually catches a deliberately fabricated fragment that
  does not reconstruct its claimed value.

### Phase 4: Core-tier shaving demo (no `amenable` involved)

The core tier's success case: prove structural shaving — `Scope`,
`Locality`, `Selection`, `Source`, `Binding` — actually works end to end,
by hand, on a type with no `amenable` bound at all, before the bridge
crate exists.

- [ ] Implement `Inline`, `Reference`, and `Omit` as `Locality` and confirm
  the round-trip obligation holds through `scope()`, not just `code()`.
- [ ] Hand-implement `Code`/`Scope` directly for a stoplight-shaped enum
  (three states, three hand-written transition methods) with no
  `StateMachine`/`Exchange`-style trait bound at all — just `Scope`'s own
  required methods, to confirm the core tier stands on its own.
- [ ] Shave the stoplight down to a smaller valid result (e.g. two states
  and the one relation between them) and confirm the result is edge-closed,
  rooted, and compiles standalone.
- [ ] Model a calculator by hand and confirm that isolating one calculation
  is exactly extracting the subgraph along one walked path — no separate
  tracking mechanism required.
- [ ] Define `Selection` and `Source`, and `scope_with(selection, source)`.
- [ ] Implement `Bound`/`Free` as `Binding` and confirm the round-trip
  obligation holds for both — a `Bound` slot round-trips to the same
  value; a `Free` slot round-trips to a program that accepts a new one.
- [ ] Reproduce both motivating configurations on the calculator example —
  a frozen replay of one session (`Selection` = the walked path, `Source`
  = that session's values) and a general `+ - * /` calculator (`Selection`
  = an arithmetic filter, `Source` = always `None`) — from the same two
  composable pieces, with no mode-specific code written for either.
- [ ] Decide how `scope()` and `scope_with()` relate.

### Phase 5: Composition

- [ ] Implement `Code for Vec<T>` (or an equivalent first composite) and
  let its actual composition needs settle the lateralizing-trait arity and
  node-shape-count questions empirically.
- [ ] Name and implement the lateralizing composition traits that fall out
  of that exercise.

### Phase 6: The bridge crate — proof-preserving shaving

Not started until Phase 4 proves the core tier's mechanics actually work —
the bridge crate builds directly on `Scope`/`Locality`/`Selection`/
`Source`, it doesn't reinvent them.

- [ ] Scaffold the bridge crate (name TBD) as a new workspace member,
  depending on `homecoming_core` and `amenable_core`.
- [ ] Implement the blanket `Scope` impl gated on real
  `amenable_core::StateMachine` + `amenable_core::Exchange`.
- [ ] Design and implement the concrete mechanism that checks a shaved
  subset still upholds the proof obligations the original `Exchange`
  established — the open question from "The bridge tier's success case"
  above.
- [ ] Re-run the stoplight/calculator demos from Phase 4 through a type
  that additionally implements real `amenable::StateMachine`/`Exchange`,
  and confirm the bridge tier's stronger guarantee actually holds where the
  core tier's did not attempt to make that claim.

### Phase 7: `amenable` becomes a consumer

- [ ] `amenable` adds `homecoming` (and the bridge crate) as a dependency.
- [ ] At least one `amenable` trait (`Witness` is the leading candidate)
  implements or requires `Code`.
- [ ] Exercise the strong-form round-trip check against a real
  `Witness`-bearing example once `amenable`'s proof-emission machinery
  exists.

### Phase 8: The derive macro

Not started until Phases 3–6 have produced enough hand-written `impl Code`/
`impl Scope` examples, core and bridge tier both, to extract a reliable
pattern from — designing the macro before the representation it targets is
proven repeats the mistake that motivated this crate's own existence.

- [ ] Extract the repeatable shape of the hand-written examples into macro
  logic.
- [ ] Decide the derive macro's final name.
- [ ] Confirm `#[derive(..)]`-generated impls satisfy the same round-trip
  obligation as hand-written ones, with no special exemption for
  macro-generated code.

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
- Is `Box<dyn Locality>` in `Scope::boundary()`'s item type the right
  tradeoff, or should `Scope` instead be generic over one `Locality` type,
  trading flexibility (heterogeneous localities per fragment) for avoiding
  `dyn` dispatch?
- Is "edge-closed and rooted" a sufficient definition of "complete logical
  entity" for core-tier shaving, or does even the core tier need a
  stronger check for some cases?
- What concrete mechanism does the bridge crate use to check that a shaved
  subset upholds the proof obligations the original `Exchange` established
  — a method that compares `Precondition`/`Postcondition` `Evidence`
  directly, something that re-runs `Witness` against the shaved result, or
  something else not yet considered?
- Should the derive macro's name mirror `Code` (or `Scope`), or diverge
  entirely, the way `#[derive(Elicit)]` diverges from any single trait name
  in `elicitation`?
- What should the bridge crate be named?
- How does `scope_with(selection, source)` relate to `scope()` — a
  convenience wrapper with trivial always-include/always-bind defaults, or
  a genuinely separate entry point?
- What is `SlotId` (or whatever identifies a value for `Source::value_for`)
  concretely — a position in a captured call's argument list, something
  derived from the `Fragment` graph itself, or something else?
- Does `Selection` need the same `dyn`-dispatch treatment `Locality` does,
  for the same reason (heterogeneous policies), or can it stay a single
  concrete type per `scope_with()` call?

## Success Condition

This plan succeeds when any type implementing `Scope` gets exact,
minimally-shaved sub-programs — provably edge-closed and rooted, assembled
only from real captured pieces — entirely for free at the core tier, with
no `amenable` dependency required to get there. When a type additionally
bound to real `amenable::StateMachine`/`Exchange` gets the stronger
guarantee that a shaved subset provably preserves the proof obligations
the original upheld, not just its graph structure. When any point on the
shifting-scope spectrum, from a frozen replay of one session to a fully
general capability-filtered program, is reachable by composing `Selection`
and `Source` rather than by choosing among a fixed set of modes. And when
`amenable`'s proof-bearing traits can lean on this machinery to prove, not
just assert, that the code a proof ran over is the code that actually
ships.
