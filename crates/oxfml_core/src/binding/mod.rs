mod reference;

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub use reference::{
    AddressMode, AreaRef, CellCoord, CellRef, ErrorRef, ExternalRef, NameKind, NameRef,
    NormalizedReference, ReferenceExpr, WholeColumnRef, WholeRowRef,
};

use crate::red::RedProjection;
use crate::source::{FormulaSourceRecord, FormulaToken, StructureContextVersion};
use crate::syntax::green::{GreenChild, GreenNode, GreenTreeRoot, SyntaxKind};
use crate::syntax::token::TextSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindDiagnostic {
    pub message: String,
    pub span: TextSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundExpr {
    NumberLiteral(String),
    StringLiteral(String),
    HelperParameterName(String),
    Binary {
        op: BinaryOp,
        left: Box<BoundExpr>,
        right: Box<BoundExpr>,
    },
    FunctionCall {
        function_name: String,
        args: Vec<BoundExpr>,
    },
    Invocation {
        callee: Box<BoundExpr>,
        args: Vec<BoundExpr>,
    },
    Reference(ReferenceExpr),
    ImplicitIntersection(Box<BoundExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySeed {
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedReferenceRecord {
    pub source_text: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFormula {
    pub formula_stable_id: String,
    pub green_tree_key: String,
    pub structure_context_version: String,
    pub bind_context_fingerprint: String,
    pub bind_hash: String,
    pub root: BoundExpr,
    pub normalized_references: Vec<NormalizedReference>,
    pub dependency_seeds: Vec<DependencySeed>,
    pub unresolved_references: Vec<UnresolvedReferenceRecord>,
    pub capability_requirements: Vec<String>,
    pub diagnostics: Vec<BindDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindContext {
    pub workbook_id: String,
    pub sheet_id: String,
    pub caller_row: u32,
    pub caller_col: u32,
    pub formula_token: FormulaToken,
    pub structure_context_version: StructureContextVersion,
    pub names: BTreeMap<String, NameKind>,
}

impl Default for BindContext {
    fn default() -> Self {
        Self {
            workbook_id: "book:default".to_string(),
            sheet_id: "sheet:default".to_string(),
            caller_row: 1,
            caller_col: 1,
            formula_token: FormulaToken("fixture".to_string()),
            structure_context_version: StructureContextVersion("struct:v1".to_string()),
            names: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindRequest {
    pub source: FormulaSourceRecord,
    pub green_tree: GreenTreeRoot,
    pub red_projection: RedProjection,
    pub context: BindContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindResult {
    pub bound_formula: BoundFormula,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalBindResult {
    pub bound_formula: BoundFormula,
    pub reused_bound_formula: bool,
}

pub fn bind_formula(request: BindRequest) -> BindResult {
    let bind_context_fingerprint = hash_debug(&(
        request.context.workbook_id.clone(),
        request.context.sheet_id.clone(),
        request.context.caller_row,
        request.context.caller_col,
        request.context.formula_token.0.clone(),
        request.context.structure_context_version.0.clone(),
        request.context.names.clone(),
    ));

    let mut binder = Binder {
        context: request.context,
        diagnostics: Vec::new(),
        normalized_references: Vec::new(),
        dependency_seeds: Vec::new(),
        unresolved_references: Vec::new(),
        capability_requirements: Vec::new(),
        helper_local_names: Vec::new(),
    };

    let expr_node = request
        .green_tree
        .root
        .children
        .iter()
        .find_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        })
        .expect("formula root should contain an expression node");

    let root = binder.bind_expr(expr_node);
    let bind_hash = hash_debug(&root);

    BindResult {
        bound_formula: BoundFormula {
            formula_stable_id: request.source.formula_stable_id.0,
            green_tree_key: request.green_tree.green_tree_key,
            structure_context_version: binder.context.structure_context_version.0.clone(),
            bind_context_fingerprint,
            bind_hash,
            root,
            normalized_references: binder.normalized_references,
            dependency_seeds: binder.dependency_seeds,
            unresolved_references: binder.unresolved_references,
            capability_requirements: binder.capability_requirements,
            diagnostics: binder.diagnostics,
        },
    }
}

pub fn bind_formula_incremental(
    request: BindRequest,
    previous_bound_formula: Option<&BoundFormula>,
) -> IncrementalBindResult {
    let bind_context_fingerprint = hash_debug(&(
        request.context.workbook_id.clone(),
        request.context.sheet_id.clone(),
        request.context.caller_row,
        request.context.caller_col,
        request.context.formula_token.0.clone(),
        request.context.structure_context_version.0.clone(),
        request.context.names.clone(),
    ));

    if let Some(previous_bound_formula) = previous_bound_formula {
        if previous_bound_formula.formula_stable_id == request.source.formula_stable_id.0
            && previous_bound_formula.green_tree_key == request.green_tree.green_tree_key
            && previous_bound_formula.bind_context_fingerprint == bind_context_fingerprint
        {
            return IncrementalBindResult {
                bound_formula: previous_bound_formula.clone(),
                reused_bound_formula: true,
            };
        }
    }

    let bind = bind_formula(request);
    IncrementalBindResult {
        bound_formula: bind.bound_formula,
        reused_bound_formula: false,
    }
}

struct Binder {
    context: BindContext,
    diagnostics: Vec<BindDiagnostic>,
    normalized_references: Vec<NormalizedReference>,
    dependency_seeds: Vec<DependencySeed>,
    unresolved_references: Vec<UnresolvedReferenceRecord>,
    capability_requirements: Vec<String>,
    helper_local_names: Vec<String>,
}

impl Binder {
    fn bind_expr(&mut self, node: &GreenNode) -> BoundExpr {
        match node.kind {
            SyntaxKind::FormulaRoot | SyntaxKind::GroupingExpr => self.bind_first_child_expr(node),
            SyntaxKind::NumberLiteralExpr => {
                BoundExpr::NumberLiteral(self.first_token_text(node).unwrap_or_default())
            }
            SyntaxKind::StringLiteralExpr => {
                BoundExpr::StringLiteral(self.first_token_text(node).unwrap_or_default())
            }
            SyntaxKind::IdentifierExpr | SyntaxKind::QuotedIdentifierExpr => {
                self.bind_identifier(node)
            }
            SyntaxKind::QualifiedReferenceExpr => self.bind_qualified_reference(node),
            SyntaxKind::RangeExpr => self.bind_range(node),
            SyntaxKind::UnionExpr => self.bind_union(node),
            SyntaxKind::IntersectionExpr => self.bind_intersection(node),
            SyntaxKind::PrefixExpr => {
                let child = self
                    .first_child_node(node)
                    .expect("prefix should have child");
                BoundExpr::ImplicitIntersection(Box::new(self.bind_expr(child)))
            }
            SyntaxKind::PostfixExpr => {
                let child = self
                    .first_child_node(node)
                    .expect("postfix should have child");
                match self.bind_expr(child) {
                    BoundExpr::Reference(reference) => BoundExpr::Reference(ReferenceExpr::Spill {
                        anchor: Box::new(reference),
                    }),
                    other => {
                        self.diagnostics.push(BindDiagnostic {
                            message: "spill suffix applied to non-reference expression".to_string(),
                            span: node.span,
                        });
                        other
                    }
                }
            }
            SyntaxKind::BinaryExpr => {
                let mut child_nodes = node.children.iter().filter_map(|child| match child {
                    GreenChild::Node(node) => Some(node.as_ref()),
                    GreenChild::Token(_) => None,
                });
                let left = child_nodes.next().expect("binary left");
                let right = child_nodes.next().expect("binary right");
                let op = if token_text(node, "+").is_some() {
                    BinaryOp::Add
                } else {
                    BinaryOp::Subtract
                };
                BoundExpr::Binary {
                    op,
                    left: Box::new(self.bind_expr(left)),
                    right: Box::new(self.bind_expr(right)),
                }
            }
            SyntaxKind::CallExpr => self.bind_call(node),
            SyntaxKind::InvokeExpr => self.bind_invoke(node),
            SyntaxKind::MissingExpr => {
                self.diagnostics.push(BindDiagnostic {
                    message: "missing expression cannot be bound".to_string(),
                    span: node.span,
                });
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                    error_class: "#PARSE!".to_string(),
                    source_text: String::new(),
                })))
            }
            SyntaxKind::ArgumentList => {
                self.diagnostics.push(BindDiagnostic {
                    message: "argument list is not a standalone expression".to_string(),
                    span: node.span,
                });
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                    error_class: "#ARG!".to_string(),
                    source_text: String::new(),
                })))
            }
        }
    }

    fn bind_first_child_expr(&mut self, node: &GreenNode) -> BoundExpr {
        let child = self
            .first_child_node(node)
            .expect("node should have child expression");
        self.bind_expr(child)
    }

    fn bind_identifier(&mut self, node: &GreenNode) -> BoundExpr {
        let text = self.first_token_text(node).unwrap_or_default();
        if let Some(cell_ref) = parse_cell_reference(&text, &self.context.sheet_id, &self.context) {
            let normalized = NormalizedReference::Cell(cell_ref);
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else if self.helper_local_names.iter().any(|name| name == &text) {
            let normalized = NormalizedReference::Name(NameRef {
                name: text,
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind: NameKind::HelperLocal,
                caller_context_dependent: false,
            });
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else if let Some(kind) = self.context.names.get(&text).cloned() {
            let normalized = NormalizedReference::Name(NameRef {
                name: text,
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind,
                caller_context_dependent: false,
            });
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else {
            self.unresolved_references.push(UnresolvedReferenceRecord {
                source_text: text.clone(),
                reason: "unknown identifier or name".to_string(),
            });
            self.diagnostics.push(BindDiagnostic {
                message: format!("unresolved identifier '{text}'"),
                span: node.span,
            });
            BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                error_class: "#NAME?".to_string(),
                source_text: text,
            })))
        }
    }

    fn bind_qualified_reference(&mut self, node: &GreenNode) -> BoundExpr {
        let qualifier = node
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Token(token)
                    if matches!(
                        token.kind,
                        crate::syntax::token::TokenKind::Identifier
                            | crate::syntax::token::TokenKind::QuotedIdentifier
                            | crate::syntax::token::TokenKind::BracketedQualifier
                    ) =>
                {
                    Some(token.text.clone())
                }
                _ => None,
            })
            .unwrap_or_default();
        let qualifier = parse_reference_qualifier(&qualifier);

        let target = node
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Node(node) => Some(node.as_ref()),
                GreenChild::Token(_) => None,
            })
            .expect("qualified reference should contain target node");

        match target.kind {
            SyntaxKind::IdentifierExpr | SyntaxKind::QuotedIdentifierExpr => {
                let text = self.first_token_text(target).unwrap_or_default();
                if qualifier.is_external {
                    let normalized = NormalizedReference::External(ExternalRef {
                        external_target_id: qualifier
                            .external_target_id
                            .clone()
                            .unwrap_or_else(|| qualifier.raw.clone()),
                        sheet_selector_summary: qualifier.sheet_id.clone(),
                        capability_requirement: "external_reference".to_string(),
                        external_reference_class: "workbook_sheet_qualified".to_string(),
                        target_summary: format!("{}!{text}", qualifier.raw),
                    });
                    self.capability_requirements
                        .push("external_reference".to_string());
                    self.push_reference_seed(&normalized);
                    BoundExpr::Reference(ReferenceExpr::Atom(normalized))
                } else if let Some(cell_ref) =
                    parse_cell_reference(&text, &qualifier.sheet_id, &self.context)
                {
                    let normalized = NormalizedReference::Cell(cell_ref);
                    self.push_reference_seed(&normalized);
                    BoundExpr::Reference(ReferenceExpr::Atom(normalized))
                } else {
                    let normalized = NormalizedReference::Name(NameRef {
                        name: format!("{}!{text}", qualifier.sheet_id),
                        workbook_id: self.context.workbook_id.clone(),
                        sheet_id: qualifier.sheet_id.clone(),
                        kind: NameKind::ReferenceLike,
                        caller_context_dependent: false,
                    });
                    self.push_reference_seed(&normalized);
                    BoundExpr::Reference(ReferenceExpr::Atom(normalized))
                }
            }
            _ => {
                self.diagnostics.push(BindDiagnostic {
                    message: "qualified reference target did not bind as identifier".to_string(),
                    span: node.span,
                });
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                    error_class: "#REF!".to_string(),
                    source_text: qualifier.raw,
                })))
            }
        }
    }

    fn bind_range(&mut self, node: &GreenNode) -> BoundExpr {
        let mut child_nodes = node.children.iter().filter_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        });
        let left_node = child_nodes.next().expect("range left");
        let right_node = child_nodes.next().expect("range right");

        if let Some(normalized) = self.try_bind_whole_row_or_column_range(left_node, right_node) {
            self.push_reference_seed(&normalized);
            return BoundExpr::Reference(ReferenceExpr::Atom(normalized));
        }

        let left = self.bind_expr(left_node);
        let right = self.bind_expr(right_node);

        match (left, right) {
            (
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Cell(start))),
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Cell(end))),
            ) if start.workbook_id == end.workbook_id && start.sheet_id == end.sheet_id => {
                self.pop_recent_reference_seed();
                self.pop_recent_reference_seed();
                let top_row = start.coord.row.min(end.coord.row);
                let left_col = start.coord.col.min(end.coord.col);
                let bottom_row = start.coord.row.max(end.coord.row);
                let right_col = start.coord.col.max(end.coord.col);
                let area = NormalizedReference::Area(AreaRef {
                    workbook_id: start.workbook_id.clone(),
                    sheet_id: start.sheet_id.clone(),
                    top_left: CellCoord {
                        row: top_row,
                        col: left_col,
                    },
                    height: bottom_row - top_row + 1,
                    width: right_col - left_col + 1,
                    address_mode: AddressMode::default(),
                    caller_anchor_used: start.caller_anchor_used || end.caller_anchor_used,
                });
                self.push_reference_seed(&area);
                BoundExpr::Reference(ReferenceExpr::Atom(area))
            }
            (BoundExpr::Reference(start), BoundExpr::Reference(end)) => {
                BoundExpr::Reference(ReferenceExpr::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                })
            }
            _ => {
                self.diagnostics.push(BindDiagnostic {
                    message: "range operands did not both bind as references".to_string(),
                    span: node.span,
                });
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                    error_class: "#REF!".to_string(),
                    source_text: "range".to_string(),
                })))
            }
        }
    }

    fn try_bind_whole_row_or_column_range(
        &mut self,
        left_node: &GreenNode,
        right_node: &GreenNode,
    ) -> Option<NormalizedReference> {
        let left_simple = try_parse_simple_reference_fragment(left_node, &self.context)?;
        let right_simple = try_parse_simple_reference_fragment(right_node, &self.context)?;
        if left_simple.qualifier.raw != right_simple.qualifier.raw
            || left_simple.qualifier.is_external
            || right_simple.qualifier.is_external
        {
            return None;
        }

        if let (Some(start_row), Some(end_row)) = (
            parse_row_reference(&left_simple.target_text),
            parse_row_reference(&right_simple.target_text),
        ) {
            let top_row = start_row.min(end_row);
            let bottom_row = start_row.max(end_row);
            return Some(NormalizedReference::WholeRow(WholeRowRef {
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: left_simple.qualifier.sheet_id,
                row_start: top_row,
                row_count: bottom_row - top_row + 1,
                address_mode: AddressMode::default(),
            }));
        }

        if let (Some(start_col), Some(end_col)) = (
            parse_column_reference(&left_simple.target_text),
            parse_column_reference(&right_simple.target_text),
        ) {
            let left_col = start_col.min(end_col);
            let right_col = start_col.max(end_col);
            return Some(NormalizedReference::WholeColumn(WholeColumnRef {
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: left_simple.qualifier.sheet_id,
                col_start: left_col,
                col_count: right_col - left_col + 1,
                address_mode: AddressMode::default(),
            }));
        }

        None
    }

    fn bind_union(&mut self, node: &GreenNode) -> BoundExpr {
        let (left, right) = self.bind_reference_pair(node, "union");
        match (left, right) {
            (Some(left), Some(right)) => BoundExpr::Reference(ReferenceExpr::Union {
                left: Box::new(left),
                right: Box::new(right),
            }),
            _ => BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                error_class: "#REF!".to_string(),
                source_text: "union".to_string(),
            }))),
        }
    }

    fn bind_intersection(&mut self, node: &GreenNode) -> BoundExpr {
        let (left, right) = self.bind_reference_pair(node, "intersection");
        match (left, right) {
            (Some(left), Some(right)) => BoundExpr::Reference(ReferenceExpr::Intersection {
                left: Box::new(left),
                right: Box::new(right),
            }),
            _ => BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                error_class: "#NULL!".to_string(),
                source_text: "intersection".to_string(),
            }))),
        }
    }

    fn bind_call(&mut self, node: &GreenNode) -> BoundExpr {
        let function_name = node
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Token(token) => Some(token.text.clone()),
                GreenChild::Node(_) => None,
            })
            .unwrap_or_default();
        let uppercase_function_name = function_name.to_ascii_uppercase();

        let arg_nodes = node
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Node(arg_list) if arg_list.kind == SyntaxKind::ArgumentList => Some(
                    arg_list
                        .children
                        .iter()
                        .filter_map(|grandchild| match grandchild {
                            GreenChild::Node(expr) => Some(expr.as_ref()),
                            GreenChild::Token(_) => None,
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .unwrap_or_default();

        let args = match uppercase_function_name.as_str() {
            "LET" => self.bind_let_args(&arg_nodes),
            "LAMBDA" => self.bind_lambda_args(&arg_nodes),
            _ => arg_nodes
                .into_iter()
                .map(|expr| self.bind_expr(expr))
                .collect::<Vec<_>>(),
        };

        if self
            .helper_local_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&function_name))
        {
            let callee = self.bind_identifier_expr_from_name(&function_name);
            return BoundExpr::Invocation {
                callee: Box::new(callee),
                args,
            };
        }

        BoundExpr::FunctionCall {
            function_name: uppercase_function_name,
            args,
        }
    }

    fn bind_invoke(&mut self, node: &GreenNode) -> BoundExpr {
        let mut child_nodes = node.children.iter().filter_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        });
        let callee_node = child_nodes.next().expect("invoke callee");
        let args_node = child_nodes.next().expect("invoke arg list");
        let callee = self.bind_expr(callee_node);
        let args = args_node
            .children
            .iter()
            .filter_map(|child| match child {
                GreenChild::Node(expr) => Some(self.bind_expr(expr.as_ref())),
                GreenChild::Token(_) => None,
            })
            .collect::<Vec<_>>();
        BoundExpr::Invocation {
            callee: Box::new(callee),
            args,
        }
    }

    fn first_child_node<'a>(&self, node: &'a GreenNode) -> Option<&'a GreenNode> {
        node.children.iter().find_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        })
    }

    fn first_token_text(&self, node: &GreenNode) -> Option<String> {
        node.children.iter().find_map(|child| match child {
            GreenChild::Token(token) => Some(token.text.clone()),
            GreenChild::Node(_) => None,
        })
    }

    fn push_reference_seed(&mut self, normalized: &NormalizedReference) {
        self.normalized_references.push(normalized.clone());
        self.dependency_seeds.push(DependencySeed {
            summary: normalized.to_string(),
        });
    }

    fn pop_recent_reference_seed(&mut self) {
        self.normalized_references.pop();
        self.dependency_seeds.pop();
    }

    fn bind_reference_pair(
        &mut self,
        node: &GreenNode,
        label: &str,
    ) -> (Option<ReferenceExpr>, Option<ReferenceExpr>) {
        let mut child_nodes = node.children.iter().filter_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        });
        let left = self.bind_expr(child_nodes.next().expect("left reference expr"));
        let right = self.bind_expr(child_nodes.next().expect("right reference expr"));

        let left = match left {
            BoundExpr::Reference(reference) => Some(reference),
            _ => {
                self.diagnostics.push(BindDiagnostic {
                    message: format!("{label} left operand did not bind as reference"),
                    span: node.span,
                });
                None
            }
        };

        let right = match right {
            BoundExpr::Reference(reference) => Some(reference),
            _ => {
                self.diagnostics.push(BindDiagnostic {
                    message: format!("{label} right operand did not bind as reference"),
                    span: node.span,
                });
                None
            }
        };

        (left, right)
    }

    fn bind_let_args(&mut self, arg_nodes: &[&GreenNode]) -> Vec<BoundExpr> {
        let mut bound_args = Vec::with_capacity(arg_nodes.len());
        let mut pushed_names = 0usize;
        let last_index = arg_nodes.len().saturating_sub(1);

        for (index, arg_node) in arg_nodes.iter().enumerate() {
            let is_binding_name_position = index < last_index && index % 2 == 0;
            if is_binding_name_position {
                if let Some(name) = self.try_helper_parameter_name(arg_node) {
                    self.helper_local_names.push(name.clone());
                    pushed_names += 1;
                    bound_args.push(BoundExpr::HelperParameterName(name));
                } else {
                    self.diagnostics.push(BindDiagnostic {
                        message: "LET binding name did not bind as helper parameter".to_string(),
                        span: arg_node.span,
                    });
                    bound_args.push(self.bind_expr(arg_node));
                }
            } else {
                bound_args.push(self.bind_expr(arg_node));
            }
        }

        for _ in 0..pushed_names {
            self.helper_local_names.pop();
        }

        bound_args
    }

    fn bind_lambda_args(&mut self, arg_nodes: &[&GreenNode]) -> Vec<BoundExpr> {
        let mut bound_args = Vec::with_capacity(arg_nodes.len());
        let mut pushed_names = 0usize;
        let body_index = arg_nodes.len().saturating_sub(1);

        for (index, arg_node) in arg_nodes.iter().enumerate() {
            if index < body_index {
                if let Some(name) = self.try_helper_parameter_name(arg_node) {
                    self.helper_local_names.push(name.clone());
                    pushed_names += 1;
                    bound_args.push(BoundExpr::HelperParameterName(name));
                } else {
                    self.diagnostics.push(BindDiagnostic {
                        message: "LAMBDA parameter did not bind as helper parameter".to_string(),
                        span: arg_node.span,
                    });
                    bound_args.push(self.bind_expr(arg_node));
                }
            } else {
                bound_args.push(self.bind_expr(arg_node));
            }
        }

        for _ in 0..pushed_names {
            self.helper_local_names.pop();
        }

        bound_args
    }

    fn try_helper_parameter_name(&self, node: &GreenNode) -> Option<String> {
        if node.kind == SyntaxKind::IdentifierExpr {
            self.first_token_text(node)
        } else {
            None
        }
    }

    fn bind_identifier_expr_from_name(&mut self, text: &str) -> BoundExpr {
        if let Some(cell_ref) = parse_cell_reference(text, &self.context.sheet_id, &self.context) {
            let normalized = NormalizedReference::Cell(cell_ref);
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else if self.helper_local_names.iter().any(|name| name == text) {
            let normalized = NormalizedReference::Name(NameRef {
                name: text.to_string(),
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind: NameKind::HelperLocal,
                caller_context_dependent: false,
            });
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else if let Some(kind) = self.context.names.get(text).cloned() {
            let normalized = NormalizedReference::Name(NameRef {
                name: text.to_string(),
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind,
                caller_context_dependent: false,
            });
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else {
            self.unresolved_references.push(UnresolvedReferenceRecord {
                source_text: text.to_string(),
                reason: "unknown identifier or name".to_string(),
            });
            self.diagnostics.push(BindDiagnostic {
                message: format!("unresolved identifier '{text}'"),
                span: TextSpan::new(0, 0),
            });
            BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                error_class: "#NAME?".to_string(),
                source_text: text.to_string(),
            })))
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedQualifier {
    raw: String,
    sheet_id: String,
    external_target_id: Option<String>,
    is_external: bool,
}

#[derive(Debug, Clone)]
struct SimpleReferenceFragment {
    qualifier: ParsedQualifier,
    target_text: String,
}

fn token_text<'a>(node: &'a GreenNode, expected: &str) -> Option<&'a str> {
    node.children.iter().find_map(|child| match child {
        GreenChild::Token(token) if token.text == expected => Some(token.text.as_str()),
        _ => None,
    })
}

