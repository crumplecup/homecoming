# The Homecoming Trait Family

High-fidelity capture, isolation, and replay of Rust source code — two tiers, not one. A core tier (Code, Scope, Locality, Extent, Selection, Source, Binding, in `homecoming_core`) works for any type, has no dependency on `amenable`, and makes no claim beyond structural validity. `homecoming_amenable`, surfaced through the top-level `homecoming` facade behind an `amenable` Cargo feature, targets types bound to `amenable`'s real StateMachine/Exchange traits and makes the stronger claim that a shaved subset preserves the proof obligations the original upheld. Neither tier targets arbitrary Rust. The dependency arrow runs one way only: `homecoming` depends on `amenable`, never the reverse.

## Trait Fragment

Fragment is the interface Code, Scope, and Locality are actually written against, not a concrete type. `Ir`, this crate's own petgraph-backed representation, is one implementation of it — used for this crate's own std-lib Code impls — not the definition of it. A type earns the capability by explicitly declaring it (`ToTokens` + `Clone`), no blanket coverage. Getting this wrong once (making Fragment a concrete struct) was the same "structure instead of capability" mistake Provenance's redesign already corrected — Code, Locality, and Scope couldn't actually express their real relationship to a type that wasn't itself an interface.

```rust
pub trait Fragment: quote::ToTokens + Clone {}
```

## Trait Code

Code is a leaf capability: a type implements Code directly on the value it describes, and `code()` gives back the source that produced that value instead of the value itself. Everything in a Rust program starts out as code — Code is about high-fidelity capture and replay of that original source as a value flows through the program, not about generating a plausible-looking substitute after the fact.

```rust
pub trait Code {
    type Fragment: Fragment;
    fn code(&self) -> Self::Fragment;
}
```

## Trait Scope

Anyone can hand back the whole source of a program. Scope is for a narrower, harder job: given one specific point of interest, isolate the minimal slice of code that actually contributes to it, not everything transitively reachable from it — program slicing, applied to Rust source.

The guiding intuition is a tracing span. A span doesn't require a pre-declared taxonomy of "request scope" versus "database-call scope" — any code that wants to be traceable enters its own span, and isolating everything that happened during that span works identically no matter how deep it's nested. Scope works the same way: no fixed taxonomy of scope kinds, just a boundary each implementor reports for itself.

```rust
pub trait Scope: Code {
    fn boundary(&self) -> impl Iterator<Item = (Self::Fragment, Box<dyn Locality<Self::Fragment>>)>;
    fn scope(&self) -> Self::Fragment;
}
```

`scope()` has no default body — combining `boundary()`'s contributions into one fragment is a composition operation, and Fragment's minimal interface deliberately carries none. Each implementor writes its own for now, pending the lateralizing composition traits.

## Trait Locality

Locality decides how one boundary entry contributes to a scoped fragment — reproduced in full, referenced by name, omitted entirely, or something else. This is a trait rather than a closed enum on purpose: an enum would need to grow every time a new rendering strategy showed up. As a trait, new localities are just new implementors, and nothing about Scope has to change to add one.

```rust
pub trait Locality<F: Fragment> {
    fn contribute(&self, dependency: &F) -> Option<F>;
}
```

`Inline` (reproduce in full), `Reference` (a caller-supplied replacement fragment, since Fragment itself carries no construction capability to build a name from), and `Omit` (shave away entirely — hence `Option<F>`, not a bare `F`) are the implementors motivated so far. `Omit` wasn't anticipated when Locality was designed — it fell out of the first real domain the design was tried against, with zero changes needed to Scope or Locality's own definition.

## Trait Extent

Programming language theory already draws this distinction: scope is the static, lexical question (what does this structure look like), extent is the dynamic, temporal question (when is a recording live). Extent answers which — a grammar for naming which methods and types are meaningful, isolatable units at all. But "live" here is a compiler's sense of liveness, not a runtime-tracing one: a name marks an anchor already present in the dependency graph a Scope implementor builds, and the live code for that name is whatever Scope's existing edge-closed/rooted closure reaches from that anchor — the same graph `boundary()` already describes, not a new traversal to build. Code unreachable from any named anchor is simply dead, the same sense a compiler's liveness analysis gives the term.

