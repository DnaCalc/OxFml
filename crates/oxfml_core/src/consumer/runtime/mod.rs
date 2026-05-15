use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use oxfunc_core::registry::builtin_registry;
use oxfunc_core::value::EvalValue;

use crate::binding::{
    BindContext, BindDiagnostic, BoundFormula, NameKind, NormalizedReference, bind_formula,
};
use crate::consumer::ConsumerLibraryContextState;
use crate::eval::{DefinedNameBinding, EvaluationBackend, EvaluationOutput, EvaluationTraceMode};
use crate::host::{
    ArtifactReuseReport, FirstHostReplayCapturePacket, HostRecalcOutput, SingleFormulaHost,
};
use crate::interface::{
    LibraryContextProvider, LibraryContextSnapshotRef, RegisteredExternalCatalogController,
    RegisteredExternalCatalogMutationRequest, RegisteredExternalCatalogMutationResult,
    ReturnedValueSurface, TableCallerRegion, TableDescriptor, TableRef, TypedContextQueryBundle,
    TypedContextQueryBundleSpec,
};
use crate::publication::{
    VerificationComparisonView, VerificationPublicationContext, VerificationPublicationSurface,
    build_verification_comparison_views,
};
use crate::red::project_red_view;
use crate::scheduler::ExecutionContract;
use crate::seam::{
    AcceptDecision, AcceptedCandidateResult, ExecutionOutcomeSurface, FenceSnapshot, Locus,
    RejectRecord, TraceEvent, execution_outcome_surface_commit_boundary_reject,
    execution_outcome_surface_executed_result,
};
use crate::semantics::{
    CompileSemanticPlanRequest, LibraryContextSnapshot, SemanticPlan, compile_semantic_plan,
};
use crate::session::{
    CapabilityViewSpec, ExecuteRequest, OverlayEntry, PrepareRequest, SessionPhase, SessionRecord,
    SessionService,
};
use crate::source::{FormulaSourceRecord, StructureContextVersion};
use crate::syntax::parser::{ParseRequest, parse_formula};
use crate::syntax::token::SyntaxDiagnostic;
use oxfunc_core::functions::call_register_id_family::RegisteredExternalProviderError;

pub struct RuntimeEnvironment<'a> {
    structure_context_version: StructureContextVersion,
    caller_row: u32,
    caller_col: u32,
    primary_locus: Locus,
    defined_names: BTreeMap<String, DefinedNameBinding>,
    cell_values: BTreeMap<String, EvalValue>,
    formal_input_bindings: Vec<RuntimeFormalInputBinding>,
    table_catalog: Vec<TableDescriptor>,
    enclosing_table_ref: Option<TableRef>,
    caller_table_region: Option<TableCallerRegion>,
    library_context: ConsumerLibraryContextState<'a>,
    oxfunc_bridge_metadata: RuntimeOxFuncBridgeMetadata,
}

impl<'a> RuntimeEnvironment<'a> {
    pub fn new() -> Self {
        Self {
            structure_context_version: StructureContextVersion("runtime-struct-v1".to_string()),
            caller_row: 1,
            caller_col: 1,
            primary_locus: Locus {
                sheet_id: "sheet:default".to_string(),
                row: 1,
                col: 1,
            },
            defined_names: BTreeMap::new(),
            cell_values: BTreeMap::new(),
            formal_input_bindings: Vec::new(),
            table_catalog: Vec::new(),
            enclosing_table_ref: None,
            caller_table_region: None,
            library_context: ConsumerLibraryContextState::new(),
            oxfunc_bridge_metadata: RuntimeOxFuncBridgeMetadata::default(),
        }
    }

    pub fn execute<'q>(
        &self,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeFormulaResult, String> {
        let mut host = self.build_host(request.source());
        self.execute_with_host(&mut host, request)
    }

