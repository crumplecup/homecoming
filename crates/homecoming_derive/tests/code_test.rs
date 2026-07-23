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

#[derive(Code)]
struct Empty;

#[test]
fn derived_code_on_a_unit_struct_emits_a_bare_path() -> Result<(), syn::Error> {
    let fragment = Empty.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert_eq!(rendered, "Empty");
    Ok(())
}

#[derive(Code)]
struct Pair(i32, i32);

#[test]
fn derived_code_on_a_tuple_struct_emits_positional_construction() -> Result<(), syn::Error> {
    let pair = Pair(3, 4);

    let fragment = pair.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("Pair"), "rendered: {rendered}");
    assert!(rendered.contains("3i32"), "rendered: {rendered}");
    assert!(rendered.contains("4i32"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn derived_code_on_a_tuple_struct_reflects_actual_values() {
    let a = Pair(1, 2);
    let b = Pair(9, 9);

    assert_ne!(a.code().expr(), b.code().expr());
}

/// Mixes all three variant shapes in one enum -- the case that actually
/// needs the match-arm dispatch and per-variant path segments, not just
/// the constructor-shape logic already validated on structs.
#[derive(Code)]
enum Shape {
    Empty,
    Circle(f64),
    Rectangle { width: f64, height: f64 },
}

#[test]
fn derived_code_on_a_unit_variant_emits_a_qualified_path() -> Result<(), syn::Error> {
    let fragment = Shape::Empty.code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert_eq!(rendered, "Shape :: Empty");
    Ok(())
}

#[test]
fn derived_code_on_a_tuple_variant_emits_qualified_construction() -> Result<(), syn::Error> {
    let fragment = Shape::Circle(1.5).code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("Shape"), "rendered: {rendered}");
    assert!(rendered.contains("Circle"), "rendered: {rendered}");
    assert!(rendered.contains("1.5f64"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn derived_code_on_a_named_variant_emits_qualified_struct_construction() -> Result<(), syn::Error> {
    let fragment = Shape::Rectangle {
        width: 2.0,
        height: 3.0,
    }
    .code();
    let tokens = fragment.to_token_stream();
    let reparsed: syn::Expr = syn::parse2(tokens.clone())?;
    assert_eq!(fragment.expr(), &reparsed);

    let rendered = tokens.to_string();
    assert!(rendered.contains("Shape"), "rendered: {rendered}");
    assert!(rendered.contains("Rectangle"), "rendered: {rendered}");
    assert!(rendered.contains("width"), "rendered: {rendered}");
    assert!(rendered.contains("2.0f64"), "rendered: {rendered}");
    assert!(rendered.contains("height"), "rendered: {rendered}");
    assert!(rendered.contains("3.0f64"), "rendered: {rendered}");
    Ok(())
}

#[test]
fn derived_code_on_an_enum_reflects_actual_values_not_defaults() {
    let a = Shape::Circle(1.0);
    let b = Shape::Circle(2.0);
    assert_ne!(a.code().expr(), b.code().expr());

    // Different variants must never accidentally emit the same code.
    assert_ne!(Shape::Empty.code().expr(), Shape::Circle(0.0).code().expr());
}
