//! `Ir`: this crate's own concrete `Fragment` implementation.

use crate::Fragment;
use petgraph::graph::{DiGraph, NodeIndex};

/// This crate's own intermediate representation for a captured piece of
/// Rust source — a small graph of `syn` expressions rather than a flat
/// token stream, so composition and isolation are structural operations
/// on the graph, not text or token surgery. A leaf fragment has one node
/// and no edges.
///
/// `Ir` is what this crate's own `Code` impls for std primitives use. It
/// is one implementation of [`Fragment`], not the definition of it —
/// callers with a different internal representation in mind can satisfy
/// `Fragment` on their own terms instead.
#[derive(Debug, Clone)]
pub struct Ir {
    graph: DiGraph<syn::Expr, ()>,
    root: NodeIndex,
}

impl Ir {
    /// Wrap a single `syn` expression as a leaf fragment with no
    /// dependencies.
    pub fn leaf(expr: syn::Expr) -> Self {
        let mut graph = DiGraph::new();
        let root = graph.add_node(expr);
        Self { graph, root }
    }

    /// The root expression this fragment renders as.
    pub fn expr(&self) -> &syn::Expr {
        &self.graph[self.root]
    }
}

impl quote::ToTokens for Ir {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr().to_tokens(tokens);
    }
}

impl Fragment for Ir {}
