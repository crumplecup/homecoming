# The Homecoming Trait Family

High-fidelity capture, isolation, and replay of Rust source code — two tiers, not one. A core tier (Code, Scope, Locality, Selection, Source, Binding) works for any type and makes no claim beyond structural validity. A bridge tier, in its own crate, targets types bound to `amenable`'s real StateMachine/Exchange traits and makes the stronger claim that a shaved subset preserves the proof obligations the original upheld. Neither tier targets arbitrary Rust.

## Trait Code

Code is a leaf capability: a type implements Code directly on the value it describes, and `code()` gives back the source that produced that value instead of the value itself. Everything in a Rust program starts out as code — Code is about high-fidelity capture and replay of that original source as a value flows through the program, not about generating a plausible-looking substitute after the fact.

```rust
pub trait Code {
    fn code(&self) -> Fragment;
}
```

## Trait Scope

Anyone can hand back the whole source of a program. Scope is for a narrower, harder job: given one specific point of interest, isolate the minimal slice of code that actually contributes to it, not everything transitively reachable from it — program slicing, applied to Rust source.

The guiding intuition is a tracing span. A span doesn't require a pre-declared taxonomy of "request scope" versus "database-call scope" — any code that wants to be traceable enters its own span, and isolating everything that happened during that span works identically no matter how deep it's nested. Scope works the same way: no fixed taxonomy of scope kinds, just a boundary each implementor reports for itself, and a default derivation that walks it.

```rust
pub trait Scope: Code {
    fn boundary(&self) -> impl Iterator<Item = (Fragment, Box<dyn Locality>)>;

    fn scope(&self) -> Fragment {
        // default, built from code() + boundary()
    }
}
```

## Trait Locality

Locality decides how one boundary entry contributes to a scoped fragment — reproduced in full, referenced by name, omitted entirely, or something else. This is a trait rather than a closed enum on purpose: an enum would need to grow every time a new rendering strategy showed up. As a trait, new localities are just new implementors, and nothing about Scope has to change to add one.

```rust
pub trait Locality {
    fn contribute(&self, dependency: &dyn Code) -> Fragment;
}
```

`Inline` (reproduce in full), `Reference` (name without reproducing), and `Omit` (shave away entirely) are the implementors motivated so far. `Omit` wasn't anticipated when Locality was designed — it fell out of the first real domain the design was tried against, with zero changes needed to Scope or Locality's own definition.

## The two tiers

Any type implementing Scope gets shaving for free at the core tier, via a blanket-derivable `scope()` built on `boundary()` and `Locality` — no hand-written isolation logic required, the same way implementing Iterator's required methods unlocks a toolkit of default ones. What makes a shaved result valid rather than a broken fragment: no Inlined entry may reference an Omitted one (edge-closed), and at least one entry must be present that doesn't depend on an Inlined one to justify its presence (rooted — the same "states are roots" principle `amenable` already names). Both conditions are graph-native, computed by petgraph, not bespoke logic. This is the whole claim the core tier makes: structurally sound, nothing more.

The stronger claim — that a shaved subset preserves the proof obligations the *original* upheld, not just its graph structure — needs real proof machinery to check against, so it lives in a separate bridge crate bound directly to `amenable`'s actual `StateMachine`/`Exchange` traits, proof tokens included. A homecoming-native lookalike trait was tried first and rejected: it could copy `StateMachine`/`Exchange`'s shape, but not the `Evidence`/`ProofToken` machinery that makes the stronger claim checkable rather than aspirational — and that machinery is the entire reason the bridge tier is worth having. `amenable_core::StateMachine` itself is minimal enough (`{ type State; type Invariant; }`) that reusing it directly costs nothing; only `Exchange` needed the real thing, not a mirror.

## Trait Selection, Source, and Binding

