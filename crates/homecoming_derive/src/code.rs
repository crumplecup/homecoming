//! `#[derive(Code)]` expansion: a struct-literal `Code` impl built from
//! each field's own captured code.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

/// Expand `#[derive(Code)]` for a non-generic struct with named fields,
/// each of which must implement `Code<Fragment = Ir>`.
///
/// Generic types and non-named-field structs (tuple structs, unit
/// structs, enums) are deliberately out of scope for this first pass —
/// see `HOMECOMING_PLAN.md` Phase 8: extract the derive from validated
/// hand-written patterns, not designed ahead of a real case that needs
/// them.
pub fn expand_code(input: &DeriveInput) -> syn::Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "derive(Code) does not yet support generic types",
        ));
    }

    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "derive(Code) only supports structs with named fields",
        ));
    };

    let Fields::Named(fields) = &data.fields else {
        return Err(Error::new_spanned(
            &data.fields,
            "derive(Code) only supports structs with named fields",
        ));
    };

    let name = &input.ident;
    let name_str = name.to_string();

    let field_entries = fields
        .named
        .iter()
        .map(|field| {
            // `Fields::Named` guarantees every field has an ident; this
            // still answers with a proper `Err` instead of assuming it,
            // the same discipline as everywhere else in this workspace.
            let field_ident = field
                .ident
                .as_ref()
                .ok_or_else(|| Error::new_spanned(field, "derive(Code) requires named fields"))?;
            let field_name = field_ident.to_string();
            Ok(quote! {
                (#field_name, ::homecoming_core::Code::code(&self.#field_ident).expr().clone())
            })
        })
        .collect::<syn::Result<Vec<TokenStream>>>()?;

    Ok(quote! {
        impl ::homecoming_core::Code for #name {
            type Fragment = ::homecoming_core::Ir;

            fn code(&self) -> ::homecoming_core::Ir {
                let fields = ::std::vec![#(#field_entries),*];
                ::homecoming_core::Ir::leaf(::homecoming_core::struct_expr(&[#name_str], fields))
            }
        }
    })
}
