# HOMECOMING_PLAN.md

## Goal

`homecoming` is a small crate family defining `Code`, the trait for
high-fidelity capture, isolation, and replay of Rust source code, plus the
composition and scoping machinery needed to assemble fragments into real
programs and extract minimal slices back out of them.

The design deliberately does not target arbitrary Rust programs, and it is
deliberately two-tiered rather than one design applied uniformly.

**Core tier** — `Code`, `Scope`, `Locality`, `Extent` in `homecoming_core`
— has no dependency on `amenable` and works for any type. What it buys: a
program that could previously only *perform* an operation can now also
*emit* the source of that operation, and isolate a minimal, structurally
valid slice (edge-closed, rooted) of a larger program. No claim beyond
structural validity is made at this tier, because nothing about a bare
`Code` implementor says anything was ever proven.

**`homecoming_amenable`** — a workspace member depending on *both*
`homecoming_core` and `amenable_core`, surfaced through the top-level
`homecoming` facade behind an `amenable` Cargo feature so the core trait
infrastructure stays lean by default — targets programs shaped like a
state machine with exchange properties, using `amenable`'s actual
`StateMachine`/`Exchange` traits directly, proof-token machinery included,
not a homecoming-native lookalike. That is a deliberate choice, not an
oversight: `amenable_core::StateMachine` is minimal enough (`{ type State;
type Invariant; }`) that reinventing it bought nothing, but
`amenable_core::Exchange` carries real `Evidence`/`ProofToken` machinery
that a lookalike trait *couldn't* carry, because that machinery is the
whole reason this crate is worth having: `StateMachine`/`Exchange` bounds
let `Extent` reason about which code is live directly from proven
transition structure, not only from `Scope`'s generic edge-closed/rooted
graph closure — shaving a state machine bound to real `Exchange` doesn't
just preserve graph connectivity, it preserves the actual proof
obligations the original upheld, checkably, not by convention. The exact
mechanism — what method checks this, and how — is not yet designed (see
Open Questions).

The core tier alone is the success case for the earlier plan version's
"conform to the shape, unlock isolation for free" thesis; `homecoming_amenable`
is the stronger version of that thesis, available only where real proof
machinery already exists to make the stronger claim honest. General,
unstructured Rust stays out of scope for either on purpose; it is a much
harder problem and not the one this crate exists to solve.

This started as `amenable_code`, a scaffolded slot inside the `amenable`
workspace (`github.com/crumplecup/amenable`). It has been split off into its
own repository on further thought: source-code capture is orthogonal to
`amenable`'s constitutional proof-role family and useful to anything that
wants exact source capture, not only formal-verification frameworks —
keeping `homecoming_core` independent avoids entangling `amenable`'s
dependency-light core with `syn`/`quote`/`proc-macro2`/`petgraph`
unconditionally.

The dependency direction is settled as one arrow, not two: `homecoming`
depends on `amenable`, never the reverse. The motivating use case is
generating modular MCP tool code from compartmentalized operations —
`Extent`'s naming/liveness machinery — where every generated tool's code
must be traceable back to the specific formal verification it came from,
which needs both crates' capabilities cooperating, not merely
interoperating. Some audit tooling originally sketched as living inside
`amenable` (see `AMENABLE_PLAN.md`) turns out to belong in
`homecoming_amenable` instead — a change of location, not a contradiction,
once it's tooling that inherently needs both `Code`/`Extent` and
`StateMachine`/`Exchange` in scope at once. Where `amenable`'s own traits
(`Witness` in particular) want to cooperate with source capture, that
cooperation lives in `homecoming_amenable` as glue/adapter code, not as a
dependency `amenable` itself takes on — `amenable_core` stays exactly as
dependency-light as it already is.

## Status

Early design, no implementation yet. `syn`/`quote`/`proc-macro2`/`petgraph`
dependencies are resolved and pinned. The crate is a workspace mirroring
`amenable`'s shape, with one more member than previously planned:

- **`homecoming_core`** — the `Code`/`Scope`/`Locality`/`Extent` trait
  family and `Fragment`, plus `Ir` (this crate's own `petgraph`-backed
  `Fragment` implementation). No dependency on `amenable`.
  `Selection`/`Source`/`Binding` are designed (see below) but not yet
  implemented. No longer defines `Family`/`Sibling`/`Relation`/`Tradition`
  — that was an earlier pre-`homecoming_amenable` design and is
  superseded; see "Goal" above.
