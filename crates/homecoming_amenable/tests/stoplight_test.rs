//! Hand-implements `Code` for a real `amenable::Exchange` transition — the
//! first attempt to bind homecoming's capture/isolation machinery to
//! `amenable`'s actual proof-role traits, on `amenable_kani`'s real
//! `Stoplight` rather than a homecoming-native lookalike.

use amenable_core::Exchange;
use amenable_kani::{Green, GreenToken, Red, Stoplight, Yellow};
use homecoming_core::{Code, Inline, Ir, Locality, Scope};
use quote::ToTokens;

// --- syn AST construction helpers, direct construction, no parsing ---

fn ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

fn path_expr(segments: &[&str]) -> syn::Expr {
    let mut punctuated = syn::punctuated::Punctuated::new();
    for segment in segments {
        punctuated.push(syn::PathSegment {
            ident: ident(segment),
            arguments: syn::PathArguments::None,
        });
    }
    syn::Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: punctuated,
        },
    })
}

fn call_expr(func: syn::Expr, args: Vec<syn::Expr>) -> syn::Expr {
    syn::Expr::Call(syn::ExprCall {
        attrs: Vec::new(),
        func: Box::new(func),
        paren_token: Default::default(),
        args: args.into_iter().collect(),
    })
}

fn method_call_expr(receiver: syn::Expr, method: &str, args: Vec<syn::Expr>) -> syn::Expr {
    syn::Expr::MethodCall(syn::ExprMethodCall {
        attrs: Vec::new(),
        receiver: Box::new(receiver),
        dot_token: Default::default(),
        method: ident(method),
        turbofish: None,
        paren_token: Default::default(),
        args: args.into_iter().collect(),
    })
}

fn wildcard_pat() -> syn::Pat {
    syn::Pat::Wild(syn::PatWild {
        attrs: Vec::new(),
        underscore_token: Default::default(),
    })
}

fn ident_pat(name: &str) -> syn::Pat {
    syn::Pat::Ident(syn::PatIdent {
        attrs: Vec::new(),
        by_ref: None,
        mutability: None,
        ident: ident(name),
        subpat: None,
    })
}

fn tuple_struct_pat(segments: &[&str], elems: Vec<syn::Pat>) -> syn::Pat {
    let mut punctuated = syn::punctuated::Punctuated::new();
    for segment in segments {
        punctuated.push(syn::PathSegment {
            ident: ident(segment),
            arguments: syn::PathArguments::None,
        });
    }
    syn::Pat::TupleStruct(syn::PatTupleStruct {
        attrs: Vec::new(),
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: punctuated,
        },
        paren_token: Default::default(),
        elems: elems.into_iter().collect(),
    })
}

fn tuple_pat(elems: Vec<syn::Pat>) -> syn::Pat {
    syn::Pat::Tuple(syn::PatTuple {
        attrs: Vec::new(),
        paren_token: Default::default(),
        elems: elems.into_iter().collect(),
    })
}

fn match_arm(pat: syn::Pat, body: syn::Expr) -> syn::Arm {
    syn::Arm {
        attrs: Vec::new(),
        pat,
        guard: None,
        fat_arrow_token: Default::default(),
        body: Box::new(body),
        comma: Some(Default::default()),
    }
}

fn match_expr(scrutinee: syn::Expr, arms: Vec<syn::Arm>) -> syn::Expr {
    syn::Expr::Match(syn::ExprMatch {
        attrs: Vec::new(),
        match_token: Default::default(),
        expr: Box::new(scrutinee),
        brace_token: Default::default(),
        arms,
    })
}

/// `match <scrutinee> { Ok(<ok_pat>) => <ok_pat's bound name>, Err(never)
/// => match never {} }` — the honest way to discharge a
/// `Result<T, Infallible>` without `.unwrap()`: the `Err` arm's body is
/// itself a match with zero arms, which only typechecks because
/// `Infallible` has zero variants, so it is unreachable by construction,
/// not by convention.
fn unwrap_infallible_expr(
    scrutinee: syn::Expr,
    ok_pat: syn::Pat,
    ok_value: syn::Expr,
) -> syn::Expr {
    let never_arm = match_arm(
        tuple_struct_pat(&["Err"], vec![ident_pat("never")]),
        match_expr(path_expr(&["never"]), Vec::new()),
    );
    let ok_arm = match_arm(tuple_struct_pat(&["Ok"], vec![ok_pat]), ok_value);
    match_expr(scrutinee, vec![ok_arm, never_arm])
}