Shaving turned out to be two independent decisions, not one. Think of a program like a buffet: an agent pulls components off the table, in any order, to build a platter. Selection decides which components land on the platter at all — Locality's Inline/Omit choice, but driven by an external, pluggable policy rather than fixed per item, since a platter isn't necessarily one walked session's path, it can be any deliberately curated selection ("just the `+ - * /` keys," which no single session ever walks but which is still a valid, edge-closed, rooted subset).

```rust
pub trait Selection {
    fn includes(&self, item: &dyn Code) -> bool;
}
```

Binding decides, separately, whether an included item's values are frozen or still open — the same question Rust's own block scoping answers for a variable's lifetime. Emission time is one scope; the emitted program's later runtime is a different one. A value crosses that boundary as a frozen literal (Bound) or doesn't cross at all, with the emitted program declaring its own fresh, open slot instead (Free). Whether a bound value came from a replayed file or the last live session doesn't matter to this question — both are just data sitting in the emission-time scope; a Source answers whether it has a value for a given slot, not where that value came from.

```rust
pub trait Binding {
    fn contribute(&self, value: &dyn Code) -> Fragment;
}

pub trait Source {
    fn value_for(&self, slot: SlotId) -> Option<Fragment>;
}
```

Selection and Source are independent, composable pieces, a Lego kit rather than a fixed set of modes. "The keys pressed this session, hardcoded forever" is a session-walked Selection paired with a Source that answers every value. "A calculator with just `+ - * /`" is a capability-filter Selection paired with a Source that answers nothing, leaving every value free. Neither needed its own bespoke mode — they're two different pairs of the same two swappable parts, and any other pair is a new, valid configuration nobody had to design for in advance.

## Narrative

Code's fidelity is not just a convenience for agents assembling programs from tool calls. It is part of the formal verification process itself: when a solver finishes checking a proof, the code that proof ran over is only meaningful if it can be tied, exactly, to the code that actually ships. Code is the receipt establishing that connection, and an approximate receipt is worse than no receipt — it looks like assurance while providing none.

Isolation is the harder half of that promise. A receipt that reproduces the entire program is technically accurate but not actually useful — nobody reviews a proof by reading everything. A calculator makes the case for why this can't be solved by capture alone: it takes an unbounded number of runtime inputs, so there is no enumerable set of "all the code for all possible sessions" to capture. But modeled as a state machine, "did you subtract or divide first" stops needing a separate answer — it is simply which path was walked through the state graph, and isolating one calculation is extracting the subgraph along that one walk. Two instincts were tried and rejected on the way here: a fixed taxonomy of scope kinds (Program, Operation, Case, Value — structure again, the exact mistake `amenable`'s Provenance redesign already corrected once), and targeting arbitrary Rust programs at all, which is a much harder problem than this crate needs to solve.

Code's own soundness is not trusted by construction. It is a checkable claim, held to the same discipline as everything else the `amenable` family will eventually depend on: emit a value's code, reconstruct the value from it, and confirm it matches the original — the baseline obligation for Code. Scope carries the same obligation one level up: the isolated slice, compiled standalone, must behave identically to the equivalent code extracted by hand. And once `amenable` depends on this crate, Witness and Code must never be allowed to independently drift apart on the same claim: reconstruct a whole program from its emitted fragments, regenerate its proofs, and compare them to the original program's proofs — the receipt made concrete, proving the two agree without requiring them to be the same mechanism. That's the bridge tier's job specifically, since it's the tier with real proofs to compare against.

A shaved result, at either tier, is a reassembly of real, previously captured pieces — every part it contains really executed, really compiled, really was part of the original. What would cross into generation is fabricating a smaller program that merely looks like what a subset might have done. That line matters more here than anywhere else in the design, because a subset program is the one thing in this crate that's actively being asked to be smaller than the truth — it has to stay honest about which smaller truth it's telling.

The name is deliberate. Given a value, however far it has traveled from its origin — through function calls, storage, transformation — this crate is the way back to the source it came from. Not a reconstruction of what the source probably looked like, but the actual homecoming: the same code, returned, and no more of it than was asked for.
