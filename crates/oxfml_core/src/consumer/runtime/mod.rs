use std::collections::BTreeMap;

use oxfunc_core::value::EvalValue;

use crate::binding::{BindContext, BindDiagnostic, NameKind, bind_formula};
use crate::consumer::ConsumerLibraryContextState;
use crate::eval::{DefinedNameBinding, EvaluationBackend, EvaluationOutput};
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
    AcceptDecision, AcceptedCandidateResult, FenceSnapshot, Locus, RejectRecord, TraceEvent,
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
    table_catalog: Vec<TableDescriptor>,
    enclosing_table_ref: Option<TableRef>,
    caller_table_region: Option<TableCallerRegion>,
    library_context: ConsumerLibraryContextState<'a>,
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
            table_catalog: Vec::new(),
            enclosing_table_ref: None,
            caller_table_region: None,
            library_context: ConsumerLibraryContextState::new(),
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

    fn execute_with_host<'q>(
        &self,
        host: &mut SingleFormulaHost,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeFormulaResult, String> {
        self.apply_to_host(host, request.source());
        let output = host.recalc_with_library_context_view(
            request.backend(),
            request.typed_query_bundle,
            self.library_context.pinned_view(),
        )?;
        Ok(RuntimeFormulaResult::from_host_output(
            output,
            request.typed_query_bundle.locale_ctx,
            request.verification_publication_context(),
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
    pub comparison_views: Vec<VerificationComparisonView>,
    pub verification_publication_surface: VerificationPublicationSurface,
    pub candidate_result: AcceptedCandidateResult,
    pub commit_decision: AcceptDecision,
    pub trace_events: Vec<TraceEvent>,
    pub artifact_reuse: ArtifactReuseReport,
    pub first_host_replay_capture_packet: FirstHostReplayCapturePacket,
}

impl RuntimeFormulaResult {
    fn from_host_output(
        host_output: HostRecalcOutput,
        locale_ctx: Option<&oxfunc_core::locale_format::LocaleFormatContext<'_>>,
        verification_publication_context: Option<&VerificationPublicationContext>,
    ) -> Self {
        let first_host_replay_capture_packet = host_output
            .to_first_host_replay_capture_packet_with_context(
                locale_ctx,
                verification_publication_context,
            );
        let comparison_views = build_verification_comparison_views(
            &first_host_replay_capture_packet.verification_publication_surface,
        );
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
            comparison_views,
            verification_publication_surface: first_host_replay_capture_packet
                .verification_publication_surface
                .clone(),
            candidate_result: host_output.candidate_result,
            commit_decision: host_output.commit_decision,
            trace_events: host_output.trace_events,
            artifact_reuse: host_output.artifact_reuse,
            first_host_replay_capture_packet,
        }
    }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeManagedExecutionResult {
    pub formula_stable_id: String,
    pub session_id: String,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub candidate_result: AcceptedCandidateResult,
    pub typed_query_bundle_spec: TypedContextQueryBundleSpec,
    pub trace_events: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeManagedCommitResult {
    pub session: RuntimeManagedSessionSnapshot,
    pub commit_decision: AcceptDecision,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeManagedTerminationResult {
    pub session: RuntimeManagedSessionSnapshot,
    pub reject_record: RejectRecord,
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
    pub trace_events: Vec<TraceEvent>,
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
        let candidate_result = self.managed_service.execute(ExecuteRequest {
            session_id: session_id.clone(),
            backend: request.backend(),
            caller_row: self.environment.caller_row as usize,
            caller_col: self.environment.caller_col as usize,
            cell_values: self.environment.cell_values.clone(),
            defined_names: self.environment.defined_names.clone(),
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
        Ok(RuntimeManagedCommitResult {
            session,
            commit_decision,
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
        Ok(RuntimeManagedTerminationResult {
            session,
            reject_record,
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
        Ok(RuntimeManagedTerminationResult {
            session,
            reject_record,
        })
    }

    pub fn managed_session_snapshot(&self) -> Option<RuntimeManagedSessionSnapshot> {
        let session_id = self.managed_session_id.as_ref()?;
        let record = self.managed_service.session(session_id)?;
        Some(runtime_managed_session_snapshot(record))
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
    let bind = bind_formula(crate::binding::BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection,
        context: BindContext {
            structure_context_version: environment.structure_context_version.clone(),
            caller_row: environment.caller_row,
            caller_col: environment.caller_col,
            formula_token: source.formula_token(),
            names: environment
                .defined_names
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

fn runtime_managed_session_snapshot(record: &SessionRecord) -> RuntimeManagedSessionSnapshot {
    RuntimeManagedSessionSnapshot {
        formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
        session_id: record.session_id.clone(),
        phase: runtime_managed_phase(record.phase.clone()),
        library_context_snapshot_ref: record.prepared.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: record.typed_query_bundle_spec.clone(),
        candidate_result_id: record
            .candidate_result
            .as_ref()
            .map(|candidate| candidate.candidate_result_id.clone()),
        last_reject: record.last_reject.clone(),
        trace_events: record.trace_events.clone(),
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

fn syntax_diagnostic_execution_error(diagnostics: &[SyntaxDiagnostic]) -> String {
    let first = diagnostics
        .first()
        .expect("syntax diagnostics should be non-empty");
    format!(
        "formula execution rejected due to syntax diagnostics: {} at {}:{}",
        first.message, first.span.start, first.span.len
    )
}