fn let_stmt(pat: syn::Pat, init: syn::Expr) -> syn::Stmt {
    syn::Stmt::Local(syn::Local {
        attrs: Vec::new(),
        let_token: Default::default(),
        pat,
        init: Some(syn::LocalInit {
            eq_token: Default::default(),
            expr: Box::new(init),
            diverge: None,
        }),
        semi_token: Default::default(),
    })
}

fn block_expr(stmts: Vec<syn::Stmt>, tail: syn::Expr) -> syn::Expr {
    let mut stmts = stmts;
    stmts.push(syn::Stmt::Expr(tail, None));
    syn::Expr::Block(syn::ExprBlock {
        attrs: Vec::new(),
        label: None,
        block: syn::Block {
            brace_token: Default::default(),
            stmts,
        },
    })
}

/// The `Green -> Yellow` transition as a real, callable operation that can
/// also emit the exact code that performs it — the same quote/eval duality
/// `homecoming_core`'s calculator example demonstrated, now bound to a real
/// `amenable::Exchange` impl instead of a homecoming-native stand-in.
struct GreenToYellow;

impl GreenToYellow {
    fn apply(&self) -> Yellow {
        let stoplight = Stoplight;
        match stoplight.exchange(Green, GreenToken::new(Green)) {
            Ok((yellow, _token)) => yellow,
            Err(never) => match never {},
        }
    }
}

impl Code for GreenToYellow {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let expr = method_call_expr(
            path_expr(&["Stoplight"]),
            "exchange",
            vec![
                path_expr(&["Green"]),
                call_expr(
                    path_expr(&["GreenToken", "new"]),
                    vec![path_expr(&["Green"])],
                ),
            ],
        );
        Ir::leaf(expr)
    }
}

#[test]
fn green_to_yellow_executes_and_emits_matching_code() -> Result<(), syn::Error> {
    let transition = GreenToYellow;

    // Real execution, through amenable's actual Exchange impl.
    assert_eq!(transition.apply(), Yellow);

    // The exact code that performs it, round-tripped through tokens.
    let fragment = transition.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("exchange"), "rendered: {rendered}");
    assert!(rendered.contains("GreenToken"), "rendered: {rendered}");
    Ok(())
}

/// The `Yellow -> Red` transition — unlike `GreenToYellow`, this one has a
/// real dependency: `YellowToken` has no `::new()`, so a valid one only
/// ever comes from actually performing `Exchange<Green, Yellow>` first.
/// That is a genuine depends-on edge in the call graph, not a modeling
/// convenience — `code()` emits only this transition's own call,
/// referencing `yellow_token` by name; `scope()` reconstructs the minimal
/// standalone code by walking that edge, binding just the token the
/// dependency produces, nothing else pulled in.
struct YellowToRed;

impl YellowToRed {
    fn apply(&self) -> Red {
        let stoplight = Stoplight;
        let (_yellow, yellow_token) = match stoplight.exchange(Green, GreenToken::new(Green)) {
            Ok(pair) => pair,
            Err(never) => match never {},
        };
        match stoplight.exchange(Yellow, yellow_token) {
            Ok((red, _token)) => red,
            Err(never) => match never {},
        }
    }
}

impl Code for YellowToRed {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let expr = method_call_expr(
            path_expr(&["Stoplight"]),
            "exchange",
            vec![path_expr(&["Yellow"]), path_expr(&["yellow_token"])],
        );
        Ir::leaf(expr)
    }
}

impl Scope for YellowToRed {
    fn boundary(&self) -> impl Iterator<Item = (Ir, Box<dyn Locality<Ir>>)> {
        vec![(
            GreenToYellow.code(),
            Box::new(Inline) as Box<dyn Locality<Ir>>,
        )]
        .into_iter()
    }

