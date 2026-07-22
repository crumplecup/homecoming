//! Derive macro generating `Code` impls.
//!
//! `#[derive(Code)]` implements `Code` for a struct as long as every field
//! also implements `Code<Fragment = Ir>` — the same struct-literal pattern
//! hand-written throughout this crate's own examples (`Calculator`,
//! `Stoplight`), extracted here once that pattern had been validated, not
//! designed in advance of it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod code;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use code::expand_code;

/// Implements `Code` for a struct with named fields, so long as every
/// field's own type already implements `Code<Fragment = Ir>`. Emits a
/// struct-literal expression built from each field's own captured code —
/// see `homecoming_core::struct_expr`.
#[proc_macro_derive(Code)]
pub fn derive_code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_code(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
