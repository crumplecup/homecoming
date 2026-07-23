//! `Code`: the leaf capability, capture in place of generation.

use crate::{Fragment, Ir, array_expr, call_expr, path_expr, tuple_expr};

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

/// Build a suffixed float literal expression from a non-negative
/// magnitude, mirroring `magnitude_expr`'s integer counterpart.
///
/// `{magnitude:?}` (`Debug`), not `{magnitude}` (`Display`): `Display`
/// for a whole-number float like `0.0` renders as `"0"`, with no decimal
/// point — and `"0f64"` is genuinely ambiguous, constructible as
/// `Lit::Float` directly via `LitFloat::new`, but re-lexed as `Lit::Int`
/// by the real tokenizer `syn::parse2` uses, a mismatch the round-trip
/// check caught immediately. `Debug` always includes a decimal point
/// (`"0.0"`), which is unambiguously a float token regardless of suffix.
fn float_magnitude_expr(magnitude: impl std::fmt::Debug, suffix: &str) -> syn::Expr {
    let repr = format!("{magnitude:?}{suffix}");
    let lit = syn::LitFloat::new(&repr, proc_macro2::Span::call_site());
    lit_expr(syn::Lit::Float(lit))
}

macro_rules! impl_code_float {
    ($ty:ty, $suffix:literal) => {
        impl Code for $ty {
            type Fragment = Ir;

            fn code(&self) -> Ir {
                // NaN and the infinities have no literal token at all --
                // `f64::NAN`/`f64::INFINITY`/`f64::NEG_INFINITY` are
                // associated constants, referenced by path, not written
                // as a number. `is_sign_negative()`, not `< 0.0`,
                // distinguishes `-0.0` from `0.0` -- a real, meaningful
                // bit-pattern difference in IEEE-754 this capture stays
                // faithful to rather than rounding away.
                let expr = if self.is_nan() {
                    path_expr(&[stringify!($ty), "NAN"])
                } else if self.is_infinite() {
                    if self.is_sign_positive() {
                        path_expr(&[stringify!($ty), "INFINITY"])
                    } else {
                        path_expr(&[stringify!($ty), "NEG_INFINITY"])
                    }
                } else {
                    let magnitude = float_magnitude_expr(self.abs(), $suffix);
                    if self.is_sign_negative() {
                        negate_expr(magnitude)
                    } else {
                        magnitude
                    }
                };
                Ir::leaf(expr)
            }
        }
    };
}

impl_code_float!(f32, "f32");
impl_code_float!(f64, "f64");

impl<T> Code for Option<T>
where
    T: Code<Fragment = Ir>,
{
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let expr = match self {
            None => path_expr(&["None"]),
            Some(value) => call_expr(path_expr(&["Some"]), vec![value.code().expr().clone()]),
        };
        Ir::leaf(expr)
    }
}

impl<T, const N: usize> Code for [T; N]
where
    T: Code<Fragment = Ir>,
{
    type Fragment = Ir;

    fn code(&self) -> Ir {
        let elems = self.iter().map(|item| item.code().expr().clone()).collect();
        Ir::leaf(array_expr(elems))
    }
}

impl<T> Code for Vec<T>
where
    T: Code<Fragment = Ir>,
{
    type Fragment = Ir;

    fn code(&self) -> Ir {
        // `Vec::from([elems...])`, not the `vec![...]` macro: syn
        // represents a macro invocation as an opaque token stream
        // (`Expr::Macro`), not a typed AST -- reconstructing one honestly
        // would mean hand-assembling raw tokens instead of reusing
        // `call_expr`/`array_expr`. `Vec::from([...])` says the same
        // thing with fully typed nodes this crate already builds
        // everywhere else.
        let elems = self.iter().map(|item| item.code().expr().clone()).collect();
        let expr = call_expr(path_expr(&["Vec", "from"]), vec![array_expr(elems)]);
        Ir::leaf(expr)
    }
}

macro_rules! impl_code_tuple {
    ($(($member:ident, $index:tt)),+) => {
        impl<$($member),+> Code for ($($member,)+)
        where
            $($member: Code<Fragment = Ir>,)+
        {
            type Fragment = Ir;

            fn code(&self) -> Ir {
                let elems = vec![$(self.$index.code().expr().clone()),+];
                Ir::leaf(tuple_expr(elems))
            }
        }
    };
}

impl_code_tuple!((A, 0), (B, 1));
impl_code_tuple!((A, 0), (B, 1), (C, 2));
impl_code_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
