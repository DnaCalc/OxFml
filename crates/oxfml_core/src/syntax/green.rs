use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::syntax::token::{SyntaxDiagnostic, TextSpan, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    FormulaRoot,
    NumberLiteralExpr,
    StringLiteralExpr,
    IdentifierExpr,
    CallExpr,
    InvokeExpr,
    ArgumentList,
    BinaryExpr,
    PrefixExpr,
    PostfixExpr,
    GroupingExpr,
    RangeExpr,
    IntersectionExpr,
    UnionExpr,
    MissingExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GreenChild {
    Node(Box<GreenNode>),
    Token(Token),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GreenNode {
    pub kind: SyntaxKind,
    pub span: TextSpan,
    pub children: Vec<GreenChild>,
}

impl GreenNode {
    pub fn new(kind: SyntaxKind, children: Vec<GreenChild>) -> Self {
        let span = span_for_children(&children);
        Self {
            kind,
            span,
            children,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GreenTreeRoot {
    pub green_tree_key: String,
    pub green_tree_fingerprint: u64,
    pub root: GreenNode,
    pub full_fidelity_tokens: Vec<Token>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

impl GreenTreeRoot {
    pub fn from_parts(
        root: GreenNode,
        full_fidelity_tokens: Vec<Token>,
        diagnostics: Vec<SyntaxDiagnostic>,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        for token in &full_fidelity_tokens {
            token.kind.hash(&mut hasher);
            token.text.hash(&mut hasher);
            token.span.start.hash(&mut hasher);
            token.span.len.hash(&mut hasher);
        }
        root.kind.hash(&mut hasher);
        let fingerprint = hasher.finish();
        Self {
            green_tree_key: format!("green:{fingerprint:016x}"),
            green_tree_fingerprint: fingerprint,
            root,
            full_fidelity_tokens,
            diagnostics,
        }
    }
}

fn span_for_children(children: &[GreenChild]) -> TextSpan {
    let mut first = None;
    let mut last = None;

    for child in children {
        let span = match child {
            GreenChild::Node(node) => node.span,
            GreenChild::Token(token) => token.span,
        };
        if first.is_none() {
            first = Some(span);
        }
        last = Some(span);
    }

    match (first, last) {
        (Some(start), Some(end)) => TextSpan::covering(start, end),
        _ => TextSpan::new(0, 0),
    }
}
