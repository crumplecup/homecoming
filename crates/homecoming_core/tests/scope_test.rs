use homecoming_core::{Code, Inline, Ir, Locality, Scope, Selection, path_expr, tuple_expr};
use quote::ToTokens;

#[derive(Debug, Clone, PartialEq)]
enum Stoplight {
    Green,
    Yellow,
    Red,
}

impl Code for Stoplight {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let variant = match self {
            Stoplight::Green => "Green",
            Stoplight::Yellow => "Yellow",
            Stoplight::Red => "Red",
        };
        Ir::leaf(path_expr(&["Stoplight", variant]))
    }
}

/// A transition between two states — the state-machine-shaped case the
/// core tier's `Scope` is meant to serve.
struct Transition {
    from: Stoplight,
    to: Stoplight,
}

impl Code for Transition {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let elems = vec![
            self.from.code().expr().clone(),
            self.to.code().expr().clone(),
        ];
        Ir::leaf(tuple_expr(elems))
    }
}

impl Scope for Transition {
    fn boundary(&self) -> impl Iterator<Item = (Ir, Box<dyn Locality<Ir>>)> {
        vec![
            (self.from.code(), Box::new(Inline) as Box<dyn Locality<Ir>>),
            (self.to.code(), Box::new(Inline) as Box<dyn Locality<Ir>>),
        ]
        .into_iter()
    }

    fn scope(&self) -> Ir {
        // No lateralizing composition trait exists yet (see
        // HOMECOMING_PLAN.md Phase 5), so this hand-writes composition
        // directly against Ir, tupling whatever boundary() survivors
        // Locality contributes. Deliberately does NOT reuse code()'s
        // output as the tail the way an earlier version did — code()
        // always builds the full (from, to) tuple regardless of Locality,
        // so appending it here would silently reintroduce whatever
        // boundary() omitted (the same bug Calculator's scope() had, see
        // calculator_test.rs).
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

#[test]
fn stoplight_state_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let state = Stoplight::Green;
    let fragment = state.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;

    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn transition_scope_includes_both_boundary_states() -> Result<(), syn::Error> {
    let transition = Transition {
        from: Stoplight::Green,
        to: Stoplight::Yellow,
    };

    let scoped = transition.scope();
    let tokens = scoped.to_token_stream();
    // The scoped fragment must actually parse as valid Rust — the whole
    // point of Fragment being syn-typed rather than raw tokens.
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;

    let rendered = tokens.to_string();
    assert!(rendered.contains("Green"), "rendered: {rendered}");
    assert!(rendered.contains("Yellow"), "rendered: {rendered}");
    Ok(())
}

/// A policy that only orders the `Green` dish off the menu, regardless of
/// what `Locality` would otherwise render.
struct OnlyGreen;

impl Selection<Ir> for OnlyGreen {
    fn includes(&self, item: &Ir) -> bool {
        item.to_token_stream().to_string().contains("Green")
    }
}

#[test]
fn scope_with_excludes_entries_selection_does_not_include() -> Result<(), syn::Error> {
    let transition = Transition {
        from: Stoplight::Green,
        to: Stoplight::Yellow,
    };

    let scoped = transition.scope_with(&OnlyGreen);
    let tokens = scoped.to_token_stream();
    let _reparsed: syn::Expr = syn::parse2(tokens.clone())?;

    let rendered = tokens.to_string();
    assert!(rendered.contains("Green"), "rendered: {rendered}");
    assert!(!rendered.contains("Yellow"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn omit_locality_excludes_its_boundary_entry() {
    use homecoming_core::Omit;

    let dependency = Stoplight::Red.code();
    let contribution = Omit.contribute(&dependency);

    assert!(contribution.is_none());
}

#[test]
fn reference_locality_names_without_reproducing() -> Result<(), Box<dyn std::error::Error>> {
    use homecoming_core::Reference;

    let dependency = Stoplight::Red.code();
    let replacement = Ir::leaf(path_expr(&["Stoplight", "Green"]));
    let reference = Reference::new(replacement);
    let contribution = reference
        .contribute(&dependency)
        .ok_or("Reference must always contribute a name fragment")?;
    let tokens = contribution.to_token_stream();
    let rendered = tokens.to_string();

    // References by the replacement it was constructed with, not by
    // reproducing the dependency it was handed — the whole point of
    // Reference vs Inline.
    assert!(rendered.contains("Green"), "rendered: {rendered}");
    assert!(!rendered.contains("Red"), "rendered: {rendered}");
    let _reparsed: syn::Expr = syn::parse2(tokens)?;
    Ok(())
}
