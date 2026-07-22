//! Calculator target: validates the design against a concrete example with
//! real cut-point pressure, per HOMECOMING_PLAN.md's stated proving ground.
//!
//! Each operation is both a real, callable Rust function *and* a `Code`
//! implementor emitting its own source — the quote/eval duality that
//! motivated this crate in the first place. The calculator demonstrates
//! subsetting: the same four operations, shaved down to just `+`/`-` via
//! `Locality`, produce code that structurally excludes `*`/`/` entirely.

use homecoming_core::{Code, Extent, Inline, Ir, Locality, Omit, Scope};
use quote::ToTokens;

// --- syn AST construction helpers, direct construction, no parsing ---

fn ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

fn single_segment_path(name: &str) -> syn::Path {
    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident: ident(name),
        arguments: syn::PathArguments::None,
    });
    syn::Path {
        leading_colon: None,
        segments,
    }
}

fn path_expr(name: &str) -> syn::Expr {
    syn::Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: single_segment_path(name),
    })
}

fn type_path(name: &str) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: single_segment_path(name),
    })
}

fn typed_pat(name: &str, ty: &str) -> syn::Pat {
    syn::Pat::Type(syn::PatType {
        attrs: Vec::new(),
        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: ident(name),
            subpat: None,
        })),
        colon_token: Default::default(),
        ty: Box::new(type_path(ty)),
    })
}

fn binary_expr(left: syn::Expr, op: syn::BinOp, right: syn::Expr) -> syn::Expr {
    syn::Expr::Binary(syn::ExprBinary {
        attrs: Vec::new(),
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

fn closure_expr(params: &[(&str, &str)], body: syn::Expr) -> syn::Expr {
    let mut inputs = syn::punctuated::Punctuated::new();
    for (name, ty) in params {
        inputs.push(typed_pat(name, ty));
    }
    syn::Expr::Closure(syn::ExprClosure {
        attrs: Vec::new(),
        lifetimes: None,
        constness: None,
        movability: None,
        asyncness: None,
        capture: None,
        or1_token: Default::default(),
        inputs,
        or2_token: Default::default(),
        output: syn::ReturnType::Default,
        body: Box::new(body),
    })
}

// --- calculator operations: real function + Code, on the same operation ---

struct Add;
impl Add {
    fn apply(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
impl Code for Add {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let body = binary_expr(
            path_expr("a"),
            syn::BinOp::Add(Default::default()),
            path_expr("b"),
        );
        Ir::leaf(closure_expr(&[("a", "i32"), ("b", "i32")], body))
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
            path_expr("a"),
            syn::BinOp::Sub(Default::default()),
            path_expr("b"),
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
            path_expr("a"),
            syn::BinOp::Mul(Default::default()),
            path_expr("b"),
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
            path_expr("a"),
            syn::BinOp::Div(Default::default()),
            path_expr("b"),
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
        let body = binary_expr(path_expr("x"), syn::BinOp::Mul(Default::default()), two);
        Ir::leaf(closure_expr(&[("x", "i32")], body))
    }
}

// --- the calculator: bundles all four, subsettable via boundary/Locality ---

struct Calculator {
    add: Add,
    subtract: Subtract,
    multiply: Multiply,
    divide: Divide,
    include_advanced: bool,
}

impl Code for Calculator {
    type Fragment = Ir;
    fn code(&self) -> Ir {
        let elems = [
            self.add.code().expr().clone(),
            self.subtract.code().expr().clone(),
            self.multiply.code().expr().clone(),
            self.divide.code().expr().clone(),
        ]
        .into_iter()
        .collect();
        Ir::leaf(syn::Expr::Tuple(syn::ExprTuple {
            attrs: Vec::new(),
            paren_token: Default::default(),
            elems,
        }))
    }
}

impl Scope for Calculator {
    fn boundary(&self) -> impl Iterator<Item = (Ir, Box<dyn Locality<Ir>>)> {
        let multiply_locality: Box<dyn Locality<Ir>> = if self.include_advanced {
            Box::new(Inline)
        } else {
            Box::new(Omit)
        };
        let divide_locality: Box<dyn Locality<Ir>> = if self.include_advanced {
            Box::new(Inline)
        } else {
            Box::new(Omit)
        };
        vec![
            (self.add.code(), Box::new(Inline) as Box<dyn Locality<Ir>>),
            (
                self.subtract.code(),
                Box::new(Inline) as Box<dyn Locality<Ir>>,
            ),
            (self.multiply.code(), multiply_locality),
            (self.divide.code(), divide_locality),
        ]
        .into_iter()
    }

    fn scope(&self) -> Ir {
        // Deliberately does NOT reuse code()'s output as a tail expression
        // the way Transition's scope() (scope_test.rs) does — code() always
        // builds the full, unshaved tuple of all four operations, so
        // appending it here would silently reintroduce whatever boundary()
        // omitted. Calculator's boundary() and code() describe the same
        // underlying data, so the tail has to be built from what actually
        // survived Locality filtering, not from the always-unshaved code().
        let elems: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma> = self
            .boundary()
            .filter_map(|(dependency, locality)| locality.contribute(&dependency))
            .map(|fragment| fragment.expr().clone())
            .collect();
        Ir::leaf(syn::Expr::Tuple(syn::ExprTuple {
            attrs: Vec::new(),
            paren_token: Default::default(),
            elems,
        }))
    }
}

impl Extent for Calculator {
    // Reuses each operation's own code() directly — no new traversal
    // logic, the same closure Scope::boundary() already builds for that
    // name. Every name here is declared regardless of include_advanced:
    // naming and shaving are different questions (see HOMECOMING_PLAN.md's
    // Extent/Selection split), so a name stays a valid anchor even when
    // the current shave configuration would omit it from scope().
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
        include_advanced: true,
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
        include_advanced: false,
    };

    let scoped = calculator.scope();
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
    // Multiply/Divide are omitted from the *code*, not deleted from the
    // program. This is the distinction between "isolated from the program"
    // and "removed from the program."
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
        include_advanced: false,
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
        include_advanced: true,
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
    // A name stays a valid anchor even when include_advanced would omit it
    // from a rendered scope() — Extent only answers "is this a nameable
    // unit," never "is this included in a given shaved result." Compare
    // to shaved_calculator_scope_excludes_multiply_and_divide, where the
    // same configuration does exclude multiply/divide from scope()'s
    // *rendered* output; anchor() is a different question entirely.
    let calculator = Calculator {
        add: Add,
        subtract: Subtract,
        multiply: Multiply,
        divide: Divide,
        include_advanced: false,
    };

    let anchored = calculator
        .anchor("multiply")
        .ok_or("multiply stays a declared anchor even when shaved out of scope()")?;
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
        include_advanced: true,
    };

    // "clear" was never annotated as an isolatable unit — never a
    // candidate at all, not merely excluded by a Selection policy that
    // hasn't been implemented yet.
    assert!(calculator.anchor("clear").is_none());
}
