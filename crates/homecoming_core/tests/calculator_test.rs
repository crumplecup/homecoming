//! Calculator target: validates the design against a concrete example with
//! real cut-point pressure, per HOMECOMING_PLAN.md's stated proving ground.
//!
//! Each operation is both a real, callable Rust function *and* a `Code`
//! implementor emitting its own source — the quote/eval duality that
//! motivated this crate in the first place. The calculator demonstrates
//! subsetting: the same four operations, shaved down to just `+`/`-` via
//! an external `Selection` policy, produce code that structurally
//! excludes `*`/`/` entirely — "a calculator with just `+` and `-`" as a
//! pluggable filter, not a field baked into the type.

use homecoming_core::{
    Args, Binding, Bound, Code, Extent, Free, Inline, Ir, Locality, Scope, Selection, Source,
    binary_expr, closure_expr, path_expr, tuple_expr,
};
use quote::ToTokens;
use std::collections::VecDeque;

// --- calculator operations: real function + Code, on the same operation ---

struct Add;
impl Add {
    fn apply(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Emits this calculation's code with each argument independently
    /// frozen or left open, depending on what `source` has for it — a
    /// finer-grained shifting-scope question than `Selection` answers:
    /// not "is this operation included," but "is this one value, within
    /// an included operation, frozen or open." Both slots bound produces
    /// a direct substitution (`3 + 4`); both free matches `code()`
    /// exactly (`|a: i32, b: i32| a + b`); a mix partially applies
    /// (`|b: i32| 3 + b`).
    fn code_with(&self, source: &mut dyn Source<Args = IntArg>) -> Ir {
        let mut params: Vec<(&str, &str)> = Vec::new();

        let a = match source.value_mut_for("a") {
            Some(arg) => Bound::new(arg.value().code()).contribute(),
            None => {
                params.push(("a", "i32"));
                Free::new(Ir::leaf(path_expr(&["a"]))).contribute()
            }
        };
        let b = match source.value_mut_for("b") {
            Some(arg) => Bound::new(arg.value().code()).contribute(),
            None => {
                params.push(("b", "i32"));
                Free::new(Ir::leaf(path_expr(&["b"]))).contribute()
            }
        };

        let body = binary_expr(
            a.expr().clone(),
            syn::BinOp::Add(Default::default()),
            b.expr().clone(),
        );

        if params.is_empty() {
            Ir::leaf(body)
        } else {
            Ir::leaf(closure_expr(&params, body))
        }
    }
}
impl Code for Add {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let body = binary_expr(
            path_expr(&["a"]),
            syn::BinOp::Add(Default::default()),
            path_expr(&["b"]),
        );
        Ir::leaf(closure_expr(&[("a", "i32"), ("b", "i32")], body))
    }
}

/// A raw `i32` handed back by [`AddArgs`] — the `Args` half of the
/// `Source`/`Args` split: `Source` deals in this, not in `Fragment`
/// directly, so `Code`'s existing `i32` impl is what turns it into one.
struct IntArg(i32);

impl Args for IntArg {
    type Value = i32;
    fn value(&self) -> i32 {
        self.0
    }
}

/// A queue-shaped `Source`: user-submitted values held for later draining
/// into a real `add(x, y)` call, one slot's queue at a time. An empty
/// queue behaves as "always free" without needing a separate unit-struct
/// `Source` for that case.
struct AddArgs {
    a: VecDeque<i32>,
    b: VecDeque<i32>,
}

impl AddArgs {
    fn new() -> Self {
        Self {
            a: VecDeque::new(),
            b: VecDeque::new(),
        }
    }
}

impl Source for AddArgs {
    type Args = IntArg;

    fn value_for(&self, slot: &str) -> Option<IntArg> {
        match slot {
            "a" => self.a.front().copied().map(IntArg),
            "b" => self.b.front().copied().map(IntArg),
            _ => None,
        }
    }

    fn value_mut_for(&mut self, slot: &str) -> Option<IntArg> {
        match slot {
            "a" => self.a.pop_front().map(IntArg),
            "b" => self.b.pop_front().map(IntArg),
            _ => None,
        }
    }
}

struct Subtract;
impl Subtract {
    fn apply(&self, a: i32, b: i32) -> i32 {
        a - b
    }
}
impl Code for Subtract {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let body = binary_expr(
            path_expr(&["a"]),
            syn::BinOp::Sub(Default::default()),
            path_expr(&["b"]),
        );
        Ir::leaf(closure_expr(&[("a", "i32"), ("b", "i32")], body))
    }
}

