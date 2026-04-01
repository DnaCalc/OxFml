use crate::binding::BindContext;
use crate::consumer::ConsumerLibraryContextState;
use crate::interface::LibraryContextProvider;
use crate::language_service::{
    CompletionRequest, CompletionValidationRequest, CompletionValidationResult, FormulaEditRequest,
    FormulaEditResult, apply_completion_proposal, apply_formula_edit,
    build_function_help_lookup_request, build_intelligent_completion_context,
    collect_completion_proposals, signature_help_context_at_cursor, validate_completion_candidate,
};
use crate::semantics::LibraryContextSnapshot;
use crate::source::FormulaSourceRecord;

mod types;

pub use types::{
    CompletionProposal, CompletionProposalKind, CompletionResult, EditorAnalysisStage,
    EditorDocument, EditorPlanOptions, EditorSyntaxSnapshot, EditorToken, EditorTrivia,
    EditorTriviaKind, FormulaEditReuseSummary, FormulaTextChangeRange, FunctionHelpPacket,
    FunctionHelpSignatureForm, IntelligentCompletionContext, LiveDiagnostic,
    LiveDiagnosticSeverity, LiveDiagnosticSnapshot, LiveDiagnosticStage, SignatureHelpContext,
};

#[derive(Clone)]
pub struct EditorEnvironment<'a> {
    bind_context: BindContext,
    library_context: ConsumerLibraryContextState<'a>,
}

impl EditorEnvironment<'_> {
    pub fn new(bind_context: BindContext) -> Self {
        Self {
            bind_context,
            library_context: ConsumerLibraryContextState::new(),
        }
    }
}

impl<'a> EditorEnvironment<'a> {
    pub fn bind_context(&self) -> &BindContext {
        &self.bind_context
    }

    pub fn with_bind_context(mut self, bind_context: BindContext) -> Self {
        self.bind_context = bind_context;
        self
    }

    pub fn with_library_context_provider(
        mut self,
        provider: &'a dyn LibraryContextProvider,
    ) -> Self {
        self.library_context.provider = Some(provider);
        self
    }

    pub fn with_library_context_snapshot_ref(
        mut self,
        snapshot_ref: crate::interface::LibraryContextSnapshotRef,
    ) -> Self {
        self.library_context.snapshot_ref = Some(snapshot_ref);
        self.library_context.snapshot = None;
        self
    }

    pub fn with_inline_library_context_snapshot(
        mut self,
        snapshot: LibraryContextSnapshot,
    ) -> Self {
        self.library_context.snapshot = Some(snapshot);
        self.library_context.snapshot_ref = None;
        self
    }

    pub fn with_pinned_library_context(
        self,
        provider: &'a dyn LibraryContextProvider,
        snapshot_ref: crate::interface::LibraryContextSnapshotRef,
    ) -> Self {
        self.with_library_context_provider(provider)
            .with_library_context_snapshot_ref(snapshot_ref)
    }

