//! Direct `syn` AST construction helpers, shared across `Code`/`Scope`/
//! `Extent` implementors instead of hand-built per call site.
//!
//! Every function here builds real `syn` nodes field by field — no
//! `quote!`/parsing round trip, so nothing here can fail. That matters
//! because `Code::code()`'s signature has no room for a `Result`:
//! constructing the AST a value's source actually looked like has to be
//! infallible by construction, the same discipline every `Code` impl in
//! this crate already follows.
//!
//! Promoted here once the same handful of builders had been hand-written,
//! nearly identically, in three separate places (`homecoming_core`'s own
//! calculator and stoplight examples, and `homecoming_amenable`'s real
//! `amenable`-bound stoplight) — not designed in advance of that need.

/// A bare identifier, at the call site's span.
pub fn ident(name: &str) -> syn::Ident {
    syn::Ident::new(name, proc_macro2::Span::call_site())
}

/// A `::`-free path from segment names, e.g. `path(&["Stoplight", "Green"])`
/// for `Stoplight::Green`.
pub fn path(segments: &[&str]) -> syn::Path {
    let mut punctuated = syn::punctuated::Punctuated::new();
    for segment in segments {
        punctuated.push(syn::PathSegment {
            ident: ident(segment),
            arguments: syn::PathArguments::None,
        });
    }
    syn::Path {
        leading_colon: None,
        segments: punctuated,
    }
}

/// A path used as an expression, e.g. a bare variable, unit struct, or
/// enum variant reference.
pub fn path_expr(segments: &[&str]) -> syn::Expr {
    syn::Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: path(segments),
    })
}

/// A path used as a type.
pub fn type_path(segments: &[&str]) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: path(segments),
    })
}

/// A function call, `func(args...)`.
pub fn call_expr(func: syn::Expr, args: Vec<syn::Expr>) -> syn::Expr {
    syn::Expr::Call(syn::ExprCall {
        attrs: Vec::new(),
        func: Box::new(func),
        paren_token: Default::default(),
        args: args.into_iter().collect(),
    })
}

/// A method call, `receiver.method(args...)`.
pub fn method_call_expr(receiver: syn::Expr, method: &str, args: Vec<syn::Expr>) -> syn::Expr {
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

/// A binary expression, `left op right`.
pub fn binary_expr(left: syn::Expr, op: syn::BinOp, right: syn::Expr) -> syn::Expr {
    syn::Expr::Binary(syn::ExprBinary {
        attrs: Vec::new(),
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

/// A tuple expression, `(elems...)`.
pub fn tuple_expr(elems: Vec<syn::Expr>) -> syn::Expr {
    syn::Expr::Tuple(syn::ExprTuple {
        attrs: Vec::new(),
        paren_token: Default::default(),
        elems: elems.into_iter().collect(),
    })
}

/// A struct-literal expression, `Name { field: value, ... }` — the shape
/// `#[derive(Code)]` needs to emit a struct's own source from its fields'
/// captured code.
pub fn struct_expr(segments: &[&str], fields: Vec<(&str, syn::Expr)>) -> syn::Expr {
    let mut punctuated = syn::punctuated::Punctuated::new();
    for (name, value) in fields {
        punctuated.push(syn::FieldValue {
            attrs: Vec::new(),
            member: syn::Member::Named(ident(name)),
            colon_token: Some(Default::default()),
            expr: value,
        });
    }
    syn::Expr::Struct(syn::ExprStruct {
        attrs: Vec::new(),
        qself: None,
        path: path(segments),
        brace_token: Default::default(),
        fields: punctuated,
        dot2_token: None,
        rest: None,
    })
}

/// A type-annotated identifier pattern, `name: Type` — a closure or
/// function parameter.
pub fn typed_pat(name: &str, ty: &str) -> syn::Pat {
    syn::Pat::Type(syn::PatType {
        attrs: Vec::new(),
        pat: Box::new(ident_pat(name)),
        colon_token: Default::default(),
        ty: Box::new(type_path(&[ty])),
    })
}

/// A closure expression, `|params...| body`.
pub fn closure_expr(params: &[(&str, &str)], body: syn::Expr) -> syn::Expr {
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

/// A wildcard pattern, `_`.
pub fn wildcard_pat() -> syn::Pat {
    syn::Pat::Wild(syn::PatWild {
        attrs: Vec::new(),
        underscore_token: Default::default(),
    })
}

/// A bare identifier pattern, binding a name.
pub fn ident_pat(name: &str) -> syn::Pat {
    syn::Pat::Ident(syn::PatIdent {
        attrs: Vec::new(),
        by_ref: None,
        mutability: None,
        ident: ident(name),
        subpat: None,
    })
}

/// A tuple pattern, `(elems...)`.
pub fn tuple_pat(elems: Vec<syn::Pat>) -> syn::Pat {
    syn::Pat::Tuple(syn::PatTuple {
        attrs: Vec::new(),
        paren_token: Default::default(),
        elems: elems.into_iter().collect(),
    })
}

/// A tuple-struct or enum-tuple-variant pattern, `Name(elems...)`.
pub fn tuple_struct_pat(segments: &[&str], elems: Vec<syn::Pat>) -> syn::Pat {
    syn::Pat::TupleStruct(syn::PatTupleStruct {
        attrs: Vec::new(),
        qself: None,
        path: path(segments),
        paren_token: Default::default(),
        elems: elems.into_iter().collect(),
    })
}

/// One `match` arm, `pat => body,`.
pub fn match_arm(pat: syn::Pat, body: syn::Expr) -> syn::Arm {
    syn::Arm {
        attrs: Vec::new(),
        pat,
        guard: None,
        fat_arrow_token: Default::default(),
        body: Box::new(body),
        comma: Some(Default::default()),
    }
}

/// A `match` expression.
pub fn match_expr(scrutinee: syn::Expr, arms: Vec<syn::Arm>) -> syn::Expr {
    syn::Expr::Match(syn::ExprMatch {
        attrs: Vec::new(),
        match_token: Default::default(),
        expr: Box::new(scrutinee),
        brace_token: Default::default(),
        arms,
    })
}

/// `match <scrutinee> { Ok(<ok_pat>) => <ok_value>, Err(never) => match
/// never {} }` — the honest way to discharge a `Result<T, Infallible>`
/// without `.unwrap()`: the `Err` arm's body is itself a match with zero
/// arms, which only typechecks because `Infallible` has zero variants, so
/// it is unreachable by construction, not by convention. Any `Code` impl
/// emitting a call to an infallible-but-`Result`-typed API needs exactly
/// this pattern, not a bespoke one per call site.
pub fn unwrap_infallible_expr(
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

/// A `let` statement, `let pat = init;`.
pub fn let_stmt(pat: syn::Pat, init: syn::Expr) -> syn::Stmt {
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

/// A block expression, `{ stmts...; tail }`.
pub fn block_expr(stmts: Vec<syn::Stmt>, tail: syn::Expr) -> syn::Expr {
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
