use crate::binding::ProfilePayload;
use crate::semantics::LibraryContextSnapshot;
use crate::source::{FormulaChannelKind, FormulaSourceRecord};
use crate::syntax::green::SyntaxKind;
use crate::syntax::token::{TextSpan, TokenKind};

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

/// Full-fidelity editor token projection for a formula source.
///
/// Invariant for ordinary consumers: concatenating each token's
/// `leading_trivia` text, then token `text`, in source order, plus the
/// final token's `trailing_trivia`, reproduces the original source text
/// character-for-character. The projection is built from the lexer token
/// stream rather than semantic parse ownership so editor rendering does not
/// drop characters that are skipped, syntax-significant, or invalid in the
/// parser.
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
    pub worksheet_error_class: Option<String>,
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
pub enum EditorAnalysisStage {
    SyntaxOnly,
    SyntaxAndBind,
    FullSemanticPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorPlanOptions {
    pub oxfunc_catalog_identity: String,
    pub locale_profile: Option<String>,
    pub date_system: Option<String>,
    pub format_profile: Option<String>,
    pub library_context_snapshot: Option<LibraryContextSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormulaEditReuseSummary {
    pub reused_green_tree: bool,
    pub reused_red_projection: bool,
    pub reused_bound_formula: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionProposalKind {
    Function,
    DefinedName,
    TableName,
    TableColumn,
    StructuredSelector,
    SyntaxAssist,
    ProfileReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionProposal {
    pub proposal_id: String,
    pub proposal_kind: CompletionProposalKind,
    pub display_text: String,
    pub insert_text: String,
    pub replacement_span: Option<TextSpan>,
    pub documentation_ref: Option<String>,
    pub profile_payload: Option<ProfilePayload>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHelpSignatureForm {
    pub display_signature: String,
    pub min_arity: usize,
    pub max_arity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHelpPacket {
    pub lookup_key: String,
    pub library_context_snapshot_ref: Option<crate::interface::LibraryContextSnapshotRef>,
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
    pub library_context_snapshot_ref: Option<crate::interface::LibraryContextSnapshotRef>,
    pub active_diagnostics: Vec<LiveDiagnostic>,
    pub signature_help_context: Option<SignatureHelpContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDocument {
    pub source: FormulaSourceRecord,
    pub text_change_range: Option<FormulaTextChangeRange>,
    pub editor_syntax_snapshot: EditorSyntaxSnapshot,
    pub green_tree: crate::syntax::green::GreenTreeRoot,
    pub red_projection: crate::red::RedProjection,
    pub bound_formula: Option<crate::binding::BoundFormula>,
    pub semantic_plan: Option<crate::semantics::SemanticPlan>,
    pub live_diagnostics: LiveDiagnosticSnapshot,
    pub reuse_summary: FormulaEditReuseSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorHostReferenceTarget {
    HostName {
        canonical_name: String,
    },
    HostReferenceCollection {
        base_canonical_name: Option<String>,
        collection_family: String,
    },
    HostStructuralSelector {
        base_canonical_name: String,
        selector_family: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorHostReferenceInsertionRequest {
    pub target: EditorHostReferenceTarget,
    pub replacement_span: Option<TextSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorHostReferenceInsertionResult {
    pub inserted_text: String,
    pub applied_span: TextSpan,
    pub interaction_result: super::EditorInteractionResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorHostReferenceInsertionError {
    EmptyHostName,
    EmptySelectorFamily,
    UnknownCollectionFamily(String),
    UnknownStructuralSelectorFamily(String),
}