    pub fn open_session(self) -> RuntimeSessionFacade<'a> {
        RuntimeSessionFacade::new(self)
    }

    pub fn apply_registered_external_catalog_mutation(
        &self,
        controller: &dyn RegisteredExternalCatalogController,
        request: &RegisteredExternalCatalogMutationRequest,
    ) -> Result<RegisteredExternalCatalogMutationResult, RegisteredExternalProviderError> {
        controller.apply_mutation(request)
    }

    pub fn with_structure_context_version(
        mut self,
        structure_context_version: StructureContextVersion,
    ) -> Self {
        self.structure_context_version = structure_context_version;
        self
    }

    pub fn with_caller_position(mut self, caller_row: u32, caller_col: u32) -> Self {
        self.caller_row = caller_row;
        self.caller_col = caller_col;
        self
    }

    pub fn with_primary_locus(mut self, primary_locus: Locus) -> Self {
        self.primary_locus = primary_locus;
        self
    }

    pub fn with_defined_names(
        mut self,
        defined_names: BTreeMap<String, DefinedNameBinding>,
    ) -> Self {
        self.defined_names = defined_names;
        self
    }

    pub fn with_cell_values(mut self, cell_values: BTreeMap<String, EvalValue>) -> Self {
        self.cell_values = cell_values;
        self
    }

    pub fn with_formal_input_bindings(
        mut self,
        formal_input_bindings: Vec<RuntimeFormalInputBinding>,
    ) -> Self {
        self.formal_input_bindings = formal_input_bindings;
        self
    }

    pub fn with_table_context(
        mut self,
        table_catalog: Vec<TableDescriptor>,
        enclosing_table_ref: Option<TableRef>,
        caller_table_region: Option<TableCallerRegion>,
    ) -> Self {
        self.table_catalog = table_catalog;
        self.enclosing_table_ref = enclosing_table_ref;
        self.caller_table_region = caller_table_region;
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
        snapshot_ref: LibraryContextSnapshotRef,
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
        snapshot_ref: LibraryContextSnapshotRef,
    ) -> Self {
        self.with_library_context_provider(provider)
            .with_library_context_snapshot_ref(snapshot_ref)
    }

    pub fn with_resolved_library_context(
        mut self,
        provider: Option<&'a dyn LibraryContextProvider>,
        snapshot_ref: Option<LibraryContextSnapshotRef>,
        snapshot: Option<LibraryContextSnapshot>,
    ) -> Self {
        self.library_context =
            ConsumerLibraryContextState::from_parts(provider, snapshot_ref, snapshot);
        self
    }

    pub fn with_oxfunc_bridge_metadata(mut self, metadata: RuntimeOxFuncBridgeMetadata) -> Self {
        self.oxfunc_bridge_metadata = metadata;
        self
    }

    pub fn with_semantic_kernel_metadata_version(mut self, version: impl Into<String>) -> Self {
        self.oxfunc_bridge_metadata.semantic_kernel_metadata_version = Some(version.into());
        self
    }

    pub fn with_arg_admission_metadata_version(mut self, version: impl Into<String>) -> Self {
        self.oxfunc_bridge_metadata.arg_admission_metadata_version = Some(version.into());
        self
    }

    fn execute_with_host<'q>(
        &self,
        host: &mut SingleFormulaHost,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeFormulaResult, String> {
        let compiled = compile_runtime_prepare_request(self, &request)?;
        let prepared_formula_identity = runtime_prepared_formula_identity(
            &compiled.prepare_request.source,
            &compiled.prepare_request.bound_formula,
            &compiled.prepare_request.semantic_plan,
            &self.primary_locus,
            &self.oxfunc_bridge_metadata,
        );
        if compiled
            .prepare_request
            .semantic_plan
            .execution_profile
            .requires_locale
            && request.typed_query_bundle.locale_ctx.is_none()
        {
            return Err(
                "capability denied: locale_format_context unavailable for runtime execution"
                    .to_string(),
            );
        }
        self.apply_to_host(host, request.source());
        apply_formal_input_bindings_to_host(
            host,
            &self.formal_input_bindings,
            &prepared_formula_identity,
        )?;
        host.set_trace_mode(request.trace_mode());
        let output = host.recalc_with_library_context_view(
            request.backend(),
            request.typed_query_bundle,
            self.library_context.pinned_view(),
            request.verification_publication_context(),
        )?;
        Ok(RuntimeFormulaResult::from_host_output(
            output,
            prepared_formula_identity,
        ))
    }

    fn build_host(&self, source: &FormulaSourceRecord) -> SingleFormulaHost {
        let mut host = SingleFormulaHost::new(
            source.formula_stable_id.0.clone(),
            source.entered_formula_text.clone(),
        );
        self.apply_to_host(&mut host, source);
        host
    }

    fn apply_to_host(&self, host: &mut SingleFormulaHost, source: &FormulaSourceRecord) {
        host.set_formula_source(source);
        host.structure_context_version = self.structure_context_version.0.clone();
        host.caller_row = self.caller_row;
        host.caller_col = self.caller_col;
        host.primary_locus = self.primary_locus.clone();
        host.defined_names = self.defined_names.clone();
        host.cell_values = self.cell_values.clone();
        host.table_catalog = self.table_catalog.clone();
        host.enclosing_table_ref = self.enclosing_table_ref.clone();
        host.caller_table_region = self.caller_table_region.clone();
    }
}

fn formal_input_defined_names(
    formal_input_bindings: &[RuntimeFormalInputBinding],
) -> BTreeMap<String, DefinedNameBinding> {
    formal_input_bindings
        .iter()
        .map(|binding| {
            (
                formal_input_binding_name(&binding.reference_descriptor),
                binding.binding.clone(),
            )
        })
        .collect()
}

fn formal_input_binding_name(reference_descriptor: &str) -> String {
    reference_descriptor
        .strip_prefix("name:")
        .unwrap_or(reference_descriptor)
        .to_string()
}

fn apply_formal_input_bindings_to_host(
    host: &mut SingleFormulaHost,
    formal_input_bindings: &[RuntimeFormalInputBinding],
    prepared_formula_identity: &RuntimePreparedFormulaIdentity,
) -> Result<(), String> {
    let formal_references_by_handle = prepared_formula_identity
        .formal_references
        .iter()
        .map(|reference| (reference.reference_handle.as_str(), reference))
        .collect::<BTreeMap<_, _>>();

    for binding in formal_input_bindings {
        if let Some(reference_handle) = &binding.reference_handle {
            let formal_reference = formal_references_by_handle
                .get(reference_handle.as_str())
                .ok_or_else(|| {
                    format!("formal input binding references unknown handle {reference_handle}")
                })?;
            if formal_reference.reference_descriptor != binding.reference_descriptor {
                return Err(format!(
                    "formal input binding descriptor mismatch for {reference_handle}: expected {}, got {}",
                    formal_reference.reference_descriptor, binding.reference_descriptor
                ));
            }
        }
        host.defined_names.insert(
            formal_input_binding_name(&binding.reference_descriptor),
            binding.binding.clone(),
        );
    }
    Ok(())
}

impl Default for RuntimeEnvironment<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeFormulaRequest<'a> {
    source: FormulaSourceRecord,
    backend: EvaluationBackend,
    typed_query_bundle: TypedContextQueryBundle<'a>,
    verification_publication_context: Option<VerificationPublicationContext>,
    trace_mode: EvaluationTraceMode,
}

impl<'a> RuntimeFormulaRequest<'a> {
    pub fn new(
        source: FormulaSourceRecord,
        typed_query_bundle: TypedContextQueryBundle<'a>,
    ) -> Self {
        Self {
            source,
            backend: EvaluationBackend::OxFuncBacked,
            typed_query_bundle,
            verification_publication_context: None,
            trace_mode: EvaluationTraceMode::default(),
        }
    }

    pub fn with_backend(mut self, backend: EvaluationBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_verification_publication_context(
        mut self,
        verification_publication_context: VerificationPublicationContext,
    ) -> Self {
        self.verification_publication_context = Some(verification_publication_context);
        self
    }

    pub fn with_trace_mode(mut self, trace_mode: EvaluationTraceMode) -> Self {
        self.trace_mode = trace_mode;
        self
    }

    pub fn source(&self) -> &FormulaSourceRecord {
        &self.source
    }

    pub fn backend(&self) -> EvaluationBackend {
        self.backend
    }

    pub fn typed_query_bundle(&self) -> &TypedContextQueryBundle<'a> {
        &self.typed_query_bundle
    }

