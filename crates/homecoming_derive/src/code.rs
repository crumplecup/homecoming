//! `#[derive(Code)]` expansion: a `Code` impl built from each field's own
//! captured code, in whatever constructor shape the type actually has.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields};

/// A struct's or enum variant's fields, reduced to what `code()` needs to
/// reconstruct the same constructor shape: nothing, positional
/// expressions, or named expressions. `syn::Fields` is the same type for
/// a struct's own body and a variant's body, so this reduction serves
/// both.
enum FieldEntries {
    /// `Name` — a unit struct/variant, no data to capture.
    Unit,
    /// `Name(a, b, ...)` — positional fields.
    Tuple(Vec<TokenStream>),
    /// `Name { a: .., b: .., ... }` — named fields, each paired with its
    /// own name for `struct_expr`.
    Named(Vec<(String, TokenStream)>),
}

/// Collect a struct's own fields, accessed as `self.field`/`self.0` —
/// valid because a struct's `code()` body always has `self` in scope
/// directly, unlike an enum variant's body, which only has whatever a
/// match arm's pattern bound.
fn collect_struct_fields(fields: &Fields) -> syn::Result<FieldEntries> {
    match fields {
        Fields::Unit => Ok(FieldEntries::Unit),
        Fields::Unnamed(unnamed) => {
            let codes = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    // syn::Index, not a bare integer literal interpolated
                    // via quote! -- a raw `0` would render as the float
                    // `0.0` in some quote! contexts. Index renders the
                    // correct `self.0` tuple-field access instead.
                    let index = syn::Index::from(index);
                    quote! { ::homecoming_core::Code::code(&self.#index).expr().clone() }
                })
                .collect();
            Ok(FieldEntries::Tuple(codes))
        }
        Fields::Named(named) => {
            let entries = named
                .named
                .iter()
                .map(|field| {
                    // `Fields::Named` guarantees every field has an
                    // ident; this still answers with a proper `Err`
                    // instead of assuming it, the same discipline as
                    // everywhere else in this workspace.
                    let field_ident = field.ident.as_ref().ok_or_else(|| {
                        Error::new_spanned(field, "derive(Code) requires named fields")
                    })?;
                    let field_name = field_ident.to_string();
                    let code = quote! {
                        ::homecoming_core::Code::code(&self.#field_ident).expr().clone()
                    };
                    Ok((field_name, code))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(FieldEntries::Named(entries))
        }
    }
}

/// Build one enum variant's match-arm pattern (`Self::Name`, `Self::Name
/// (field_0, field_1, ...)`, or `Self::Name { a, b, ... }`) alongside its
/// field entries — referencing the pattern's own local bindings, not
/// `self.field`, since `self` is the whole enum, not this one variant.
/// Matching on `&Self` binds those locals as references already (Rust's
/// match ergonomics), the same shape `Code::code(&self.field)` needs, so
/// no extra `&` is added the way `collect_struct_fields` needs one.
fn variant_pattern_and_entries(
    variant_name: &syn::Ident,
    fields: &Fields,
) -> syn::Result<(TokenStream, FieldEntries)> {
    match fields {
        Fields::Unit => Ok((quote! { Self::#variant_name }, FieldEntries::Unit)),
        Fields::Unnamed(unnamed) => {
            let bindings: Vec<syn::Ident> = (0..unnamed.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect();
            let pattern = quote! { Self::#variant_name(#(#bindings),*) };
            let codes = bindings
                .iter()
                .map(|binding| quote! { ::homecoming_core::Code::code(#binding).expr().clone() })
                .collect();
            Ok((pattern, FieldEntries::Tuple(codes)))
        }
        Fields::Named(named) => {
            let idents = named
                .named
                .iter()
                .map(|field| {
                    field.ident.as_ref().ok_or_else(|| {
                        Error::new_spanned(field, "derive(Code) requires named fields")
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let pattern = quote! { Self::#variant_name { #(#idents),* } };
            let entries = idents
                .iter()
                .map(|ident| {
                    let name = ident.to_string();
                    let code = quote! { ::homecoming_core::Code::code(#ident).expr().clone() };
                    (name, code)
                })
                .collect();
            Ok((pattern, FieldEntries::Named(entries)))
        }
    }
}

/// Build the `Ir::leaf(...)` expression that reconstructs a constructor
/// call from `segments` (the full path to the struct or `[EnumName,
/// VariantName]`) and its fields' captured code.
fn constructor_expr(entries: FieldEntries, segments: &[&str]) -> TokenStream {
    match entries {
        FieldEntries::Unit => quote! {
            ::homecoming_core::Ir::leaf(::homecoming_core::path_expr(&[#(#segments),*]))
        },
        FieldEntries::Tuple(codes) => quote! {
            ::homecoming_core::Ir::leaf(::homecoming_core::call_expr(
                ::homecoming_core::path_expr(&[#(#segments),*]),
                ::std::vec![#(#codes),*],
            ))
        },
        FieldEntries::Named(entries) => {
            let names = entries.iter().map(|(name, _)| name);
            let codes = entries.iter().map(|(_, code)| code);
            quote! {
                ::homecoming_core::Ir::leaf(::homecoming_core::struct_expr(
                    &[#(#segments),*],
                    ::std::vec![#((#names, #codes)),*],
                ))
            }
        }
    }
}

/// Expand `#[derive(Code)]` for a non-generic struct or enum, whose
/// fields (if any) each implement `Code<Fragment = Ir>`.
///
/// Generic types and unions are out of scope — see `HOMECOMING_PLAN.md`
/// Phase 8: extract the derive from validated hand-written patterns, not
/// designed ahead of a real case that needs them.
pub fn expand_code(input: &DeriveInput) -> syn::Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "derive(Code) does not yet support generic types",
        ));
    }

    let name = &input.ident;
    let name_str = name.to_string();

    let body = match &input.data {
        Data::Struct(data) => {
            let entries = collect_struct_fields(&data.fields)?;
            constructor_expr(entries, &[&name_str])
        }
        Data::Enum(data) => {
            let arms = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let variant_name_str = variant_name.to_string();
                    let (pattern, entries) =
                        variant_pattern_and_entries(variant_name, &variant.fields)?;
                    let constructor = constructor_expr(entries, &[&name_str, &variant_name_str]);
                    Ok(quote! { #pattern => #constructor, })
                })
                .collect::<syn::Result<Vec<TokenStream>>>()?;

            quote! {
                match self {
                    #(#arms)*
                }
            }
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                input,
                "derive(Code) does not support unions",
            ));
        }
    };

    Ok(quote! {
        impl ::homecoming_core::Code for #name {
            type Fragment = ::homecoming_core::Ir;

            fn code(&self) -> ::homecoming_core::Ir {
                #body
            }
        }
    })
}
