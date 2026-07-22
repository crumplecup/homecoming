//! `Binding`: whether a value crosses from emission-time scope into the
//! emitted program as a frozen literal, or stays open.

use crate::Fragment;

/// Decides how a value crosses from emission-time scope into the emitted
/// program — reproduced as a frozen literal, or left as a fresh, open
/// slot for the emitted program to declare itself.
///
/// The same question Rust's own block scoping answers for a variable's
/// lifetime, applied to code emission: emission time is one scope, the
/// emitted program's later runtime is a different one.
pub trait Binding<F: Fragment> {
    /// This binding's rendering.
    fn contribute(&self) -> F;
}

/// Freezes a value into a literal, reproduced exactly.
#[derive(Debug, Clone)]
pub struct Bound<F> {
    value: F,
}

impl<F> Bound<F> {
    /// Freeze `value` as this binding's rendering.
    pub fn new(value: F) -> Self {
        Self { value }
    }
}

impl<F: Fragment> Binding<F> for Bound<F> {
    fn contribute(&self) -> F {
        self.value.clone()
    }
}

/// Leaves a value open — the emitted program declares its own fresh slot
/// instead of freezing whatever value happened to be present at emission
/// time.
///
/// `placeholder` is supplied by the caller rather than built here:
/// `Fragment`'s interface carries no construction capability (deliberately
/// — see [`crate::Reference`], which needs the same thing for the same
/// reason).
#[derive(Debug, Clone)]
pub struct Free<F> {
    placeholder: F,
}

impl<F> Free<F> {
    /// Leave this slot open, rendered as `placeholder`.
    pub fn new(placeholder: F) -> Self {
        Self { placeholder }
    }
}

impl<F: Fragment> Binding<F> for Free<F> {
    fn contribute(&self) -> F {
        self.placeholder.clone()
    }
}