fn try_parse_simple_reference_fragment(
    node: &GreenNode,
    context: &BindContext,
) -> Option<SimpleReferenceFragment> {
    match node.kind {
        SyntaxKind::IdentifierExpr
        | SyntaxKind::QuotedIdentifierExpr
        | SyntaxKind::NumberLiteralExpr => Some(SimpleReferenceFragment {
            qualifier: ParsedQualifier {
                raw: context.sheet_id.clone(),
                sheet_id: context.sheet_id.clone(),
                external_target_id: None,
                is_external: false,
            },
            target_text: first_token_text_free(node)?,
        }),
        SyntaxKind::QualifiedReferenceExpr => {
            let qualifier = node.children.iter().find_map(|child| match child {
                GreenChild::Token(token) => Some(parse_reference_qualifier(&token.text)),
                GreenChild::Node(_) => None,
            })?;
            let target = node.children.iter().find_map(|child| match child {
                GreenChild::Node(node) => Some(node.as_ref()),
                GreenChild::Token(_) => None,
            })?;
            match target.kind {
                SyntaxKind::IdentifierExpr
                | SyntaxKind::QuotedIdentifierExpr
                | SyntaxKind::NumberLiteralExpr => Some(SimpleReferenceFragment {
                    qualifier,
                    target_text: first_token_text_free(target)?,
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

fn first_token_text_free(node: &GreenNode) -> Option<String> {
    node.children.iter().find_map(|child| match child {
        GreenChild::Token(token) => Some(token.text.clone()),
        GreenChild::Node(_) => None,
    })
}

fn parse_reference_qualifier(text: &str) -> ParsedQualifier {
    if let Some(rest) = text.strip_prefix('[') {
        if let Some(close_index) = rest.find(']') {
            let external_target_id = rest[..close_index].to_string();
            let sheet_id = rest[close_index + 1..].to_string();
            return ParsedQualifier {
                raw: text.to_string(),
                sheet_id: if sheet_id.is_empty() {
                    "sheet:external".to_string()
                } else {
                    sheet_id
                },
                external_target_id: Some(external_target_id),
                is_external: true,
            };
        }
    }

    let sheet_id = if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
        text[1..text.len() - 1].replace("''", "'")
    } else {
        text.to_string()
    };

    ParsedQualifier {
        raw: text.to_string(),
        sheet_id,
        external_target_id: None,
        is_external: false,
    }
}

fn parse_cell_reference(text: &str, sheet_id: &str, context: &BindContext) -> Option<CellRef> {
    let mut chars = text.chars().peekable();
    let mut col_text = String::new();
    while matches!(chars.peek(), Some(c) if c.is_ascii_alphabetic() || *c == '$') {
        let ch = chars.next().unwrap();
        if ch != '$' {
            col_text.push(ch);
        }
    }

    let mut row_text = String::new();
    while matches!(chars.peek(), Some(c) if c.is_ascii_digit() || *c == '$') {
        let ch = chars.next().unwrap();
        if ch != '$' {
            row_text.push(ch);
        }
    }

    if col_text.is_empty() || row_text.is_empty() || chars.next().is_some() {
        return None;
    }

    let col = column_to_index(&col_text)?;
    let row = row_text.parse::<u32>().ok()?;

    Some(CellRef {
        workbook_id: context.workbook_id.clone(),
        sheet_id: sheet_id.to_string(),
        coord: CellCoord { row, col },
        address_mode: AddressMode::default(),
        caller_anchor_used: true,
    })
}

fn parse_row_reference(text: &str) -> Option<u32> {
    if text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    text.parse::<u32>().ok()
}

fn parse_column_reference(text: &str) -> Option<u32> {
    if text.is_empty() || text.len() > 3 || !text.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    column_to_index(text)
}

fn column_to_index(text: &str) -> Option<u32> {
    let mut result = 0u32;
    for ch in text.chars() {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_alphabetic() {
            return None;
        }
        result = result
            .checked_mul(26)?
            .checked_add((upper as u32) - ('A' as u32) + 1)?;
    }
    Some(result)
}

fn hash_debug<T: std::fmt::Debug>(value: &T) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{value:?}").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
