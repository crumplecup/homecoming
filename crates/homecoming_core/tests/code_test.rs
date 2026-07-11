use homecoming_core::Code;
use quote::ToTokens;

#[test]
fn primitive_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let value = 42i32;
    let fragment = value.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;

    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn bool_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let value = true;
    let fragment = value.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;

    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn char_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let value = 'x';
    let fragment = value.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;

    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn fabricated_fragment_fails_the_round_trip() -> Result<(), syn::Error> {
    // A default()-fallback style failure: claim to capture 42 but actually
    // emit 0. The round-trip check must be able to catch this mechanically.
    let claimed = 42i32;
    let fabricated_fragment = 0i32.code();
    let tokens = fabricated_fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    let real_fragment = claimed.code();

    assert_ne!(real_fragment.expr(), &reparsed);
    Ok(())
}