```rust
pub trait Extent: Code {
    fn anchor(&self, name: &str) -> Option<Self::Fragment>;
}
```

Declaring an extent is a compile-time fact, not a runtime event, so there is no start/stop lifecycle and no guard to drop: the derive is expected to emit a static `(type, method, name)` descriptor per annotated method via `inventory::submit!`, collected into one process-wide, read-only registry — closer to how `#[instrument]` doesn't need a central span registry either, just a marker at the definition site. A query by name reads the registry to find the anchor, then asks that instance's `anchor()` for the closure computed fresh from its current graph, not a value tracked as something happened. Recording user-submitted runtime values is a different, already-solved problem — a queue-shaped `Source` draining into `Binding`, not Extent's job at all.

The buffet framing still holds: Extent decides which dishes exist on the table at all — a method never annotated is never its own isolatable unit, just plumbing absorbed into whatever encloses it. Selection decides what ends up on a given plate, operating only over what Extent already made a candidate.

## The two tiers

Any type implementing Scope gets shaving for free at the core tier, via a blanket-derivable `scope()` built on `boundary()` and `Locality` — no hand-written isolation logic required, the same way implementing Iterator's required methods unlocks a toolkit of default ones. What makes a shaved result valid rather than a broken fragment: no Inlined entry may reference an Omitted one (edge-closed), and at least one entry must be present that doesn't depend on an Inlined one to justify its presence (rooted — the same "states are roots" principle `amenable` already names). Both conditions are graph-native, computed by petgraph, not bespoke logic. This is the whole claim the core tier makes: structurally sound, nothing more.

The stronger claim — that a shaved subset preserves the proof obligations the *original* upheld, not just its graph structure — needs real proof machinery to check against, so it lives in `homecoming_amenable`, a workspace member bound directly to `amenable`'s actual `StateMachine`/`Exchange` traits, proof tokens included, and feature-gated out of `homecoming`'s default dependency graph. A homecoming-native lookalike trait was tried first and rejected: it could copy `StateMachine`/`Exchange`'s shape, but not the `Evidence`/`ProofToken` machinery that makes the stronger claim checkable rather than aspirational — and that machinery is the entire reason `homecoming_amenable` is worth having. `amenable_core::StateMachine` itself is minimal enough (`{ type State; type Invariant; }`) that reusing it directly costs nothing; only `Exchange` needed the real thing, not a mirror. The same real bound is what lets `Extent` reason about liveness from proven transition structure directly, rather than only from `Scope`'s generic edge-closed/rooted graph closure.

## Trait Selection, Source, and Binding

Shaving turned out to be two independent decisions, not one — three, counting Extent above, which decides what's even a candidate before Selection chooses among candidates. Think of a program like a buffet: an agent pulls components off the table, in any order, to build a platter. Selection decides which of the available components land on the platter — Locality's Inline/Omit choice, but driven by an external, pluggable policy rather than fixed per item, since a platter isn't necessarily one walked session's path, it can be any deliberately curated selection ("just the `+ - * /` keys," which no single session ever walks but which is still a valid, edge-closed, rooted subset).

```rust
pub trait Selection<F: Fragment> {
    fn includes(&self, item: &F) -> bool;
}
```

Binding decides, separately, whether an included item's values are frozen or still open — the same question Rust's own block scoping answers for a variable's lifetime. Emission time is one scope; the emitted program's later runtime is a different one. A value crosses that boundary as a frozen literal (Bound) or doesn't cross at all, with the emitted program declaring its own fresh, open slot instead (Free).

```rust
pub trait Binding<F: Fragment> {
    fn contribute(&self) -> F;
}
```

`Bound<F>`/`Free<F>` are self-contained, constructed with whatever they need up front, rather than mirroring `Locality::contribute(&self, dependency: &F)`'s shape. `boundary()` always hands `Locality` a real dependency to decide about; `Binding` doesn't have that guarantee — `Free` has nothing to be handed when `Source` (below) answered `None`, so passing a parameter would mean passing a dummy.

Source answers whether a value is available for a named slot, decoupled entirely from `Fragment`/`Code` — it hands back an `Args`, a raw value with no code representation, and whoever wants a `Fragment` reaches for whatever `Code` impl that raw value's own type already has (`i32: Code`, unchanged).

