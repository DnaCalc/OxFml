use std::collections::{BTreeMap, BTreeSet};

use crate::binding::{BindContext, BindRequest, BoundFormula, bind_formula_incremental};
use crate::interface::{LibraryContextProvider, LibraryContextSnapshotRef};
use crate::red::{RedProjection, project_red_view_incremental};
use crate::semantics::{
    CompileSemanticPlanRequest, LibraryContextSnapshot, SemanticPlan, compile_semantic_plan,
};
use crate::source::{FormulaChannelKind, FormulaSourceRecord, FormulaTextVersion};
use crate::syntax::green::{GreenChild, GreenNode, GreenTreeRoot, SyntaxKind};
use crate::syntax::parser::{ParseRequest, parse_formula_incremental};
use crate::syntax::token::{SyntaxTrivia, SyntaxTriviaKind, TextSpan, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTriviaKind {
    Whitespace,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorTrivia {
    pub kind: EditorTriviaKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorToken {
    pub kind: TokenKind,
    pub text: String,
    pub leading_trivia: Vec<EditorTrivia>,
    pub trailing_trivia: Vec<EditorTrivia>,
    pub span: TextSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSyntaxSnapshot {
    pub formula_stable_id: String,
    pub formula_channel_kind: FormulaChannelKind,
    pub green_tree_key: String,
    pub tokens: Vec<EditorToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveDiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveDiagnosticStage {
    Syntax,
    Bind,
    SemanticPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveDiagnostic {
    pub diagnostic_id: String,
    pub severity: LiveDiagnosticSeverity,
    pub stage: LiveDiagnosticStage,
    pub message: String,
    pub primary_span: TextSpan,
    pub related_spans: Vec<TextSpan>,
    pub code: Option<String>,
    pub suggested_fix_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveDiagnosticSnapshot {
    pub formula_stable_id: String,
    pub formula_token: String,
    pub diagnostics: Vec<LiveDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormulaTextChangeRange {
    pub start: usize,
    pub old_len: usize,
    pub new_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditFollowOnStage {
    ParseOnly,
    ParseAndBind,
    ParseBindAndPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticPlanEditOptions {
    pub oxfunc_catalog_identity: String,
    pub locale_profile: Option<String>,
    pub date_system: Option<String>,
    pub format_profile: Option<String>,
    pub library_context_snapshot: Option<LibraryContextSnapshot>,
}

#[derive(Debug, Clone)]
pub struct FormulaEditRequest<'a> {
    pub source: FormulaSourceRecord,
    pub bind_context: BindContext,
    pub previous_green_tree: Option<&'a GreenTreeRoot>,
    pub previous_red_projection: Option<&'a RedProjection>,
    pub previous_bound_formula: Option<&'a BoundFormula>,
    pub follow_on_stage: EditFollowOnStage,
    pub semantic_plan_options: Option<SemanticPlanEditOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormulaEditReuseSummary {
    pub reused_green_tree: bool,
    pub reused_red_projection: bool,
    pub reused_bound_formula: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaEditResult {
    pub source: FormulaSourceRecord,
    pub text_change_range: Option<FormulaTextChangeRange>,
    pub editor_syntax_snapshot: EditorSyntaxSnapshot,
    pub green_tree: GreenTreeRoot,
    pub red_projection: RedProjection,
    pub bound_formula: Option<BoundFormula>,
    pub semantic_plan: Option<SemanticPlan>,
    pub live_diagnostics: LiveDiagnosticSnapshot,
    pub reuse_summary: FormulaEditReuseSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionProposalKind {
    Function,
    DefinedName,
    TableName,
    TableColumn,
    StructuredSelector,
    SyntaxAssist,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionProposal {
    pub proposal_id: String,
    pub proposal_kind: CompletionProposalKind,
    pub display_text: String,
    pub insert_text: String,
    pub replacement_span: Option<TextSpan>,
    pub documentation_ref: Option<String>,
    pub requires_revalidation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelpContext {
    pub callee_text: String,
    pub call_span: TextSpan,
    pub active_argument_index: usize,
    pub invocation_kind: SyntaxKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionResult {
    pub replacement_span: Option<TextSpan>,
    pub proposals: Vec<CompletionProposal>,
    pub signature_help_context: Option<SignatureHelpContext>,
}

#[derive(Debug, Clone)]
pub struct CompletionValidationRequest<'a> {
    pub source: FormulaSourceRecord,
    pub bind_context: BindContext,
    pub previous_green_tree: Option<&'a GreenTreeRoot>,
    pub previous_red_projection: Option<&'a RedProjection>,
    pub previous_bound_formula: Option<&'a BoundFormula>,
    pub replacement_span: Option<TextSpan>,
    pub insert_text: String,
    pub follow_on_stage: EditFollowOnStage,
    pub semantic_plan_options: Option<SemanticPlanEditOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionValidationResult {
    pub proposal_id: Option<String>,
    pub applied_span: TextSpan,
    pub updated_source: FormulaSourceRecord,
    pub edit_result: FormulaEditResult,
}

#[derive(Clone)]
pub struct CompletionRequest<'a> {
    pub source: &'a FormulaSourceRecord,
    pub green_tree: &'a GreenTreeRoot,
    pub red_projection: &'a RedProjection,
    pub bind_context: &'a BindContext,
    pub library_context_provider: Option<&'a dyn LibraryContextProvider>,
    pub library_context_snapshot_ref: Option<&'a LibraryContextSnapshotRef>,
    pub library_context_snapshot: Option<&'a LibraryContextSnapshot>,
    pub cursor_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHelpLookupRequest {
    pub lookup_key: String,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHelpSignatureForm {
    pub display_signature: String,
    pub min_arity: usize,
    pub max_arity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHelpPacket {
    pub display_name: String,
    pub signature_forms: Vec<FunctionHelpSignatureForm>,
    pub argument_help: Vec<String>,
    pub short_description: Option<String>,
    pub availability_summary: Option<String>,
    pub deferred_or_profile_limited: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelligentCompletionContext {
    pub formula_text: String,
    pub formula_channel_kind: FormulaChannelKind,
    pub cursor_offset: usize,
    pub green_tree_key: String,
    pub red_context_summary: String,
    pub visible_defined_names: Vec<String>,
    pub visible_tables: Vec<String>,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub active_diagnostics: Vec<LiveDiagnostic>,
    pub signature_help_context: Option<SignatureHelpContext>,
}

pub fn build_function_help_lookup_request(
    source: &FormulaSourceRecord,
    green_tree: &GreenTreeRoot,
    cursor_offset: usize,
    library_context_provider: Option<&dyn LibraryContextProvider>,
    library_context_snapshot_ref: Option<&LibraryContextSnapshotRef>,
    library_context_snapshot: Option<&LibraryContextSnapshot>,
) -> Option<FunctionHelpLookupRequest> {
    let signature_help = signature_help_context_at_cursor(source, green_tree, cursor_offset)?;
    if signature_help.callee_text.is_empty() {
        return None;
    }

    Some(FunctionHelpLookupRequest {
        lookup_key: signature_help.callee_text,
        library_context_snapshot_ref: effective_library_context_snapshot_ref(
            library_context_provider,
            library_context_snapshot_ref,
            library_context_snapshot,
        ),
    })
}

pub fn build_editor_syntax_snapshot(
    source: &FormulaSourceRecord,
    green_tree: &GreenTreeRoot,
) -> EditorSyntaxSnapshot {
    let mut tokens = Vec::new();
    collect_editor_tokens(&green_tree.root, &mut tokens);

    EditorSyntaxSnapshot {
        formula_stable_id: source.formula_stable_id.0.clone(),
        formula_channel_kind: source.formula_channel_kind,
        green_tree_key: green_tree.green_tree_key.clone(),
        tokens,
    }
}

pub fn apply_formula_edit(request: FormulaEditRequest<'_>) -> FormulaEditResult {
    let text_change_range = request.previous_green_tree.and_then(|previous_green_tree| {
        compute_text_change_range(
            &green_tree_text(previous_green_tree),
            &request.source.entered_formula_text,
        )
    });
    let parse = parse_formula_incremental(
        ParseRequest {
            source: request.source.clone(),
        },
        request.previous_green_tree,
    );
    let red = project_red_view_incremental(
        request.source.formula_stable_id.clone(),
        &parse.green_tree,
        request.previous_red_projection,
    );

    let (bound_formula, reused_bound_formula) = match request.follow_on_stage {
        EditFollowOnStage::ParseOnly => (None, false),
        EditFollowOnStage::ParseAndBind | EditFollowOnStage::ParseBindAndPlan => {
            let bind = bind_formula_incremental(
                BindRequest {
                    source: request.source.clone(),
                    green_tree: parse.green_tree.clone(),
                    red_projection: red.red_projection.clone(),
                    context: request.bind_context.clone(),
                },
                request.previous_bound_formula,
            );
            (Some(bind.bound_formula), bind.reused_bound_formula)
        }
    };

    let semantic_plan = match request.follow_on_stage {
        EditFollowOnStage::ParseBindAndPlan => bound_formula.as_ref().map(|bound_formula| {
            let options =
                request
                    .semantic_plan_options
                    .clone()
                    .unwrap_or(SemanticPlanEditOptions {
                        oxfunc_catalog_identity: "editor-catalog".to_string(),
                        locale_profile: None,
                        date_system: None,
                        format_profile: None,
                        library_context_snapshot: None,
                    });
            compile_semantic_plan(CompileSemanticPlanRequest {
                bound_formula: bound_formula.clone(),
                oxfunc_catalog_identity: options.oxfunc_catalog_identity,
                locale_profile: options.locale_profile,
                date_system: options.date_system,
                format_profile: options.format_profile,
                library_context_snapshot: options.library_context_snapshot,
            })
            .semantic_plan
        }),
        _ => None,
    };

    let live_diagnostics = build_live_diagnostics(
        &request.source,
        &parse.green_tree,
        bound_formula.as_ref(),
        semantic_plan.as_ref(),
    );

    FormulaEditResult {
        source: request.source.clone(),
        text_change_range,
        editor_syntax_snapshot: build_editor_syntax_snapshot(&request.source, &parse.green_tree),
        green_tree: parse.green_tree,
        red_projection: red.red_projection,
        bound_formula,
        semantic_plan,
        live_diagnostics,
        reuse_summary: FormulaEditReuseSummary {
            reused_green_tree: parse.reused_green_tree,
            reused_red_projection: red.reused_red_projection,
            reused_bound_formula,
        },
    }
}

pub fn apply_completion_proposal(
    mut request: CompletionValidationRequest<'_>,
    proposal: &CompletionProposal,
) -> CompletionValidationResult {
    request.replacement_span = proposal.replacement_span;
    request.insert_text = proposal.insert_text.clone();
    let mut result = validate_completion_candidate(request);
    result.proposal_id = Some(proposal.proposal_id.clone());
    result
}

pub fn validate_completion_candidate(
    request: CompletionValidationRequest<'_>,
) -> CompletionValidationResult {
    let applied_span = request
        .replacement_span
        .unwrap_or_else(|| TextSpan::new(request.source.entered_formula_text.chars().count(), 0));
    let updated_formula_text = apply_text_replacement(
        &request.source.entered_formula_text,
        applied_span,
        &request.insert_text,
    );
    let updated_source = FormulaSourceRecord {
        entered_formula_text: updated_formula_text,
        formula_text_version: FormulaTextVersion(request.source.formula_text_version.0 + 1),
        ..request.source.clone()
    };
    let edit_result = apply_formula_edit(FormulaEditRequest {
        source: updated_source.clone(),
        bind_context: request.bind_context,
        previous_green_tree: request.previous_green_tree,
        previous_red_projection: request.previous_red_projection,
        previous_bound_formula: request.previous_bound_formula,
        follow_on_stage: request.follow_on_stage,
        semantic_plan_options: request.semantic_plan_options,
    });

    CompletionValidationResult {
        proposal_id: None,
        applied_span,
        updated_source,
        edit_result,
    }
}

pub fn build_live_diagnostics(
    source: &FormulaSourceRecord,
    green_tree: &GreenTreeRoot,
    bound_formula: Option<&BoundFormula>,
    semantic_plan: Option<&SemanticPlan>,
) -> LiveDiagnosticSnapshot {
    let mut diagnostics = Vec::new();

    for (index, diagnostic) in green_tree.diagnostics.iter().enumerate() {
        diagnostics.push(LiveDiagnostic {
            diagnostic_id: format!("{}:syntax:{index}", source.formula_stable_id.0),
            severity: LiveDiagnosticSeverity::Error,
            stage: LiveDiagnosticStage::Syntax,
            message: diagnostic.message.clone(),
            primary_span: diagnostic.span,
            related_spans: Vec::new(),
            code: Some("syntax".to_string()),
            suggested_fix_kind: None,
        });
    }

    if let Some(bound_formula) = bound_formula {
        for (index, diagnostic) in bound_formula.diagnostics.iter().enumerate() {
            diagnostics.push(LiveDiagnostic {
                diagnostic_id: format!("{}:bind:{index}", source.formula_stable_id.0),
                severity: LiveDiagnosticSeverity::Error,
                stage: LiveDiagnosticStage::Bind,
                message: diagnostic.message.clone(),
                primary_span: diagnostic.span,
                related_spans: Vec::new(),
                code: Some("bind".to_string()),
                suggested_fix_kind: suggested_fix_kind_for_bind_message(&diagnostic.message),
            });
        }
    }

    if let Some(semantic_plan) = semantic_plan {
        for (index, diagnostic) in semantic_plan.diagnostics.iter().enumerate() {
            diagnostics.push(LiveDiagnostic {
                diagnostic_id: format!("{}:semantic:{index}", source.formula_stable_id.0),
                severity: LiveDiagnosticSeverity::Warning,
                stage: LiveDiagnosticStage::SemanticPlan,
                message: diagnostic.message.clone(),
                primary_span: TextSpan::new(0, source.entered_formula_text.chars().count()),
                related_spans: Vec::new(),
                code: diagnostic.function_name.clone(),
                suggested_fix_kind: None,
            });
        }
    }

    LiveDiagnosticSnapshot {
        formula_stable_id: source.formula_stable_id.0.clone(),
        formula_token: source.formula_token().0,
        diagnostics,
    }
}

pub fn collect_completion_proposals(request: CompletionRequest<'_>) -> CompletionResult {
    let (replacement_span, prefix) =
        completion_prefix(&request.source.entered_formula_text, request.cursor_offset);
    let normalized_prefix = prefix.to_ascii_lowercase();
    let signature_help_context =
        signature_help_context_at_cursor(request.source, request.green_tree, request.cursor_offset);

    let mut proposals = BTreeMap::<(u8, String), CompletionProposal>::new();

    if request.source.formula_channel_kind == FormulaChannelKind::WorksheetR1C1 {
        for assist in ["R", "C", "RC", "R[", "C["] {
            if assist.to_ascii_lowercase().starts_with(&normalized_prefix) {
                insert_proposal(
                    &mut proposals,
                    0,
                    CompletionProposal {
                        proposal_id: format!("syntax-assist:{assist}"),
                        proposal_kind: CompletionProposalKind::SyntaxAssist,
                        display_text: assist.to_string(),
                        insert_text: assist.to_string(),
                        replacement_span: Some(replacement_span),
                        documentation_ref: Some("syntax:r1c1".to_string()),
                        requires_revalidation: true,
                    },
                );
            }
        }
    }

    if is_structured_selector_context(&request.source.entered_formula_text, replacement_span) {
        for selector in ["#All", "#Data", "#Headers", "#Totals", "#This Row"] {
            if selector
                .to_ascii_lowercase()
                .starts_with(&normalized_prefix)
            {
                insert_proposal(
                    &mut proposals,
                    0,
                    CompletionProposal {
                        proposal_id: format!("structured-selector:{selector}"),
                        proposal_kind: CompletionProposalKind::StructuredSelector,
                        display_text: selector.to_string(),
                        insert_text: selector.to_string(),
                        replacement_span: Some(replacement_span),
                        documentation_ref: None,
                        requires_revalidation: true,
                    },
                );
            }
        }
    }

    if let Some(table_name) =
        structured_column_table_name(&request.source.entered_formula_text, replacement_span)
    {
        if let Some(table) = request
            .bind_context
            .table_catalog
            .iter()
            .find(|table| table.table_name.eq_ignore_ascii_case(&table_name))
        {
            for column in &table.columns {
                if normalized_prefix.is_empty()
                    || column
                        .column_name
                        .to_ascii_lowercase()
                        .starts_with(&normalized_prefix)
                {
                    insert_proposal(
                        &mut proposals,
                        1,
                        CompletionProposal {
                            proposal_id: format!("table-column:{}", column.column_id),
                            proposal_kind: CompletionProposalKind::TableColumn,
                            display_text: column.column_name.clone(),
                            insert_text: column.column_name.clone(),
                            replacement_span: Some(replacement_span),
                            documentation_ref: None,
                            requires_revalidation: true,
                        },
                    );
                }
            }
        }
    }

    for name in request.bind_context.names.keys() {
        if normalized_prefix.is_empty() || name.to_ascii_lowercase().starts_with(&normalized_prefix)
        {
            insert_proposal(
                &mut proposals,
                2,
                CompletionProposal {
                    proposal_id: format!("defined-name:{name}"),
                    proposal_kind: CompletionProposalKind::DefinedName,
                    display_text: name.clone(),
                    insert_text: name.clone(),
                    replacement_span: Some(replacement_span),
                    documentation_ref: None,
                    requires_revalidation: true,
                },
            );
        }
    }

    for table in &request.bind_context.table_catalog {
        if normalized_prefix.is_empty()
            || table
                .table_name
                .to_ascii_lowercase()
                .starts_with(&normalized_prefix)
        {
            insert_proposal(
                &mut proposals,
                3,
                CompletionProposal {
                    proposal_id: format!("table-name:{}", table.table_id),
                    proposal_kind: CompletionProposalKind::TableName,
                    display_text: table.table_name.clone(),
                    insert_text: table.table_name.clone(),
                    replacement_span: Some(replacement_span),
                    documentation_ref: None,
                    requires_revalidation: true,
                },
            );
        }
    }

    let library_context_snapshot = resolve_library_context_snapshot(&request);
    if let Some(snapshot) = library_context_snapshot.as_ref() {
        let mut seen_functions = BTreeSet::new();
        for entry in &snapshot.entries {
            if seen_functions.insert(entry.surface_name.to_ascii_lowercase())
                && (normalized_prefix.is_empty()
                    || entry
                        .surface_name
                        .to_ascii_lowercase()
                        .starts_with(&normalized_prefix))
            {
                insert_proposal(
                    &mut proposals,
                    4,
                    CompletionProposal {
                        proposal_id: format!("function:{}", entry.surface_name),
                        proposal_kind: CompletionProposalKind::Function,
                        display_text: entry.surface_name.clone(),
                        insert_text: entry.surface_name.clone(),
                        replacement_span: Some(replacement_span),
                        documentation_ref: entry.interface_contract_ref.clone(),
                        requires_revalidation: true,
                    },
                );
            }
        }
    }

    CompletionResult {
        replacement_span: Some(replacement_span),
        proposals: proposals.into_values().collect(),
        signature_help_context,
    }
}

pub fn build_intelligent_completion_context(
    request: CompletionRequest<'_>,
    diagnostics: &LiveDiagnosticSnapshot,
) -> IntelligentCompletionContext {
    let signature_help_context =
        signature_help_context_at_cursor(request.source, request.green_tree, request.cursor_offset);
    IntelligentCompletionContext {
        formula_text: request.source.entered_formula_text.clone(),
        formula_channel_kind: request.source.formula_channel_kind,
        cursor_offset: request.cursor_offset,
        green_tree_key: request.green_tree.green_tree_key.clone(),
        red_context_summary: red_context_summary(request.red_projection, request.cursor_offset),
        visible_defined_names: request.bind_context.names.keys().cloned().collect(),
        visible_tables: request
            .bind_context
            .table_catalog
            .iter()
            .map(|table| table.table_name.clone())
            .collect(),
        library_context_snapshot_ref: effective_library_context_snapshot_ref(
            request.library_context_provider,
            request.library_context_snapshot_ref,
            request.library_context_snapshot,
        ),
        active_diagnostics: diagnostics
            .diagnostics
            .iter()
            .filter(|diagnostic| spans_touch_cursor(diagnostic.primary_span, request.cursor_offset))
            .cloned()
            .collect(),
        signature_help_context,
    }
}

fn resolve_library_context_snapshot(
    request: &CompletionRequest<'_>,
) -> Option<LibraryContextSnapshot> {
    match (
        request.library_context_provider,
        request.library_context_snapshot_ref,
        request.library_context_snapshot,
    ) {
        (Some(provider), Some(snapshot_ref), _) => provider.snapshot_by_identity(snapshot_ref),
        (_, _, Some(snapshot)) => Some(snapshot.clone()),
        (Some(provider), None, None) => Some(provider.current_snapshot()),
        (None, Some(_), None) => None,
        (None, None, None) => None,
    }
}

fn effective_library_context_snapshot_ref(
    library_context_provider: Option<&dyn LibraryContextProvider>,
    library_context_snapshot_ref: Option<&LibraryContextSnapshotRef>,
    library_context_snapshot: Option<&LibraryContextSnapshot>,
) -> Option<LibraryContextSnapshotRef> {
    library_context_snapshot_ref
        .cloned()
        .or_else(|| library_context_snapshot.map(LibraryContextSnapshotRef::from))
        .or_else(|| {
            library_context_provider.map(|provider| {
                let current_snapshot = provider.current_snapshot();
                LibraryContextSnapshotRef::from(&current_snapshot)
            })
        })
}

pub fn signature_help_context_at_cursor(
    source: &FormulaSourceRecord,
    green_tree: &GreenTreeRoot,
    cursor_offset: usize,
) -> Option<SignatureHelpContext> {
    let call_node = smallest_call_like_node(&green_tree.root, cursor_offset)?;
    let arg_list = call_node.children.iter().find_map(|child| match child {
        GreenChild::Node(node) if node.kind == SyntaxKind::ArgumentList => Some(node.as_ref()),
        _ => None,
    })?;

    Some(SignatureHelpContext {
        callee_text: callee_text_for_call(source, call_node),
        call_span: call_node.span,
        active_argument_index: active_argument_index(arg_list, cursor_offset),
        invocation_kind: call_node.kind,
    })
}

fn editor_trivia_kind(trivia_kind: SyntaxTriviaKind) -> EditorTriviaKind {
    match trivia_kind {
        SyntaxTriviaKind::Whitespace => EditorTriviaKind::Whitespace,
        SyntaxTriviaKind::Unknown => EditorTriviaKind::Unknown,
    }
}

fn collect_editor_tokens(node: &GreenNode, tokens: &mut Vec<EditorToken>) {
    for child in &node.children {
        match child {
            GreenChild::Node(child_node) => collect_editor_tokens(child_node, tokens),
            GreenChild::Token(token) if token.kind != TokenKind::Eof && !token.kind.is_trivia() => {
                tokens.push(EditorToken {
                    kind: token.kind,
                    text: token.text.clone(),
                    leading_trivia: token
                        .leading_trivia
                        .iter()
                        .map(editor_trivia_from_syntax)
                        .collect(),
                    trailing_trivia: token
                        .trailing_trivia
                        .iter()
                        .map(editor_trivia_from_syntax)
                        .collect(),
                    span: token.span,
                });
            }
            GreenChild::Token(_) => {}
        }
    }
}

fn editor_trivia_from_syntax(trivia: &SyntaxTrivia) -> EditorTrivia {
    EditorTrivia {
        kind: editor_trivia_kind(trivia.kind),
        text: trivia.text.clone(),
    }
}

fn suggested_fix_kind_for_bind_message(message: &str) -> Option<String> {
    if message.contains("enclosing table context") {
        Some("supply_enclosing_table_context".to_string())
    } else if message.contains("#This Row must not be combined") {
        Some("remove_illegal_this_row_combination".to_string())
    } else {
        None
    }
}

fn completion_prefix(text: &str, cursor_offset: usize) -> (TextSpan, String) {
    let chars: Vec<char> = text.chars().collect();
    let cursor = cursor_offset.min(chars.len());
    let mut start = cursor;

    while start > 0 {
        let ch = chars[start - 1];
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '#' | '.') {
            start -= 1;
        } else {
            break;
        }
    }

    (
        TextSpan::new(start, cursor.saturating_sub(start)),
        chars[start..cursor].iter().collect(),
    )
}

fn is_structured_selector_context(text: &str, replacement_span: TextSpan) -> bool {
    let chars: Vec<char> = text.chars().collect();
    replacement_span.start > 0 && chars[replacement_span.start - 1] == '['
}

fn structured_column_table_name(text: &str, replacement_span: TextSpan) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if replacement_span.start == 0 || chars[replacement_span.start - 1] != '[' {
        return None;
    }
    let bracket_index = replacement_span.start - 1;
    let mut table_end = bracket_index;
    while table_end > 0 && chars[table_end - 1].is_ascii_whitespace() {
        table_end -= 1;
    }
    let mut table_start = table_end;
    while table_start > 0 {
        let ch = chars[table_start - 1];
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.') {
            table_start -= 1;
        } else {
            break;
        }
    }
    if table_start == table_end {
        None
    } else {
        Some(chars[table_start..table_end].iter().collect())
    }
}

fn insert_proposal(
    proposals: &mut BTreeMap<(u8, String), CompletionProposal>,
    bucket: u8,
    proposal: CompletionProposal,
) {
    proposals.insert(
        (bucket, proposal.display_text.to_ascii_lowercase()),
        proposal,
    );
}

fn smallest_call_like_node<'a>(node: &'a GreenNode, cursor_offset: usize) -> Option<&'a GreenNode> {
    if !span_contains(node.span, cursor_offset) {
        return None;
    }

    let mut best =
        matches!(node.kind, SyntaxKind::CallExpr | SyntaxKind::InvokeExpr).then_some(node);

    for child in &node.children {
        if let GreenChild::Node(child_node) = child {
            if let Some(candidate) = smallest_call_like_node(child_node, cursor_offset) {
                best = match best {
                    Some(current) if current.span.len <= candidate.span.len => Some(current),
                    _ => Some(candidate),
                };
            }
        }
    }

    best
}

fn callee_text_for_call(source: &FormulaSourceRecord, call_node: &GreenNode) -> String {
    match call_node.kind {
        SyntaxKind::CallExpr => call_node
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Token(token) => Some(token.text.clone()),
                _ => None,
            })
            .unwrap_or_default(),
        SyntaxKind::InvokeExpr => call_node
            .children
            .iter()
            .find_map(|child| match child {
                GreenChild::Node(node) => {
                    Some(text_for_span(&source.entered_formula_text, node.span))
                }
                _ => None,
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn active_argument_index(arg_list: &GreenNode, cursor_offset: usize) -> usize {
    let mut index = 0usize;
    for child in &arg_list.children {
        if let GreenChild::Token(token) = child {
            if token.kind == TokenKind::Comma && token.span.start < cursor_offset {
                index += 1;
            }
        }
    }
    index
}

fn red_context_summary(red_projection: &RedProjection, cursor_offset: usize) -> String {
    let mut best_index: Option<usize> = None;
    for node in &red_projection.nodes {
        if span_contains(node.span, cursor_offset) {
            best_index = match best_index {
                Some(current) => {
                    let current_node = &red_projection.nodes[current];
                    if node.span.len < current_node.span.len {
                        Some(node.index)
                    } else {
                        Some(current)
                    }
                }
                None => Some(node.index),
            };
        }
    }

    if let Some(index) = best_index {
        let node = &red_projection.nodes[index];
        format!(
            "kind={:?};depth={}",
            node.kind,
            node_depth(red_projection, index)
        )
    } else {
        "kind=<none>;depth=0".to_string()
    }
}

fn node_depth(red_projection: &RedProjection, mut index: usize) -> usize {
    let mut depth = 0usize;
    while let Some(parent) = red_projection.nodes[index].parent {
        depth += 1;
        index = parent;
    }
    depth
}

fn span_contains(span: TextSpan, cursor_offset: usize) -> bool {
    cursor_offset >= span.start && cursor_offset <= span.end()
}

fn spans_touch_cursor(span: TextSpan, cursor_offset: usize) -> bool {
    cursor_offset >= span.start.saturating_sub(1) && cursor_offset <= span.end().saturating_add(1)
}

fn text_for_span(text: &str, span: TextSpan) -> String {
    text.chars().skip(span.start).take(span.len).collect()
}

fn compute_text_change_range(
    previous_text: &str,
    new_text: &str,
) -> Option<FormulaTextChangeRange> {
    if previous_text == new_text {
        return None;
    }

    let previous_chars: Vec<char> = previous_text.chars().collect();
    let new_chars: Vec<char> = new_text.chars().collect();

    let mut prefix = 0usize;
    while prefix < previous_chars.len()
        && prefix < new_chars.len()
        && previous_chars[prefix] == new_chars[prefix]
    {
        prefix += 1;
    }

    let mut previous_suffix = previous_chars.len();
    let mut new_suffix = new_chars.len();
    while previous_suffix > prefix
        && new_suffix > prefix
        && previous_chars[previous_suffix - 1] == new_chars[new_suffix - 1]
    {
        previous_suffix -= 1;
        new_suffix -= 1;
    }

    Some(FormulaTextChangeRange {
        start: prefix,
        old_len: previous_suffix.saturating_sub(prefix),
        new_len: new_suffix.saturating_sub(prefix),
    })
}

fn apply_text_replacement(text: &str, span: TextSpan, insert_text: &str) -> String {
    let before: String = text.chars().take(span.start).collect();
    let after: String = text.chars().skip(span.end()).collect();
    format!("{before}{insert_text}{after}")
}

fn green_tree_text(green_tree: &GreenTreeRoot) -> String {
    green_tree
        .full_fidelity_tokens
        .iter()
        .filter(|token| token.kind != TokenKind::Eof)
        .map(|token| token.text.as_str())
        .collect()
}