    fn scope(&self) -> Ir {
        // The one boundary entry produces (Yellow, YellowToken); only the
        // token is live here -- the Yellow value itself is discarded
        // (`_`), since code() never references it.
        let dependency_call = GreenToYellow.code().expr().clone();
        let token_value = unwrap_infallible_expr(
            dependency_call,
            tuple_pat(vec![wildcard_pat(), ident_pat("token")]),
            path_expr(&["token"]),
        );
        let binding = let_stmt(ident_pat("yellow_token"), token_value);
        let tail = self.code().expr().clone();
        Ir::leaf(block_expr(vec![binding], tail))
    }

    fn scope_with(&self, selection: &dyn homecoming_core::Selection<Ir>) -> Ir {
        let dependency = GreenToYellow.code();
        if selection.includes(&dependency) {
            self.scope()
        } else {
            // Unlike Calculator's Multiply/Divide, this boundary entry
            // isn't an optional inclusion choice -- it's a proof
            // precondition. YellowToken has no ::new(), so excluding the
            // dependency here can't fall back to some other valid
            // rendering the way omitting an operation could; the honest
            // answer is this transition's own leaf code, which the
            // caller already knows (from code()'s dangling
            // `yellow_token` reference) does not compile standalone.
            self.code()
        }
    }
}

#[test]
fn yellow_to_red_executes_and_emits_a_standalone_scope() -> Result<(), syn::Error> {
    let transition = YellowToRed;

    // Real execution, chaining through the real Green -> Yellow exchange
    // first to obtain a genuine YellowToken.
    assert_eq!(transition.apply(), Red);

    // code() alone is not standalone -- it references `yellow_token`,
    // which nothing in code()'s own output binds.
    let leaf = transition.code();
    let leaf_tokens = leaf.to_token_stream().to_string();
    assert!(leaf_tokens.contains("yellow_token"), "leaf: {leaf_tokens}");

    // scope() reconstructs the minimal standalone code: bind the token
    // the boundary dependency produces, then this transition's own call.
    let scoped = transition.scope();
    let tokens = scoped.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(scoped.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(
        rendered.contains("let yellow_token"),
        "rendered: {rendered}"
    );
    assert!(rendered.contains("GreenToken"), "rendered: {rendered}");
    assert!(rendered.contains("Yellow"), "rendered: {rendered}");
    Ok(())
}

/// Names the two transitions above as isolatable units — the same naming
/// grammar `Calculator::anchor` used, but exercised against a case where
/// the distinction it was designed around actually bites: `code()` alone
/// is standalone for a leaf transition with no dependency (`green_to_
/// yellow`), but not for one with a real depends-on edge (`yellow_to_
/// red`), where the *live* code for that name is `scope()`'s
/// reconstruction, not the bare leaf.
struct StoplightCycle;

impl Code for StoplightCycle {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let elems = [
            GreenToYellow.code().expr().clone(),
            YellowToRed.scope().expr().clone(),
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

impl homecoming_core::Extent for StoplightCycle {
    fn anchor(&self, name: &str) -> Option<Ir> {
        match name {
            "green_to_yellow" => Some(GreenToYellow.code()),
            // Not YellowToRed.code() -- that leaf alone references an
            // unbound `yellow_token`. The live code for this name is the
            // reconstructed, standalone scope.
            "yellow_to_red" => Some(YellowToRed.scope()),
            _ => None,
        }
    }
}

#[test]
fn anchor_gives_back_live_code_not_just_leaf_code() -> Result<(), Box<dyn std::error::Error>> {
    use homecoming_core::Extent;

    let cycle = StoplightCycle;

    let green_to_yellow = cycle
        .anchor("green_to_yellow")
        .ok_or("green_to_yellow must be a declared anchor")?;
    assert_eq!(green_to_yellow.expr(), GreenToYellow.code().expr());

    let yellow_to_red = cycle
        .anchor("yellow_to_red")
        .ok_or("yellow_to_red must be a declared anchor")?;
    // The anchor's answer must be the reconstructed, standalone scope --
    // not the bare leaf, which references an unbound name and would not
    // compile if emitted on its own.
    assert_eq!(yellow_to_red.expr(), YellowToRed.scope().expr());
    assert_ne!(yellow_to_red.expr(), YellowToRed.code().expr());

    let tokens = yellow_to_red.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    assert_eq!(yellow_to_red.expr(), &reparsed);

    assert!(cycle.anchor("red_to_green").is_none());
    Ok(())
}