struct Multiply;
impl Multiply {
    fn apply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}
impl Code for Multiply {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let body = binary_expr(
            path_expr(&["a"]),
            syn::BinOp::Mul(Default::default()),
            path_expr(&["b"]),
        );
        Ir::leaf(closure_expr(&[("a", "i32"), ("b", "i32")], body))
    }
}

struct Divide;
impl Divide {
    fn apply(&self, a: i32, b: i32) -> i32 {
        a / b
    }
}
impl Code for Divide {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let body = binary_expr(
            path_expr(&["a"]),
            syn::BinOp::Div(Default::default()),
            path_expr(&["b"]),
        );
        Ir::leaf(closure_expr(&[("a", "i32"), ("b", "i32")], body))
    }
}

/// "Multiply by two" — the minimal quote/eval duality example: a real
/// callable operation that also emits the exact code it performs.
struct Double;
impl Double {
    fn apply(&self, x: i32) -> i32 {
        x * 2
    }
}
impl Code for Double {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        // Reuse the existing Code impl for i32 for the literal `2`, rather
        // than hand-building a second literal-construction path.
        let two = 2i32.code().expr().clone();
        let body = binary_expr(path_expr(&["x"]), syn::BinOp::Mul(Default::default()), two);
        Ir::leaf(closure_expr(&[("x", "i32")], body))
    }
}

// --- the calculator: bundles all four, subsettable via Selection ---

struct Calculator {
    add: Add,
    subtract: Subtract,
    multiply: Multiply,
    divide: Divide,
}

impl Code for Calculator {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let elems = vec![
            self.add.code().expr().clone(),
            self.subtract.code().expr().clone(),
            self.multiply.code().expr().clone(),
            self.divide.code().expr().clone(),
        ];
        Ir::leaf(tuple_expr(elems))
    }
}

impl Scope for Calculator {
    fn boundary(&self) -> impl Iterator<Item = (Ir, Box<dyn Locality<Ir>>)> {
        // Every entry is unconditionally Inline here — which operations
        // actually end up in a given result is Selection's job
        // (scope_with), not a choice hardcoded per instance the way an
        // earlier version's include_advanced field made it.
        vec![
            (self.add.code(), Box::new(Inline) as Box<dyn Locality<Ir>>),
            (
                self.subtract.code(),
                Box::new(Inline) as Box<dyn Locality<Ir>>,
            ),
            (
                self.multiply.code(),
                Box::new(Inline) as Box<dyn Locality<Ir>>,
            ),
            (
                self.divide.code(),
                Box::new(Inline) as Box<dyn Locality<Ir>>,
            ),
        ]
        .into_iter()
    }

    fn scope(&self) -> Ir {
        // Deliberately does NOT reuse code()'s output as a tail expression
        // the way Transition's scope() (scope_test.rs) once did — code()
        // always builds the full, unshaved tuple of all four operations,
        // so appending it here would silently reintroduce whatever
        // boundary() omitted. Calculator's boundary() and code() describe
        // the same underlying data, so the tail has to be built from what
        // actually survived Locality filtering, not from the
        // always-unshaved code().
        let elems: Vec<syn::Expr> = self
            .boundary()
            .filter_map(|(dependency, locality)| locality.contribute(&dependency))
            .map(|fragment| fragment.expr().clone())
            .collect();
        Ir::leaf(tuple_expr(elems))
    }

    fn scope_with(&self, selection: &dyn Selection<Ir>) -> Ir {
        let elems: Vec<syn::Expr> = self
            .boundary()
            .filter(|(dependency, _)| selection.includes(dependency))
            .filter_map(|(dependency, locality)| locality.contribute(&dependency))
            .map(|fragment| fragment.expr().clone())
            .collect();
        Ir::leaf(tuple_expr(elems))
    }
}

impl Extent for Calculator {
    // Reuses each operation's own code() directly — no new traversal
    // logic, the same closure Scope::boundary() already builds for that
    // name. Every name here is declared regardless of any Selection
    // policy: naming and shaving are different questions (see
    // HOMECOMING_PLAN.md's Extent/Selection split), so a name stays a
    // valid anchor even when a given scope_with() call would omit it.
    fn anchor(&self, name: &str) -> Option<Ir> {
        match name {
            "add" => Some(self.add.code()),
            "subtract" => Some(self.subtract.code()),
            "multiply" => Some(self.multiply.code()),
            "divide" => Some(self.divide.code()),
            _ => None,
        }
    }
}