    pub fn verification_publication_context(&self) -> Option<&VerificationPublicationContext> {
        self.verification_publication_context.as_ref()
    }

    pub fn trace_mode(&self) -> EvaluationTraceMode {
        self.trace_mode
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFormulaResult {
    pub source: FormulaSourceRecord,
    pub syntax_diagnostics: Vec<SyntaxDiagnostic>,
    pub bind_diagnostics: Vec<BindDiagnostic>,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub semantic_plan: SemanticPlan,
    pub execution_contract: ExecutionContract,
    pub typed_query_bundle_spec: TypedContextQueryBundleSpec,
    pub evaluation: EvaluationOutput,
    pub published_worksheet_value: EvalValue,
    pub returned_value_surface: ReturnedValueSurface,
    pub execution_outcome_surface: ExecutionOutcomeSurface,
    pub comparison_views: Vec<VerificationComparisonView>,
    pub verification_publication_surface: VerificationPublicationSurface,
    pub candidate_result: AcceptedCandidateResult,
    pub commit_decision: AcceptDecision,
    pub trace_events: Vec<TraceEvent>,
    pub artifact_reuse: ArtifactReuseReport,
    pub first_host_replay_capture_packet: FirstHostReplayCapturePacket,
    pub prepared_formula_identity: RuntimePreparedFormulaIdentity,
}

impl RuntimeFormulaResult {
    pub fn prepared_formula_package(&self) -> RuntimePreparedFormulaPackage {
        self.prepared_formula_identity.prepared_formula_package()
    }

    fn from_host_output(
        host_output: HostRecalcOutput,
        mut prepared_formula_identity: RuntimePreparedFormulaIdentity,
    ) -> Self {
        refresh_runtime_prepared_formula_identity_for_plan(
            &mut prepared_formula_identity,
            &host_output.semantic_plan,
        );
        let verification_publication_surface = host_output.verification_publication_surface.clone();
        let first_host_replay_capture_packet = host_output.to_first_host_replay_capture_packet();
        let comparison_views =
            build_verification_comparison_views(&verification_publication_surface);
        Self {
            source: host_output.source,
            syntax_diagnostics: host_output.syntax_diagnostics,
            bind_diagnostics: host_output.bind_diagnostics,
            library_context_snapshot_ref: host_output.library_context_snapshot_ref,
            semantic_plan: host_output.semantic_plan,
            execution_contract: host_output.execution_contract,
            typed_query_bundle_spec: host_output.typed_query_bundle_spec,
            evaluation: host_output.evaluation,
            published_worksheet_value: host_output.published_worksheet_value,
            returned_value_surface: host_output.returned_value_surface,
            execution_outcome_surface: host_output.execution_outcome_surface,
            comparison_views,
            verification_publication_surface,
            candidate_result: host_output.candidate_result,
            commit_decision: host_output.commit_decision,
            trace_events: host_output.trace_events,
            artifact_reuse: host_output.artifact_reuse,
            first_host_replay_capture_packet,
            prepared_formula_identity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreparedFormulaIdentity {
    pub prepared_formula_key: String,
    pub formula_stable_id: String,
    pub formula_text_version: u64,
    pub formula_token: String,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub structure_context_version: String,
    pub caller_context_key: Option<String>,
    pub semantic_kernel_metadata_version: Option<String>,
    pub arg_admission_metadata_version: Option<String>,
    pub plan_template: RuntimePlanTemplateIdentity,
    pub hole_binding: RuntimeHoleBindingIdentity,
    pub formal_references: Vec<RuntimeFormalReference>,
    pub projection_status: String,
}

impl RuntimePreparedFormulaIdentity {
    pub fn prepared_formula_package(&self) -> RuntimePreparedFormulaPackage {
        RuntimePreparedFormulaPackage {
            package_key: self.prepared_formula_key.clone(),
            identity: self.clone(),
            plan_template: self.plan_template.clone(),
            hole_binding: self.hole_binding.clone(),
            formal_references: self.formal_references.clone(),
            projection_status: "current_floor:runtime_identity_package".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreparedFormulaPackage {
    pub package_key: String,
    pub identity: RuntimePreparedFormulaIdentity,
    pub plan_template: RuntimePlanTemplateIdentity,
    pub hole_binding: RuntimeHoleBindingIdentity,
    pub formal_references: Vec<RuntimeFormalReference>,
    pub projection_status: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeOxFuncBridgeMetadata {
    pub semantic_kernel_metadata_version: Option<String>,
    pub arg_admission_metadata_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanTemplateIdentity {
    pub shape_key: Option<String>,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
    pub folded_plan_key: Option<String>,
    pub template_holes: Vec<RuntimeTemplateHole>,
    pub projection_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTemplateHole {
    pub hole_id: String,
    pub ordinal: usize,
    pub path: Option<String>,
    pub hole_kind: String,
    pub hole_kind_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHoleBindingIdentity {
    pub hole_binding_fingerprint: String,
    pub binding_count: usize,
    pub projection_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFormalReference {
    pub reference_handle: String,
    pub reference_descriptor: String,
    pub reference_family: String,
    pub caller_context_dependent: bool,
    pub host_mappable_identity: Option<String>,
    pub linked_hole_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFormalInputBinding {
    pub reference_handle: Option<String>,
    pub reference_descriptor: String,
    pub binding: DefinedNameBinding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeManagedSessionError {
    Preparation(String),
    Reject(RejectRecord),
}

impl From<RejectRecord> for RuntimeManagedSessionError {
    fn from(value: RejectRecord) -> Self {
        Self::Reject(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeManagedOpenResult {
    pub session_id: String,
    pub fence_snapshot: FenceSnapshot,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub syntax_diagnostics: Vec<SyntaxDiagnostic>,
    pub bind_diagnostics: Vec<BindDiagnostic>,
    pub semantic_plan: SemanticPlan,
    pub prepared_formula_identity: RuntimePreparedFormulaIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeManagedExecutionResult {
    pub formula_stable_id: String,
    pub session_id: String,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub candidate_result: AcceptedCandidateResult,
    pub typed_query_bundle_spec: TypedContextQueryBundleSpec,
    pub trace_events: Vec<TraceEvent>,
    pub prepared_formula_identity: RuntimePreparedFormulaIdentity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeManagedCommitResult {
    pub session: RuntimeManagedSessionSnapshot,
    pub commit_decision: AcceptDecision,
    pub execution_outcome_surface: ExecutionOutcomeSurface,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeManagedTerminationResult {
    pub session: RuntimeManagedSessionSnapshot,
    pub reject_record: RejectRecord,
    pub execution_outcome_surface: ExecutionOutcomeSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeManagedSessionPhase {
    Open,
    CapabilityViewEstablished,
    Executed,
    Committed,
    Rejected,
    Aborted,
    Expired,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeManagedSessionSnapshot {
    pub formula_stable_id: String,
    pub session_id: String,
    pub phase: RuntimeManagedSessionPhase,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub typed_query_bundle_spec: Option<TypedContextQueryBundleSpec>,
    pub candidate_result_id: Option<String>,
    pub last_reject: Option<RejectRecord>,
    pub execution_outcome_surface: Option<ExecutionOutcomeSurface>,
    pub trace_events: Vec<TraceEvent>,
    pub prepared_formula_identity: RuntimePreparedFormulaIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeManagedOverlaySummary {
    pub overlay_entry_id: String,
    pub overlay_scope_key: String,
    pub overlay_family: String,
    pub formula_stable_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeManagedSessionDiagnostics {
    pub session_id: String,
    pub formula_stable_id: String,
    pub phase: RuntimeManagedSessionPhase,
    pub capability_view_key: Option<String>,
    pub overlay_entries: Vec<RuntimeManagedOverlaySummary>,
    pub active_locus_claim_owner: Option<String>,
}

pub struct RuntimeSessionFacade<'a> {
    environment: RuntimeEnvironment<'a>,
    host: Option<SingleFormulaHost>,
    managed_service: SessionService,
    managed_session_id: Option<String>,
    managed_formula_token: Option<String>,
    managed_syntax_diagnostics: Vec<SyntaxDiagnostic>,
}

impl<'a> RuntimeSessionFacade<'a> {
    pub fn new(environment: RuntimeEnvironment<'a>) -> Self {
        Self {
            environment,
            host: None,
            managed_service: SessionService::new(),
            managed_session_id: None,
            managed_formula_token: None,
            managed_syntax_diagnostics: Vec::new(),
        }
    }

    pub fn environment(&self) -> &RuntimeEnvironment<'a> {
        &self.environment
    }

    pub fn execute<'q>(
        &mut self,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeFormulaResult, String> {
        let host = self
            .host
            .get_or_insert_with(|| self.environment.build_host(request.source()));
        self.environment.execute_with_host(host, request)
    }

    pub fn open_managed_session<'q>(
        &mut self,
        request: &RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeManagedOpenResult, RuntimeManagedSessionError> {
        let prepared = compile_runtime_prepare_request(&self.environment, request)
            .map_err(RuntimeManagedSessionError::Preparation)?;
        let syntax_diagnostics = prepared.syntax_diagnostics;
        let bind_diagnostics = prepared.bind_diagnostics.clone();
        let semantic_plan = prepared.prepare_request.semantic_plan.clone();
        let prepared_formula_identity = runtime_prepared_formula_identity(
            &prepared.prepare_request.source,
            &prepared.prepare_request.bound_formula,
            &prepared.prepare_request.semantic_plan,
            &self.environment.primary_locus,
            &self.environment.oxfunc_bridge_metadata,
        );
        let prepared_session = self.managed_service.prepare(prepared.prepare_request)?;
        let open = self.managed_service.open_session(prepared_session);
        self.managed_formula_token = Some(request.source().formula_token().0);
        self.managed_session_id = Some(open.session_id.clone());
        self.managed_syntax_diagnostics = syntax_diagnostics.clone();
        Ok(RuntimeManagedOpenResult {
            session_id: open.session_id,
            fence_snapshot: open.fence_snapshot,
            library_context_snapshot_ref: open.library_context_snapshot_ref,
            syntax_diagnostics,
            bind_diagnostics,
            semantic_plan,
            prepared_formula_identity,
        })
    }

    pub fn execute_managed<'q>(
        &mut self,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeManagedExecutionResult, RuntimeManagedSessionError> {
        self.ensure_managed_session_for(&request)?;
        if !self.managed_syntax_diagnostics.is_empty() {
            return Err(RuntimeManagedSessionError::Preparation(
                syntax_diagnostic_execution_error(&self.managed_syntax_diagnostics),
            ));
        }
        let session_id = self.managed_session_id.clone().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation("managed session not open".to_string())
        })?;
        self.ensure_managed_capability_view_for(&session_id, &request)?;
        let snapshot_before_execute = self.managed_session_snapshot().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation(
                "managed session missing before execute".to_string(),
            )
        })?;
        let mut defined_names = self.environment.defined_names.clone();
        for binding in &self.environment.formal_input_bindings {
            if let Some(reference_handle) = &binding.reference_handle {
                let formal_reference = snapshot_before_execute
                    .prepared_formula_identity
                    .formal_references
                    .iter()
                    .find(|reference| reference.reference_handle == *reference_handle)
                    .ok_or_else(|| {
                        RuntimeManagedSessionError::Preparation(format!(
                            "formal input binding references unknown handle {reference_handle}"
                        ))
                    })?;
                if formal_reference.reference_descriptor != binding.reference_descriptor {
                    return Err(RuntimeManagedSessionError::Preparation(format!(
                        "formal input binding descriptor mismatch for {reference_handle}: expected {}, got {}",
                        formal_reference.reference_descriptor, binding.reference_descriptor
                    )));
                }
            }
            defined_names.insert(
                formal_input_binding_name(&binding.reference_descriptor),
                binding.binding.clone(),
            );
        }
        let candidate_result = self.managed_service.execute(ExecuteRequest {
            session_id: session_id.clone(),
            backend: request.backend(),
            caller_row: self.environment.caller_row as usize,
            caller_col: self.environment.caller_col as usize,
            cell_values: self.environment.cell_values.clone(),
            defined_names,
            typed_query_bundle: *request.typed_query_bundle(),
        })?;
        let snapshot = self.managed_session_snapshot().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation(
                "managed session missing after execute".to_string(),
            )
        })?;
        let typed_query_bundle_spec =
            snapshot.typed_query_bundle_spec.clone().ok_or_else(|| {
                RuntimeManagedSessionError::Preparation(
                    "managed session missing typed query bundle spec".to_string(),
                )
            })?;
        Ok(RuntimeManagedExecutionResult {
            formula_stable_id: snapshot.formula_stable_id,
            session_id,
            library_context_snapshot_ref: snapshot.library_context_snapshot_ref,
            candidate_result,
            typed_query_bundle_spec,
            trace_events: snapshot.trace_events,
            prepared_formula_identity: snapshot.prepared_formula_identity,
        })
    }

    pub fn execute_and_commit_managed<'q>(
        &mut self,
        request: RuntimeFormulaRequest<'q>,
        commit_attempt_id: impl Into<String>,
    ) -> Result<RuntimeManagedCommitResult, RuntimeManagedSessionError> {
        let execution = self.execute_managed(request)?;
        self.commit_managed(commit_attempt_id, execution.candidate_result.fence_snapshot)
    }

    pub fn commit_managed(
        &mut self,
        commit_attempt_id: impl Into<String>,
        observed_fence: FenceSnapshot,
    ) -> Result<RuntimeManagedCommitResult, RuntimeManagedSessionError> {
        let session_id = self.managed_session_id.clone().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation("managed session not open".to_string())
        })?;
        let commit_decision =
            self.managed_service
                .commit(&session_id, commit_attempt_id, observed_fence);
        let session = self.managed_session_snapshot().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation(
                "managed session missing after commit".to_string(),
            )
        })?;
        let execution_outcome_surface =
            runtime_execution_outcome_surface_from_commit_decision(&commit_decision);
        Ok(RuntimeManagedCommitResult {
            session,
            commit_decision,
            execution_outcome_surface,
        })
    }

    pub fn abort_managed(
        &mut self,
        cause: Option<String>,
    ) -> Result<RuntimeManagedTerminationResult, RuntimeManagedSessionError> {
        let session_id = self.managed_session_id.clone().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation("managed session not open".to_string())
        })?;
        let reject_record = self.managed_service.abort_session(&session_id, cause);
        let session = self.managed_session_snapshot().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation(
                "managed session missing after abort".to_string(),
            )
        })?;
        let execution_outcome_surface =
            runtime_execution_outcome_surface_from_reject_record(&reject_record);
        Ok(RuntimeManagedTerminationResult {
            session,
            reject_record,
            execution_outcome_surface,
        })
    }

    pub fn expire_managed(
        &mut self,
        cause: Option<String>,
    ) -> Result<RuntimeManagedTerminationResult, RuntimeManagedSessionError> {
        let session_id = self.managed_session_id.clone().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation("managed session not open".to_string())
        })?;
        let reject_record = self.managed_service.expire_session(&session_id, cause);
        let session = self.managed_session_snapshot().ok_or_else(|| {
            RuntimeManagedSessionError::Preparation(
                "managed session missing after expiry".to_string(),
            )
        })?;
        let execution_outcome_surface =
            runtime_execution_outcome_surface_from_reject_record(&reject_record);
        Ok(RuntimeManagedTerminationResult {
            session,
            reject_record,
            execution_outcome_surface,
        })
    }

    pub fn managed_session_snapshot(&self) -> Option<RuntimeManagedSessionSnapshot> {
        let session_id = self.managed_session_id.as_ref()?;
        let record = self.managed_service.session(session_id)?;
        Some(runtime_managed_session_snapshot(
            record,
            &self.environment.oxfunc_bridge_metadata,
        ))
    }

    pub fn managed_session_diagnostics(&self) -> Option<RuntimeManagedSessionDiagnostics> {
        let session_id = self.managed_session_id.as_ref()?;
        let record = self.managed_service.session(session_id)?;
        Some(RuntimeManagedSessionDiagnostics {
            session_id: record.session_id.clone(),
            formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
            phase: runtime_managed_phase(record.phase.clone()),
            capability_view_key: record
                .capability_view
                .as_ref()
                .map(|view| view.capability_view_key.clone()),
            overlay_entries: self
                .managed_service
                .overlay_entries(session_id)
                .iter()
                .map(runtime_overlay_summary)
                .collect(),
            active_locus_claim_owner: self
                .managed_service
                .active_locus_claim_owner(&record.prepared.primary_locus)
                .map(ToString::to_string),
        })
    }

    fn ensure_managed_session_for<'q>(
        &mut self,
        request: &RuntimeFormulaRequest<'q>,
    ) -> Result<(), RuntimeManagedSessionError> {
        let formula_token = request.source().formula_token().0;
        if self.managed_session_id.is_none()
            || self.managed_formula_token.as_deref() != Some(formula_token.as_str())
        {
            let _ = self.open_managed_session(request)?;
        }
        Ok(())
    }

    fn ensure_managed_capability_view_for<'q>(
        &mut self,
        session_id: &str,
        request: &RuntimeFormulaRequest<'q>,
    ) -> Result<(), RuntimeManagedSessionError> {
        let Some(record) = self.managed_service.session(session_id) else {
            return Err(RuntimeManagedSessionError::Preparation(
                "managed session missing before capability view establishment".to_string(),
            ));
        };
        if !matches!(record.phase, SessionPhase::Open) {
            return Ok(());
        }
        let spec = runtime_capability_view_spec(request);
        self.managed_service
            .establish_capability_view(session_id, spec)?;
        Ok(())
    }
}

fn runtime_prepared_formula_identity(
    source: &FormulaSourceRecord,
    bound_formula: &BoundFormula,
    semantic_plan: &SemanticPlan,
    primary_locus: &Locus,
    oxfunc_bridge_metadata: &RuntimeOxFuncBridgeMetadata,
) -> RuntimePreparedFormulaIdentity {
    let formula_token = source.formula_token().0;
    let oxfunc_bridge_metadata =
        runtime_oxfunc_bridge_metadata_for_plan(semantic_plan, oxfunc_bridge_metadata);
    let caller_context_key = Some(format!(
        "locus:{}:{}:{}",
        primary_locus.sheet_id, primary_locus.row, primary_locus.col
    ));
    let dispatch_skeleton_key = runtime_hash_debug(&(
        &semantic_plan.function_bindings,
        &semantic_plan.availability_summaries,
        &semantic_plan.oxfunc_catalog_identity,
        &semantic_plan.library_context_snapshot_ref,
    ));
    let plan_template_key = semantic_plan.semantic_plan_key.clone();
    let hole_binding_fingerprint = runtime_hash_debug(&(
        &bound_formula.normalized_references,
        &bound_formula.unresolved_references,
        &semantic_plan.helper_profile,
        &semantic_plan.capability_requirements,
    ));
    let prepared_formula_key = runtime_hash_debug(&(
        &source.formula_stable_id.0,
        source.formula_text_version.0,
        &formula_token,
        &bound_formula.bind_hash,
        &semantic_plan.semantic_plan_key,
        &semantic_plan.library_context_snapshot_ref,
        &bound_formula.structure_context_version,
        &caller_context_key,
        &oxfunc_bridge_metadata.semantic_kernel_metadata_version,
        &oxfunc_bridge_metadata.arg_admission_metadata_version,
    ));

    RuntimePreparedFormulaIdentity {
        prepared_formula_key,
        formula_stable_id: source.formula_stable_id.0.clone(),
        formula_text_version: source.formula_text_version.0,
        formula_token,
        library_context_snapshot_ref: semantic_plan.library_context_snapshot_ref.clone(),
        structure_context_version: bound_formula.structure_context_version.clone(),
        caller_context_key,
        semantic_kernel_metadata_version: oxfunc_bridge_metadata
            .semantic_kernel_metadata_version
            .clone(),
        arg_admission_metadata_version: oxfunc_bridge_metadata
            .arg_admission_metadata_version
            .clone(),
        plan_template: RuntimePlanTemplateIdentity {
            shape_key: None,
            dispatch_skeleton_key,
            plan_template_key,
            folded_plan_key: None,
            template_holes: runtime_template_holes(bound_formula),
            projection_status: "current_floor:template_key_and_reference_holes".to_string(),
        },
        hole_binding: RuntimeHoleBindingIdentity {
            hole_binding_fingerprint,
            binding_count: bound_formula.normalized_references.len()
                + bound_formula.unresolved_references.len(),
            projection_status:
                "current_floor:fingerprint_from_public_bind_references;canonical_holes_deferred"
                    .to_string(),
        },
        formal_references: runtime_formal_references(bound_formula),
        projection_status: "current_floor:derived_from_public_runtime_prepare".to_string(),
    }
}

fn runtime_oxfunc_bridge_metadata_for_plan(
    semantic_plan: &SemanticPlan,
    override_metadata: &RuntimeOxFuncBridgeMetadata,
) -> RuntimeOxFuncBridgeMetadata {
    RuntimeOxFuncBridgeMetadata {
        semantic_kernel_metadata_version: override_metadata
            .semantic_kernel_metadata_version
            .clone()
            .or_else(|| {
                runtime_metadata_version_summary(
                    semantic_plan
                        .function_bindings
                        .iter()
                        .filter_map(|binding| {
                            builtin_registry()
                                .lookup_by_id(binding.function_id)
                                .map(|entry| entry.meta.semantic_kernel_metadata_version.clone())
                        }),
                    "semantic_kernel_metadata",
                )
            }),
        arg_admission_metadata_version: override_metadata
            .arg_admission_metadata_version
            .clone()
            .or_else(|| {
                runtime_metadata_version_summary(
                    semantic_plan
                        .function_bindings
                        .iter()
                        .filter_map(|binding| {
                            builtin_registry()
                                .lookup_by_id(binding.function_id)
                                .map(|entry| entry.meta.arg_admission_metadata_version.clone())
                        }),
                    "arg_admission_metadata",
                )
            }),
    }
}

fn runtime_metadata_version_summary(
    versions: impl IntoIterator<Item = String>,
    version_family: &str,
) -> Option<String> {
    let versions = versions.into_iter().collect::<BTreeSet<_>>();
    match versions.len() {
        0 => None,
        1 => versions.into_iter().next(),
        _ => Some(format!(
            "{version_family}.set.v1;{}",
            versions.into_iter().collect::<Vec<_>>().join("|")
        )),
    }
}

fn refresh_runtime_prepared_formula_identity_for_plan(
    identity: &mut RuntimePreparedFormulaIdentity,
    semantic_plan: &SemanticPlan,
) {
    identity.library_context_snapshot_ref = semantic_plan.library_context_snapshot_ref.clone();
    identity.plan_template.dispatch_skeleton_key = runtime_hash_debug(&(
        &semantic_plan.function_bindings,
        &semantic_plan.availability_summaries,
        &semantic_plan.oxfunc_catalog_identity,
        &semantic_plan.library_context_snapshot_ref,
    ));
    identity.plan_template.plan_template_key = semantic_plan.semantic_plan_key.clone();
    identity.prepared_formula_key = runtime_hash_debug(&(
        &identity.formula_stable_id,
        identity.formula_text_version,
        &identity.formula_token,
        &semantic_plan.bind_hash,
        &semantic_plan.semantic_plan_key,
        &semantic_plan.library_context_snapshot_ref,
        &identity.structure_context_version,
        &identity.caller_context_key,
        &identity.semantic_kernel_metadata_version,
        &identity.arg_admission_metadata_version,
    ));
}

fn runtime_formal_references(bound_formula: &BoundFormula) -> Vec<RuntimeFormalReference> {
    let mut references = bound_formula
        .normalized_references
        .iter()
        .enumerate()
        .map(|(index, reference)| RuntimeFormalReference {
            reference_handle: format!(
                "formal-ref:{}:{index}",
                runtime_hash_debug(&(bound_formula.bind_hash.as_str(), reference))
            ),
            reference_descriptor: reference.to_string(),
            reference_family: runtime_reference_family(reference).to_string(),
            caller_context_dependent: runtime_reference_caller_context_dependent(reference),
            host_mappable_identity: Some(reference.to_string()),
            linked_hole_id: Some(runtime_reference_hole_id(index)),
        })
        .collect::<Vec<_>>();
    references.extend(bound_formula.unresolved_references.iter().enumerate().map(
        |(index, unresolved)| RuntimeFormalReference {
            reference_handle: format!(
                "formal-ref:{}:unresolved:{index}",
                runtime_hash_debug(&(
                    bound_formula.bind_hash.as_str(),
                    unresolved.source_text.as_str(),
                    unresolved.reason.as_str()
                ))
            ),
            reference_descriptor: unresolved.source_text.clone(),
            reference_family: "unresolved".to_string(),
            caller_context_dependent: false,
            host_mappable_identity: None,
            linked_hole_id: Some(runtime_unresolved_hole_id(index)),
        },
    ));
    references
}

fn runtime_template_holes(bound_formula: &BoundFormula) -> Vec<RuntimeTemplateHole> {
    let mut holes = bound_formula
        .normalized_references
        .iter()
        .enumerate()
        .map(|(index, reference)| {
            let hole_kind = runtime_template_hole_kind(reference).to_string();
            RuntimeTemplateHole {
                hole_id: runtime_reference_hole_id(index),
                ordinal: index,
                path: Some(reference.to_string()),
                hole_kind_key: format!("{hole_kind}:{}", runtime_reference_family(reference)),
                hole_kind,
            }
        })
        .collect::<Vec<_>>();
    let offset = holes.len();
    holes.extend(bound_formula.unresolved_references.iter().enumerate().map(
        |(index, unresolved)| RuntimeTemplateHole {
            hole_id: runtime_unresolved_hole_id(index),
            ordinal: offset + index,
            path: Some(unresolved.source_text.clone()),
            hole_kind: "UnresolvedReferenceHole".to_string(),
            hole_kind_key: format!("UnresolvedReferenceHole:{}", unresolved.reason),
        },
    ));
    holes
}

fn runtime_reference_hole_id(index: usize) -> String {
    format!("hole:reference:{index}")
}

fn runtime_unresolved_hole_id(index: usize) -> String {
    format!("hole:unresolved:{index}")
}

fn runtime_template_hole_kind(reference: &NormalizedReference) -> &'static str {
    match reference {
        NormalizedReference::WholeRow(_)
        | NormalizedReference::WholeColumn(_)
        | NormalizedReference::Structured(_) => "ShapeSensitiveHole",
        NormalizedReference::External(_) => "RichValueHole",
        NormalizedReference::Error(_) => "UnresolvedReferenceHole",
        NormalizedReference::Cell(_)
        | NormalizedReference::Area(_)
        | NormalizedReference::Name(_) => "RefOrValueHole",
    }
}

fn runtime_reference_family(reference: &NormalizedReference) -> &'static str {
    match reference {
        NormalizedReference::Cell(_) | NormalizedReference::Area(_) => "direct",
        NormalizedReference::WholeRow(_) | NormalizedReference::WholeColumn(_) => {
            "shape_topology_sensitive"
        }
        NormalizedReference::Name(name) if name.caller_context_dependent => {
            "relative_or_caller_sensitive"
        }
        NormalizedReference::Name(name) if matches!(name.kind, NameKind::MixedOrDeferred) => {
            "host_sensitive"
        }
        NormalizedReference::Name(_) => "direct",
        NormalizedReference::External(_) => "dynamic_potential",
        NormalizedReference::Structured(structured) if structured.caller_row_sensitive => {
            "relative_or_caller_sensitive"
        }
        NormalizedReference::Structured(_) => "direct",
        NormalizedReference::Error(_) => "unresolved",
    }
}

fn runtime_reference_caller_context_dependent(reference: &NormalizedReference) -> bool {
    match reference {
        NormalizedReference::Cell(cell) => cell.caller_anchor_used,
        NormalizedReference::Area(area) => area.caller_anchor_used,
        NormalizedReference::Name(name) => name.caller_context_dependent,
        NormalizedReference::Structured(structured) => structured.caller_row_sensitive,
        NormalizedReference::WholeRow(_)
        | NormalizedReference::WholeColumn(_)
        | NormalizedReference::External(_)
        | NormalizedReference::Error(_) => false,
    }
}

fn runtime_hash_debug<T: std::fmt::Debug>(value: &T) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{value:?}").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

struct CompiledRuntimePrepareRequest {
    prepare_request: PrepareRequest,
    syntax_diagnostics: Vec<SyntaxDiagnostic>,
    bind_diagnostics: Vec<BindDiagnostic>,
}

fn compile_runtime_prepare_request(
    environment: &RuntimeEnvironment<'_>,
    request: &RuntimeFormulaRequest<'_>,
) -> Result<CompiledRuntimePrepareRequest, String> {
    let source = request.source().clone();
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let syntax_diagnostics = parse.green_tree.diagnostics.clone();
    let red_projection = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let mut bind_names = environment.defined_names.clone();
    bind_names.extend(formal_input_defined_names(
        &environment.formal_input_bindings,
    ));
    let bind = bind_formula(crate::binding::BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection,
        context: BindContext {
            structure_context_version: environment.structure_context_version.clone(),
            caller_row: environment.caller_row,
            caller_col: environment.caller_col,
            formula_token: source.formula_token(),
            names: bind_names
                .iter()
                .map(|(name, binding)| {
                    (
                        name.clone(),
                        match binding {
                            DefinedNameBinding::Value(_) => NameKind::ValueLike,
                            DefinedNameBinding::Reference(_) => NameKind::ReferenceLike,
                            DefinedNameBinding::Callable(_) => NameKind::ValueLike,
                        },
                    )
                })
                .collect(),
            table_catalog: environment.table_catalog.clone(),
            enclosing_table_ref: environment.enclosing_table_ref.clone(),
            caller_table_region: environment.caller_table_region.clone(),
            ..BindContext::default()
        },
    });
    let library_context_view = environment.library_context.pinned_view();
    let library_context_snapshot = library_context_view.resolve_snapshot();
    if let Some(snapshot_ref) = library_context_view.snapshot_ref() {
        if library_context_snapshot.is_none() {
            return Err(format!(
                "requested library context snapshot {}@{} did not resolve",
                snapshot_ref.snapshot_id, snapshot_ref.snapshot_version
            ));
        }
    }
    let locale_profile = request
        .typed_query_bundle()
        .locale_ctx
        .map(|ctx| format!("{:?}", ctx.profile.id));
    let date_system = request
        .typed_query_bundle()
        .locale_ctx
        .map(|ctx| format!("{:?}", ctx.date_system));
    let format_profile = request
        .typed_query_bundle()
        .locale_ctx
        .map(|_| "locale-format-context".to_string());
    let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:runtime-facade-session".to_string(),
        locale_profile,
        date_system,
        format_profile,
        library_context_snapshot,
    })
    .semantic_plan;
    let bind_diagnostics = bind.bound_formula.diagnostics.clone();

    Ok(CompiledRuntimePrepareRequest {
        prepare_request: PrepareRequest {
            source,
            bound_formula: bind.bound_formula,
            semantic_plan,
            primary_locus: environment.primary_locus.clone(),
        },
        syntax_diagnostics,
        bind_diagnostics,
    })
}