- **`homecoming_amenable`** — depends on both `homecoming_core` and
  `amenable_core`; provides the `StateMachine`/`Exchange`-gated
  capability. Not yet scaffolded.
- **`homecoming_derive`** — a `#[derive(...)]` macro generating `Code`
  impls. Deliberately not started — see "Build order" below.
- **`homecoming`** — the top-level facade, re-exporting `homecoming_core`
  (and `homecoming_amenable`, behind an `amenable` feature, and
  `homecoming_derive`, once they have content) for user convenience, the
  same sanctioned-exception pattern `amenable` uses for its own facade.

Naming is settled for the core trait family (`Code`, `Scope`, `Locality`,
`Extent`, `Selection`, `Source`, `Binding`), for `homecoming_amenable`, and
for the convention that a trait's primary method mirrors the trait's own
name in lowercase (`Code::code()`, `Scope::scope()`). Open for the derive
macro's name — `Homecoming` is the working name, matching the crate, but
whether the macro should mirror a trait name or diverge from it (both
patterns exist in this ecosystem — see `elicitation`/`elicitation_derive`,
where `#[derive(Elicit)]` does not match any single trait name) is
undecided until there's a macro to name.

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

## `homecoming_amenable`'s success case: proof-preserving shaving

This is where the stronger claim lives, and it requires the real
`amenable_core::StateMachine`/`amenable_core::Exchange` traits, proof
tokens included — not a lookalike. A type satisfying `amenable`'s actual
`Exchange` has transitions that were *proven* lawful, not merely present;
`Exchange`'s `Precondition`/`Postcondition` `Evidence` and `ProofToken`s
are exactly the machinery needed to check whether a shaved subset still
upholds what the original proved, rather than only whether it's still
graph-connected. A homecoming-native lookalike trait could copy the shape
but not this property, because there would be nothing to check against.

`homecoming_amenable`'s blanket impl is expected to look like:

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
means for that check to fail. This was asserted as `homecoming_amenable`'s
payoff in conversation before it was designed as a signature; resolving it
concretely is `homecoming_amenable` work, not `homecoming_core` work, and
comes after the core tier is implemented and compiling (see Phased
Implementation Plan).

## `Extent`: the user's grammar for what's worth isolating

`Scope` and `Extent` borrow their names from the same distinction
programming language theory draws between *scope* and *extent*: scope is
the static, lexical question (what does this structure look like), extent
is about a binding's lifetime. This design uses that borrowed vocabulary
for a narrower purpose than PL theory's own dynamic/runtime sense of the
word: `Scope` answers what; `Extent` answers *which* — a grammar for
declaring which methods and types are meaningful units worth being able to
isolate at all.

"Live" here is deliberately the compiler's sense of liveness, not a
runtime-tracing one: a definition is live at a point if it is used to
compute a result reachable from there, and dead otherwise. `Extent` does
not track anything as it happens — it names an anchor already present in
the dependency graph a `Scope` implementor builds (`Ir`'s `petgraph` graph,
concretely, at the core tier), and querying a name just reuses `Scope`'s
existing edge-closed/rooted closure computed from that anchor. An earlier
pass modeled this as an RAII guard mirroring `tracing`'s span lifecycle
(`start()` returning a `Guard`, recording ending on drop), and that was a
wrong turn: it borrowed the *temporal* half of the tracing analogy (spans
start and stop at runtime) when what's actually needed is the *naming*
half (spans are addressable by name). Recording user-submitted runtime
values live is a different, already-solved problem — a queue-shaped
`Source` draining into `Binding` (below) — not `Extent`'s job at all.

```rust
pub trait Extent: Code {
    /// The live code for a named, isolatable unit, or `None` if `name`
    /// was never declared for this type.
    fn anchor(&self, name: &str) -> Option<Self::Fragment>;
}
```

Declaring an extent is a compile-time fact recorded once, not a runtime
event repeated per call: the derive is expected to emit a static
`(type, method, name)` descriptor per annotated method via
`inventory::submit!`, collected into one process-wide, read-only
registry — `inventory` fits because the set of declared extents is
genuinely static, known at compile time from the annotations in the user's
own source, unlike the mutable, per-call state a `tracing` span registry
actually needs. Querying a name resolves it through that registry to find
which type/method it names, then calls `anchor()` on a live instance of
that type to get back the closure computed fresh from its current
dependency graph — nothing is tracked between queries; each one is
answered from scratch, the same way `Scope::boundary()` is recomputed
rather than cached.