/// A capability-filter policy: only the additive operations are on offer,
/// regardless of what Calculator's boundary() otherwise includes — "a
/// calculator with just `+` and `-`" (HOMECOMING_PLAN.md), expressed as an
/// external, pluggable policy rather than a struct field.
struct PlusMinusOnly {
    allowed: Vec<syn::Expr>,
}

impl PlusMinusOnly {
    fn new() -> Self {
        Self {
            allowed: vec![Add.code().expr().clone(), Subtract.code().expr().clone()],
        }
    }
}

impl Selection<Ir> for PlusMinusOnly {
    fn includes(&self, item: &Ir) -> bool {
        self.allowed.contains(item.expr())
    }
}

// --- tests ---

#[test]
fn add_executes_and_emits_matching_code() -> Result<(), syn::Error> {
    let add = Add;
    assert_eq!(add.apply(3, 4), 7);

    let fragment = add.code();
    let tokens = fragment.to_token_stream();
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    let rendered = tokens.to_string();
    assert!(rendered.contains('+'), "rendered: {rendered}");
    Ok(())
}

#[test]
fn double_executes_and_also_emits_the_code_that_performs_it() -> Result<(), syn::Error> {
    let double = Double;

    // "produce a binary that does this operation" — real execution.
    assert_eq!(double.apply(21), 42);

    // "also emits the code we are using ... for the user" — capture, not
    // a description of the operation, the actual expression.
    let fragment = double.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains('*'), "rendered: {rendered}");
    assert!(rendered.contains('2'), "rendered: {rendered}");
    Ok(())
}

#[test]
fn full_calculator_scope_includes_all_four_operations() -> Result<(), syn::Error> {
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
    };

    let scoped = calculator.scope();
    let tokens = scoped.to_token_stream();
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;

    let rendered = tokens.to_string();
    assert!(rendered.contains('+'), "rendered: {rendered}");
    assert!(rendered.contains('-'), "rendered: {rendered}");
    assert!(rendered.contains('*'), "rendered: {rendered}");
    assert!(rendered.contains('/'), "rendered: {rendered}");
    Ok(())
}

#[test]
fn shaved_calculator_scope_excludes_multiply_and_divide() -> Result<(), syn::Error> {
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
    };

    let scoped = calculator.scope_with(&PlusMinusOnly::new());
    let tokens = scoped.to_token_stream();
    // The shaved result must still compile standalone — the whole point of
    // Fragment being syn-typed rather than raw tokens or a string cut.
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;

    let rendered = tokens.to_string();
    assert!(rendered.contains('+'), "rendered: {rendered}");
    assert!(rendered.contains('-'), "rendered: {rendered}");
    assert!(!rendered.contains('*'), "rendered: {rendered}");
    assert!(!rendered.contains('/'), "rendered: {rendered}");
    Ok(())
}

#[test]
fn shaved_calculator_operations_still_execute_correctly() {
    // Shaving the emitted code doesn't touch the real operations at all —
    // Multiply/Divide can be omitted from the *code* via Selection,
    // without ever being deleted from the program. This is the
    // distinction between "isolated from the program" and "removed from
    // the program."
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
    };

    assert_eq!(calculator.add.apply(2, 3), 5);
    assert_eq!(calculator.subtract.apply(5, 3), 2);
    assert_eq!(calculator.multiply.apply(3, 4), 12);
    assert_eq!(calculator.divide.apply(10, 2), 5);
}

#[test]
fn calculator_anchor_resolves_named_operations_to_their_own_code()
-> Result<(), Box<dyn std::error::Error>> {
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
    };

    let anchored = calculator
        .anchor("add")
        .ok_or("add must be a declared anchor")?;
    assert_eq!(anchored.expr(), calculator.add.code().expr());

    let anchored = calculator
        .anchor("subtract")
        .ok_or("subtract must be a declared anchor")?;
    assert_eq!(anchored.expr(), calculator.subtract.code().expr());

    let anchored = calculator
        .anchor("multiply")
        .ok_or("multiply must be a declared anchor")?;
    assert_eq!(anchored.expr(), calculator.multiply.code().expr());

    let anchored = calculator
        .anchor("divide")
        .ok_or("divide must be a declared anchor")?;
    assert_eq!(anchored.expr(), calculator.divide.code().expr());

    Ok(())
}