fn runtime_capability_view_spec(request: &RuntimeFormulaRequest<'_>) -> CapabilityViewSpec {
    CapabilityViewSpec {
        host_query_enabled: request.typed_query_bundle().host_info.is_some(),
        locale_format_enabled: request.typed_query_bundle().locale_ctx.is_some(),
        caller_context_enabled: true,
        external_provider_enabled: request
            .typed_query_bundle()
            .registered_external_provider
            .is_some(),
    }
}

fn runtime_managed_session_snapshot(
    record: &SessionRecord,
    oxfunc_bridge_metadata: &RuntimeOxFuncBridgeMetadata,
) -> RuntimeManagedSessionSnapshot {
    let phase = runtime_managed_phase(record.phase.clone());
    let last_reject = record.last_reject.clone();
    RuntimeManagedSessionSnapshot {
        formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
        session_id: record.session_id.clone(),
        phase,
        library_context_snapshot_ref: record.prepared.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: record.typed_query_bundle_spec.clone(),
        candidate_result_id: record
            .candidate_result
            .as_ref()
            .map(|candidate| candidate.candidate_result_id.clone()),
        execution_outcome_surface: runtime_execution_outcome_surface_from_managed_session_state(
            phase,
            last_reject.as_ref(),
        ),
        last_reject,
        trace_events: record.trace_events.clone(),
        prepared_formula_identity: runtime_prepared_formula_identity(
            &record.prepared.source,
            &record.prepared.bound_formula,
            &record.prepared.semantic_plan,
            &record.prepared.primary_locus,
            oxfunc_bridge_metadata,
        ),
    }
}

