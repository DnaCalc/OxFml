mod profile;
mod reference;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub use profile::{
    BindProfile, FormulaSourceIdentity, FormulaTemplateIdentity, InstantiatedReference,
    PlacedFormulaIdentity, ProfilePayload, ProfileReferenceRecord, ProfileVersion,
    ReferenceAtomBindRequest, ReferenceAtomBindResult, ReferenceBindProfile,
    ReferenceDependencyEnvelope, ReferenceFingerprintPolicy, ReferenceInstantiationPurpose,
    ReferenceInstantiationRequest, ReferenceNormalFormKey, ReferenceOperatorCapabilities,
    ReferencePolicy, ReferenceProfileFingerprint, ReferenceProfileFingerprintContext,
    ReferenceRangeBindRequest, ReferenceRangeBindResult, ReferenceRangeEndpointBindRequest,
    ReferenceRenderRequest, ReferenceRenderResult, ReferenceSourceInfo,
    ReferenceSyntaxCapabilities, ReferenceTransformKind, ReferenceTransformOutcome,
    ReferenceTransformRequest, ReferenceTransformResult, ReferenceValidity,
    RuntimeDependencyIdentity, RuntimeHostFormulaContext,
};
pub use reference::{
    AddressMode, AreaRef, CellCoord, CellRef, ErrorRef, ExternalRef, NameKind, NameRef,
    NormalizedReference, ReferenceExpr, StructuredEmptyAreaRef, StructuredRef,
    StructuredReferenceBindDiagnosticLink, StructuredReferenceBindRecord,
    StructuredReferenceSelectedRegion, StructuredReferenceSourceTokenKind, StructuredResolvedRef,
    StructuredSectionKind, StructuredSelectorKind, WholeColumnRef, WholeRowRef,
};

