//! `Code`: the leaf capability, capture in place of generation.

use crate::{Fragment, Ir};

/// A value that can give back the exact source that produced it, in place
/// of the value itself.
///
/// `self` already *is* the value; [`Code::code`] is the alternate channel
/// for getting its source instead of using it directly. Implementors must
/// capture real source, never a plausible-looking reconstruction — see
/// `HOMECOMING_PLAN.md`'s "Capture and replay, not generation".
pub trait Code {
    /// The concrete [`Fragment`] representation this implementor's source
    /// is captured as.
    type Fragment: Fragment;

    /// The source that produced this value.
    fn code(&self) -> Self::Fragment;
}

fn lit_expr(lit: syn::Lit) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit,
    })
}

fn negate_expr(expr: syn::Expr) -> syn::Expr {
    syn::Expr::Unary(syn::ExprUnary {
        attrs: Vec::new(),
        op: syn::UnOp::Neg(Default::default()),
        expr: Box::new(expr),
    })
}

/// Build a suffixed integer literal expression from a non-negative
/// magnitude — the shared core of the signed and unsigned impls below.
/// Suffixed unconditionally: an unsuffixed literal defaults to `i32` in an
/// unconstrained context, silently changing the captured type.
fn magnitude_expr(magnitude: impl std::fmt::Display, suffix: &str) -> syn::Expr {
    let repr = format!("{magnitude}{suffix}");
    let lit = syn::LitInt::new(&repr, proc_macro2::Span::call_site());
    lit_expr(syn::Lit::Int(lit))
}

macro_rules! impl_code_signed_integer {
    ($ty:ty, $suffix:literal) => {
        impl Code for $ty {
            type Fragment = Ir;

            fn code(&self) -> Ir {
                // Rust's literal grammar has no negative-literal token:
                // "-5" always parses as unary negation of the literal 5.
                // unsigned_abs() (not abs(), which overflows on MIN) gives
                // a magnitude that always fits its unsigned counterpart.
                let magnitude = magnitude_expr(self.unsigned_abs(), $suffix);
                let expr = if *self < 0 {
                    negate_expr(magnitude)
                } else {
                    magnitude
                };
                Ir::leaf(expr)
            }
        }
    };
}

macro_rules! impl_code_unsigned_integer {
    ($ty:ty, $suffix:literal) => {
        impl Code for $ty {
            type Fragment = Ir;

            fn code(&self) -> Ir {
                Ir::leaf(magnitude_expr(*self, $suffix))
            }
        }
    };
}

impl_code_signed_integer!(i8, "i8");
impl_code_signed_integer!(i16, "i16");
impl_code_signed_integer!(i32, "i32");
impl_code_signed_integer!(i64, "i64");
impl_code_signed_integer!(i128, "i128");
impl_code_signed_integer!(isize, "isize");
impl_code_unsigned_integer!(u8, "u8");
impl_code_unsigned_integer!(u16, "u16");
impl_code_unsigned_integer!(u32, "u32");
impl_code_unsigned_integer!(u64, "u64");
impl_code_unsigned_integer!(u128, "u128");
impl_code_unsigned_integer!(usize, "usize");

impl Code for bool {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let lit = syn::LitBool::new(*self, proc_macro2::Span::call_site());
        Ir::leaf(lit_expr(syn::Lit::Bool(lit)))
    }
}

impl Code for char {
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let lit = syn::LitChar::new(*self, proc_macro2::Span::call_site());
        Ir::leaf(lit_expr(syn::Lit::Char(lit)))
    }
}