### Why this matters more than a mechanism: it's the user's actual API

The point of `Extent` isn't just "a well-shaped way to name a cut point."
It is how a user expresses which cut points in *their own program* are
meaningful, in their own vocabulary, without hand-writing `Code`/`Scope`
impls at all. The derive-driven build order (Phase 8, not yet started) is
expected to land here specifically: `#[derive(Homecoming)]` on a type, plus
an attribute on the methods the user considers meaningful units, registers
those methods' anchors via `inventory::submit!` automatically — the same
role `#[instrument]` plays for `tracing`, minus the runtime enter/exit,
since nothing here needs to happen at call time. A method the user does
*not* annotate never becomes its own isolatable unit; it is just plumbing,
absorbed into whatever encloses it, invisible to the isolation machinery
entirely.

That reframes something about `Selection` (above) worth stating plainly:
`Extent`-annotation is what creates the *menu* `Selection` later chooses
from, not the other way around. Revisit the calculator: annotating
`divide`, `multiply`, `add`, and `subtract` with an extent attribute is
what makes "a calculator with just `+ - * /`" possible to ask for at all —
those four methods become individually isolatable units because the user
declared them meaningful. If the calculator also has `clear`,
`memory_store`, or display-formatting methods that were never annotated,
they are never on the menu — not omitted by a `Selection` policy at
shave-time, simply never candidates in the first place. `Selection` then
picks among whatever the user's own `Extent` annotations made available —
"just `+` and `-`," say, out of all four. The buffet metaphor gets a
missing piece: `Extent` is what decides which dishes exist on the table at
all; `Selection` is what ends up on a given plate.

`homecoming` is expected to use the same grammar for its own work, not a
special internal-only mechanism — its own std-lib and example `Code`/
`Scope` impls should eventually be expressible through `Extent`
annotations too, once the derive exists, rather than the hand-written
`impl Scope` this crate currently relies on being a permanently separate
path from what user code does.

## Shifting scope: `Selection`, `Source`, and `Binding`

Core-tier concept, not specific to either tier's shaving mechanism — any
`Scope` implementor's `boundary()` entries can be filtered by `Selection`
and value-bound by `Source`, whether the implementor is a bare `Code`
value or a `StateMachine`/`Exchange`-bound `homecoming_amenable` type.
Which items are *available* to select among is `Extent`'s job (above),
not `Selection`'s — `Selection` only ever operates over what an `Extent`
annotation already made isolatable.

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
then, is an external, pluggable policy, layered on top of `Locality` rather
than replacing it — an entry survives only if `Selection` includes it *and*
its `Locality::contribute` still answers `Some`:

```rust
pub trait Selection<F: Fragment> {
    fn includes(&self, item: &F) -> bool;
}
```

Implemented via `Scope::scope_with`, a second required method alongside
`scope()` — siblings, not one wrapping the other:

```rust
fn scope_with(&self, selection: &dyn Selection<Self::Fragment>) -> Self::Fragment
```

`scope_with` has no default body for the same reason `scope()` doesn't:
`Fragment` still carries no composition capability to build a generic
default from. Each `Scope` implementor hand-writes both, pending the
lateralizing composition traits (below).

**Whether an included item's values are frozen or open** is a different
question, and it doesn't map onto `Locality` at all — an `Inline`d entry
can still have inputs that are either fixed to a specific observed value
or left as an open parameter for a future caller to supply. It also
doesn't route through `Scope::boundary()`/`scope_with()` at all: `Locality`
and `Selection` decide whether a whole boundary *entry* is included, but a
value like one argument of one call is finer-grained than an entry, so
`Source`/`Binding` are queried directly by whichever method is assembling
that entry's own code (`Add::code_with` in the calculator example, not
`Scope::scope_with`).

The right mental model turned out to be Rust's own block scoping: a value's
lifetime is bounded by the scope it's declared in, and whether it survives
past that scope is a choice, not a fact about the value itself. Emission
time is one scope; the emitted, compiled program's own later runtime is a
different, separate one. A value crosses that boundary one of two ways —
copied across as a frozen literal (`Bound`), or not copied across at all,
with the emitted program instead declaring its own fresh, empty slot in its
own scope, to be filled by whoever calls *it*, later (`Free`):

