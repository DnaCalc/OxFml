use crate::binding::{BindContext, ReferenceBindProfile};
use crate::consumer::ConsumerLibraryContextState;
use crate::interface::LibraryContextProvider;
use crate::language_service::{
    CompletionRequest, CompletionValidationRequest, CompletionValidationResult, FormulaEditRequest,
    FormulaEditResult, apply_completion_proposal, apply_formula_edit,
    build_function_help_lookup_request, build_intelligent_completion_context,
    build_profile_reference_info_at_cursor, collect_completion_proposals,
    signature_help_context_at_cursor, validate_completion_candidate,
};
use crate::semantics::LibraryContextSnapshot;
use crate::source::{FormulaChannelKind, FormulaSourceRecord};
use oxfunc_core::registry::{
    CapabilityOverlay, FunctionAvailability, FunctionEntry, FunctionRegistry, SignatureForm,
    builtin_registry,
};

mod types;

pub use types::{
    CompletionProposal, CompletionProposalKind, CompletionResult, EditorAnalysisStage,
    EditorDocument, EditorHostReferenceInsertionError, EditorHostReferenceInsertionRequest,
    EditorHostReferenceInsertionResult, EditorHostReferenceTarget, EditorPlanOptions,
    EditorReferenceInfo, EditorSyntaxSnapshot, EditorToken, EditorTrivia, EditorTriviaKind,
    FormulaEditReuseSummary, FormulaTextChangeRange, FunctionHelpPacket, FunctionHelpSignatureForm,
    IntelligentCompletionContext, LiveDiagnostic, LiveDiagnosticSeverity, LiveDiagnosticSnapshot,
    LiveDiagnosticStage, SignatureHelpContext,
};

#[derive(Clone)]
pub struct EditorEnvironment<'a> {
    bind_context: BindContext,
    library_context: ConsumerLibraryContextState<'a>,
    function_registry: &'a FunctionRegistry,
    capability_overlay: Option<&'a CapabilityOverlay>,
    reference_bind_profile: Option<&'a dyn ReferenceBindProfile>,
}

impl EditorEnvironment<'_> {
    pub fn new(bind_context: BindContext) -> Self {
        Self {
            bind_context,
            library_context: ConsumerLibraryContextState::new(),
            function_registry: builtin_registry(),
            capability_overlay: None,
            reference_bind_profile: None,
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

    pub fn with_function_registry(mut self, function_registry: &'a FunctionRegistry) -> Self {
        self.function_registry = function_registry;
        self
    }

    pub fn with_capability_overlay(mut self, capability_overlay: &'a CapabilityOverlay) -> Self {
        self.capability_overlay = Some(capability_overlay);
        self
    }

    pub fn with_reference_bind_profile(
        mut self,
        reference_bind_profile: &'a dyn ReferenceBindProfile,
    ) -> Self {
        self.reference_bind_profile = Some(reference_bind_profile);
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
    pub reference_info: Option<EditorReferenceInfo>,
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
            reference_bind_profile: self.environment.reference_bind_profile,
            analysis_stage,
            plan_options,
        });

        EditorInteractionResult {
            document: document_from_edit_result(edit_result),
            completion_result: None,
            signature_help_context: None,
            function_help_packet: None,
            intelligent_completion_context: None,
            reference_info: None,
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
            function_registry: self.environment.function_registry,
            capability_overlay: self.environment.capability_overlay,
            reference_bind_profile: self.environment.reference_bind_profile,
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
        .and_then(|request| {
            build_function_help_packet(request, &self.environment, signature_help_context.as_ref())
        });
        let intelligent_completion_context = Some(build_intelligent_completion_context(
            CompletionRequest {
                source: &document.source,
                green_tree: &document.green_tree,
                red_projection: &document.red_projection,
                bind_context: self.environment.bind_context(),
                library_context: self.environment.library_context.pinned_view(),
                function_registry: self.environment.function_registry,
                capability_overlay: self.environment.capability_overlay,
                reference_bind_profile: self.environment.reference_bind_profile,
                cursor_offset,
            },
            &document.live_diagnostics,
        ));
        let reference_info = self.reference_info_at_cursor(document, cursor_offset, None);

        EditorInteractionResult {
            document: document.clone(),
            completion_result: Some(completion_result),
            signature_help_context,
            function_help_packet,
            intelligent_completion_context,
            reference_info,
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

    pub fn reference_info_at_cursor(
        &self,
        document: &EditorDocument,
        cursor_offset: usize,
        target_channel: Option<FormulaChannelKind>,
    ) -> Option<EditorReferenceInfo> {
        build_profile_reference_info_at_cursor(
            &document.source,
            document.bound_formula.as_ref(),
            self.environment.reference_bind_profile,
            cursor_offset,
            target_channel.unwrap_or(document.source.formula_channel_kind),
        )
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
            reference_bind_profile: self.environment.reference_bind_profile,
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
                reference_bind_profile: self.environment.reference_bind_profile,
                replacement_span: None,
                insert_text: String::new(),
                analysis_stage,
                plan_options,
            },
            proposal,
        );
        completion_application_from_validation(self, validation)
    }

    pub fn insert_host_reference(
        &self,
        document: &EditorDocument,
        request: EditorHostReferenceInsertionRequest,
        analysis_stage: EditorAnalysisStage,
        plan_options: Option<EditorPlanOptions>,
    ) -> Result<EditorHostReferenceInsertionResult, EditorHostReferenceInsertionError> {
        let inserted_text = compose_host_reference_text(
            &request.target,
            &self.environment.bind_context.host_reference_syntax,
        )?;
        let application = self.validate_completion(
            document,
            request.replacement_span,
            inserted_text.clone(),
            analysis_stage,
            plan_options,
        );

        Ok(EditorHostReferenceInsertionResult {
            inserted_text,
            applied_span: application.applied_span,
            interaction_result: application.interaction_result,
        })
    }
}