#[test]
fn calculator_anchor_ignores_shave_configuration() -> Result<(), Box<dyn std::error::Error>> {
    // A name stays a valid anchor even when a Selection policy would omit
    // it from a rendered scope_with() result — Extent only answers "is
    // this a nameable unit," never "is this included in a given shaved
    // result." Compare to
    // shaved_calculator_scope_excludes_multiply_and_divide, where the
    // same PlusMinusOnly selection does exclude multiply/divide from
    // scope_with()'s *rendered* output; anchor() is a different question
    // entirely.
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
    };

    let shaved = calculator.scope_with(&PlusMinusOnly::new());
    let rendered = shaved.to_token_stream().to_string();
    assert!(!rendered.contains('*'), "rendered: {rendered}");

    let anchored = calculator
        .anchor("multiply")
        .ok_or("multiply stays a declared anchor even when shaved out of scope_with()")?;
    assert_eq!(anchored.expr(), calculator.multiply.code().expr());

    Ok(())
}

#[test]
fn calculator_anchor_returns_none_for_undeclared_names() {
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
    };

    // "clear" was never annotated as an isolatable unit — never a
    // candidate at all, not merely excluded by a Selection policy.
    assert!(calculator.anchor("clear").is_none());
}

#[test]
fn bound_binding_round_trips_to_the_same_value() -> Result<(), syn::Error> {
    let value = 3i32.code();
    let bound = Bound::new(value.clone());
    let contribution = bound.contribute();

    let tokens = contribution.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    assert_eq!(contribution.expr(), &reparsed);
    assert_eq!(contribution.expr(), value.expr());
    Ok(())
}

#[test]
fn free_binding_round_trips_to_an_accepting_placeholder() -> Result<(), syn::Error> {
    let placeholder = Ir::leaf(path_expr(&["a"]));
    let free = Free::new(placeholder.clone());
    let contribution = free.contribute();

    let tokens = contribution.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    assert_eq!(contribution.expr(), &reparsed);
    // A Free slot is a bare identifier, not tied to any particular frozen
    // value — ready for the emitted program to supply its own.
    assert_eq!(contribution.expr(), placeholder.expr());
    Ok(())
}

#[test]
fn add_code_with_fully_bound_produces_direct_substitution() -> Result<(), syn::Error> {
    let add = Add;
    let mut args = AddArgs::new();
    args.a.push_back(3);
    args.b.push_back(4);

    let fragment = add.code_with(&mut args);
    let tokens = fragment.to_token_stream();
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;

    let rendered = tokens.to_string();
    assert!(rendered.contains('3'), "rendered: {rendered}");
    assert!(rendered.contains('4'), "rendered: {rendered}");
    assert!(!rendered.contains('|'), "rendered: {rendered}");
    Ok(())
}

#[test]
fn add_code_with_fully_free_matches_plain_code() -> Result<(), syn::Error> {
    let add = Add;
    let mut args = AddArgs::new();

    let fragment = add.code_with(&mut args);
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    // Fully free must match code()'s own unparameterized rendering
    // exactly -- an empty Source and no Source at all are the same thing.
    assert_eq!(fragment.expr(), add.code().expr());
    Ok(())
}

#[test]
fn add_code_with_mixed_binding_partially_applies() -> Result<(), syn::Error> {
    let add = Add;
    let mut args = AddArgs::new();
    args.a.push_back(3);
    // b left empty -- stays free.

    let fragment = add.code_with(&mut args);
    let tokens = fragment.to_token_stream();
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;

    let rendered = tokens.to_string();
    assert!(rendered.contains('3'), "rendered: {rendered}");
    assert!(rendered.contains('b'), "rendered: {rendered}");
    // `a` is no longer a parameter -- it was frozen, not just referenced.
    assert!(!rendered.contains("a :"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn add_code_with_drains_the_queue() {
    let add = Add;
    let mut args = AddArgs::new();
    args.a.push_back(3);
    args.a.push_back(30);
    args.b.push_back(4);

    let _ = add.code_with(&mut args);

    // The first call must have drained the front of each queue -- the
    // whole point of value_mut_for over value_for.
    assert_eq!(args.a.front().copied(), Some(30));
    assert_eq!(args.b.front().copied(), None);
}
