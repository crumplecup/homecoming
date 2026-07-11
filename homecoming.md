# The Homecoming Trait

High-fidelity capture and replay of Rust source code.

## Trait Homecoming

Homecoming is a leaf capability: a type implements Homecoming directly on the value it describes, and `code()` gives back the source that produced that value instead of the value itself. Everything in a Rust program starts out as code — Homecoming is about high-fidelity capture and replay of that original source as a value flows through the program, not about generating a plausible-looking substitute after the fact.

```rust
pub trait Homecoming {
    fn code(&self) -> Fragment;
}
```

Homecoming fragments compose like Lego bricks, but the composition itself has to be an explicit, invokable operation, rather than a blanket impl — an agent choosing how to assemble a pile of fragments at runtime (sequence them into a block, thread them as call arguments, assemble them as struct fields) is a decision that needs to stay visible and auditable, not buried in generic code. See `HOMECOMING_PLAN.md` for the lateralizing composition traits that carry out that assembly.

## Narrative

Homecoming's fidelity is not just a convenience for agents assembling programs from tool calls. It is part of the formal verification process itself: when a solver finishes checking a proof, the code that proof ran over is only meaningful if it can be tied, exactly, to the code that actually ships. Homecoming is the receipt establishing that connection, and an approximate receipt is worse than no receipt — it looks like assurance while providing none.

The name is deliberate. Given a value, however far it has traveled from its origin — through function calls, storage, transformation — Homecoming is the way back to the source it came from. Not a reconstruction of what the source probably looked like, but the actual homecoming: the same code, returned.

Homecoming's own soundness is not trusted by construction. It is a checkable claim, held to the same discipline as everything else in the `amenable` family that will eventually depend on it: emit a value's code, reconstruct the value from it, and confirm it matches the original — the baseline obligation for any Homecoming implementor. Reconstruct a whole program from its emitted fragments, regenerate its proofs, and compare them to the original program's proofs — the stronger form, and the one that makes the receipt concrete: not "the code looks the same" but "a verifier run over the reconstructed code reaches the same conclusions a verifier run over the original code did."
