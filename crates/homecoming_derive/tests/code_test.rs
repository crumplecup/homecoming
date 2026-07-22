//! Exercises `#[derive(Code)]` against a real struct whose fields already
//! implement `Code<Fragment = Ir>` via `homecoming_core`'s own std-lib
//! impls.

use homecoming_core::Code;
use homecoming_derive::Code;
use quote::ToTokens;

#[derive(Code)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn derived_code_emits_a_struct_literal_built_from_field_code() -> Result<(), syn::Error> {
    let point = Point { x: 3, y: 4 };

    let fragment = point.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("Point"), "rendered: {rendered}");
    assert!(rendered.contains("x"), "rendered: {rendered}");
    assert!(rendered.contains("3i32"), "rendered: {rendered}");
    assert!(rendered.contains("y"), "rendered: {rendered}");
    assert!(rendered.contains("4i32"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn derived_code_reflects_the_actual_field_values_not_defaults() -> Result<(), syn::Error> {
    // The motivating failure case this whole crate exists to catch: a
    // derive that silently falls back to a plausible-looking default for
    // a field it didn't actually inspect. Two different instances must
    // emit two different, correct fragments.
    let a = Point { x: 1, y: 2 };
    let b = Point { x: 9, y: 9 };

    assert_ne!(a.code().expr(), b.code().expr());

    let rendered_a = a.code().to_token_stream().to_string();
    assert!(rendered_a.contains("1i32"), "rendered: {rendered_a}");
    assert!(rendered_a.contains("2i32"), "rendered: {rendered_a}");
    Ok(())
}