```rust
pub trait Binding<F: Fragment> {
    fn contribute(&self) -> F;
}

pub struct Bound<F> { value: F }
impl<F: Fragment> Binding<F> for Bound<F> {
    fn contribute(&self) -> F { self.value.clone() }
}

pub struct Free<F> { placeholder: F }
impl<F: Fragment> Binding<F> for Free<F> {
    fn contribute(&self) -> F { self.placeholder.clone() }
}
```

Both are self-contained, constructed with whatever they need up front,
rather than mirroring `Locality::contribute(&self, dependency: &F)`'s
shape — `boundary()` always hands `Locality` a real dependency to decide
about, but `Free` has nothing to be handed when `Source` answered `None`;
passing a parameter there would mean passing a dummy.

`Source` answers whether a value is available for a slot at all,
decoupled entirely from `Fragment`/`Code` — it hands back an `Args`, a raw
value with no code representation, and whoever wants a `Fragment` reaches
for whatever `Code` impl that raw value's own type already has:

```rust
pub trait Args {
    type Value;
    fn value(&self) -> Self::Value;
}

pub trait Source {
    type Args: Args;

    /// A non-consuming peek — safe to call more than once with the same
    /// answer.
    fn value_for(&self, slot: &str) -> Option<Self::Args>;

    /// Permitted to consume or mutate whatever backs it. Defaults to a
    /// non-consuming peek — most `Source`s (a fixed, already-captured
    /// record) have nothing to drain and never override this.
    fn value_mut_for(&mut self, slot: &str) -> Option<Self::Args> {
        self.value_for(slot)
    }
}
```

An earlier pass tried to put the stable/mutating distinction on the
*value* `Source` hands back (an `ArgsMut` trait alongside `Args`) instead
of on `Source` itself — but a value with no way back to its `Source` has
nothing to mutate. Putting `value_mut_for` on `Source`, taking `&mut
self`, is what makes real draining mechanically possible: a queue-shaped
`Source` (`AddArgs` in the calculator example, holding user-submitted
values one slot's queue at a time) can actually pop from its own state
inside `value_mut_for`, something no amount of cleverness on the returned
value alone could do.

`Selection` and `Source` are independent, composable pieces — a Lego kit,
not a fixed set of "modes." Neither knows the other exists, and neither is
routed through the other's entry point:

- *"The keys the user pressed this session, hardcoded forever"* —
  `Selection` = the path this session actually walked; `Source` =
  `AddArgs` populated with that session's actual values, drained via
  `Add::code_with` as each operation's code is emitted.
- *"A calculator with just `+ - * /`"* — `Selection` = an arithmetic
  capability filter (`PlusMinusOnly`); `Source` = an empty `AddArgs`,
  answering `None` for everything and leaving every value free — no
  separate always-free type needed.

Any other pair — a capability filter with some values pre-bound and others
left open, say — is a new, valid point on the spectrum nobody had to
explicitly design for, which is the same payoff `Locality`'s extensibility
already demonstrated once with `Omit`. `Add::code_with` demonstrates this
concretely: both slots bound is direct substitution (`3 + 4`), both free
matches `code()` exactly (`|a: i32, b: i32| a + b`), and a mix partially
applies (`|b: i32| 3 + b`) — a spectrum, not a binary switch, at the value
level in a way `Selection` alone (whole-entry, not per-value) can't reach.

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
  `homecoming_amenable`-shaved result, "behave identically" means
  preserving the proof obligations the original's `Exchange` upheld, not
  just compiling.
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
  `homecoming_core` specifically — `Code`/`Scope`/`Locality`/`Extent` stay
  consumable by anything that wants exact source capture and structural
  subsetting, not only formal-verification frameworks. The `amenable`
  dependency is confined to `homecoming_amenable`, surfaced through the
  top-level `homecoming` facade behind an `amenable` Cargo feature, not
  part of `homecoming_core`'s or `homecoming`'s default/required graph.
  The dependency arrow runs one way only — `homecoming` depends on
  `amenable`, never the reverse; any cooperation `amenable`'s own traits
  need with source capture is glue code living in `homecoming_amenable`,
  not a dependency `amenable_core` takes on.
- Where `amenable` traits are reused (`homecoming_amenable`'s
  `StateMachine`/`Exchange`), reuse the real thing, proof tokens included —
  no homecoming-native lookalike traits that copy the shape without the
  substance, since the substance (real proof machinery) is the entire
  reason `homecoming_amenable` exists.
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
- No treating this as a bag of independent traits that happen to share a
  crate — `Code`, `Locality`, `Scope`, and `Extent` are a family, and their
  members should constrain each other through associated types the way
  `Iterator`/`IntoIterator` and `amenable`'s `Evidence`/`Witness` already
  do, not merely be independently satisfiable. `Fragment` was a concrete
  type in an earlier pass specifically because this wasn't being honored —
  `Code`, `Locality<F>`, and `Scope` couldn't actually express their real
  relationship to a type that wasn't itself an interface.
- No targeting arbitrary, unstructured Rust programs — the design scopes
  deliberately to programs with a `Scope` implementation (core tier) or a
  state-machine shape (`homecoming_amenable`), where isolation has a
  precise, checkable meaning, not a harder general program-slicing problem
  this crate doesn't need to solve to be useful.
- No fixed "modes" for configuring emission (a replay mode, an interactive
  mode) — `Selection` and `Source` are independent, composable pieces, and
  any combination of them is a valid configuration, not just the two that
  motivated the design.
- No conflating "which units can be isolated at all" with "which of those
  units are chosen for a given shaved result" — `Extent` (author-declared,
  what's on the menu) and `Selection` (shave-time policy, what's on a
  given plate) are different questions answered at different times, not
  one mechanism wearing two names.

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

