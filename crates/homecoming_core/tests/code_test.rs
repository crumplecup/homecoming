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
fn negative_integer_round_trips_through_tokens() -> Result<(), syn::Error> {
    // Rust's literal grammar has no negative-literal token: "-5" always
    // parses as unary negation of the literal 5, never as one literal
    // token spelled "-5". A captured Expr::Lit("-5") would reparse as
    // Expr::Unary(Neg, Expr::Lit(5)) instead — a different AST shape the
    // structural round-trip check below would catch immediately.
    let value = -5i32;
    let fragment = value.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;

    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn signed_min_value_round_trips_through_tokens() -> Result<(), syn::Error> {
    // i32::MIN's magnitude (2147483648) does not fit i32 on its own — only
    // the negation of it does. unsigned_abs() (not abs(), which overflows)
    // is what makes this representable at all.
    let value = i32::MIN;
    let fragment = value.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;

    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn unsigned_integer_code_carries_a_type_suffix() {
    // An unsuffixed literal like `200` defaults to i32 in an unconstrained
    // context, silently changing a captured u8's concrete type. Every
    // integer Code impl must emit its own type suffix to stay unambiguous
    // regardless of where the fragment ends up embedded.
    let value = 200u8;
    let fragment = value.code();
    let rendered = fragment.to_token_stream().to_string();

    assert!(rendered.contains("u8"), "rendered: {rendered}");
}

#[test]
fn every_integer_width_round_trips_through_tokens() -> Result<(), syn::Error> {
    macro_rules! check {
        ($value:expr) => {{
            let fragment = $value.code();
            let tokens = fragment.to_token_stream();
            let reparsed: syn::Expr = syn::parse2(tokens)?;
            assert_eq!(fragment.expr(), &reparsed);
        }};
    }

    check!(i8::MIN);
    check!(i8::MAX);
    check!(i16::MIN);
    check!(i16::MAX);
    check!(i32::MIN);
    check!(i32::MAX);
    check!(i64::MIN);
    check!(i64::MAX);
    check!(i128::MIN);
    check!(i128::MAX);
    check!(isize::MIN);
    check!(isize::MAX);
    check!(u8::MAX);
    check!(u16::MAX);
    check!(u32::MAX);
    check!(u64::MAX);
    check!(u128::MAX);
    check!(usize::MAX);
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
fn float_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    macro_rules! check {
        ($value:expr) => {{
            let fragment = $value.code();
            let tokens = fragment.to_token_stream();
            let reparsed: syn::Expr = syn::parse2(tokens)?;
            assert_eq!(fragment.expr(), &reparsed);
        }};
    }

    check!(3.5f64);
    check!(-3.5f64);
    check!(0.0f64);
    check!(-0.0f64);
    check!(1.0f32);
    check!(-1.0f32);
    Ok(())
}

#[test]
fn float_code_uses_associated_constants_for_non_finite_values() -> Result<(), syn::Error> {
    // NaN and the infinities have no literal token at all -- capturing
    // them as a number would be dishonest even if some numeral happened
    // to compare equal at runtime (nothing does, for NaN).
    macro_rules! check {
        ($value:expr, $contains:literal) => {{
            let fragment = $value.code();
            let tokens = fragment.to_token_stream();
            let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
            assert_eq!(fragment.expr(), &reparsed);
            let rendered = tokens.to_string();
            assert!(rendered.contains($contains), "rendered: {rendered}");
        }};
    }

    check!(f64::NAN, "NAN");
    check!(f64::INFINITY, "INFINITY");
    check!(f64::NEG_INFINITY, "NEG_INFINITY");
    Ok(())
}

#[test]
fn float_code_distinguishes_negative_zero_from_zero() {
    // -0.0 == 0.0 numerically, but they are different bit patterns and
    // this capture stays faithful to which one was actually there.
    let positive = 0.0f64.code().to_token_stream().to_string();
    let negative = (-0.0f64).code().to_token_stream().to_string();
    assert_ne!(positive, negative);
}

#[test]
fn option_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let none: Option<i32> = None;
    let fragment = none.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    assert_eq!(fragment.expr(), &reparsed);

    let some = Some(7i32);
    let fragment = some.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("Some"), "rendered: {rendered}");
    assert!(rendered.contains("7i32"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn tuple_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let pair = (1i32, true);
    let fragment = pair.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("1i32"), "rendered: {rendered}");
    assert!(rendered.contains("true"), "rendered: {rendered}");

    let triple = (1i32, true, 'x');
    let fragment = triple.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    assert_eq!(fragment.expr(), &reparsed);

    let quad = (1i32, true, 'x', 2u8);
    let fragment = quad.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens)?;
    assert_eq!(fragment.expr(), &reparsed);
    Ok(())
}

#[test]
fn array_code_round_trips_through_tokens() -> Result<(), syn::Error> {
    let values = [1i32, 2, 3];
    let fragment = values.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("1i32"), "rendered: {rendered}");
    assert!(rendered.contains("2i32"), "rendered: {rendered}");
    assert!(rendered.contains("3i32"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn array_code_reflects_actual_elements_not_defaults() {
    let a = [1i32, 2, 3];
    let b = [9i32, 9, 9];
    assert_ne!(a.code().expr(), b.code().expr());
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