fn compose_host_reference_text(
    target: &EditorHostReferenceTarget,
    host_reference_syntax: &crate::syntax::parser::HostReferenceSyntaxProfile,
) -> Result<String, EditorHostReferenceInsertionError> {
    match target {
        EditorHostReferenceTarget::HostName { canonical_name } => compose_host_name(canonical_name),
        EditorHostReferenceTarget::HostReferenceCollection {
            base_canonical_name,
            collection_family,
        } => {
            if collection_family.is_empty() {
                return Err(EditorHostReferenceInsertionError::EmptySelectorFamily);
            }
            let token = host_reference_syntax
                .collection_members
                .iter()
                .find(|member| member.collection_family == *collection_family)
                .map(|member| member.token_text.as_str())
                .ok_or_else(|| {
                    EditorHostReferenceInsertionError::UnknownCollectionFamily(
                        collection_family.clone(),
                    )
                })?;
            let selector_text = compose_member_selector_token(token);
            match base_canonical_name {
                Some(base) => Ok(format!("{}.{}", compose_host_name(base)?, selector_text)),
                None => Ok(compose_owner_relative_selector_token(token)),
            }
        }
        EditorHostReferenceTarget::HostStructuralSelector {
            base_canonical_name,
            selector_family,
        } => {
            if selector_family.is_empty() {
                return Err(EditorHostReferenceInsertionError::EmptySelectorFamily);
            }
            let token = host_reference_syntax
                .structural_selectors
                .iter()
                .find(|selector| selector.selector_family == *selector_family)
                .map(|selector| selector.token_text.as_str())
                .ok_or_else(|| {
                    EditorHostReferenceInsertionError::UnknownStructuralSelectorFamily(
                        selector_family.clone(),
                    )
                })?;
            Ok(format!(
                "{}.{}",
                compose_host_name(base_canonical_name)?,
                compose_member_selector_token(token)
            ))
        }
    }
}

fn compose_owner_relative_selector_token(token: &str) -> String {
    if token == "*" {
        ".*".to_string()
    } else {
        compose_member_selector_token(token)
    }
}

fn compose_member_selector_token(token: &str) -> String {
    if token == "*" || token.starts_with('@') {
        token.to_string()
    } else {
        format!("@{token}")
    }
}

