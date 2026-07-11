//! `Fragment`: a captured piece of Rust source.

use petgraph::graph::{DiGraph, NodeIndex};

/// A captured piece of Rust source, represented as a small graph of `syn`
/// expressions rather than a flat token stream, so composition and
/// isolation are structural operations on the graph, not text or token
/// surgery. A leaf fragment has one node and no edges; a composite
/// fragment's edges are what [`crate::Scope::boundary`] walks.
#[derive(Debug, Clone)]
pub struct Fragment {
    graph: DiGraph<syn::Expr, ()>,
    root: NodeIndex,
}

impl Fragment {
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

impl quote::ToTokens for Fragment {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr().to_tokens(tokens);
    }
}