    pub fn with_resolved_library_context(
        mut self,
        provider: Option<&'a dyn LibraryContextProvider>,
        snapshot_ref: Option<crate::interface::LibraryContextSnapshotRef>,
        snapshot: Option<LibraryContextSnapshot>,
    ) -> Self {
        self.library_context =
            ConsumerLibraryContextState::from_parts(provider, snapshot_ref, snapshot);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorInteractionResult {
    pub document: EditorDocument,
    pub completion_result: Option<CompletionResult>,
    pub signature_help_context: Option<SignatureHelpContext>,
    pub function_help_packet: Option<FunctionHelpPacket>,
    pub intelligent_completion_context: Option<IntelligentCompletionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCompletionApplicationResult {
    pub proposal_id: Option<String>,
    pub applied_span: crate::syntax::token::TextSpan,
    pub interaction_result: EditorInteractionResult,
}

pub struct EditorEditService<'a> {
    environment: EditorEnvironment<'a>,
}

impl<'a> EditorEditService<'a> {
    pub fn new(environment: EditorEnvironment<'a>) -> Self {
        Self { environment }
    }

    pub fn open_document(
        &self,
        source: FormulaSourceRecord,
        plan_options: Option<EditorPlanOptions>,
    ) -> EditorDocument {
        self.apply_edit(
            source,
            None,
            EditorAnalysisStage::FullSemanticPlan,
            plan_options,
        )
        .document
    }

    pub fn open_and_interact(
        &self,
        source: FormulaSourceRecord,
        cursor_offset: usize,
        plan_options: Option<EditorPlanOptions>,
    ) -> EditorInteractionResult {
        let document = self.open_document(source, plan_options);
        self.interact_at_cursor(&document, cursor_offset)
    }

    pub fn apply_edit(
        &self,
        source: FormulaSourceRecord,
        previous_document: Option<&EditorDocument>,
        analysis_stage: EditorAnalysisStage,
        plan_options: Option<EditorPlanOptions>,
    ) -> EditorInteractionResult {
        let edit_result = apply_formula_edit(FormulaEditRequest {
            source,
            bind_context: self.environment.bind_context().clone(),
            previous_green_tree: previous_document.map(|document| &document.green_tree),
            previous_red_projection: previous_document.map(|document| &document.red_projection),
            previous_bound_formula: previous_document
                .and_then(|document| document.bound_formula.as_ref()),
            analysis_stage,
            plan_options,
        });

        EditorInteractionResult {
            document: document_from_edit_result(edit_result),
            completion_result: None,
            signature_help_context: None,
            function_help_packet: None,
            intelligent_completion_context: None,
        }
    }

    pub fn interact_at_cursor(
        &self,
        document: &EditorDocument,
        cursor_offset: usize,
    ) -> EditorInteractionResult {
        let completion_result = collect_completion_proposals(CompletionRequest {
            source: &document.source,
            green_tree: &document.green_tree,
            red_projection: &document.red_projection,
            bind_context: self.environment.bind_context(),
            library_context: self.environment.library_context.pinned_view(),
            cursor_offset,
        });
        let signature_help_context =
            signature_help_context_at_cursor(&document.source, &document.green_tree, cursor_offset);
        let function_help_packet = build_function_help_lookup_request(
            &document.source,
            &document.green_tree,
            cursor_offset,
            self.environment.library_context.pinned_view(),
        )
        .as_ref()
        .map(|request| {
            build_function_help_packet(request, &self.environment, signature_help_context.as_ref())
        });
        let intelligent_completion_context = Some(build_intelligent_completion_context(
            CompletionRequest {
                source: &document.source,
                green_tree: &document.green_tree,
                red_projection: &document.red_projection,
                bind_context: self.environment.bind_context(),
                library_context: self.environment.library_context.pinned_view(),
                cursor_offset,
            },
            &document.live_diagnostics,
        ));

        EditorInteractionResult {
            document: document.clone(),
            completion_result: Some(completion_result),
            signature_help_context,
            function_help_packet,
            intelligent_completion_context,
        }
    }

    pub fn completion_at_cursor(
        &self,
        document: &EditorDocument,
        cursor_offset: usize,
    ) -> CompletionResult {
        self.interact_at_cursor(document, cursor_offset)
            .completion_result
            .expect("completion should always be produced for an editor interaction")
    }

    pub fn function_help_at_cursor(
        &self,
        document: &EditorDocument,
        cursor_offset: usize,
    ) -> Option<FunctionHelpPacket> {
        self.interact_at_cursor(document, cursor_offset)
            .function_help_packet
    }

    pub fn signature_help_at_cursor(
        &self,
        document: &EditorDocument,
        cursor_offset: usize,
    ) -> Option<SignatureHelpContext> {
        self.interact_at_cursor(document, cursor_offset)
            .signature_help_context
    }

    pub fn intelligent_completion_context_at_cursor(
        &self,
        document: &EditorDocument,
        cursor_offset: usize,
    ) -> IntelligentCompletionContext {
        self.interact_at_cursor(document, cursor_offset)
            .intelligent_completion_context
            .expect("intelligent completion context should be produced for an editor interaction")
    }

    pub fn validate_completion(
        &self,
        document: &EditorDocument,
        replacement_span: Option<crate::syntax::token::TextSpan>,
        insert_text: impl Into<String>,
        analysis_stage: EditorAnalysisStage,
        plan_options: Option<EditorPlanOptions>,
    ) -> EditorCompletionApplicationResult {
        let validation = validate_completion_candidate(CompletionValidationRequest {
            source: document.source.clone(),
            bind_context: self.environment.bind_context().clone(),
            previous_green_tree: Some(&document.green_tree),
            previous_red_projection: Some(&document.red_projection),
            previous_bound_formula: document.bound_formula.as_ref(),
            replacement_span,
            insert_text: insert_text.into(),
            analysis_stage,
            plan_options,
        });
        completion_application_from_validation(self, validation)
    }

    pub fn apply_completion_proposal(
        &self,
        document: &EditorDocument,
        proposal: &CompletionProposal,
        analysis_stage: EditorAnalysisStage,
        plan_options: Option<EditorPlanOptions>,
    ) -> EditorCompletionApplicationResult {
        let validation = apply_completion_proposal(
            CompletionValidationRequest {
                source: document.source.clone(),
                bind_context: self.environment.bind_context().clone(),
                previous_green_tree: Some(&document.green_tree),
                previous_red_projection: Some(&document.red_projection),
                previous_bound_formula: document.bound_formula.as_ref(),
                replacement_span: None,
                insert_text: String::new(),
                analysis_stage,
                plan_options,
            },
            proposal,
        );
        completion_application_from_validation(self, validation)
    }
}

fn build_function_help_packet(
    request: &crate::language_service::FunctionHelpLookupRequest,
    environment: &EditorEnvironment<'_>,
    signature_help_context: Option<&SignatureHelpContext>,
) -> FunctionHelpPacket {
    let snapshot_entry = lookup_snapshot_entry(environment, request);
    let display_name = snapshot_entry
        .as_ref()
        .map(|entry| entry.surface_name.clone())
        .unwrap_or_else(|| request.lookup_key.clone());
    let (min_arity, max_arity, signature_suffix) = snapshot_entry
        .as_ref()
        .and_then(|entry| entry.arity_shape_note.as_deref())
        .map(parse_arity_shape_note)
        .unwrap_or((0, None, "...".to_string()));

    let availability_summary = snapshot_entry.as_ref().map(|entry| {
        let mut parts = vec![
            format!("parse_bind={:?}", entry.parse_bind_state),
            format!("semantic_plan={:?}", entry.semantic_plan_state),
        ];
        if let Some(runtime_capability_state) = entry.runtime_capability_state {
            parts.push(format!("runtime={runtime_capability_state:?}"));
        }
        if let Some(post_dispatch_state) = entry.post_dispatch_state {
            parts.push(format!("post_dispatch={post_dispatch_state:?}"));
        }
        parts.join("; ")
    });
    let deferred_or_profile_limited = snapshot_entry.as_ref().is_some_and(|entry| {
        entry.parse_bind_state != crate::semantics::LibraryAvailabilityState::CatalogKnown
            || entry.semantic_plan_state != crate::semantics::LibraryAvailabilityState::CatalogKnown
            || entry.runtime_capability_state
                != Some(crate::semantics::LibraryAvailabilityState::CatalogKnown)
            || entry.post_dispatch_state.is_some()
    });

    FunctionHelpPacket {
        lookup_key: request.lookup_key.clone(),
        library_context_snapshot_ref: request.library_context_snapshot_ref.clone(),
        display_name: display_name.clone(),
        signature_forms: vec![FunctionHelpSignatureForm {
            display_signature: format!("{display_name}({signature_suffix})"),
            min_arity,
            max_arity,
        }],
        argument_help: build_argument_help(min_arity, max_arity, signature_help_context),
        short_description: snapshot_entry
            .as_ref()
            .and_then(|entry| entry.interface_contract_ref.clone()),
        availability_summary,
        deferred_or_profile_limited,
    }
}

fn lookup_snapshot_entry(
    environment: &EditorEnvironment<'_>,
    request: &crate::language_service::FunctionHelpLookupRequest,
) -> Option<crate::semantics::LibraryContextSnapshotEntry> {
    if let (Some(provider), Some(snapshot_ref)) = (
        environment.library_context.provider,
        request.library_context_snapshot_ref.as_ref(),
    ) {
        if let Some(entry) = crate::interface::LibraryContextProvider::lookup_surface(
            provider,
            snapshot_ref,
            &request.lookup_key,
        ) {
            return Some(entry);
        }
    }

    environment
        .library_context
        .snapshot
        .as_ref()
        .and_then(|snapshot| {
            snapshot
                .entries
                .iter()
                .find(|entry| entry.surface_name.eq_ignore_ascii_case(&request.lookup_key))
                .cloned()
        })
}

fn parse_arity_shape_note(note: &str) -> (usize, Option<usize>, String) {
    if note.eq_ignore_ascii_case("variadic") {
        return (1, None, "...".to_string());
    }
    if let Some(prefix) = note.strip_suffix('+') {
        if let Ok(min_arity) = prefix.parse::<usize>() {
            return (min_arity, None, signature_suffix(min_arity, None));
        }
    }
    if let Some((start, end)) = note.split_once("..") {
        if let Ok(min_arity) = start.parse::<usize>() {
            let max_arity = if end == "*" {
                None
            } else {
                end.parse::<usize>().ok()
            };
            return (min_arity, max_arity, signature_suffix(min_arity, max_arity));
        }
    }
    if let Ok(arity) = note.parse::<usize>() {
        return (arity, Some(arity), signature_suffix(arity, Some(arity)));
    }
    (0, None, note.to_string())
}

fn signature_suffix(min_arity: usize, max_arity: Option<usize>) -> String {
    let max_display = max_arity.unwrap_or(min_arity.max(3));
    let mut parts = Vec::new();
    for index in 0..max_display {
        parts.push(format!("arg{}", index + 1));
    }
    if max_arity.is_none() {
        parts.push("...".to_string());
    }
    parts.join(", ")
}

fn build_argument_help(
    min_arity: usize,
    max_arity: Option<usize>,
    signature_help_context: Option<&SignatureHelpContext>,
) -> Vec<String> {
    let max_display = max_arity.unwrap_or(min_arity.max(3));
    let mut args = (0..max_display)
        .map(|index| format!("arg{}", index + 1))
        .collect::<Vec<_>>();
    if max_arity.is_none() && min_arity > 0 {
        args.push("additional_args".to_string());
    }
    if let Some(context) = signature_help_context {
        if let Some(active) = args.get_mut(context.active_argument_index) {
            *active = format!("*{active}");
        }
    }
    args
}

fn document_from_edit_result(edit_result: FormulaEditResult) -> EditorDocument {
    EditorDocument {
        source: edit_result.source,
        text_change_range: edit_result.text_change_range,
        editor_syntax_snapshot: edit_result.editor_syntax_snapshot,
        green_tree: edit_result.green_tree,
        red_projection: edit_result.red_projection,
        bound_formula: edit_result.bound_formula,
        semantic_plan: edit_result.semantic_plan,
        live_diagnostics: edit_result.live_diagnostics,
        reuse_summary: edit_result.reuse_summary,
    }
}

fn completion_application_from_validation(
    service: &EditorEditService<'_>,
    validation: CompletionValidationResult,
) -> EditorCompletionApplicationResult {
    let document = document_from_edit_result(validation.edit_result);
    let cursor_offset = validation.applied_span.start + validation.applied_span.len;
    let interaction_result = service.interact_at_cursor(&document, cursor_offset);

    EditorCompletionApplicationResult {
        proposal_id: validation.proposal_id,
        applied_span: validation.applied_span,
        interaction_result,
    }
}
