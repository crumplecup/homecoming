//! `Code`: the leaf capability, capture in place of generation.

use crate::Fragment;

/// A value that can give back the exact source that produced it, in place
/// of the value itself.
///
/// `self` already *is* the value; [`Code::code`] is the alternate channel
/// for getting its source instead of using it directly. Implementors must
/// capture real source, never a plausible-looking reconstruction — see
/// `HOMECOMING_PLAN.md`'s "Capture and replay, not generation".
pub trait Code {
    /// The source that produced this value.
    fn code(&self) -> Fragment;
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
            fn code(&self) -> Fragment {
                let lit = syn::LitInt::new(&self.to_string(), proc_macro2::Span::call_site());
                Fragment::leaf(lit_expr(syn::Lit::Int(lit)))
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
    fn code(&self) -> Fragment {
        let lit = syn::LitBool::new(*self, proc_macro2::Span::call_site());
        Fragment::leaf(lit_expr(syn::Lit::Bool(lit)))
    }
}

impl Code for char {
    fn code(&self) -> Fragment {
        let lit = syn::LitChar::new(*self, proc_macro2::Span::call_site());
        Fragment::leaf(lit_expr(syn::Lit::Char(lit)))
    }
}