### Phase 3: `Fragment` and minimal `Code` leaves — done, first pass

- [x] Define `Fragment` as a `petgraph` graph with `syn`-typed node payloads
  (leaf nodes wrap `syn::Expr`; composite nodes carry a shape tag plus child
  edges). Currently only the leaf case (`Fragment::leaf`) is implemented —
  a single-node graph. Composite nodes are Phase 5 work.
- [x] Implement `Code` for enough std primitives to exercise the design
  (`bool`, `char`, all integer types) — literal construction via `syn::Lit`
  built directly from the value, not string-parsing, so there is no
  fallible path and no `.unwrap()`/`.expect()` anywhere in the impl.
- [x] Write the simple-form round-trip check for every leaf `Code` impl,
  and confirm it actually catches a deliberately fabricated fragment that
  does not reconstruct its claimed value — `tests/code_test.rs`'s
  `fabricated_fragment_fails_the_round_trip` does exactly this and passes.

### Phase 4: Core-tier shaving demo (no `amenable` involved)

The core tier's success case: prove structural shaving — `Scope`,
`Locality`, `Extent`, `Selection`, `Source`, `Binding` — actually works
end to end, by hand, on a type with no `amenable` bound at all, before
`homecoming_amenable` exists.

- [x] Implement `Inline`, `Reference`, and `Omit` as `Locality`. Two real
  refinements surfaced only once this was actually written, not while it
  was still a sketch: `Locality::contribute` takes `&Fragment`, not
  `&dyn Code` as originally sketched (avoids needing `Code` to be
  object-safe for no real benefit); it returns `Option<Fragment>`, not a
  bare `Fragment` (`Omit` has nothing honest to return otherwise — `None`
  is the correct answer, not an empty/unit fragment standing in for
  "nothing"). `Reference` also turned out to need to carry its own
  `syn::Path`, supplied by the caller — nothing in `&Fragment` alone
  identifies what name to reference by.