```rust
pub trait Args {
    type Value;
    fn value(&self) -> Self::Value;
}

pub trait Source {
    type Args: Args;

    fn value_for(&self, slot: &str) -> Option<Self::Args>;

    fn value_mut_for(&mut self, slot: &str) -> Option<Self::Args> {
        self.value_for(slot)
    }
}
```

`value_for` is a non-consuming peek; `value_mut_for` is permitted to consume or mutate whatever backs it, and defaults to `value_for` — most `Source`s (a fixed, already-captured record) have nothing to drain and never override it. This is what makes real draining possible at all: an earlier pass tried to put the stable/mutating distinction on the *value* `Source` hands back instead of on `Source` itself, but a value with no way back to its `Source` has nothing to mutate. Putting `value_mut_for` on `Source`, taking `&mut self`, gives a queue-shaped `Source` (holding user-submitted values, drained one at a time into a real call — "just the `+ - * /` keys" pressed this session, replayed) a real place to pop from.

Selection and Source are independent, composable pieces, a Lego kit rather than a fixed set of modes. "The keys pressed this session, hardcoded forever" is a session-walked Selection paired with a Source built from that session's actual values. "A calculator with just `+ - * /`" is a capability-filter Selection paired with a Source that answers nothing, leaving every value free — an empty queue-shaped Source already behaves this way, with no separate always-free type needed. Neither needed its own bespoke mode — they're two different pairs of the same two swappable parts, and any other pair is a new, valid configuration nobody had to design for in advance.

## Narrative

Code's fidelity is not just a convenience for agents assembling programs from tool calls. It is part of the formal verification process itself: when a solver finishes checking a proof, the code that proof ran over is only meaningful if it can be tied, exactly, to the code that actually ships. Code is the receipt establishing that connection, and an approximate receipt is worse than no receipt — it looks like assurance while providing none.

Isolation is the harder half of that promise. A receipt that reproduces the entire program is technically accurate but not actually useful — nobody reviews a proof by reading everything. A calculator makes the case for why this can't be solved by capture alone: it takes an unbounded number of runtime inputs, so there is no enumerable set of "all the code for all possible sessions" to capture. But modeled as a state machine, "did you subtract or divide first" stops needing a separate answer — it is simply which path was walked through the state graph, and isolating one calculation is extracting the subgraph along that one walk. Two instincts were tried and rejected on the way here: a fixed taxonomy of scope kinds (Program, Operation, Case, Value — structure again, the exact mistake `amenable`'s Provenance redesign already corrected once), and targeting arbitrary Rust programs at all, which is a much harder problem than this crate needs to solve.

Code's own soundness is not trusted by construction. It is a checkable claim, held to the same discipline this whole family answers to: emit a value's code, reconstruct the value from it, and confirm it matches the original — the baseline obligation for Code. Scope carries the same obligation one level up: the isolated slice, compiled standalone, must behave identically to the equivalent code extracted by hand. And once `homecoming_amenable` cooperates with `amenable`'s real `Witness`, Witness and Code must never be allowed to independently drift apart on the same claim: reconstruct a whole program from its emitted fragments, regenerate its proofs, and compare them to the original program's proofs — the receipt made concrete, proving the two agree without requiring them to be the same mechanism. That cooperation lives in `homecoming_amenable` as glue code holding both capabilities, not as a dependency `amenable` takes on `homecoming` — the dependency arrow points one way, and that's `homecoming_amenable`'s job specifically, since it's the crate with real proofs to compare against.

A shaved result, at either tier, is a reassembly of real, previously captured pieces — every part it contains really executed, really compiled, really was part of the original. What would cross into generation is fabricating a smaller program that merely looks like what a subset might have done. That line matters more here than anywhere else in the design, because a subset program is the one thing in this crate that's actively being asked to be smaller than the truth — it has to stay honest about which smaller truth it's telling.

The name is deliberate. Given a value, however far it has traveled from its origin — through function calls, storage, transformation — this crate is the way back to the source it came from. Not a reconstruction of what the source probably looked like, but the actual homecoming: the same code, returned, and no more of it than was asked for.