fn compose_host_name(name: &str) -> Result<String, EditorHostReferenceInsertionError> {
    if name.is_empty() {
        return Err(EditorHostReferenceInsertionError::EmptyHostName);
    }
    if is_plain_host_name_identifier(name) && !is_formula_literal_identifier(name) {
        Ok(name.to_string())
    } else {
        Ok(format!("[{}]", escape_bracketed_host_reference_text(name)))
    }
}

fn is_plain_host_name_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || matches!(first, '_' | '$')) {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$'))
}

fn is_formula_literal_identifier(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "TRUE"
            | "FALSE"
            | "#NULL!"
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

fn escape_bracketed_host_reference_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(ch, '#' | '[' | ']' | '@' | '\'') {
            output.push('\'');
        }
        output.push(ch);
    }
    output
}

fn build_function_help_packet(
    request: &crate::language_service::FunctionHelpLookupRequest,
    environment: &EditorEnvironment<'_>,
    signature_help_context: Option<&SignatureHelpContext>,
) -> Option<FunctionHelpPacket> {
    let registry_entry = lookup_registry_entry(environment, request)?;
    let snapshot_entry = lookup_snapshot_entry(environment, request);
    let display_name = registry_entry.entry.surface_name.clone();
    let min_arity = registry_entry.entry.meta.arity.min;
    let max_arity = Some(registry_entry.entry.meta.arity.max);

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
    let snapshot_limited = snapshot_entry.as_ref().is_some_and(|entry| {
        entry.parse_bind_state != crate::semantics::LibraryAvailabilityState::CatalogKnown
            || entry.semantic_plan_state != crate::semantics::LibraryAvailabilityState::CatalogKnown
            || entry.runtime_capability_state
                != Some(crate::semantics::LibraryAvailabilityState::CatalogKnown)
            || entry.post_dispatch_state.is_some()
    });
    let registry_limited = matches!(
        registry_entry.availability,
        FunctionAvailability::Unavailable { .. }
    );

    Some(FunctionHelpPacket {
        lookup_key: request.lookup_key.clone(),
        library_context_snapshot_ref: request.library_context_snapshot_ref.clone(),
        display_name: display_name.clone(),
        signature_forms: vec![FunctionHelpSignatureForm {
            display_signature: registry_entry
                .entry
                .display_signature
                .signature_display
                .clone(),
            min_arity,
            max_arity,
        }],
        argument_help: argument_help_from_signature(
            &registry_entry.entry.display_signature,
            signature_help_context,
        ),
        short_description: registry_entry.entry.short_description.clone().or_else(|| {
            snapshot_entry
                .as_ref()
                .and_then(|entry| entry.interface_contract_ref.clone())
        }),
        availability_summary: availability_summary.or_else(|| {
            if let FunctionAvailability::Unavailable { reason } = registry_entry.availability {
                Some(format!("registry_capability=Unavailable({reason})"))
            } else {
                None
            }
        }),
        deferred_or_profile_limited: snapshot_limited || registry_limited,
    })
}

struct RegistryHelpEntry<'a> {
    entry: &'a FunctionEntry,
    availability: FunctionAvailability,
}

fn lookup_registry_entry<'a>(
    environment: &'a EditorEnvironment<'a>,
    request: &crate::language_service::FunctionHelpLookupRequest,
) -> Option<RegistryHelpEntry<'a>> {
    if let Some(overlay) = environment.capability_overlay {
        let scoped = environment
            .function_registry
            .with_capability_overlay(overlay)
            .lookup_by_surface_name(&request.lookup_key)?;
        return Some(RegistryHelpEntry {
            entry: scoped.entry,
            availability: scoped.availability,
        });
    }

    environment
        .function_registry
        .lookup_by_surface_name(&request.lookup_key)
        .map(|entry| RegistryHelpEntry {
            entry,
            availability: FunctionAvailability::Available,
        })
}

fn argument_help_from_signature(
    signature: &SignatureForm,
    signature_help_context: Option<&SignatureHelpContext>,
) -> Vec<String> {
    signature
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            let mut label = parameter.name.clone();
            if parameter.repeats {
                label.push_str("...");
            }
            if signature_help_context.is_some_and(|context| context.active_argument_index == index)
            {
                label = format!("*{label}");
            }
            label
        })
        .collect()
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