- [x] Confirm the round-trip obligation holds through `scope()`, not just
  `code()` — `tests/scope_test.rs`'s `transition_scope_includes_both_
  boundary_states` re-parses `scope()`'s output as `syn::Expr` and confirms
  it actually compiles as valid Rust, not just that it looks plausible.
- [x] Hand-implement `Code`/`Scope` directly for a stoplight-shaped enum
  (three states) and a `Transition` type with no `StateMachine`/
  `Exchange`-style trait bound at all — just `Scope`'s own required
  methods, confirming the core tier stands on its own. A further
  refinement surfaced here too: `Fragment` had to become a trait, not a
  concrete type (see "`Code`, `Scope`, `Locality`" above) — a properly
  abstract `Fragment` carries no composition capability, so `scope()` has
  no default body at all now. `Transition`'s `scope()` hand-sequences its
  boundary contributions into a `syn::Block`, specific to `Ir`, which is a
  real, if simple, composition (see Phase 5) — not the final answer to how
  contributions ought to combine, but honest and round-trip-checkable as
  far as it goes.
- [ ] Shave the stoplight down to a smaller valid result (e.g. two states
  and the one relation between them) and confirm the result is edge-closed,
  rooted, and compiles standalone. Not yet done — the current test exercises
  `Inline` universally; `Omit`-driven shaving of a multi-state example is
  still open.
- [ ] Model a calculator by hand and confirm that isolating one calculation
  is exactly extracting the subgraph along one walked path — no separate
  tracking mechanism required.
- [x] Define `Selection` and `scope_with(selection)`. `Selection<F:
  Fragment> { fn includes(&self, item: &F) -> bool }` is a filter layered
  on top of `Locality`, not a replacement for it, confirmed by
  `scope_with`'s implementation: an entry survives only if `selection`
  includes it *and* its assigned `Locality::contribute` still answers
  `Some`. Hand-implemented for both `Transition` (`scope_test.rs`,
  `OnlyGreen`) and `Calculator` (`calculator_test.rs`, `PlusMinusOnly`,
  replacing the earlier hardcoded `include_advanced` field — "a
  calculator with just `+` and `-`" is now an external, pluggable policy
  rather than a struct field). Implementing this surfaced and fixed a
  latent bug in `Transition::scope()`: it reused `code()`'s output as its
  tail expression, which always built the full `(from, to)` tuple
  regardless of `Locality`, the same shaving bug `Calculator::scope()`
  had before its own fix — never caught because no prior test exercised
  exclusion through `Transition::scope()` itself.
- [x] Decide how `scope()` and `scope_with()` relate: siblings, not a
  wrapper — `scope_with` is a second required `Scope` method with no
  default body, for the same reason `scope()` has none (`Fragment`
  carries no composition capability to build a generic default from).
  Each implementor hand-writes both for now, pending Phase 5.
- [x] Define `Source` and implement `Bound`/`Free` as `Binding`, and
  confirm the round-trip obligation holds for both —
  `bound_binding_round_trips_to_the_same_value` and
  `free_binding_round_trips_to_an_accepting_placeholder` in
  `calculator_test.rs`. The design changed shape twice on the way here,
  both times toward less machinery, not more:
  - No separate `Calculation` example type was needed. A single resolved
    calculation and one entry drained from a queue-shaped `Source` are
    the same data — inventing a bespoke type to hold it would have
    duplicated what the `Source` already represents.
  - `Source` ended up fully decoupled from `Fragment`/`Code`: it hands
    back an `Args` (`{ type Value; fn value(&self) -> Self::Value; }`),
    a raw value with no code representation at all. Turning that into a
    `Fragment` is left to whatever `Code` impl the raw value's own type
    already has (`i32: Code`, unchanged). `SlotId` is a plain `&str`,
    matching `Extent::anchor`'s own naming convention.
  - `Binding`'s stable/mutating split moved off the `Args` side (no
    `ArgsMut` trait) and onto `Source` itself: `value_for(&self, ..)` is
    a non-consuming peek, `value_mut_for(&mut self, ..)` is permitted to
    consume/mutate, defaulting to `value_for` so read-only `Source`s
    never need to override it. This is what makes real draining
    mechanically possible at all — the earlier sketch tried to put
    mutation on the *returned* value without `Source` itself ever taking
    `&mut self`, which had no way to reach back into a `Source`'s own
    state.
  - `Bound<F>`/`Free<F>` ended up self-contained
    (`fn contribute(&self) -> F`, no argument), not mirroring
    `Locality::contribute(&self, dependency: &F)`'s shape — `boundary()`
    always hands `Locality` a real dependency to decide about, but
    `Free` has no "value from `Source`" to be handed when `Source`
    answered `None`, so keeping the parameter would have meant passing a
    dummy.
  - `Add::code_with(&self, source: &mut dyn Source<Args = IntArg>) -> Ir`
    demonstrates all three renderings on one operation, not just a bound/
    free toggle: both slots bound is direct substitution (`3 + 4`), both
    free matches `code()` exactly (`|a: i32, b: i32| a + b`), and a mix
    partially applies (`|b: i32| 3 + b`) — a real illustration of
    "shifting scope" as a spectrum at the value level, something
    `Selection` alone (whole-entry, not per-value) can't produce.
  - `AddArgs` is the concrete queue-shaped `Source`
    (`add_code_with_drains_the_queue` confirms `value_mut_for` actually
    pops), doubling as "always free" when empty — no separate
    `AlwaysFree` unit struct was needed either.
- [x] Reproduce both motivating configurations on the calculator example —
  a frozen replay of one session (`Selection` = the walked path, `Source`
  = that session's values) and a general `+ - * /` calculator (`Selection`
  = an arithmetic filter, `Source` = always `None`) — from the same two
  composable pieces, with no mode-specific code written for either. The
  `Selection` half (general calculator) is `PlusMinusOnly`; the
  session-replay half is `AddArgs` populated with one session's actual
  values, drained via `Add::code_with`. Both are ordinary implementors of
  the same two traits, not two hand-written modes.
- [x] Define `Extent`, hand-implement it for the calculator example
  (`anchor()` matching on `"divide"`/`"multiply"`/`"add"`/`"subtract"`, by
  hand, no derive/`inventory` yet), and confirm each name resolves to the
  same closure `Scope::boundary()` would already compute for that
  operation — no separate traversal logic written for `Extent` itself.
  `calculator_anchor_resolves_named_operations_to_their_own_code` and
  `calculator_anchor_ignores_shave_configuration` confirm this, the latter
  also confirming naming and shaving stay independent questions: an
  anchor stays valid even when `include_advanced` shaves it out of a
  rendered `scope()`.
- [x] Confirm a name never declared in `anchor()` answers `None`
  (`calculator_anchor_returns_none_for_undeclared_names`) — the
  precondition for being invisible to `Selection`. The full claim (a
  never-declared name is invisible to `Selection` specifically, not just
  to `anchor()`) is still open until `Selection` itself exists to check
  against.
- [ ] Prototype the `inventory`-backed registry once the hand-written
  `anchor()` case above works: a static descriptor type, `inventory::submit!`
  calls standing in for what the derive will eventually emit, and a lookup
  function that resolves a name to the type/method it names.

### Phase 5: Composition

- [x] Implement `Code for Vec<T>`. This did *not* turn out to force the
  lateralizing-trait questions the way this checklist item originally
  expected — `impl<T: Code<Fragment = Ir>> Code for Vec<T>` is exactly as
  mechanical as `[T; N]`/tuples/`Option<T>` were: `Vec::from([elem0,
  elem1, ...])`, reusing `call_expr`/`array_expr`, no new builder, no
  `Scope`, no runtime choice of shape involved at all, since a `Vec<T>`
  value only ever has the one shape its own type dictates. The two
  questions this checklist item conflated turned out to be genuinely
  separate: "capture a `Vec<T>` *value's* code" (solved, mechanical) is
  not the same problem as "let an agent choose, at runtime, how to
  combine a pile of already-captured fragments not tied to any one Rust
  type" (still fully open — see "Lateralizing composition traits" above).
- [ ] Name and implement the lateralizing composition traits. Still needs
  its own forcing case — something that actually needs an agent to choose
  a composition shape at runtime over fragments with no single owning
  Rust type, which `Vec<T>` never was.

### Phase 6: `homecoming_amenable` — proof-preserving shaving

Not started until Phase 4 proves the core tier's mechanics actually work —
`homecoming_amenable` builds directly on `Scope`/`Locality`/`Extent`/
`Selection`/`Source`, it doesn't reinvent them.

- [ ] Scaffold `homecoming_amenable` as a new workspace member, depending
  on `homecoming_core` and `amenable_core`, and wire it into the
  top-level `homecoming` facade behind an `amenable` Cargo feature so the
  default dependency graph stays lean.
- [ ] Implement the blanket `Scope` impl gated on real
  `amenable_core::StateMachine` + `amenable_core::Exchange`.
- [ ] Implement `Extent` for `StateMachine`/`Exchange`-bound types using
  their real transition structure directly, rather than `Scope`'s generic
  edge-closed/rooted graph closure — the concrete payoff of taking a real
  `amenable` dependency instead of a homecoming-native lookalike.
- [ ] Design and implement the concrete mechanism that checks a shaved
  subset still upholds the proof obligations the original `Exchange`
  established — the open question from "`homecoming_amenable`'s success
  case" above.
- [ ] Re-run the stoplight/calculator demos from Phase 4 through a type
  that additionally implements real `amenable::StateMachine`/`Exchange`,
  and confirm `homecoming_amenable`'s stronger guarantee actually holds
  where the core tier's did not attempt to make that claim.

### Phase 7: relocate audit tooling into `homecoming_amenable`

`amenable` never takes `homecoming` as a dependency — the dependency arrow
runs one way. Any cooperation between `amenable`'s traits (`Witness` in
particular) and source capture is implemented in `homecoming_amenable`,
which already has both `homecoming_core` and `amenable_core` in scope,
rather than by `amenable_core` depending on `homecoming`.

- [ ] Identify which audit-tooling designs sketched in `AMENABLE_PLAN.md`
  actually need both `Code`/`Extent` and `StateMachine`/`Exchange` in
  scope at once, and relocate their design notes into this plan —
  a change of location, not a re-derivation.
- [ ] Implement adapter/glue code in `homecoming_amenable` that lets a
  `Witness`-bearing `amenable` type also produce `Code`/`Extent` output —
  composition in `homecoming_amenable`, not a `Code` bound added to
  `Witness` itself.
- [ ] Exercise the strong-form round-trip check against a real
  `Witness`-bearing example once `amenable`'s proof-emission machinery
  exists.
- [ ] Confirm `amenable_core`'s own `Cargo.toml` never gains a
  `homecoming`/`homecoming_core` dependency at any point in this process —
  the check that the one-way arrow actually held.

### Phase 8: The derive macro

Not started until Phases 3–6 have produced enough hand-written `impl Code`/
`impl Scope`/`impl Extent` examples, `homecoming_core` and
`homecoming_amenable` both, to extract a reliable pattern from —
designing the macro before the representation it targets is proven
repeats the mistake that motivated this crate's own existence.

- [ ] Extract the repeatable shape of the hand-written examples into macro
  logic.
- [ ] Decide the derive macro's final name.
- [ ] Decide the attribute users apply to individual methods/types to mark
  them as meaningful `Extent` units — this is expected to be the primary
  user-facing API surface, the same role `#[instrument]` plays for
  `tracing`, not an internal implementation detail.