fn runtime_managed_phase(phase: SessionPhase) -> RuntimeManagedSessionPhase {
    match phase {
        SessionPhase::Open => RuntimeManagedSessionPhase::Open,
        SessionPhase::CapabilityViewEstablished => {
            RuntimeManagedSessionPhase::CapabilityViewEstablished
        }
        SessionPhase::Executed => RuntimeManagedSessionPhase::Executed,
        SessionPhase::Committed => RuntimeManagedSessionPhase::Committed,
        SessionPhase::Rejected => RuntimeManagedSessionPhase::Rejected,
        SessionPhase::Aborted => RuntimeManagedSessionPhase::Aborted,
        SessionPhase::Expired => RuntimeManagedSessionPhase::Expired,
    }
}

fn runtime_overlay_summary(overlay: &OverlayEntry) -> RuntimeManagedOverlaySummary {
    RuntimeManagedOverlaySummary {
        overlay_entry_id: overlay.overlay_entry_id.clone(),
        overlay_scope_key: overlay.overlay_scope_key.clone(),
        overlay_family: overlay.overlay_family.clone(),
        formula_stable_id: overlay.formula_stable_id.clone(),
    }
}

fn runtime_execution_outcome_surface_from_managed_session_state(
    phase: RuntimeManagedSessionPhase,
    last_reject: Option<&RejectRecord>,
) -> Option<ExecutionOutcomeSurface> {
    match phase {
        RuntimeManagedSessionPhase::Committed => Some(execution_outcome_surface_executed_result()),
        RuntimeManagedSessionPhase::Rejected
        | RuntimeManagedSessionPhase::Aborted
        | RuntimeManagedSessionPhase::Expired => {
            last_reject.map(runtime_execution_outcome_surface_from_reject_record)
        }
        RuntimeManagedSessionPhase::Open
        | RuntimeManagedSessionPhase::CapabilityViewEstablished
        | RuntimeManagedSessionPhase::Executed => None,
    }
}

fn runtime_execution_outcome_surface_from_commit_decision(
    commit_decision: &AcceptDecision,
) -> ExecutionOutcomeSurface {
    match commit_decision {
        AcceptDecision::Accepted(_) => execution_outcome_surface_executed_result(),
        AcceptDecision::Rejected(reject) => {
            runtime_execution_outcome_surface_from_reject_record(reject)
        }
    }
}

fn runtime_execution_outcome_surface_from_reject_record(
    reject_record: &RejectRecord,
) -> ExecutionOutcomeSurface {
    execution_outcome_surface_commit_boundary_reject(reject_record.reject_code)
}

fn syntax_diagnostic_execution_error(diagnostics: &[SyntaxDiagnostic]) -> String {
    let first = diagnostics
        .first()
        .expect("syntax diagnostics should be non-empty");
    format!(
        "formula execution rejected due to syntax diagnostics: {} at {}:{}",
        first.message, first.span.start, first.span.len
    )
}
