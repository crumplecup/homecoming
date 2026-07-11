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

macro_rules! impl_code_integer {
    ($ty:ty) => {
        impl Code for $ty {
            type Fragment = Ir;

            fn code(&self) -> Ir {
                let lit = syn::LitInt::new(&self.to_string(), proc_macro2::Span::call_site());
                Ir::leaf(lit_expr(syn::Lit::Int(lit)))
            }
        }
    };
}

impl_code_integer!(i8);
impl_code_integer!(i16);
impl_code_integer!(i32);
impl_code_integer!(i64);
impl_code_integer!(i128);
impl_code_integer!(isize);
impl_code_integer!(u8);
impl_code_integer!(u16);
impl_code_integer!(u32);
impl_code_integer!(u64);
impl_code_integer!(u128);
impl_code_integer!(usize);

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