- [ ] Confirm `#[derive(..)]`-generated impls satisfy the same round-trip
  obligation as hand-written ones, with no special exemption for
  macro-generated code.
- [ ] Confirm `homecoming`'s own hand-written examples can be re-expressed
  through the derive and `Extent` attributes, not left as a permanently
  separate hand-written path from what user code does.

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
- What concrete mechanism does `homecoming_amenable` use to check that a
  shaved subset upholds the proof obligations the original `Exchange`
  established — a method that compares `Precondition`/`Postcondition`
  `Evidence` directly, something that re-runs `Witness` against the
  shaved result, or something else not yet considered?
- Should the derive macro's name mirror `Code` (or `Scope`), or diverge
  entirely, the way `#[derive(Elicit)]` diverges from any single trait name
  in `elicitation`?
- What attribute grammar does the derive use to mark a method as an
  `Extent` unit — a bare `#[extent]`, something carrying a name/config
  like `#[instrument]` does, or something else?
- What does the `homecoming_amenable` glue between `Witness` and
  `Code`/`Extent` concretely look like — a wrapper type that holds a
  `Witness`-bearing value alongside its own `Code`/`Extent` impl, a
  blanket impl bound on `Witness` (which would still not require
  `amenable_core` to depend on anything, since the blanket impl itself
  lives in `homecoming_amenable`), or something else?
