use homecoming_core::{Code, Fragment, Inline, Locality, Scope};
use quote::ToTokens;

#[derive(Debug, Clone, PartialEq)]
enum Stoplight {
    Green,
    Yellow,
    Red,
}

fn variant_path(variant: &str) -> syn::Path {
    let mut segments = syn::punctuated::Punctuated::new();
    segments.push(syn::PathSegment {
        ident: syn::Ident::new("Stoplight", proc_macro2::Span::call_site()),
        arguments: syn::PathArguments::None,
    });
    segments.push(syn::PathSegment {
        ident: syn::Ident::new(variant, proc_macro2::Span::call_site()),
        arguments: syn::PathArguments::None,
    });
    syn::Path {
        leading_colon: None,
        segments,
    }
}

impl Code for Stoplight {
    fn code(&self) -> Fragment {
        let variant = match self {
            Stoplight::Green => "Green",
            Stoplight::Yellow => "Yellow",
            Stoplight::Red => "Red",
        };
        let expr = syn::Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: variant_path(variant),
        });
        Fragment::leaf(expr)
    }
}

/// A transition between two states — the state-machine-shaped case the
/// core tier's `Scope` is meant to serve.
struct Transition {
    from: Stoplight,
    to: Stoplight,
}

impl Code for Transition {
    fn code(&self) -> Fragment {
        let elems = [
            self.from.code().expr().clone(),
            self.to.code().expr().clone(),
        ]
        .into_iter()
        .collect();
        Fragment::leaf(syn::Expr::Tuple(syn::ExprTuple {
            attrs: Vec::new(),
            paren_token: Default::default(),
            elems,
        }))
    }
}

impl Scope for Transition {
    fn boundary(&self) -> impl Iterator<Item = (Fragment, Box<dyn Locality>)> {
        vec![
            (self.from.code(), Box::new(Inline) as Box<dyn Locality>),
            (self.to.code(), Box::new(Inline) as Box<dyn Locality>),
        ]
        .into_iter()
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
    let reference = Reference::new(variant_path("Green"));
    let contribution = reference
        .contribute(&dependency)
        .ok_or("Reference must always contribute a name fragment")?;
    let tokens = contribution.to_token_stream();
    let rendered = tokens.to_string();

    // References by the given path, not by reproducing the dependency it
    // was handed — the whole point of Reference vs Inline.
    assert!(rendered.contains("Green"), "rendered: {rendered}");
    assert!(!rendered.contains("Red"), "rendered: {rendered}");
    let _reparsed: syn::Expr = syn::parse2(tokens)?;
    Ok(())
}
