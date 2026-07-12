use std::sync::Arc;

use crate::source::FormulaStableId;
use crate::syntax::green::{GreenNode, GreenTreeRoot, SyntaxKind};
use crate::syntax::token::TextSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedNode {
    pub index: usize,
    pub kind: SyntaxKind,
    pub span: TextSpan,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedProjection {
    pub formula_stable_id: FormulaStableId,
    pub green_tree_key: String,
    pub nodes: Vec<RedNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalRedProjectionResult {
    pub red_projection: RedProjection,
    pub reused_red_projection: bool,
}

impl RedProjection {
    pub fn root(&self) -> &RedNode {
        &self.nodes[0]
    }
}

pub fn project_red_view(
    formula_stable_id: FormulaStableId,
    green_tree: &GreenTreeRoot,
) -> RedProjection {
    let mut nodes = Vec::new();
    flatten(&green_tree.root, None, &mut nodes);
    RedProjection {
        formula_stable_id,
        green_tree_key: green_tree.green_tree_key.clone(),
        nodes,
    }
}

pub fn project_red_view_incremental(
    formula_stable_id: FormulaStableId,
    green_tree: &GreenTreeRoot,
    previous_red_projection: Option<&RedProjection>,
) -> IncrementalRedProjectionResult {
    if let Some(previous_red_projection) = previous_red_projection
        && previous_red_projection.formula_stable_id == formula_stable_id
        && previous_red_projection.green_tree_key == green_tree.green_tree_key
    {
        return IncrementalRedProjectionResult {
            red_projection: previous_red_projection.clone(),
            reused_red_projection: true,
        };
    }

    IncrementalRedProjectionResult {
        red_projection: project_red_view(formula_stable_id, green_tree),
        reused_red_projection: false,
    }
}

/// `Arc`-returning sibling of [`project_red_view_incremental`]. Shares the exact
/// reuse gate; on a hit it clones the `Arc` handle rather than deep-cloning the
/// red projection, on a miss it wraps a fresh projection.
pub fn project_red_view_incremental_arc(
    formula_stable_id: FormulaStableId,
    green_tree: &GreenTreeRoot,
    previous_red_projection: Option<&Arc<RedProjection>>,
) -> (Arc<RedProjection>, bool) {
    if let Some(previous_red_projection) = previous_red_projection
        && previous_red_projection.formula_stable_id == formula_stable_id
        && previous_red_projection.green_tree_key == green_tree.green_tree_key
    {
        return (Arc::clone(previous_red_projection), true);
    }

    (
        Arc::new(project_red_view(formula_stable_id, green_tree)),
        false,
    )
}

fn flatten(node: &GreenNode, parent: Option<usize>, arena: &mut Vec<RedNode>) -> usize {
    let index = arena.len();
    arena.push(RedNode {
        index,
        kind: node.kind,
        span: node.span,
        parent,
        children: Vec::new(),
    });

    let mut child_indices = Vec::new();
    for child in &node.children {
        if let crate::syntax::green::GreenChild::Node(child_node) = child {
            let child_index = flatten(child_node, Some(index), arena);
            child_indices.push(child_index);
        }
    }

    arena[index].children = child_indices;
    index
}