- How does a compile-time `inventory`-registered `(type, method, name)`
  descriptor resolve to an actual node in a specific *instance*'s
  `petgraph` graph — `petgraph::NodeIndex` only means something within one
  graph, not globally, so the registry can name a method but not a node
  directly. Does the derive-generated `anchor()` body just re-derive the
  node by re-running the same construction logic `code()`/`boundary()`
  already use, making the registry closer to "which method to call" than
  "which node to fetch"?
- Do nested `Extent` names (an extent-marked method calling another
  extent-marked method) resolve automatically, since the inner name's
  anchor is just another reachable node in the outer's dependency graph —
  or does something more need to be true for the outer's closure to
  include it?

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
ships. And when a user can declare which parts of their own program are
worth isolating in their own vocabulary — `Extent` attributes on their own
methods and types — without hand-writing `Code`/`Scope` impls, the same
way `#[instrument]` lets a `tracing` user opt into span-shaped observation
without hand-writing span management. And when the dependency arrow
between the two crate families stays a single line — `homecoming` depends
on `amenable`, never the reverse — so that generating modular MCP tool
code from `Extent`-compartmentalized operations, each traceable back to
the specific formal verification it came from, is `homecoming_amenable`
composing both crates' capabilities rather than either crate reaching back
into the other.