use crate::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
};
use crate::red::RedProjection;
use crate::semantics::lookup_function_meta;
use crate::source::{
    FormulaChannelKind, FormulaSourceRecord, FormulaToken, StructureContextVersion,
};
use crate::syntax::green::{GreenChild, GreenNode, GreenTreeRoot, SyntaxKind};
use crate::syntax::parser::HostReferenceSyntaxProfile;
use crate::syntax::token::TextSpan;
use oxfunc_core::function::{ArgPreparationProfile, FecDependencyProfile, FunctionMeta};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindDiagnostic {
    pub message: String,
    pub span: TextSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCallSourceRecord {
    pub function_name: String,
    pub callee_span: TextSpan,
    pub call_span: TextSpan,
    pub arg_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostNameBindRecord {
    pub host_name_handle: String,
    pub canonical_name: String,
    pub host_dependency_key: Option<String>,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub resolution_layer: String,
    pub binding_kind: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
    pub replay_identity_contribution: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundHostStructuralSelector {
    pub selector_handle: String,
    pub selector_family: String,
    pub base: Box<BoundExpr>,
    pub members: Vec<BoundExpr>,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub resolution_layer: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
    pub replay_identity_contribution: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundHostReferenceCollection {
    pub collection_handle: String,
    pub collection_family: String,
    pub base: Option<Box<BoundExpr>>,
    pub members: Vec<BoundExpr>,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub resolution_layer: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
    pub replay_identity_contribution: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostNameResolveRequest {
    pub source: FormulaSourceRecord,
    pub source_span: TextSpan,
    pub source_token_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostNameResolveResult {
    pub kind: NameKind,
    pub bind_record: HostNameBindRecord,
}

pub trait HostNameResolver {
    fn resolve_host_name(&self, request: &HostNameResolveRequest) -> Option<HostNameResolveResult>;

    fn resolve_host_reference_collection(
        &self,
        _request: &HostReferenceCollectionResolveRequest,
    ) -> Option<HostReferenceCollectionResolveResult> {
        None
    }

    fn resolve_host_structural_selector(
        &self,
        _request: &HostStructuralSelectorResolveRequest,
    ) -> Option<HostStructuralSelectorResolveResult> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostReferenceCollectionResolveRequest {
    pub source: FormulaSourceRecord,
    pub collection_handle: String,
    pub collection_family: String,
    pub base: Option<BoundExpr>,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub member_token_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostReferenceCollectionResolveResult {
    pub collection: BoundHostReferenceCollection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostStructuralSelectorResolveRequest {
    pub source: FormulaSourceRecord,
    pub selector_handle: String,
    pub selector_family: String,
    pub base: BoundExpr,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub member_token_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostStructuralSelectorResolveResult {
    pub selector: BoundHostStructuralSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindFunctionSurfaceKind {
    BuiltIn,
    Udf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundExpr {
    NumberLiteral(String),
    StringLiteral(String),
    LogicalLiteral(bool),
    ArrayLiteral(Vec<Vec<BoundExpr>>),
    OmittedArgument,
    HelperParameterName(String),
    HelperOptionalParameterName(String),
    Binary {
        op: BinaryOp,
        left: Box<BoundExpr>,
        right: Box<BoundExpr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<BoundExpr>,
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
    HostReference(HostNameBindRecord),
    HostStructuralSelector(BoundHostStructuralSelector),
    HostReferenceCollection(BoundHostReferenceCollection),
    ImplicitIntersection(Box<BoundExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Power,
    Multiply,
    Divide,
    Concat,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Negate,
    Percent,
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
    pub formula_source_identity: FormulaSourceIdentity,
    pub formula_template_identity: FormulaTemplateIdentity,
    pub placed_formula_identity: PlacedFormulaIdentity,
    pub runtime_dependency_identity: RuntimeDependencyIdentity,
    pub root: BoundExpr,
    pub root_expression_is_grouped: bool,
    pub normalized_references: Vec<NormalizedReference>,
    pub structured_reference_bind_records: Vec<StructuredReferenceBindRecord>,
    pub dependency_seeds: Vec<DependencySeed>,
    pub unresolved_references: Vec<UnresolvedReferenceRecord>,
    pub capability_requirements: Vec<String>,
    pub diagnostics: Vec<BindDiagnostic>,
    pub function_call_sources: Vec<FunctionCallSourceRecord>,
    pub host_name_bind_records: Vec<HostNameBindRecord>,
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
    pub host_name_bind_records: BTreeMap<String, HostNameBindRecord>,
    pub function_surfaces: BTreeMap<String, BindFunctionSurfaceKind>,
    pub name_caller_context_dependencies: BTreeMap<String, bool>,
    pub table_catalog: Vec<TableDescriptor>,
    pub enclosing_table_ref: Option<TableRef>,
    pub caller_table_region: Option<TableCallerRegion>,
    pub host_reference_syntax: HostReferenceSyntaxProfile,
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
            host_name_bind_records: BTreeMap::new(),
            function_surfaces: BTreeMap::new(),
            name_caller_context_dependencies: BTreeMap::new(),
            table_catalog: Vec::new(),
            enclosing_table_ref: None,
            caller_table_region: None,
            host_reference_syntax: HostReferenceSyntaxProfile::default(),
        }
    }
}

pub struct BindRequest<'a> {
    pub source: FormulaSourceRecord,
    pub green_tree: GreenTreeRoot,
    pub red_projection: RedProjection,
    pub context: BindContext,
    pub host_name_resolver: Option<&'a dyn HostNameResolver>,
    pub reference_bind_profile: Option<&'a dyn ReferenceBindProfile>,
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

pub fn bind_formula(request: BindRequest<'_>) -> BindResult {
    let profile_context = reference_profile_fingerprint_context(&request.context);
    let profile_fingerprint =
        reference_profile_fingerprint(request.reference_bind_profile, &profile_context);
    let fingerprint_policy = request
        .reference_bind_profile
        .map(|profile| profile.fingerprint_policy())
        .unwrap_or(ReferenceFingerprintPolicy::IncludeCallerAnchor);
    let bind_context_fingerprint = bind_context_fingerprint_for(
        &request.context,
        profile_fingerprint.as_ref(),
        fingerprint_policy,
    );
    let formula_source_identity =
        formula_source_identity_for(&request.source, request.reference_bind_profile);

    let mut binder = Binder {
        source: request.source.clone(),
        context: request.context,
        host_name_resolver: request.host_name_resolver,
        reference_bind_profile: request.reference_bind_profile,
        formula_channel_kind: request.source.formula_channel_kind,
        diagnostics: Vec::new(),
        normalized_references: Vec::new(),
        structured_reference_bind_records: Vec::new(),
        dependency_seeds: Vec::new(),
        unresolved_references: Vec::new(),
        capability_requirements: Vec::new(),
        helper_local_names: Vec::new(),
        function_call_sources: Vec::new(),
        host_name_bind_records: Vec::new(),
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

    let root_expression_is_grouped = expr_node.kind == SyntaxKind::GroupingExpr;
    let root = binder.bind_expr(expr_node);
    let bind_hash = if binder.structured_reference_bind_records.is_empty() {
        hash_debug(&(root_expression_is_grouped, &root))
    } else {
        hash_debug(&(
            root_expression_is_grouped,
            &root,
            &binder.structured_reference_bind_records,
        ))
    };
    let formula_template_identity = formula_template_identity_for(
        &request.source,
        request.reference_bind_profile,
        profile_fingerprint.as_ref(),
        &bind_hash,
        &root,
    );
    let placed_formula_identity = placed_formula_identity_for(
        &formula_template_identity,
        &binder.context,
        profile_fingerprint.as_ref(),
    );
    let runtime_dependency_identity = runtime_dependency_identity_for(&placed_formula_identity);

    BindResult {
        bound_formula: BoundFormula {
            formula_stable_id: request.source.formula_stable_id.0,
            green_tree_key: request.green_tree.green_tree_key,
            structure_context_version: binder.context.structure_context_version.0.clone(),
            bind_context_fingerprint,
            bind_hash,
            formula_source_identity,
            formula_template_identity,
            placed_formula_identity,
            runtime_dependency_identity,
            root,
            root_expression_is_grouped,
            normalized_references: binder.normalized_references,
            structured_reference_bind_records: binder.structured_reference_bind_records,
            dependency_seeds: binder.dependency_seeds,
            unresolved_references: binder.unresolved_references,
            capability_requirements: binder.capability_requirements,
            diagnostics: binder.diagnostics,
            function_call_sources: binder.function_call_sources,
            host_name_bind_records: binder.host_name_bind_records,
        },
    }
}

pub fn bind_formula_incremental(
    request: BindRequest<'_>,
    previous_bound_formula: Option<&BoundFormula>,
) -> IncrementalBindResult {
    let profile_context = reference_profile_fingerprint_context(&request.context);
    let profile_fingerprint =
        reference_profile_fingerprint(request.reference_bind_profile, &profile_context);
    let fingerprint_policy = request
        .reference_bind_profile
        .map(|profile| profile.fingerprint_policy())
        .unwrap_or(ReferenceFingerprintPolicy::IncludeCallerAnchor);
    let bind_context_fingerprint = bind_context_fingerprint_for(
        &request.context,
        profile_fingerprint.as_ref(),
        fingerprint_policy,
    );

    if let Some(previous_bound_formula) = previous_bound_formula {
        if previous_bound_formula.formula_stable_id == request.source.formula_stable_id.0
            && previous_bound_formula.green_tree_key == request.green_tree.green_tree_key
            && previous_bound_formula.bind_context_fingerprint == bind_context_fingerprint
        {
            let mut bound_formula = previous_bound_formula.clone();
            bound_formula.placed_formula_identity = placed_formula_identity_for(
                &bound_formula.formula_template_identity,
                &request.context,
                profile_fingerprint.as_ref(),
            );
            bound_formula.runtime_dependency_identity =
                runtime_dependency_identity_for(&bound_formula.placed_formula_identity);
            return IncrementalBindResult {
                bound_formula,
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

struct Binder<'a> {
    source: FormulaSourceRecord,
    context: BindContext,
    host_name_resolver: Option<&'a dyn HostNameResolver>,
    reference_bind_profile: Option<&'a dyn ReferenceBindProfile>,
    formula_channel_kind: FormulaChannelKind,
    diagnostics: Vec<BindDiagnostic>,
    normalized_references: Vec<NormalizedReference>,
    structured_reference_bind_records: Vec<StructuredReferenceBindRecord>,
    dependency_seeds: Vec<DependencySeed>,
    unresolved_references: Vec<UnresolvedReferenceRecord>,
    capability_requirements: Vec<String>,
    helper_local_names: Vec<String>,
    function_call_sources: Vec<FunctionCallSourceRecord>,
    host_name_bind_records: Vec<HostNameBindRecord>,
}

impl Binder<'_> {
    fn bind_expr(&mut self, node: &GreenNode) -> BoundExpr {
        match node.kind {
            SyntaxKind::FormulaRoot | SyntaxKind::GroupingExpr => self.bind_first_child_expr(node),
            SyntaxKind::NumberLiteralExpr => {
                BoundExpr::NumberLiteral(self.first_token_text(node).unwrap_or_default())
            }
            SyntaxKind::StringLiteralExpr => {
                BoundExpr::StringLiteral(self.first_token_text(node).unwrap_or_default())
            }
            SyntaxKind::ArrayLiteralExpr => self.bind_array_literal(node),
            SyntaxKind::OmittedArgExpr => BoundExpr::OmittedArgument,
            SyntaxKind::IdentifierExpr | SyntaxKind::QuotedIdentifierExpr => {
                self.bind_identifier(node)
            }
            SyntaxKind::QualifiedReferenceExpr => self.bind_qualified_reference(node),
            SyntaxKind::HostMemberReferenceExpr => self.bind_host_member_reference(node),
            SyntaxKind::HostReferenceCollectionExpr => self.bind_host_reference_collection(node),
            SyntaxKind::RangeExpr => self.bind_range(node),
            SyntaxKind::UnionExpr => self.bind_union(node),
            SyntaxKind::IntersectionExpr => self.bind_intersection(node),
            SyntaxKind::PrefixExpr => {
                let child = self
                    .first_child_node(node)
                    .expect("prefix should have child");
                let bound_child = self.bind_expr(child);
                match token_text(node, "@") {
                    Some(_) => BoundExpr::ImplicitIntersection(Box::new(bound_child)),
                    None if token_text(node, "-").is_some() => BoundExpr::Unary {
                        op: UnaryOp::Negate,
                        expr: Box::new(bound_child),
                    },
                    None if token_text(node, "+").is_some() => BoundExpr::Unary {
                        op: UnaryOp::Plus,
                        expr: Box::new(bound_child),
                    },
                    None => {
                        self.diagnostics.push(BindDiagnostic {
                            message: "unsupported prefix operator".to_string(),
                            span: node.span,
                        });
                        bound_child
                    }
                }
            }
            SyntaxKind::PostfixExpr => {
                let child = self
                    .first_child_node(node)
                    .expect("postfix should have child");
                let bound_child = self.bind_expr(child);
                if token_text(node, "#").is_some() {
                    match bound_child {
                        BoundExpr::Reference(reference) => {
                            BoundExpr::Reference(ReferenceExpr::Spill {
                                anchor: Box::new(reference),
                            })
                        }
                        other => {
                            self.diagnostics.push(BindDiagnostic {
                                message: "spill suffix applied to non-reference expression"
                                    .to_string(),
                                span: node.span,
                            });
                            other
                        }
                    }
                } else if token_text(node, "%").is_some() {
                    BoundExpr::Unary {
                        op: UnaryOp::Percent,
                        expr: Box::new(bound_child),
                    }
                } else {
                    self.diagnostics.push(BindDiagnostic {
                        message: "unsupported postfix operator".to_string(),
                        span: node.span,
                    });
                    bound_child
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
                } else if token_text(node, "-").is_some() {
                    BinaryOp::Subtract
                } else if token_text(node, "^").is_some() {
                    BinaryOp::Power
                } else if token_text(node, "*").is_some() {
                    BinaryOp::Multiply
                } else if token_text(node, "&").is_some() {
                    BinaryOp::Concat
                } else if token_text(node, "<>").is_some() {
                    BinaryOp::NotEqual
                } else if token_text(node, "<=").is_some() {
                    BinaryOp::LessEqual
                } else if token_text(node, ">=").is_some() {
                    BinaryOp::GreaterEqual
                } else if token_text(node, "<").is_some() {
                    BinaryOp::LessThan
                } else if token_text(node, ">").is_some() {
                    BinaryOp::GreaterThan
                } else if token_text(node, "=").is_some() {
                    BinaryOp::Equal
                } else {
                    BinaryOp::Divide
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
        if text.eq_ignore_ascii_case("TRUE") {
            return BoundExpr::LogicalLiteral(true);
        }
        if text.eq_ignore_ascii_case("FALSE") {
            return BoundExpr::LogicalLiteral(false);
        }
        if is_worksheet_error_literal(&text) {
            return BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(
                ErrorRef {
                    error_class: text.clone(),
                    source_text: text,
                },
            )));
        }
        if let Some(host_name) = bracketed_host_name_text(&text)
            && let Some(kind) = self.context_name_kind(&host_name, node.span)
        {
            let caller_context_dependent = self.name_caller_context_dependent(&host_name);
            let normalized = NormalizedReference::Name(NameRef {
                name: host_name.clone(),
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind,
                caller_context_dependent,
            });
            self.push_reference_seed(&normalized);
            return self
                .push_host_name_bind_record(&host_name, node.span)
                .map(BoundExpr::HostReference)
                .unwrap_or_else(|| BoundExpr::Reference(ReferenceExpr::Atom(normalized)));
        }
        let sheet_id = self.context.sheet_id.clone();
        if let Some(bound) = self.try_bind_profile_reference_atom(&text, node.span, &sheet_id, None)
        {
            return bound;
        }
        if let Some(structured) = bind_structured_reference_text(
            &text,
            &text,
            &self.context,
            node.span,
            self.formula_channel_kind,
            &mut self.diagnostics,
            &mut self.unresolved_references,
            &mut self.structured_reference_bind_records,
        ) {
            self.push_reference_seed(&structured);
            BoundExpr::Reference(ReferenceExpr::Atom(structured))
        } else if self.is_helper_local_name(&text) {
            let normalized = NormalizedReference::Name(NameRef {
                name: text,
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind: NameKind::HelperLocal,
                caller_context_dependent: false,
            });
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else if let Some(cell_ref) = parse_cell_reference(
            &text,
            &self.context.sheet_id,
            &self.context,
            self.formula_channel_kind,
        ) {
            let normalized = NormalizedReference::Cell(cell_ref);
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else if let Some(kind) = self.context_name_kind(&text, node.span) {
            let caller_context_dependent = self.name_caller_context_dependent(&text);
            let normalized = NormalizedReference::Name(NameRef {
                name: text.clone(),
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind,
                caller_context_dependent,
            });
            self.push_reference_seed(&normalized);
            self.push_host_name_bind_record(&text, node.span)
                .map(BoundExpr::HostReference)
                .unwrap_or_else(|| BoundExpr::Reference(ReferenceExpr::Atom(normalized)))
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

    fn bind_host_member_reference(&mut self, node: &GreenNode) -> BoundExpr {
        let mut child_nodes = node.children.iter().filter_map(|child| match child {
            GreenChild::Node(node) => Some(node.as_ref()),
            GreenChild::Token(_) => None,
        });
        let base_node = child_nodes
            .next()
            .expect("host member reference should have base expression");
        let base = self.bind_expr(base_node);
        let member = node
            .children
            .iter()
            .rev()
            .find_map(|child| match child {
                GreenChild::Token(token)
                    if matches!(
                        token.kind,
                        crate::syntax::token::TokenKind::Identifier
                            | crate::syntax::token::TokenKind::BracketedQualifier
                            | crate::syntax::token::TokenKind::Star
                    ) =>
                {
                    Some(token.text.clone())
                }
                _ => None,
            })
            .unwrap_or_default();
        let member = bracketed_host_name_text(&member).unwrap_or(member);
        let selector_family = self.host_reference_collection_family(&member);
        let selector_handle = format!("hostref_selector_{}_{}", node.span.start, node.span.len);
        let source_token_text = node_source_text(node);
        if let Some(resolver) = self.host_name_resolver
            && let Some(result) =
                resolver.resolve_host_structural_selector(&HostStructuralSelectorResolveRequest {
                    source: self.source.clone(),
                    selector_handle: selector_handle.clone(),
                    selector_family: selector_family.clone(),
                    base: base.clone(),
                    source_span: node.span,
                    source_token_text: source_token_text.clone(),
                    member_token_text: member.clone(),
                })
        {
            self.push_reference_seed(&host_reference_handle_normalized_reference(
                &result.selector.selector_handle,
                result.selector.caller_context_dependent,
            ));
            return BoundExpr::HostStructuralSelector(result.selector);
        }
        self.push_reference_seed(&host_reference_handle_normalized_reference(
            &selector_handle,
            true,
        ));
        BoundExpr::HostStructuralSelector(BoundHostStructuralSelector {
            selector_handle,
            selector_family,
            base: Box::new(base),
            members: Vec::new(),
            source_span: node.span,
            source_token_text,
            resolution_layer: "host_member_reference_syntax".to_string(),
            shape_hint: Some("host_structural_selector".to_string()),
            caller_context_dependent: true,
            diagnostics: Vec::new(),
            replay_identity_contribution: format!(
                "host-member-selector:v1:span={}:{};member={member}",
                node.span.start, node.span.len
            ),
        })
    }

    fn bind_host_reference_collection(&mut self, node: &GreenNode) -> BoundExpr {
        let member = node
            .children
            .iter()
            .rev()
            .find_map(|child| match child {
                GreenChild::Token(token)
                    if matches!(
                        token.kind,
                        crate::syntax::token::TokenKind::Identifier
                            | crate::syntax::token::TokenKind::BracketedQualifier
                            | crate::syntax::token::TokenKind::Star
                    ) =>
                {
                    Some(token.text.clone())
                }
                _ => None,
            })
            .unwrap_or_default();
        let member = bracketed_host_name_text(&member).unwrap_or(member);
        let collection_family = self.host_reference_collection_family(&member);
        let collection_handle = format!("hostref_collection_{}_{}", node.span.start, node.span.len);
        let source_token_text = node_source_text(node);
        if let Some(resolver) = self.host_name_resolver
            && let Some(result) =
                resolver.resolve_host_reference_collection(&HostReferenceCollectionResolveRequest {
                    source: self.source.clone(),
                    collection_handle: collection_handle.clone(),
                    collection_family: collection_family.clone(),
                    base: None,
                    source_span: node.span,
                    source_token_text: source_token_text.clone(),
                    member_token_text: member.clone(),
                })
        {
            self.push_reference_seed(&host_reference_handle_normalized_reference(
                &result.collection.collection_handle,
                result.collection.caller_context_dependent,
            ));
            return BoundExpr::HostReferenceCollection(result.collection);
        }
        self.push_reference_seed(&host_reference_handle_normalized_reference(
            &collection_handle,
            true,
        ));
        BoundExpr::HostReferenceCollection(BoundHostReferenceCollection {
            collection_handle,
            collection_family,
            base: None,
            members: Vec::new(),
            source_span: node.span,
            source_token_text,
            resolution_layer: "host_reference_collection_syntax".to_string(),
            shape_hint: Some("host_reference_collection".to_string()),
            caller_context_dependent: true,
            diagnostics: Vec::new(),
            replay_identity_contribution: format!(
                "host-reference-collection:v1:span={}:{};member={member}",
                node.span.start, node.span.len
            ),
        })
    }

    fn host_reference_collection_family(&self, member: &str) -> String {
        self.context
            .host_reference_syntax
            .host_selector_family_for(member)
            .unwrap_or(member)
            .to_ascii_lowercase()
    }

    fn bind_array_literal(&mut self, node: &GreenNode) -> BoundExpr {
        let mut rows: Vec<Vec<BoundExpr>> = vec![Vec::new()];
        let mut saw_inline_lambda = false;
        for child in &node.children {
            match child {
                GreenChild::Node(expr) => {
                    let bound_expr = self.bind_expr(expr);
                    saw_inline_lambda |= array_literal_contains_inline_lambda_call(&bound_expr);
                    rows.last_mut()
                        .expect("array literal should have current row")
                        .push(bound_expr);
                }
                GreenChild::Token(token)
                    if token.kind == crate::syntax::token::TokenKind::Semicolon =>
                {
                    rows.push(Vec::new());
                }
                GreenChild::Token(_) => {}
            }
        }
        if saw_inline_lambda {
            self.diagnostics.push(BindDiagnostic {
                message: "LAMBDA cannot appear inside array constants".to_string(),
                span: node.span,
            });
        }
        BoundExpr::ArrayLiteral(rows)
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
                let source_text = node_source_text(node);
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
                } else if let Some(bound) = self.try_bind_profile_reference_atom(
                    &source_text,
                    node.span,
                    &qualifier.sheet_id,
                    Some(qualifier.raw.clone()),
                ) {
                    bound
                } else if let Some(structured) = bind_structured_reference_text_with_sheet(
                    &text,
                    &source_text,
                    &qualifier.sheet_id,
                    &self.context,
                    node.span,
                    self.formula_channel_kind,
                    &mut self.diagnostics,
                    &mut self.unresolved_references,
                    &mut self.structured_reference_bind_records,
                ) {
                    self.push_reference_seed(&structured);
                    BoundExpr::Reference(ReferenceExpr::Atom(structured))
                } else if let Some(cell_ref) = parse_cell_reference(
                    &text,
                    &qualifier.sheet_id,
                    &self.context,
                    self.formula_channel_kind,
                ) {
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

        if let Some(bound) = self.try_bind_profile_reference_range(node, left_node, right_node) {
            return bound;
        }

        if !self.profile_uses_symbolic_references() {
            if let Some(normalized) = self.try_bind_simple_reference_range(left_node, right_node) {
                self.push_reference_seed(&normalized);
                return BoundExpr::Reference(ReferenceExpr::Atom(normalized));
            }
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

    fn try_bind_simple_reference_range(
        &mut self,
        left_node: &GreenNode,
        right_node: &GreenNode,
    ) -> Option<NormalizedReference> {
        let (left_simple, right_simple) =
            harmonize_simple_reference_fragments(left_node, right_node, &self.context)?;

        if let (Some(start_row), Some(end_row)) = (
            parse_row_reference(&left_simple.target_text, self.formula_channel_kind),
            parse_row_reference(&right_simple.target_text, self.formula_channel_kind),
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
            parse_column_reference(&left_simple.target_text, self.formula_channel_kind),
            parse_column_reference(&right_simple.target_text, self.formula_channel_kind),
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

        if let (Some(start), Some(end)) = (
            parse_cell_reference(
                &left_simple.target_text,
                &left_simple.qualifier.sheet_id,
                &self.context,
                self.formula_channel_kind,
            ),
            parse_cell_reference(
                &right_simple.target_text,
                &right_simple.qualifier.sheet_id,
                &self.context,
                self.formula_channel_kind,
            ),
        ) {
            if start.workbook_id == end.workbook_id && start.sheet_id == end.sheet_id {
                let top_row = start.coord.row.min(end.coord.row);
                let left_col = start.coord.col.min(end.coord.col);
                let bottom_row = start.coord.row.max(end.coord.row);
                let right_col = start.coord.col.max(end.coord.col);
                return Some(NormalizedReference::Area(AreaRef {
                    workbook_id: start.workbook_id,
                    sheet_id: start.sheet_id,
                    top_left: CellCoord {
                        row: top_row,
                        col: left_col,
                    },
                    height: bottom_row - top_row + 1,
                    width: right_col - left_col + 1,
                    address_mode: AddressMode::default(),
                    caller_anchor_used: start.caller_anchor_used || end.caller_anchor_used,
                }));
            }
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
        let function_token = node.children.iter().find_map(|child| match child {
            GreenChild::Token(token) => Some(token),
            GreenChild::Node(_) => None,
        });
        let function_name = function_token
            .map(|token| token.text.clone())
            .unwrap_or_default();
        let callee_span = function_token.map(|token| token.span).unwrap_or(node.span);
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

        let helper_local_match = self
            .helper_local_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&function_name));
        let context_name_kind = self.context_name_kind(&function_name, callee_span);
        let builtin_function_meta = lookup_function_meta(&uppercase_function_name);
        let builtin_function_match = builtin_function_meta.is_some();
        let host_reference_name_match = context_name_kind.is_some() && !builtin_function_match;
        let _udf_function_match = !builtin_function_match
            && !host_reference_name_match
            && matches!(
                self.function_surface_kind(&uppercase_function_name),
                Some(BindFunctionSurfaceKind::Udf)
            );
        let binds_as_invocation =
            (helper_local_match && !builtin_function_match) || host_reference_name_match;

        if !binds_as_invocation
            && let Some(meta) = builtin_function_meta.as_ref()
            && let Some(message) =
                builtin_function_call_authoring_diagnostic(&uppercase_function_name, meta, &args)
        {
            self.diagnostics.push(BindDiagnostic {
                message,
                span: callee_span,
            });
        }

        if matches!(context_name_kind, Some(NameKind::ReferenceLike)) {
            self.diagnostics.push(BindDiagnostic {
                message: format!("known symbol '{function_name}' is not callable"),
                span: callee_span,
            });
        }

        if binds_as_invocation {
            let callee = self.bind_identifier_expr_from_name(&function_name, callee_span);
            return BoundExpr::Invocation {
                callee: Box::new(callee),
                args,
            };
        }

        self.function_call_sources.push(FunctionCallSourceRecord {
            function_name: uppercase_function_name.clone(),
            callee_span,
            call_span: node.span,
            arg_count: args.len(),
        });

        BoundExpr::FunctionCall {
            function_name: uppercase_function_name,
            args,
        }
    }

    fn function_surface_kind(&self, function_name: &str) -> Option<BindFunctionSurfaceKind> {
        self.context
            .function_surfaces
            .iter()
            .find_map(|(name, kind)| name.eq_ignore_ascii_case(function_name).then_some(*kind))
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

    fn profile_uses_symbolic_references(&self) -> bool {
        self.reference_bind_profile
            .map(|profile| matches!(profile.reference_policy(), ReferencePolicy::ProfileSymbolic))
            .unwrap_or(false)
    }

    fn try_bind_profile_reference_atom(
        &mut self,
        source_text: &str,
        source_span: TextSpan,
        sheet_id: &str,
        parsed_qualifier: Option<String>,
    ) -> Option<BoundExpr> {
        let profile = self.reference_bind_profile?;
        let request = ReferenceAtomBindRequest {
            source_channel: self.formula_channel_kind,
            source_span,
            source_text: source_text.to_string(),
            parsed_qualifier,
            workbook_id: self.context.workbook_id.clone(),
            sheet_id: sheet_id.to_string(),
            caller_row: self.context.caller_row,
            caller_col: self.context.caller_col,
        };
        match profile.bind_atom(&request) {
            ReferenceAtomBindResult::Bound(record) => {
                let normalized = NormalizedReference::ProfileSymbolic(record);
                self.push_reference_seed(&normalized);
                Some(BoundExpr::Reference(ReferenceExpr::Atom(normalized)))
            }
            ReferenceAtomBindResult::Rejected { validity, message } => {
                self.diagnostics.push(BindDiagnostic {
                    message: format!(
                        "reference profile '{}' rejected '{}': {message} ({validity:?})",
                        profile.profile_id(),
                        source_text
                    ),
                    span: source_span,
                });
                Some(BoundExpr::Reference(ReferenceExpr::Atom(
                    NormalizedReference::Error(ErrorRef {
                        error_class: "#REF!".to_string(),
                        source_text: source_text.to_string(),
                    }),
                )))
            }
            ReferenceAtomBindResult::Unsupported => {
                self.diagnostics.push(BindDiagnostic {
                    message: format!(
                        "reference profile '{}' does not support reference atom '{}'",
                        profile.profile_id(),
                        source_text
                    ),
                    span: source_span,
                });
                Some(BoundExpr::Reference(ReferenceExpr::Atom(
                    NormalizedReference::Error(ErrorRef {
                        error_class: "#REF!".to_string(),
                        source_text: source_text.to_string(),
                    }),
                )))
            }
            ReferenceAtomBindResult::LegacyCompatibility => None,
        }
    }

    fn try_bind_profile_reference_range(
        &mut self,
        node: &GreenNode,
        left_node: &GreenNode,
        right_node: &GreenNode,
    ) -> Option<BoundExpr> {
        let profile = self.reference_bind_profile?;
        let (left_simple, right_simple) =
            harmonize_simple_reference_fragments(left_node, right_node, &self.context)?;
        let request = ReferenceRangeBindRequest {
            source_channel: self.formula_channel_kind,
            source_span: node.span,
            source_text: node_source_text(node),
            left: reference_range_endpoint_request(left_simple),
            right: reference_range_endpoint_request(right_simple),
            workbook_id: self.context.workbook_id.clone(),
            sheet_id: self.context.sheet_id.clone(),
            caller_row: self.context.caller_row,
            caller_col: self.context.caller_col,
        };
        match profile.bind_range(&request) {
            ReferenceRangeBindResult::Bound(record) => {
                let normalized = NormalizedReference::ProfileSymbolic(record);
                self.push_reference_seed(&normalized);
                Some(BoundExpr::Reference(ReferenceExpr::Atom(normalized)))
            }
            ReferenceRangeBindResult::Rejected { validity, message } => {
                self.diagnostics.push(BindDiagnostic {
                    message: format!(
                        "reference profile '{}' rejected range '{}': {message} ({validity:?})",
                        profile.profile_id(),
                        request.source_text
                    ),
                    span: request.source_span,
                });
                Some(BoundExpr::Reference(ReferenceExpr::Atom(
                    NormalizedReference::Error(ErrorRef {
                        error_class: "#REF!".to_string(),
                        source_text: request.source_text,
                    }),
                )))
            }
            ReferenceRangeBindResult::Unsupported => {
                self.diagnostics.push(BindDiagnostic {
                    message: format!(
                        "reference profile '{}' does not support reference range '{}'",
                        profile.profile_id(),
                        request.source_text
                    ),
                    span: request.source_span,
                });
                Some(BoundExpr::Reference(ReferenceExpr::Atom(
                    NormalizedReference::Error(ErrorRef {
                        error_class: "#REF!".to_string(),
                        source_text: request.source_text,
                    }),
                )))
            }
            ReferenceRangeBindResult::LegacyCompatibility => None,
        }
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
        let mut seen_binding_names = BTreeSet::new();

        for (index, arg_node) in arg_nodes.iter().enumerate() {
            let is_binding_name_position = index < last_index && index % 2 == 0;
            if is_binding_name_position {
                if let Some(name) = self.try_helper_parameter_name(arg_node) {
                    let normalized_name = name.to_ascii_uppercase();
                    if seen_binding_names.insert(normalized_name) {
                        self.helper_local_names.push(name.clone());
                        pushed_names += 1;
                    } else {
                        self.diagnostics.push(BindDiagnostic {
                            message: format!("duplicate LET binding name '{name}'"),
                            span: arg_node.span,
                        });
                    }
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
        let mut seen_parameter_names = BTreeSet::new();
        let mut saw_optional_parameter = false;

        for (index, arg_node) in arg_nodes.iter().enumerate() {
            if index < body_index {
                if let Some((name, optional)) = self.try_lambda_parameter(arg_node) {
                    let normalized_name = name.to_ascii_uppercase();
                    if seen_parameter_names.insert(normalized_name) {
                        if !optional && saw_optional_parameter {
                            self.diagnostics.push(BindDiagnostic {
                                message: format!(
                                    "required LAMBDA parameter '{name}' cannot follow optional parameter"
                                ),
                                span: arg_node.span,
                            });
                        }
                        saw_optional_parameter |= optional;
                        self.helper_local_names.push(name.clone());
                        pushed_names += 1;
                    } else {
                        self.diagnostics.push(BindDiagnostic {
                            message: format!("duplicate LAMBDA parameter name '{name}'"),
                            span: arg_node.span,
                        });
                    }
                    bound_args.push(if optional {
                        BoundExpr::HelperOptionalParameterName(name)
                    } else {
                        BoundExpr::HelperParameterName(name)
                    });
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

    fn try_lambda_parameter(&self, node: &GreenNode) -> Option<(String, bool)> {
        if node.kind != SyntaxKind::IdentifierExpr {
            return None;
        }
        let text = self.first_token_text(node)?;
        if let Some(inner) = strip_optional_lambda_parameter_syntax(&text) {
            return Some((inner, true));
        }
        Some((text, false))
    }

    fn is_helper_local_name(&self, text: &str) -> bool {
        self.helper_local_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(text))
    }

    fn name_caller_context_dependent(&self, text: &str) -> bool {
        self.context
            .name_caller_context_dependencies
            .iter()
            .find_map(|(name, dependent)| name.eq_ignore_ascii_case(text).then_some(*dependent))
            .unwrap_or(false)
    }

    fn context_name_kind(&mut self, text: &str, source_span: TextSpan) -> Option<NameKind> {
        if let Some(kind) = self
            .context
            .names
            .iter()
            .find_map(|(name, kind)| name.eq_ignore_ascii_case(text).then_some(kind.clone()))
        {
            return Some(kind);
        }
        let resolver = self.host_name_resolver?;
        let resolved = resolver.resolve_host_name(&HostNameResolveRequest {
            source: self.source.clone(),
            source_span,
            source_token_text: text.to_string(),
        })?;
        self.context.names.insert(
            resolved.bind_record.canonical_name.clone(),
            resolved.kind.clone(),
        );
        self.context.name_caller_context_dependencies.insert(
            resolved.bind_record.canonical_name.clone(),
            resolved.bind_record.caller_context_dependent,
        );
        self.context.host_name_bind_records.insert(
            resolved.bind_record.canonical_name.clone(),
            resolved.bind_record,
        );
        self.context
            .names
            .iter()
            .find_map(|(name, kind)| name.eq_ignore_ascii_case(text).then_some(kind.clone()))
    }

    fn push_host_name_bind_record(
        &mut self,
        text: &str,
        source_span: TextSpan,
    ) -> Option<HostNameBindRecord> {
        let Some(mut record) = self
            .context
            .host_name_bind_records
            .iter()
            .find_map(|(name, record)| name.eq_ignore_ascii_case(text).then_some(record.clone()))
        else {
            return None;
        };
        record.source_span = source_span;
        record.source_token_text = text.to_string();
        self.host_name_bind_records.push(record);
        self.host_name_bind_records.last().cloned()
    }

    fn bind_identifier_expr_from_name(&mut self, text: &str, source_span: TextSpan) -> BoundExpr {
        if self.is_helper_local_name(text) {
            let normalized = NormalizedReference::Name(NameRef {
                name: text.to_string(),
                workbook_id: self.context.workbook_id.clone(),
                sheet_id: self.context.sheet_id.clone(),
                kind: NameKind::HelperLocal,
                caller_context_dependent: false,
            });
            self.push_reference_seed(&normalized);
            BoundExpr::Reference(ReferenceExpr::Atom(normalized))
        } else {
            let sheet_id = self.context.sheet_id.clone();
            if let Some(bound) =
                self.try_bind_profile_reference_atom(text, source_span, &sheet_id, None)
            {
                return bound;
            }

            if let Some(structured) = bind_structured_reference_text(
                text,
                text,
                &self.context,
                TextSpan::new(0, 0),
                self.formula_channel_kind,
                &mut self.diagnostics,
                &mut self.unresolved_references,
                &mut self.structured_reference_bind_records,
            ) {
                self.push_reference_seed(&structured);
                BoundExpr::Reference(ReferenceExpr::Atom(structured))
            } else if let Some(cell_ref) = parse_cell_reference(
                text,
                &self.context.sheet_id,
                &self.context,
                self.formula_channel_kind,
            ) {
                let normalized = NormalizedReference::Cell(cell_ref);
                self.push_reference_seed(&normalized);
                BoundExpr::Reference(ReferenceExpr::Atom(normalized))
            } else if let Some(kind) = self.context_name_kind(text, source_span) {
                let caller_context_dependent = self.name_caller_context_dependent(text);
                let normalized = NormalizedReference::Name(NameRef {
                    name: text.to_string(),
                    workbook_id: self.context.workbook_id.clone(),
                    sheet_id: self.context.sheet_id.clone(),
                    kind,
                    caller_context_dependent,
                });
                self.push_reference_seed(&normalized);
                self.push_host_name_bind_record(text, source_span)
                    .map(BoundExpr::HostReference)
                    .unwrap_or_else(|| BoundExpr::Reference(ReferenceExpr::Atom(normalized)))
            } else {
                self.unresolved_references.push(UnresolvedReferenceRecord {
                    source_text: text.to_string(),
                    reason: "unknown identifier or name".to_string(),
                });
                self.diagnostics.push(BindDiagnostic {
                    message: format!("unresolved identifier '{text}'"),
                    span: source_span,
                });
                BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Error(ErrorRef {
                    error_class: "#NAME?".to_string(),
                    source_text: text.to_string(),
                })))
            }
        }
    }
}

fn builtin_function_call_authoring_diagnostic(
    builtin_name: &str,
    meta: &FunctionMeta,
    args: &[BoundExpr],
) -> Option<String> {
    if !meta.arity.accepts(args.len()) {
        return Some(builtin_function_arity_authoring_diagnostic(
            builtin_name,
            meta.arity.min,
            meta.arity.max,
            args.len(),
        ));
    }

    if builtin_requires_reference_locator_authoring_reject(meta, args) {
        return Some(builtin_reference_locator_authoring_diagnostic(builtin_name));
    }

    None
}

fn builtin_function_arity_authoring_diagnostic(
    builtin_name: &str,
    min_arity: usize,
    max_arity: usize,
    actual_arity: usize,
) -> String {
    format!(
        "built-in function call '{builtin_name}' rejects {actual_arity} arguments at the authoring boundary (expected {min_arity}..={max_arity})"
    )
}

fn builtin_requires_reference_locator_authoring_reject(
    meta: &FunctionMeta,
    args: &[BoundExpr],
) -> bool {
    meta.fec_dependency_profile == FecDependencyProfile::CallerContext
        && meta.arg_preparation_profile == ArgPreparationProfile::RefsVisibleInAdapter
        && meta.arity.min == 0
        && meta.arity.max == 1
        && args.len() == 1
        && !builtin_reference_locator_argument_is_reference_like(&args[0])
}

fn builtin_reference_locator_authoring_diagnostic(builtin_name: &str) -> String {
    format!(
        "built-in function call '{builtin_name}' rejects non-reference arguments at the authoring boundary for caller-context locator functions"
    )
}

fn builtin_reference_locator_argument_is_reference_like(expr: &BoundExpr) -> bool {
    match expr {
        BoundExpr::Reference(reference) => match reference {
            ReferenceExpr::Atom(NormalizedReference::Cell(_))
            | ReferenceExpr::Atom(NormalizedReference::Area(_))
            | ReferenceExpr::Atom(NormalizedReference::WholeRow(_))
            | ReferenceExpr::Atom(NormalizedReference::WholeColumn(_))
            | ReferenceExpr::Atom(NormalizedReference::Structured(_))
            | ReferenceExpr::Atom(NormalizedReference::External(_))
            | ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(_)) => true,
            ReferenceExpr::Atom(NormalizedReference::Name(name)) => {
                matches!(
                    name.kind,
                    NameKind::ReferenceLike | NameKind::MixedOrDeferred
                )
            }
            ReferenceExpr::Spill { anchor } => {
                let anchor_expr = BoundExpr::Reference((**anchor).clone());
                builtin_reference_locator_argument_is_reference_like(&anchor_expr)
            }
            ReferenceExpr::Range { .. }
            | ReferenceExpr::Union { .. }
            | ReferenceExpr::Intersection { .. } => true,
            ReferenceExpr::Atom(NormalizedReference::Error(_)) => false,
        },
        BoundExpr::ImplicitIntersection(inner) => {
            builtin_reference_locator_argument_is_reference_like(inner)
        }
        _ => false,
    }
}

fn array_literal_contains_inline_lambda_call(expr: &BoundExpr) -> bool {
    match expr {
        BoundExpr::FunctionCall { function_name, .. } if function_name == "LAMBDA" => true,
        BoundExpr::FunctionCall { args, .. } => {
            args.iter().any(array_literal_contains_inline_lambda_call)
        }
        BoundExpr::Invocation { callee, args } => {
            array_literal_contains_inline_lambda_call(callee)
                || args.iter().any(array_literal_contains_inline_lambda_call)
        }
        BoundExpr::ImplicitIntersection(inner) | BoundExpr::Unary { expr: inner, .. } => {
            array_literal_contains_inline_lambda_call(inner)
        }
        BoundExpr::Binary { left, right, .. } => {
            array_literal_contains_inline_lambda_call(left)
                || array_literal_contains_inline_lambda_call(right)
        }
        BoundExpr::ArrayLiteral(rows) => rows
            .iter()
            .flatten()
            .any(array_literal_contains_inline_lambda_call),
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_)
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::Reference(_) => false,
    }
}

fn strip_optional_lambda_parameter_syntax(text: &str) -> Option<String> {
    let inner = text.strip_prefix('[')?.strip_suffix(']')?;
    if inner.is_empty() {
        return None;
    }
    let mut chars = inner.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || matches!(first, '_' | '$')) {
        return None;
    }
    if chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$')) {
        Some(inner.to_string())
    } else {
        None
    }
}

fn bracketed_host_name_text(text: &str) -> Option<String> {
    let inner = text.strip_prefix('[')?.strip_suffix(']')?;
    (!inner.is_empty()).then(|| unescape_structured_reference_text(inner))
}

fn node_source_text(node: &GreenNode) -> String {
    let mut text = String::new();
    append_node_source_text(node, &mut text);
    text
}

fn append_node_source_text(node: &GreenNode, text: &mut String) {
    for child in &node.children {
        match child {
            GreenChild::Node(node) => append_node_source_text(node, text),
            GreenChild::Token(token) => text.push_str(&token.text),
        }
    }
}

fn host_reference_handle_normalized_reference(
    handle: &str,
    caller_context_dependent: bool,
) -> NormalizedReference {
    NormalizedReference::Name(NameRef {
        name: handle.to_string(),
        workbook_id: String::new(),
        sheet_id: String::new(),
        kind: NameKind::ReferenceLike,
        caller_context_dependent,
    })
}

#[derive(Debug, Clone)]
struct ParsedQualifier {
    raw: String,
    sheet_id: String,
    external_target_id: Option<String>,
    is_external: bool,
    explicit: bool,
}

#[derive(Debug, Clone)]
struct SimpleReferenceFragment {
    qualifier: ParsedQualifier,
    source_span: TextSpan,
    source_text: String,
    target_text: String,
}

#[derive(Debug, Clone)]
struct ParsedStructuredReference {
    table_name: Option<String>,
    section_qualifiers: Vec<StructuredSectionKind>,
    column_names: Vec<String>,
    caller_row_sensitive: bool,
}

#[derive(Debug, Clone)]
struct BuiltStructuredReference {
    structured: StructuredRef,
    selected_columns: Vec<TableColumnDescriptor>,
    selected_sections: Vec<StructuredSectionKind>,
}

fn is_worksheet_error_literal(text: &str) -> bool {
    matches!(
        text.to_ascii_uppercase().as_str(),
        "#NULL!"
            | "#DIV/0!"
            | "#VALUE!"
            | "#REF!"
            | "#NAME?"
            | "#NUM!"
            | "#N/A"
            | "#BUSY!"
            | "#GETTING_DATA"
            | "#SPILL!"
            | "#CALC!"
            | "#FIELD!"
            | "#BLOCKED!"
            | "#CONNECT!"
    )
}

fn bind_structured_reference_text(
    text: &str,
    source_token_text: &str,
    context: &BindContext,
    span: TextSpan,
    formula_channel_kind: FormulaChannelKind,
    diagnostics: &mut Vec<BindDiagnostic>,
    unresolved_references: &mut Vec<UnresolvedReferenceRecord>,
    structured_reference_bind_records: &mut Vec<StructuredReferenceBindRecord>,
) -> Option<NormalizedReference> {
    bind_structured_reference_text_with_sheet(
        text,
        source_token_text,
        &context.sheet_id,
        context,
        span,
        formula_channel_kind,
        diagnostics,
        unresolved_references,
        structured_reference_bind_records,
    )
}

fn bind_structured_reference_text_with_sheet(
    text: &str,
    source_token_text: &str,
    effective_sheet_id: &str,
    context: &BindContext,
    span: TextSpan,
    formula_channel_kind: FormulaChannelKind,
    diagnostics: &mut Vec<BindDiagnostic>,
    unresolved_references: &mut Vec<UnresolvedReferenceRecord>,
    structured_reference_bind_records: &mut Vec<StructuredReferenceBindRecord>,
) -> Option<NormalizedReference> {
    let parsed = parse_structured_reference_text(text)?;
    let table = match resolve_structured_table(&parsed, effective_sheet_id, context) {
        Ok(table) => table,
        Err(message) => {
            unresolved_references.push(UnresolvedReferenceRecord {
                source_text: source_token_text.to_string(),
                reason: message.clone(),
            });
            diagnostics.push(BindDiagnostic { message, span });
            structured_reference_bind_records.push(structured_reference_error_bind_record(
                &parsed,
                source_token_text,
                span,
                None,
                diagnostics
                    .last()
                    .expect("structured reference diagnostic should have been pushed"),
            ));
            return Some(NormalizedReference::Error(ErrorRef {
                error_class: "#REF!".to_string(),
                source_text: source_token_text.to_string(),
            }));
        }
    };

    let built = match build_structured_reference(&parsed, table, context, formula_channel_kind) {
        Ok(built) => built,
        Err(message) => {
            unresolved_references.push(UnresolvedReferenceRecord {
                source_text: source_token_text.to_string(),
                reason: message.clone(),
            });
            diagnostics.push(BindDiagnostic { message, span });
            structured_reference_bind_records.push(structured_reference_error_bind_record(
                &parsed,
                source_token_text,
                span,
                Some(table),
                diagnostics
                    .last()
                    .expect("structured reference diagnostic should have been pushed"),
            ));
            return Some(NormalizedReference::Error(ErrorRef {
                error_class: "#REF!".to_string(),
                source_text: source_token_text.to_string(),
            }));
        }
    };

    structured_reference_bind_records.push(structured_reference_success_bind_record(
        &parsed,
        table,
        &built,
        source_token_text,
        span,
    ));

    Some(NormalizedReference::Structured(built.structured))
}

fn parse_structured_reference_text(text: &str) -> Option<ParsedStructuredReference> {
    let (table_name, selector_text) = if text.starts_with('[') {
        (None, text)
    } else {
        let bracket_index = text.find('[')?;
        (
            Some(text[..bracket_index].to_string()),
            &text[bracket_index..],
        )
    };

    if !selector_text.starts_with('[')
        || !selector_text.ends_with(']')
        || matching_outer_bracket_end(selector_text)? != selector_text.len() - 1
    {
        return None;
    }

    let inner = &selector_text[1..selector_text.len() - 1];
    if inner.is_empty() {
        return None;
    }

    let mut section_qualifiers = Vec::new();
    let mut column_names = Vec::new();
    let mut caller_row_sensitive = false;

    if inner.starts_with('[') {
        for segment in split_top_level_segments(inner, ',') {
            let raw_stripped = strip_structured_brackets_raw(segment)?;
            if !contains_structured_reference_escape(raw_stripped)
                && let Some(qualifier) = parse_section_qualifier(raw_stripped)
            {
                if qualifier == StructuredSectionKind::ThisRow {
                    caller_row_sensitive = true;
                }
                section_qualifiers.push(qualifier);
                continue;
            }

            let columns = parse_structured_column_segment(segment)?;
            if columns.is_empty() {
                return None;
            }
            column_names.extend(columns);
        }
    } else if let Some(rest) = inner.strip_prefix('@') {
        caller_row_sensitive = true;
        section_qualifiers.push(StructuredSectionKind::ThisRow);
        column_names.push(unescape_structured_reference_text(rest));
    } else if let Some(qualifier) = parse_section_qualifier(inner) {
        if qualifier == StructuredSectionKind::ThisRow {
            caller_row_sensitive = true;
        }
        section_qualifiers.push(qualifier);
    } else {
        column_names.push(unescape_structured_reference_text(inner));
    }

    Some(ParsedStructuredReference {
        table_name,
        section_qualifiers,
        column_names,
        caller_row_sensitive,
    })
}

fn resolve_structured_table<'a>(
    parsed: &ParsedStructuredReference,
    effective_sheet_id: &str,
    context: &'a BindContext,
) -> Result<&'a TableDescriptor, String> {
    if let Some(table_name) = &parsed.table_name {
        context
            .table_catalog
            .iter()
            .find(|table| {
                table.sheet_scope_ref == effective_sheet_id
                    && table.table_name.eq_ignore_ascii_case(table_name)
            })
            .ok_or_else(|| format!("unknown structured-reference table '{table_name}'"))
    } else {
        let enclosing = context
            .enclosing_table_ref
            .as_ref()
            .ok_or_else(|| "structured reference requires enclosing table context".to_string())?;
        context
            .table_catalog
            .iter()
            .find(|table| table.table_id == enclosing.table_id)
            .ok_or_else(|| {
                format!(
                    "enclosing structured-reference table '{}' is not present in table_catalog",
                    enclosing.table_id
                )
            })
    }
}

fn build_structured_reference(
    parsed: &ParsedStructuredReference,
    table: &TableDescriptor,
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Result<BuiltStructuredReference, String> {
    if parsed
        .section_qualifiers
        .contains(&StructuredSectionKind::ThisRow)
        && parsed.section_qualifiers.iter().any(|qualifier| {
            matches!(
                qualifier,
                StructuredSectionKind::Headers
                    | StructuredSectionKind::Totals
                    | StructuredSectionKind::Data
                    | StructuredSectionKind::All
            )
        })
    {
        return Err(
            "#This Row must not be combined with #Headers, #Total Row, #Data, or #All".to_string(),
        );
    }

    if parsed.column_names.is_empty()
        && parsed
            .section_qualifiers
            .contains(&StructuredSectionKind::ThisRow)
    {
        return Err("standalone #This Row structured references are not yet supported".to_string());
    }

    if parsed.section_qualifiers.len() > 1
        && !parsed
            .section_qualifiers
            .contains(&StructuredSectionKind::ThisRow)
    {
        return Err(
            "structured reference section unions beyond the first local floor are not yet supported"
                .to_string(),
        );
    }

    let selected_columns = select_structured_reference_columns(parsed, table)?;
    if selected_columns.is_empty() {
        return Err("structured reference did not resolve any table columns".to_string());
    }

    let selected_column_ids = selected_columns
        .iter()
        .map(|column| column.column_id.clone())
        .collect::<Vec<_>>();
    let section = effective_structured_section(parsed);
    let resolved_reference = resolve_structured_reference_target(
        table,
        &selected_columns,
        section,
        parsed.caller_row_sensitive,
        context,
        formula_channel_kind,
    )?;

    let selected_sections = if parsed.section_qualifiers.is_empty() && !parsed.caller_row_sensitive
    {
        vec![StructuredSectionKind::Data]
    } else {
        parsed.section_qualifiers.clone()
    };

    Ok(BuiltStructuredReference {
        structured: StructuredRef {
            table_id: table.table_id.clone(),
            table_name: table.table_name.clone(),
            selector_kind: if parsed.caller_row_sensitive {
                StructuredSelectorKind::ThisRowColumn
            } else if parsed.column_names.is_empty() {
                StructuredSelectorKind::Section
            } else if parsed.section_qualifiers.is_empty() {
                StructuredSelectorKind::Column
            } else {
                StructuredSelectorKind::SectionColumn
            },
            section_qualifiers: selected_sections.clone(),
            selected_column_ids,
            caller_row_sensitive: parsed.caller_row_sensitive,
            workbook_scope_ref: table.workbook_scope_ref.clone(),
            sheet_scope_ref: table.sheet_scope_ref.clone(),
            resolved_reference,
        },
        selected_columns,
        selected_sections,
    })
}

fn structured_reference_success_bind_record(
    parsed: &ParsedStructuredReference,
    table: &TableDescriptor,
    built: &BuiltStructuredReference,
    source_token_text: &str,
    source_span_utf8: TextSpan,
) -> StructuredReferenceBindRecord {
    let selected_regions = structured_reference_selected_regions(
        table,
        &built.selected_columns,
        &built.selected_sections,
    );
    structured_reference_bind_record(
        parsed,
        source_token_text,
        source_span_utf8,
        Some(table),
        built
            .selected_columns
            .iter()
            .map(|column| column.column_id.clone())
            .collect(),
        built.selected_sections.clone(),
        selected_regions,
        Some(built.structured.resolved_reference.clone()),
        Vec::new(),
    )
}

fn structured_reference_error_bind_record(
    parsed: &ParsedStructuredReference,
    source_token_text: &str,
    source_span_utf8: TextSpan,
    table: Option<&TableDescriptor>,
    diagnostic: &BindDiagnostic,
) -> StructuredReferenceBindRecord {
    let selected_sections = if parsed.section_qualifiers.is_empty() && !parsed.caller_row_sensitive
    {
        vec![StructuredSectionKind::Data]
    } else {
        parsed.section_qualifiers.clone()
    };
    let selected_columns = table
        .and_then(|table| select_structured_reference_columns(parsed, table).ok())
        .unwrap_or_default();
    let selected_column_ids = selected_columns
        .iter()
        .map(|column| column.column_id.clone())
        .collect::<Vec<_>>();
    let selected_regions = table
        .map(|table| {
            structured_reference_selected_regions(table, &selected_columns, &selected_sections)
        })
        .unwrap_or_default();
    structured_reference_bind_record(
        parsed,
        source_token_text,
        source_span_utf8,
        table,
        selected_column_ids,
        selected_sections,
        selected_regions,
        None,
        vec![StructuredReferenceBindDiagnosticLink {
            diagnostic_code: "structured_reference_bind_error".to_string(),
            message: diagnostic.message.clone(),
            source_span_utf8: diagnostic.span,
        }],
    )
}

fn structured_reference_bind_record(
    parsed: &ParsedStructuredReference,
    source_token_text: &str,
    source_span_utf8: TextSpan,
    table: Option<&TableDescriptor>,
    selected_column_ids: Vec<String>,
    selected_sections: Vec<StructuredSectionKind>,
    selected_regions: Vec<StructuredReferenceSelectedRegion>,
    resolved_reference: Option<StructuredResolvedRef>,
    diagnostics: Vec<StructuredReferenceBindDiagnosticLink>,
) -> StructuredReferenceBindRecord {
    let effective_table_id = table.map(|table| table.table_id.clone());
    let effective_table_name = table.map(|table| table.table_name.clone());
    let mut record = StructuredReferenceBindRecord {
        bind_record_handle: String::new(),
        source_span_utf8,
        source_token_text: source_token_text.to_string(),
        source_token_kind: StructuredReferenceSourceTokenKind::StructuredReference,
        explicit_table_name: parsed.table_name.clone(),
        omitted_table_name: parsed.table_name.is_none(),
        effective_table_id,
        effective_table_name,
        selected_column_ids,
        selected_sections,
        selected_regions,
        uses_this_row: parsed.caller_row_sensitive,
        caller_context_dependent: parsed.caller_row_sensitive,
        resolved_reference,
        diagnostics,
    };
    record.bind_record_handle = format!(
        "structured-ref:{}",
        hash_debug(&(
            record.source_span_utf8,
            record.source_token_text.as_str(),
            &record.explicit_table_name,
            record.omitted_table_name,
            &record.effective_table_id,
            &record.selected_column_ids,
            &record.selected_sections,
            &record.selected_regions,
            &record.uses_this_row,
            &record.resolved_reference,
            &record.diagnostics
        ))
    );
    record
}

fn structured_reference_selected_regions(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    selected_sections: &[StructuredSectionKind],
) -> Vec<StructuredReferenceSelectedRegion> {
    let data_column_range_refs = selected_columns
        .iter()
        .filter_map(|column| {
            let range_ref = column.column_range_ref.trim();
            (!range_ref.is_empty()).then(|| column.column_range_ref.clone())
        })
        .collect::<Vec<_>>();
    let data_region_is_empty = selected_columns_have_empty_data_body(selected_columns);
    selected_sections
        .iter()
        .map(|section| StructuredReferenceSelectedRegion {
            section_kind: *section,
            region_ref: match section {
                StructuredSectionKind::All => Some(table.table_range_ref.clone()),
                StructuredSectionKind::Data | StructuredSectionKind::ThisRow => None,
                StructuredSectionKind::Headers => table.header_region_ref.clone(),
                StructuredSectionKind::Totals => table.totals_region_ref.clone(),
            },
            column_range_refs: match section {
                StructuredSectionKind::Data | StructuredSectionKind::ThisRow => {
                    data_column_range_refs.clone()
                }
                _ => selected_columns
                    .iter()
                    .filter_map(|column| {
                        let range_ref = column.column_range_ref.trim();
                        (!range_ref.is_empty()).then(|| column.column_range_ref.clone())
                    })
                    .collect(),
            },
            is_empty: matches!(
                section,
                StructuredSectionKind::Data | StructuredSectionKind::ThisRow
            ) && data_region_is_empty,
        })
        .collect()
}

fn select_structured_reference_columns(
    parsed: &ParsedStructuredReference,
    table: &TableDescriptor,
) -> Result<Vec<TableColumnDescriptor>, String> {
    if parsed.column_names.is_empty() {
        return Ok(table.columns.clone());
    }

    let first_column = find_table_column(table, &parsed.column_names[0])?;
    let last_column = find_table_column(
        table,
        parsed
            .column_names
            .last()
            .expect("column_names should be non-empty"),
    )?;
    let (start_ordinal, end_ordinal) = if first_column.ordinal <= last_column.ordinal {
        (first_column.ordinal, last_column.ordinal)
    } else {
        (last_column.ordinal, first_column.ordinal)
    };
    Ok(table
        .columns
        .iter()
        .filter(|column| column.ordinal >= start_ordinal && column.ordinal <= end_ordinal)
        .cloned()
        .collect::<Vec<_>>())
}

fn effective_structured_section(parsed: &ParsedStructuredReference) -> StructuredSectionKind {
    if parsed.caller_row_sensitive {
        StructuredSectionKind::ThisRow
    } else {
        parsed
            .section_qualifiers
            .first()
            .copied()
            .unwrap_or(StructuredSectionKind::Data)
    }
}

fn resolve_structured_reference_target(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    section: StructuredSectionKind,
    caller_row_sensitive: bool,
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Result<StructuredResolvedRef, String> {
    if caller_row_sensitive {
        let region = context.caller_table_region.as_ref().ok_or_else(|| {
            "structured reference requires caller_table_region for current-row-sensitive binding"
                .to_string()
        })?;
        if region.table_id != table.table_id {
            return Err(
                "caller_table_region table_id does not match enclosing structured-reference table"
                    .to_string(),
            );
        }
        if region.region_kind != TableRegionKind::Data {
            return Err(
                "current-row structured reference requires a data-region caller_table_region"
                    .to_string(),
            );
        }
        let row_offset = region.data_row_offset.ok_or_else(|| {
            "current-row structured reference requires data_row_offset".to_string()
        })?;
        return resolve_structured_data_row_target(
            table,
            selected_columns,
            row_offset,
            context,
            formula_channel_kind,
        );
    }

    match section {
        StructuredSectionKind::Data => resolve_structured_data_area_target(
            table,
            selected_columns,
            context,
            formula_channel_kind,
        ),
        StructuredSectionKind::All => {
            let table_area = parse_area_target(
                &table.table_range_ref,
                &table.workbook_scope_ref,
                &table.sheet_scope_ref,
                context,
                formula_channel_kind,
            )
            .ok_or_else(|| {
                format!(
                    "unable to parse table_range_ref '{}' for structured reference",
                    table.table_range_ref
                )
            })?;
            let (left_col, right_col) =
                structured_column_bounds(table, selected_columns, context, formula_channel_kind)?;
            Ok(area_or_cell_from_bounds(
                &table.workbook_scope_ref,
                &table.sheet_scope_ref,
                table_area.top_left.row,
                left_col,
                table_area.top_left.row + table_area.height - 1,
                right_col,
            ))
        }
        StructuredSectionKind::Headers => {
            if !table.header_row_present {
                return Err(
                    "structured reference requested #Headers for a table without a header row"
                        .to_string(),
                );
            }
            resolve_structured_section_row_target(
                table,
                selected_columns,
                table.header_region_ref.as_deref(),
                0,
                context,
                formula_channel_kind,
            )
        }
        StructuredSectionKind::Totals => {
            if !table.totals_row_present {
                return Err(
                    "structured reference requested #Total Row for a table without a totals row"
                        .to_string(),
                );
            }
            let table_row_offset = if table.totals_region_ref.is_some() {
                0
            } else {
                let table_area = parse_area_target(
                    &table.table_range_ref,
                    &table.workbook_scope_ref,
                    &table.sheet_scope_ref,
                    context,
                    formula_channel_kind,
                )
                .ok_or_else(|| {
                    format!(
                        "unable to parse table_range_ref '{}' for structured reference",
                        table.table_range_ref
                    )
                })?;
                table_area.height - 1
            };
            resolve_structured_section_row_target(
                table,
                selected_columns,
                table.totals_region_ref.as_deref(),
                table_row_offset,
                context,
                formula_channel_kind,
            )
        }
        StructuredSectionKind::ThisRow => Err(
            "#This Row structured references require caller_table_region and are not standalone"
                .to_string(),
        ),
    }
}

fn resolve_structured_data_row_target(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    row_offset: u32,
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Result<StructuredResolvedRef, String> {
    if selected_columns_have_empty_data_body(selected_columns) {
        return Err(
            "current-row structured reference has no table data row in the selected data body"
                .to_string(),
        );
    }
    let first = parse_area_target(
        &selected_columns[0].column_range_ref,
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        context,
        formula_channel_kind,
    )
    .ok_or_else(|| "unable to parse first structured column_range_ref".to_string())?;
    let last = parse_area_target(
        &selected_columns[selected_columns.len() - 1].column_range_ref,
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        context,
        formula_channel_kind,
    )
    .ok_or_else(|| "unable to parse last structured column_range_ref".to_string())?;
    if row_offset >= first.height {
        return Err("structured reference row offset is outside the table data body".to_string());
    }
    let row = first.top_left.row + row_offset;
    Ok(area_or_cell_from_bounds(
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        row,
        first.top_left.col.min(last.top_left.col),
        row,
        first.top_left.col.max(last.top_left.col),
    ))
}

fn resolve_structured_data_area_target(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Result<StructuredResolvedRef, String> {
    if selected_columns_have_empty_data_body(selected_columns) {
        return Ok(StructuredResolvedRef::EmptyArea(structured_empty_area_ref(
            table,
            selected_columns,
            StructuredSectionKind::Data,
        )));
    }
    let first = parse_area_target(
        &selected_columns[0].column_range_ref,
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        context,
        formula_channel_kind,
    )
    .ok_or_else(|| "unable to parse first structured column_range_ref".to_string())?;
    let last = parse_area_target(
        &selected_columns[selected_columns.len() - 1].column_range_ref,
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        context,
        formula_channel_kind,
    )
    .ok_or_else(|| "unable to parse last structured column_range_ref".to_string())?;
    Ok(area_or_cell_from_bounds(
        &first.workbook_id,
        &first.sheet_id,
        first.top_left.row.min(last.top_left.row),
        first.top_left.col.min(last.top_left.col),
        (first.top_left.row + first.height - 1).max(last.top_left.row + last.height - 1),
        first.top_left.col.max(last.top_left.col),
    ))
}

fn resolve_structured_section_row_target(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    explicit_region_ref: Option<&str>,
    table_row_offset: u32,
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Result<StructuredResolvedRef, String> {
    let row_area_ref = explicit_region_ref.unwrap_or(&table.table_range_ref);
    let row_area = parse_area_target(
        row_area_ref,
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        context,
        formula_channel_kind,
    )
    .ok_or_else(|| {
        let ref_kind = if explicit_region_ref.is_some() {
            "structured section region ref"
        } else {
            "table_range_ref"
        };
        format!("unable to parse {ref_kind} '{row_area_ref}' for structured reference")
    })?;
    let (left_col, right_col) =
        structured_column_bounds(table, selected_columns, context, formula_channel_kind)?;
    let row = row_area.top_left.row + table_row_offset;
    Ok(area_or_cell_from_bounds(
        &table.workbook_scope_ref,
        &table.sheet_scope_ref,
        row,
        left_col,
        row,
        right_col,
    ))
}

fn selected_columns_have_empty_data_body(selected_columns: &[TableColumnDescriptor]) -> bool {
    !selected_columns.is_empty()
        && selected_columns
            .iter()
            .all(|column| column.column_range_ref.trim().is_empty())
}

fn structured_empty_area_ref(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    section_kind: StructuredSectionKind,
) -> StructuredEmptyAreaRef {
    StructuredEmptyAreaRef {
        workbook_id: table.workbook_scope_ref.clone(),
        sheet_id: table.sheet_scope_ref.clone(),
        section_kind,
        selected_column_ids: selected_columns
            .iter()
            .map(|column| column.column_id.clone())
            .collect(),
        column_count: selected_columns.len() as u32,
        row_membership_identity: table.row_membership_identity.clone(),
        row_order_identity: table.row_order_identity.clone(),
    }
}

fn structured_column_bounds(
    table: &TableDescriptor,
    selected_columns: &[TableColumnDescriptor],
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Result<(u32, u32), String> {
    let first_column = selected_columns
        .first()
        .ok_or_else(|| "structured reference did not resolve any table columns".to_string())?;
    let last_column = selected_columns
        .last()
        .expect("first selected column confirms last selected column");

    let first_range = first_column.column_range_ref.trim();
    let last_range = last_column.column_range_ref.trim();
    if selected_columns_have_empty_data_body(selected_columns) {
        let table_area = parse_area_target(
            &table.table_range_ref,
            &table.workbook_scope_ref,
            &table.sheet_scope_ref,
            context,
            formula_channel_kind,
        )
        .ok_or_else(|| {
            format!(
                "unable to parse table_range_ref '{}' for structured reference",
                table.table_range_ref
            )
        })?;
        let left_col = table_area.top_left.col + first_column.ordinal.saturating_sub(1);
        let right_col = table_area.top_left.col + last_column.ordinal.saturating_sub(1);
        return Ok((left_col.min(right_col), left_col.max(right_col)));
    }

    if !first_range.is_empty() && !last_range.is_empty() {
        let first = parse_area_target(
            first_range,
            &table.workbook_scope_ref,
            &table.sheet_scope_ref,
            context,
            formula_channel_kind,
        )
        .ok_or_else(|| "unable to parse first structured column_range_ref".to_string())?;
        let last = parse_area_target(
            last_range,
            &table.workbook_scope_ref,
            &table.sheet_scope_ref,
            context,
            formula_channel_kind,
        )
        .ok_or_else(|| "unable to parse last structured column_range_ref".to_string())?;
        return Ok((
            first.top_left.col.min(last.top_left.col),
            first.top_left.col.max(last.top_left.col),
        ));
    }

    Err("structured reference has partial data-column range refs; zero-row packets must leave every selected data column range empty".to_string())
}

fn area_or_cell_from_bounds(
    workbook_id: &str,
    sheet_id: &str,
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
) -> StructuredResolvedRef {
    if top_row == bottom_row && left_col == right_col {
        StructuredResolvedRef::Cell(CellRef {
            workbook_id: workbook_id.to_string(),
            sheet_id: sheet_id.to_string(),
            coord: CellCoord {
                row: top_row,
                col: left_col,
            },
            address_mode: AddressMode::default(),
            caller_anchor_used: false,
        })
    } else {
        StructuredResolvedRef::Area(AreaRef {
            workbook_id: workbook_id.to_string(),
            sheet_id: sheet_id.to_string(),
            top_left: CellCoord {
                row: top_row,
                col: left_col,
            },
            height: bottom_row - top_row + 1,
            width: right_col - left_col + 1,
            address_mode: AddressMode::default(),
            caller_anchor_used: false,
        })
    }
}

fn find_table_column<'a>(
    table: &'a TableDescriptor,
    column_name: &str,
) -> Result<&'a TableColumnDescriptor, String> {
    table
        .columns
        .iter()
        .find(|column| column.column_name.eq_ignore_ascii_case(column_name))
        .ok_or_else(|| format!("unknown structured-reference column '{column_name}'"))
}

fn parse_structured_column_segment(segment: &str) -> Option<Vec<String>> {
    if let Some((left, right)) = split_top_level_once(segment, ':') {
        return Some(vec![
            strip_structured_brackets(left)?,
            strip_structured_brackets(right)?,
        ]);
    }

    Some(vec![strip_structured_brackets(segment)?])
}

fn split_top_level_segments(text: &str, delimiter: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut skip_escaped = false;
    for (index, ch) in text.char_indices() {
        if skip_escaped {
            skip_escaped = false;
            continue;
        }
        if ch == '\'' && is_structured_reference_escape_at(text, index) {
            skip_escaped = true;
            continue;
        }
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                result.push(text[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    result.push(text[start..].trim());
    result
}

fn split_top_level_once(text: &str, delimiter: char) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let mut skip_escaped = false;
    for (index, ch) in text.char_indices() {
        if skip_escaped {
            skip_escaped = false;
            continue;
        }
        if ch == '\'' && is_structured_reference_escape_at(text, index) {
            skip_escaped = true;
            continue;
        }
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                let delimiter_len = ch.len_utf8();
                return Some((text[..index].trim(), text[index + delimiter_len..].trim()));
            }
            _ => {}
        }
    }
    None
}

fn strip_structured_brackets(text: &str) -> Option<String> {
    strip_structured_brackets_raw(text).map(unescape_structured_reference_text)
}

fn strip_structured_brackets_raw(text: &str) -> Option<&str> {
    if text.starts_with('[') && text.ends_with(']') && text.len() >= 2 {
        Some(&text[1..text.len() - 1])
    } else if !text.is_empty() {
        Some(text)
    } else {
        None
    }
}

fn matching_outer_bracket_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut skip_escaped = false;
    for (index, ch) in text.char_indices() {
        if skip_escaped {
            skip_escaped = false;
            continue;
        }
        if ch == '\'' && is_structured_reference_escape_at(text, index) {
            skip_escaped = true;
            continue;
        }
        match ch {
            '[' => depth += 1,
            ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn unescape_structured_reference_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' && matches!(chars.peek(), Some('#' | '[' | ']' | '@' | '\'')) {
            output.push(
                chars
                    .next()
                    .expect("peek confirmed escaped structured-reference character"),
            );
        } else {
            output.push(ch);
        }
    }
    output
}

fn is_structured_reference_escape_at(text: &str, index: usize) -> bool {
    text[index..]
        .chars()
        .nth(1)
        .is_some_and(|ch| matches!(ch, '#' | '[' | ']' | '@' | '\''))
}

fn contains_structured_reference_escape(text: &str) -> bool {
    text.char_indices()
        .any(|(index, ch)| ch == '\'' && is_structured_reference_escape_at(text, index))
}

fn parse_section_qualifier(text: &str) -> Option<StructuredSectionKind> {
    match text.to_ascii_uppercase().replace(' ', "").as_str() {
        "#ALL" => Some(StructuredSectionKind::All),
        "#DATA" => Some(StructuredSectionKind::Data),
        "#HEADERS" => Some(StructuredSectionKind::Headers),
        "#TOTALS" | "#TOTALROW" => Some(StructuredSectionKind::Totals),
        "#THISROW" => Some(StructuredSectionKind::ThisRow),
        _ => None,
    }
}

fn parse_area_target(
    text: &str,
    workbook_id: &str,
    default_sheet_id: &str,
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Option<AreaRef> {
    let (sheet_id, target) = split_sheet_qualified_target(text, default_sheet_id);
    let (start_text, end_text) = target.split_once(':')?;
    let start = parse_cell_reference(start_text, &sheet_id, context, formula_channel_kind)?;
    let end = parse_cell_reference(end_text, &sheet_id, context, formula_channel_kind)?;
    let top_row = start.coord.row.min(end.coord.row);
    let left_col = start.coord.col.min(end.coord.col);
    let bottom_row = start.coord.row.max(end.coord.row);
    let right_col = start.coord.col.max(end.coord.col);
    Some(AreaRef {
        workbook_id: workbook_id.to_string(),
        sheet_id,
        top_left: CellCoord {
            row: top_row,
            col: left_col,
        },
        height: bottom_row - top_row + 1,
        width: right_col - left_col + 1,
        address_mode: AddressMode::default(),
        caller_anchor_used: false,
    })
}

fn split_sheet_qualified_target<'a>(text: &'a str, default_sheet_id: &str) -> (String, &'a str) {
    if let Some((qualifier_text, target)) = text.rsplit_once('!') {
        let qualifier = parse_reference_qualifier(qualifier_text);
        (qualifier.sheet_id, target)
    } else {
        (default_sheet_id.to_string(), text)
    }
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
                explicit: false,
            },
            source_span: node.span,
            source_text: node_source_text(node),
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
                    source_span: node.span,
                    source_text: node_source_text(node),
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
                explicit: true,
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
        explicit: true,
    }
}

fn harmonize_simple_reference_fragments(
    left_node: &GreenNode,
    right_node: &GreenNode,
    context: &BindContext,
) -> Option<(SimpleReferenceFragment, SimpleReferenceFragment)> {
    let mut left_simple = try_parse_simple_reference_fragment(left_node, context)?;
    let mut right_simple = try_parse_simple_reference_fragment(right_node, context)?;

    if left_simple.qualifier.is_external || right_simple.qualifier.is_external {
        if left_simple.qualifier.raw == right_simple.qualifier.raw {
            return Some((left_simple, right_simple));
        }
        return None;
    }

    if left_simple.qualifier.raw == right_simple.qualifier.raw {
        return Some((left_simple, right_simple));
    }

    match (
        left_simple.qualifier.explicit,
        right_simple.qualifier.explicit,
    ) {
        (true, false) => {
            right_simple.qualifier = left_simple.qualifier.clone();
            Some((left_simple, right_simple))
        }
        (false, true) => {
            left_simple.qualifier = right_simple.qualifier.clone();
            Some((left_simple, right_simple))
        }
        _ => None,
    }
}

fn reference_range_endpoint_request(
    fragment: SimpleReferenceFragment,
) -> ReferenceRangeEndpointBindRequest {
    ReferenceRangeEndpointBindRequest {
        source_span: fragment.source_span,
        source_text: fragment.source_text,
        target_text: fragment.target_text,
        parsed_qualifier: fragment
            .qualifier
            .explicit
            .then_some(fragment.qualifier.raw),
        sheet_id: fragment.qualifier.sheet_id,
        external_target_id: fragment.qualifier.external_target_id,
    }
}

fn parse_cell_reference(
    text: &str,
    sheet_id: &str,
    context: &BindContext,
    formula_channel_kind: FormulaChannelKind,
) -> Option<CellRef> {
    if formula_channel_kind == FormulaChannelKind::WorksheetR1C1 {
        return parse_r1c1_cell_reference(text, sheet_id, context);
    }

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

fn parse_row_reference(text: &str, formula_channel_kind: FormulaChannelKind) -> Option<u32> {
    if formula_channel_kind == FormulaChannelKind::WorksheetR1C1 {
        return parse_r1c1_row_reference(text);
    }
    if text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    text.parse::<u32>().ok()
}

fn parse_column_reference(text: &str, formula_channel_kind: FormulaChannelKind) -> Option<u32> {
    if formula_channel_kind == FormulaChannelKind::WorksheetR1C1 {
        return parse_r1c1_column_reference(text);
    }
    if text.is_empty() || text.len() > 3 || !text.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }
    column_to_index(text)
}

fn parse_r1c1_cell_reference(text: &str, sheet_id: &str, context: &BindContext) -> Option<CellRef> {
    let (row, row_absolute, row_anchor_used, rest) =
        parse_r1c1_axis(text, 'R', context.caller_row)?;
    let (col, col_absolute, col_anchor_used, rest) =
        parse_r1c1_axis(rest, 'C', context.caller_col)?;
    if !rest.is_empty() {
        return None;
    }

    Some(CellRef {
        workbook_id: context.workbook_id.clone(),
        sheet_id: sheet_id.to_string(),
        coord: CellCoord { row, col },
        address_mode: AddressMode {
            row_absolute,
            col_absolute,
        },
        caller_anchor_used: row_anchor_used || col_anchor_used,
    })
}

fn parse_r1c1_row_reference(text: &str) -> Option<u32> {
    let remainder = text.strip_prefix('R')?;
    if remainder.starts_with('[') || remainder.is_empty() {
        return None;
    }
    remainder.parse::<u32>().ok()
}

fn parse_r1c1_column_reference(text: &str) -> Option<u32> {
    let remainder = text.strip_prefix('C')?;
    if remainder.starts_with('[') || remainder.is_empty() {
        return None;
    }
    remainder.parse::<u32>().ok()
}

fn parse_r1c1_axis<'a>(
    text: &'a str,
    axis_kind: char,
    caller_anchor: u32,
) -> Option<(u32, bool, bool, &'a str)> {
    let remainder = text.strip_prefix(axis_kind)?;
    if let Some(relative) = remainder.strip_prefix('[') {
        let close_index = relative.find(']')?;
        let delta = relative[..close_index].parse::<i32>().ok()?;
        let anchor = i64::from(caller_anchor);
        let resolved = anchor.checked_add(i64::from(delta))?;
        if resolved < 1 {
            return None;
        }
        return Some((
            u32::try_from(resolved).ok()?,
            false,
            true,
            &relative[close_index + 1..],
        ));
    }

    let digits_len = remainder
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .count();
    if digits_len == 0 {
        return None;
    }
    let absolute = remainder[..digits_len].parse::<u32>().ok()?;
    Some((absolute, true, false, &remainder[digits_len..]))
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

fn reference_profile_fingerprint_context(
    context: &BindContext,
) -> ReferenceProfileFingerprintContext {
    ReferenceProfileFingerprintContext {
        workbook_id: context.workbook_id.clone(),
        sheet_id: context.sheet_id.clone(),
        caller_row: context.caller_row,
        caller_col: context.caller_col,
        structure_context_version: context.structure_context_version.0.clone(),
    }
}

fn reference_profile_fingerprint(
    profile: Option<&dyn ReferenceBindProfile>,
    context: &ReferenceProfileFingerprintContext,
) -> Option<ReferenceProfileFingerprint> {
    profile.map(|profile| profile.fingerprint(context))
}

fn bind_context_fingerprint_for(
    context: &BindContext,
    profile_fingerprint: Option<&ReferenceProfileFingerprint>,
    fingerprint_policy: ReferenceFingerprintPolicy,
) -> String {
    let caller_anchor = match fingerprint_policy {
        ReferenceFingerprintPolicy::IncludeCallerAnchor => {
            Some((context.caller_row, context.caller_col))
        }
        ReferenceFingerprintPolicy::ExcludeCallerAnchorForTemplate => None,
    };

    hash_debug(&(
        context.workbook_id.clone(),
        context.sheet_id.clone(),
        caller_anchor,
        context.formula_token.0.clone(),
        context.structure_context_version.0.clone(),
        context.names.clone(),
        context.name_caller_context_dependencies.clone(),
        context.host_name_bind_records.clone(),
        context.table_catalog.clone(),
        context.enclosing_table_ref.clone(),
        context.caller_table_region.clone(),
        profile_fingerprint.map(|fingerprint| fingerprint.0.clone()),
    ))
}

fn formula_source_identity_for(
    source: &FormulaSourceRecord,
    profile: Option<&dyn ReferenceBindProfile>,
) -> FormulaSourceIdentity {
    FormulaSourceIdentity {
        key: hash_debug(&(
            source.formula_stable_id.0.clone(),
            source.formula_text_version.0,
            source.formula_channel_kind,
            source.entered_formula_text.clone(),
            source.stored_formula_text.clone(),
            profile.map(|profile| {
                (
                    profile.profile_id().to_string(),
                    profile.profile_version().0,
                    profile.reference_policy(),
                )
            }),
        )),
    }
}

fn formula_template_identity_for(
    source: &FormulaSourceRecord,
    profile: Option<&dyn ReferenceBindProfile>,
    profile_fingerprint: Option<&ReferenceProfileFingerprint>,
    bind_hash: &str,
    root: &BoundExpr,
) -> FormulaTemplateIdentity {
    FormulaTemplateIdentity {
        key: hash_debug(&(
            source.formula_channel_kind,
            source.entered_formula_text.clone(),
            source.stored_formula_text.clone(),
            profile.map(|profile| {
                (
                    profile.profile_id().to_string(),
                    profile.profile_version().0,
                    profile.reference_policy(),
                )
            }),
            profile_fingerprint.map(|fingerprint| fingerprint.0.clone()),
            bind_hash,
            root,
        )),
    }
}

fn placed_formula_identity_for(
    template_identity: &FormulaTemplateIdentity,
    context: &BindContext,
    profile_fingerprint: Option<&ReferenceProfileFingerprint>,
) -> PlacedFormulaIdentity {
    PlacedFormulaIdentity {
        key: hash_debug(&(
            template_identity.key.clone(),
            context.workbook_id.clone(),
            context.sheet_id.clone(),
            context.caller_row,
            context.caller_col,
            context.structure_context_version.0.clone(),
            context.names.clone(),
            context.host_name_bind_records.clone(),
            context.table_catalog.clone(),
            profile_fingerprint.map(|fingerprint| fingerprint.0.clone()),
        )),
    }
}

fn runtime_dependency_identity_for(
    placed_identity: &PlacedFormulaIdentity,
) -> RuntimeDependencyIdentity {
    RuntimeDependencyIdentity {
        key: hash_debug(&(placed_identity.key.clone(), "runtime-dependency:v1")),
    }
}

fn hash_debug<T: std::fmt::Debug>(value: &T) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{value:?}").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
