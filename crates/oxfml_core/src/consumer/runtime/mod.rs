use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use oxfunc_core::registry::{
    CapabilityOverlay, FunctionAvailability, FunctionEntry, FunctionRegistry, FunctionSource,
    builtin_registry,
};
use oxfunc_core::resolver::{
    ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
};
use oxfunc_core::value::{ArrayCellValue, EvalValue, ReferenceLike, WorksheetErrorCode};

use crate::binding::{
    BindContext, BindDiagnostic, BoundFormula, NameKind, NormalizedReference,
    StructuredReferenceBindRecord, bind_formula,
};
use crate::consumer::ConsumerLibraryContextState;
use crate::eval::{
    DefinedNameBinding, EvaluationBackend, EvaluationOutput, EvaluationTraceMode, PreparedCall,
    SparseReferenceValuesBinding,
};
use crate::host::{ArtifactReuseReport, FirstHostReplayCapturePacket};
pub use crate::host::{HostRecalcOutput, SingleFormulaHost};
use crate::interface::{
    LibraryContextProvider, LibraryContextSnapshotRef, PinnedLibraryContextView,
    RegisteredExternalCatalogController, RegisteredExternalCatalogMutationRequest,
    RegisteredExternalCatalogMutationResult, ReturnedValueSurface, TableCallerRegion,
    TableDescriptor, TableRef, TypedContextQueryBundle, TypedContextQueryBundleSpec,
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
    CompileSemanticPlanRequest, LibraryAvailabilityState, LibraryContextSnapshot,
    LibraryContextSnapshotEntry, RegistrationSourceKind, SemanticPlan, compile_semantic_plan,
};
use crate::session::{
    CapabilityViewSpec, ExecuteRequest, OverlayEntry, PrepareRequest, SessionPhase, SessionRecord,
    SessionService,
};
use crate::source::{FormulaSourceRecord, StructureContextVersion};
use crate::syntax::parser::{ParseRequest, parse_formula};
use crate::syntax::token::{SyntaxDiagnostic, TextSpan};
use oxfunc_core::functions::call_register_id_family::RegisteredExternalProviderError;
use oxfunc_core::functions::surface_dispatch::FUNC_ID_OP_DIVIDE;

pub struct RuntimeEnvironment<'a> {
    structure_context_version: StructureContextVersion,
    caller_row: u32,
    caller_col: u32,
    primary_locus: Locus,
    defined_names: BTreeMap<String, DefinedNameBinding>,
    cell_values: BTreeMap<String, EvalValue>,
    formal_input_bindings: Vec<RuntimeFormalInputBinding>,
    sparse_reference_value_bindings: Vec<RuntimeSparseReferenceValuesBinding>,
    host_formula_context: Option<RuntimeHostFormulaContext>,
    host_name_bindings: Vec<RuntimeHostNameBinding>,
    host_reference_bind_results: Vec<RuntimeHostReferenceBindResult>,
    table_catalog: Vec<TableDescriptor>,
    enclosing_table_ref: Option<TableRef>,
    caller_table_region: Option<TableCallerRegion>,
    library_context: ConsumerLibraryContextState<'a>,
    oxfunc_bridge_metadata: RuntimeOxFuncBridgeMetadata,
    function_registry: &'a FunctionRegistry,
    function_registry_explicit: bool,
    capability_overlay: Option<&'a CapabilityOverlay>,
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
            sparse_reference_value_bindings: Vec::new(),
            host_formula_context: None,
            host_name_bindings: Vec::new(),
            host_reference_bind_results: Vec::new(),
            table_catalog: Vec::new(),
            enclosing_table_ref: None,
            caller_table_region: None,
            library_context: ConsumerLibraryContextState::new(),
            oxfunc_bridge_metadata: RuntimeOxFuncBridgeMetadata::default(),
            function_registry: builtin_registry(),
            function_registry_explicit: false,
            capability_overlay: None,
        }
    }

    pub fn execute<'q>(
        &self,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeFormulaResult, String> {
        let mut host = self.build_host(request.source());
        self.execute_with_host(&mut host, request)
    }

    pub fn formula_drill_trace_for_source(&self, source: FormulaSourceRecord) -> FormulaDrillTrace {
        let parse = parse_formula(ParseRequest {
            source: source.clone(),
        });
        build_formula_drill_trace(
            &source,
            &parse.green_tree.diagnostics,
            &[],
            &EvalValue::Error(WorksheetErrorCode::Value),
        )
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

    pub fn with_sparse_reference_value_bindings(
        mut self,
        sparse_reference_value_bindings: Vec<RuntimeSparseReferenceValuesBinding>,
    ) -> Self {
        self.sparse_reference_value_bindings = sparse_reference_value_bindings;
        self
    }

    pub fn with_host_formula_context(
        mut self,
        host_formula_context: RuntimeHostFormulaContext,
    ) -> Self {
        self.host_formula_context = Some(host_formula_context);
        self
    }

    pub fn with_host_name_bindings(
        mut self,
        host_name_bindings: Vec<RuntimeHostNameBinding>,
    ) -> Self {
        self.host_name_bindings = host_name_bindings;
        self
    }

    pub fn with_host_reference_bind_results(
        mut self,
        host_reference_bind_results: Vec<RuntimeHostReferenceBindResult>,
    ) -> Self {
        self.host_reference_bind_results = host_reference_bind_results;
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

    fn table_context_fingerprint(&self) -> Option<String> {
        runtime_table_context_fingerprint(
            &self.table_catalog,
            &self.enclosing_table_ref,
            &self.caller_table_region,
        )
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

    pub fn with_function_registry(mut self, function_registry: &'a FunctionRegistry) -> Self {
        self.function_registry = function_registry;
        self.function_registry_explicit = true;
        self
    }

    pub fn with_capability_overlay(mut self, capability_overlay: &'a CapabilityOverlay) -> Self {
        self.capability_overlay = Some(capability_overlay);
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
            &self.host_formula_context,
            &self.host_name_bind_results(),
            &self.host_reference_bind_results,
            self.runtime_registry_view_identity().as_ref(),
            &compiled.registry_capability_denials,
            self.table_context_fingerprint(),
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
        let runtime_library_context_snapshot = self.runtime_registry_library_context_snapshot(
            self.library_context.pinned_view().resolve_snapshot(),
        );
        let runtime_library_context_view = if self.has_active_registry_view() {
            PinnedLibraryContextView::new(None, None, runtime_library_context_snapshot.as_ref())
        } else {
            self.library_context.pinned_view()
        };
        let output = host.recalc_with_library_context_view(
            request.backend(),
            request.typed_query_bundle,
            runtime_library_context_view,
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
        host.defined_names
            .extend(host_name_defined_names(&self.host_name_bindings));
        host.cell_values = self.cell_values.clone();
        host.sparse_reference_values =
            sparse_reference_values_map(&self.sparse_reference_value_bindings);
        host.table_catalog = self.table_catalog.clone();
        host.enclosing_table_ref = self.enclosing_table_ref.clone();
        host.caller_table_region = self.caller_table_region.clone();
    }

    fn has_active_registry_view(&self) -> bool {
        self.function_registry_explicit || self.capability_overlay.is_some()
    }

    fn runtime_registry_view_identity(&self) -> Option<RuntimeFunctionRegistryViewIdentity> {
        if !self.has_active_registry_view() {
            return None;
        }
        Some(RuntimeFunctionRegistryViewIdentity {
            registry_snapshot_identity: self.function_registry.snapshot_identity().stable_key(),
            capability_overlay_identity: self
                .capability_overlay
                .map(|overlay| format!("capability-overlay:{}", runtime_hash_debug(overlay))),
        })
    }

    fn host_name_bind_results(&self) -> Vec<RuntimeHostNameBindResult> {
        self.host_name_bindings
            .iter()
            .map(|binding| binding.bind_result.clone())
            .collect()
    }

    fn runtime_registry_library_context_snapshot(
        &self,
        base_snapshot: Option<LibraryContextSnapshot>,
    ) -> Option<LibraryContextSnapshot> {
        if !self.has_active_registry_view() {
            return base_snapshot;
        }

        let view_identity = self
            .runtime_registry_view_identity()
            .expect("active registry view should produce identity");
        let mut entries = Vec::new();
        let mut seen_surface_names = BTreeSet::new();

        if let Some(base_snapshot) = base_snapshot {
            for entry in base_snapshot.entries {
                seen_surface_names.insert(entry.surface_name.to_ascii_uppercase());
                entries.push(entry);
            }
        }

        for entry in self.function_registry.iter() {
            if seen_surface_names.contains(&entry.surface_name.to_ascii_uppercase()) {
                continue;
            }
            entries.push(runtime_registry_snapshot_entry(
                entry,
                self.capability_overlay,
            ));
        }

        Some(LibraryContextSnapshot {
            snapshot_id: "runtime-function-registry-view".to_string(),
            snapshot_version: runtime_hash_debug(&(&view_identity, &entries)),
            entries,
        })
    }
}

fn runtime_registry_snapshot_entry(
    entry: &FunctionEntry,
    capability_overlay: Option<&CapabilityOverlay>,
) -> LibraryContextSnapshotEntry {
    let availability = capability_overlay
        .map(|overlay| overlay.availability_for(&entry.meta.function_id))
        .unwrap_or(FunctionAvailability::Available);
    let runtime_capability_state = match availability {
        FunctionAvailability::Available => Some(LibraryAvailabilityState::CatalogKnown),
        FunctionAvailability::Unavailable { .. } => {
            Some(LibraryAvailabilityState::HostProfileUnavailable)
        }
    };

    LibraryContextSnapshotEntry {
        surface_name: entry.surface_name.clone(),
        canonical_id: Some(entry.meta.function_id.clone()),
        surface_stable_id: entry
            .registry_metadata
            .surface_stable_id
            .clone()
            .or_else(|| Some(entry.meta.function_id.clone())),
        name_resolution_table_ref: entry.registry_metadata.name_resolution_table_ref.clone(),
        semantic_trait_profile_ref: entry
            .registry_metadata
            .semantic_trait_profile_ref
            .clone()
            .or_else(|| Some(entry.meta.semantic_kernel_metadata_version.clone())),
        gating_profile_ref: entry.registry_metadata.gating_profile_ref.clone(),
        metadata_status: entry.registry_metadata.metadata_status.clone(),
        special_interface_kind: entry.registry_metadata.special_interface_kind.clone(),
        admission_interface_kind: entry.registry_metadata.admission_interface_kind.clone(),
        preparation_owner: entry.registry_metadata.preparation_owner.clone(),
        runtime_boundary_kind: entry.registry_metadata.runtime_boundary_kind.clone(),
        interface_contract_ref: entry.registry_metadata.interface_contract_ref.clone(),
        registration_source_kind: runtime_registration_source_kind(&entry.source),
        parse_bind_state: LibraryAvailabilityState::CatalogKnown,
        semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
        runtime_capability_state,
        post_dispatch_state: None,
    }
}

fn runtime_registration_source_kind(source: &FunctionSource) -> RegistrationSourceKind {
    match source {
        FunctionSource::BuiltIn => RegistrationSourceKind::BuiltIn,
        FunctionSource::Udf { .. } => RegistrationSourceKind::UserDefined,
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

fn host_name_defined_names(
    host_name_bindings: &[RuntimeHostNameBinding],
) -> BTreeMap<String, DefinedNameBinding> {
    host_name_bindings
        .iter()
        .map(|binding| {
            (
                binding.bind_result.canonical_name.clone(),
                binding.binding.clone(),
            )
        })
        .collect()
}

fn host_name_caller_context_dependencies(
    host_name_bindings: &[RuntimeHostNameBinding],
) -> BTreeMap<String, bool> {
    host_name_bindings
        .iter()
        .map(|binding| {
            (
                binding.bind_result.canonical_name.clone(),
                binding.bind_result.caller_context_dependent,
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

fn sparse_reference_values_map(
    bindings: &[RuntimeSparseReferenceValuesBinding],
) -> BTreeMap<String, SparseReferenceValuesBinding> {
    bindings
        .iter()
        .map(|binding| {
            (
                binding.reference.target.clone(),
                SparseReferenceValuesBinding {
                    reference: binding.reference.clone(),
                    values: binding.resolved_values(),
                },
            )
        })
        .collect()
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
    pub host_formula_context: Option<RuntimeHostFormulaContext>,
    pub host_name_bind_results: Vec<RuntimeHostNameBindResult>,
    pub host_reference_bind_results: Vec<RuntimeHostReferenceBindResult>,
    pub structured_reference_bind_records: Vec<StructuredReferenceBindRecord>,
    pub formula_drill_trace: Option<FormulaDrillTrace>,
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
        let formula_drill_trace = Some(build_formula_drill_trace(
            &host_output.source,
            &host_output.syntax_diagnostics,
            &host_output.evaluation.trace.prepared_calls,
            &host_output.published_worksheet_value,
        ));
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
            host_formula_context: prepared_formula_identity.host_formula_context.clone(),
            host_name_bind_results: prepared_formula_identity.host_name_bind_results.clone(),
            host_reference_bind_results: prepared_formula_identity
                .host_reference_bind_results
                .clone(),
            structured_reference_bind_records: prepared_formula_identity
                .structured_reference_bind_records
                .clone(),
            prepared_formula_identity,
            formula_drill_trace,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormulaDrillTrace {
    pub schema_id: &'static str,
    pub formula_stable_id: String,
    pub source_text: String,
    pub root_node_id: FormulaDrillNodeId,
    pub nodes: Vec<FormulaDrillTraceNode>,
    pub evaluation_order: Vec<FormulaDrillNodeId>,
    pub diagnostics: Vec<FormulaDrillDiagnosticLink>,
    pub final_value: EvalValue,
    pub projection_losses: Vec<FormulaDrillProjectionLoss>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormulaDrillNodeId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct FormulaDrillTraceNode {
    pub node_id: FormulaDrillNodeId,
    pub parent_node_id: Option<FormulaDrillNodeId>,
    pub source_span: Option<TextSpan>,
    pub expression_text: Option<String>,
    pub kind: FormulaDrillNodeKind,
    pub function_id: Option<String>,
    pub function_surface_name: Option<String>,
    pub operator_kind: Option<String>,
    pub argument_ordinal: Option<usize>,
    pub argument_name: Option<String>,
    pub argument_role: Option<FormulaArgumentRole>,
    pub argument_name_source: FormulaArgumentNameSource,
    pub label_user: String,
    pub label_developer: String,
    pub evaluation_state: FormulaDrillEvaluationState,
    pub branch_disposition: Option<FormulaDrillBranchDisposition>,
    pub value_before_coercion: Option<EvalValue>,
    pub value_after_coercion: Option<EvalValue>,
    pub returned_value: Option<EvalValue>,
    pub published_value: Option<EvalValue>,
    pub value_preview: Option<FormulaDrillValuePreview>,
    pub error: Option<FormulaDrillError>,
    pub child_node_ids: Vec<FormulaDrillNodeId>,
    pub prepared_call_index: Option<usize>,
    pub prepared_argument_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaDrillNodeKind {
    FormulaRoot,
    FunctionCall,
    OperatorCall,
    Argument,
    Literal,
    NameReference,
    LetBinding,
    LambdaBinding,
    ArrayLiteral,
    SpillRange,
    RichValue,
    Error,
    DiagnosticPlaceholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaDrillEvaluationState {
    Pending,
    Bound,
    Evaluated,
    Skipped,
    ShortCircuited,
    Omitted,
    Blocked,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaDrillBranchDisposition {
    Taken,
    Skipped,
    NotReached,
    ErrorWhileChoosing,
    ErrorWhileEvaluatingBranch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaArgumentRole {
    LogicalTest,
    ValueIfTrue,
    ValueIfFalse,
    Number,
    Text,
    NameSlot,
    ValueSlot,
    BodyExpression,
    Array,
    Rows,
    Columns,
    Step,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaArgumentNameSource {
    NotApplicable,
    OxFuncMetadata,
    OxFmlSpecialForm,
    OrdinalFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaDrillValuePreview {
    pub value_kind: String,
    pub array_shape: Option<FormulaDrillArrayShape>,
    pub preview: Vec<String>,
    pub truncated: bool,
    pub rich_value_type_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormulaDrillArrayShape {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormulaDrillError {
    pub code: Option<String>,
    pub message: String,
    pub causal_node_id: Option<FormulaDrillNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaDrillDiagnosticLink {
    pub diagnostic_id: String,
    pub node_id: Option<FormulaDrillNodeId>,
    pub source_span: TextSpan,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaDrillProjectionLoss {
    pub node_id: Option<FormulaDrillNodeId>,
    pub loss_kind: String,
    pub message: String,
}

fn build_formula_drill_trace(
    source: &FormulaSourceRecord,
    syntax_diagnostics: &[SyntaxDiagnostic],
    prepared_calls: &[PreparedCall],
    final_value: &EvalValue,
) -> FormulaDrillTrace {
    let source_text = source.entered_formula_text.clone();
    let parsed = parse_drill_expression_source(&source_text);
    let mut builder = FormulaDrillTraceBuilder::new(prepared_calls, final_value);
    let root_span = TextSpan::new(0, source_text.len());
    let root_node_id = builder.push_node(FormulaDrillTraceNode {
        node_id: FormulaDrillNodeId(String::new()),
        parent_node_id: None,
        source_span: Some(root_span),
        expression_text: Some(source_text.clone()),
        kind: FormulaDrillNodeKind::FormulaRoot,
        function_id: None,
        function_surface_name: None,
        operator_kind: None,
        argument_ordinal: None,
        argument_name: None,
        argument_role: None,
        argument_name_source: FormulaArgumentNameSource::NotApplicable,
        label_user: format!("Formula = {}", eval_value_label(final_value)),
        label_developer: format!("Formula = {:?}", final_value),
        evaluation_state: evaluation_state_for_value(final_value),
        branch_disposition: None,
        value_before_coercion: None,
        value_after_coercion: None,
        returned_value: Some(final_value.clone()),
        published_value: Some(final_value.clone()),
        value_preview: value_preview(final_value),
        error: error_for_value(final_value, None),
        child_node_ids: Vec::new(),
        prepared_call_index: None,
        prepared_argument_index: None,
    });

    if let Some(expr) = parsed {
        builder.build_expr_node(&expr, Some(root_node_id.clone()), false);
    } else if !syntax_diagnostics.is_empty() {
        builder.push_diagnostic_placeholder(
            Some(root_node_id.clone()),
            root_span,
            "formula syntax pending".to_string(),
        );
    }

    let diagnostics = syntax_diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| FormulaDrillDiagnosticLink {
            diagnostic_id: format!("syntax:{index}"),
            node_id: Some(root_node_id.clone()),
            source_span: diagnostic.span,
            message: diagnostic.message.clone(),
        })
        .collect();

    FormulaDrillTrace {
        schema_id: "oxfml.formula_drill_trace.v1",
        formula_stable_id: source.formula_stable_id.0.clone(),
        source_text,
        root_node_id,
        nodes: builder.nodes,
        evaluation_order: builder.evaluation_order,
        diagnostics,
        final_value: final_value.clone(),
        projection_losses: builder.projection_losses,
    }
}

struct FormulaDrillTraceBuilder<'a> {
    prepared_calls: &'a [PreparedCall],
    root_fallback_value: &'a EvalValue,
    nodes: Vec<FormulaDrillTraceNode>,
    evaluation_order: Vec<FormulaDrillNodeId>,
    projection_losses: Vec<FormulaDrillProjectionLoss>,
    next_node_ordinal: usize,
    consumed_prepared_calls: BTreeSet<usize>,
}

impl<'a> FormulaDrillTraceBuilder<'a> {
    fn new(prepared_calls: &'a [PreparedCall], root_fallback_value: &'a EvalValue) -> Self {
        Self {
            prepared_calls,
            root_fallback_value,
            nodes: Vec::new(),
            evaluation_order: Vec::new(),
            projection_losses: Vec::new(),
            next_node_ordinal: 0,
            consumed_prepared_calls: BTreeSet::new(),
        }
    }

    fn push_node(&mut self, mut node: FormulaDrillTraceNode) -> FormulaDrillNodeId {
        let id = FormulaDrillNodeId(format!("drill-node:{}", self.next_node_ordinal));
        self.next_node_ordinal += 1;
        node.node_id = id.clone();
        if let Some(parent_id) = node.parent_node_id.clone() {
            if let Some(parent) = self
                .nodes
                .iter_mut()
                .find(|candidate| candidate.node_id == parent_id)
            {
                parent.child_node_ids.push(id.clone());
            }
        }
        self.nodes.push(node);
        id
    }

    fn record_evaluation_order(&mut self, id: &FormulaDrillNodeId) {
        if self.nodes.iter().any(|node| {
            &node.node_id == id
                && matches!(
                    node.kind,
                    FormulaDrillNodeKind::FunctionCall | FormulaDrillNodeKind::OperatorCall
                )
                && matches!(
                    node.evaluation_state,
                    FormulaDrillEvaluationState::Evaluated | FormulaDrillEvaluationState::Error
                )
        }) {
            self.evaluation_order.push(id.clone());
        }
    }

    fn apply_prepared_call_to_node(
        &mut self,
        id: &FormulaDrillNodeId,
        label: &str,
        prepared: Option<&(usize, PreparedCall)>,
        skipped: bool,
        fallback_value: Option<EvalValue>,
    ) {
        let returned_value = prepared
            .and_then(|(_, call)| call.returned_value.clone())
            .or(fallback_value);
        if let Some(node) = self.nodes.iter_mut().find(|node| &node.node_id == id) {
            let causal_node_id = if matches!(node.kind, FormulaDrillNodeKind::OperatorCall) {
                Some(id.clone())
            } else {
                None
            };
            node.function_id = prepared
                .map(|(_, call)| call.function_id.to_string())
                .or_else(|| node.function_id.clone());
            node.label_user = drill_call_label(label, returned_value.as_ref(), skipped);
            node.label_developer = format!(
                "{} prepared_call={:?}",
                node.function_id.as_deref().unwrap_or(label),
                prepared.map(|(index, _)| *index)
            );
            node.evaluation_state = if skipped {
                FormulaDrillEvaluationState::Skipped
            } else {
                returned_value
                    .as_ref()
                    .map(evaluation_state_for_value)
                    .unwrap_or(FormulaDrillEvaluationState::Evaluated)
            };
            node.returned_value = returned_value.clone();
            node.value_preview = returned_value.as_ref().and_then(value_preview);
            node.error = returned_value
                .as_ref()
                .and_then(|value| error_for_value(value, causal_node_id));
            node.prepared_call_index = prepared.map(|(index, _)| *index);
        }
    }

    fn apply_prepared_argument_values(
        &mut self,
        parent: &FormulaDrillNodeId,
        call_index: usize,
        call: &PreparedCall,
    ) {
        let child_node_ids = self
            .nodes
            .iter()
            .find(|node| &node.node_id == parent)
            .map(|node| node.child_node_ids.clone())
            .unwrap_or_default();
        for child_node_id in child_node_ids {
            let Some(node) = self
                .nodes
                .iter_mut()
                .find(|node| node.node_id == child_node_id)
            else {
                continue;
            };
            if node.kind != FormulaDrillNodeKind::Argument {
                continue;
            }
            let Some(ordinal) = node.prepared_argument_index else {
                continue;
            };
            let Some(value) = prepared_argument_value(call, ordinal) else {
                continue;
            };
            node.value_before_coercion = Some(value.clone());
            node.value_after_coercion = Some(value.clone());
            node.value_preview = value_preview(&value);
            node.prepared_call_index = Some(call_index);
        }
    }

    fn build_expr_node(
        &mut self,
        expr: &DrillParsedExpr,
        parent: Option<FormulaDrillNodeId>,
        skipped: bool,
    ) -> FormulaDrillNodeId {
        match &expr.kind {
            DrillParsedExprKind::Function { name, args, closed } => {
                self.build_function_node(expr, name, args, *closed, parent, skipped)
            }
            DrillParsedExprKind::Binary { op, left, right } => {
                self.build_operator_node(expr, *op, left, right, parent, skipped)
            }
            DrillParsedExprKind::Number
            | DrillParsedExprKind::String
            | DrillParsedExprKind::Logical(_)
            | DrillParsedExprKind::Name => self.build_leaf_node(expr, parent, skipped),
            DrillParsedExprKind::Missing => self.push_diagnostic_placeholder(
                parent,
                expr.span,
                "missing expression".to_string(),
            ),
        }
    }

    fn build_function_node(
        &mut self,
        expr: &DrillParsedExpr,
        name: &str,
        args: &[DrillParsedArgument],
        closed: bool,
        parent: Option<FormulaDrillNodeId>,
        skipped: bool,
    ) -> FormulaDrillNodeId {
        let node_id = self.push_node(FormulaDrillTraceNode {
            node_id: FormulaDrillNodeId(String::new()),
            parent_node_id: parent,
            source_span: Some(expr.span),
            expression_text: Some(expr.text.clone()),
            kind: FormulaDrillNodeKind::FunctionCall,
            function_id: None,
            function_surface_name: Some(name.to_string()),
            operator_kind: None,
            argument_ordinal: None,
            argument_name: None,
            argument_role: None,
            argument_name_source: FormulaArgumentNameSource::NotApplicable,
            label_user: drill_call_label(name, None, skipped),
            label_developer: format!("{name} prepared_call=None"),
            evaluation_state: if skipped {
                FormulaDrillEvaluationState::Skipped
            } else {
                FormulaDrillEvaluationState::Pending
            },
            branch_disposition: None,
            value_before_coercion: None,
            value_after_coercion: None,
            returned_value: None,
            published_value: None,
            value_preview: None,
            error: None,
            child_node_ids: Vec::new(),
            prepared_call_index: None,
            prepared_argument_index: None,
        });

        if name.eq_ignore_ascii_case("LET") {
            self.build_let_children(args, node_id.clone(), skipped);
        } else if name.eq_ignore_ascii_case("IF") {
            self.build_if_children(args, node_id.clone(), skipped);
        } else if name.eq_ignore_ascii_case("LAMBDA") {
            self.build_lambda_children(args, node_id.clone(), skipped);
        } else {
            for (ordinal, arg) in args.iter().enumerate() {
                self.build_argument_node(name, ordinal, arg, node_id.clone(), skipped, None, None);
            }
        }

        let prepared = if skipped {
            None
        } else {
            self.take_prepared_call(name, None)
        };
        self.apply_prepared_call_to_node(&node_id, name, prepared.as_ref(), skipped, None);
        if let Some((index, call)) = prepared.as_ref() {
            self.apply_prepared_argument_values(&node_id, *index, call);
        }

        if !closed {
            self.push_diagnostic_placeholder(
                Some(node_id.clone()),
                TextSpan::new(expr.span.end(), 0),
                "expected ')'".to_string(),
            );
        }
        self.record_evaluation_order(&node_id);
        node_id
    }

    fn build_operator_node(
        &mut self,
        expr: &DrillParsedExpr,
        op: DrillParsedOperator,
        left: &DrillParsedExpr,
        right: &DrillParsedExpr,
        parent: Option<FormulaDrillNodeId>,
        skipped: bool,
    ) -> FormulaDrillNodeId {
        let function_id = match op {
            DrillParsedOperator::Divide => FUNC_ID_OP_DIVIDE,
        };
        let fallback_value = if parent.is_some() {
            None
        } else {
            Some(self.root_fallback_value.clone())
        };
        let node_id = self.push_node(FormulaDrillTraceNode {
            node_id: FormulaDrillNodeId(String::new()),
            parent_node_id: parent,
            source_span: Some(expr.span),
            expression_text: Some(expr.text.clone()),
            kind: FormulaDrillNodeKind::OperatorCall,
            function_id: Some(function_id.to_string()),
            function_surface_name: None,
            operator_kind: Some(op.label().to_string()),
            argument_ordinal: None,
            argument_name: None,
            argument_role: None,
            argument_name_source: FormulaArgumentNameSource::NotApplicable,
            label_user: drill_call_label(op.label(), None, skipped),
            label_developer: format!("{function_id} prepared_call=None"),
            evaluation_state: if skipped {
                FormulaDrillEvaluationState::Skipped
            } else {
                FormulaDrillEvaluationState::Pending
            },
            branch_disposition: None,
            value_before_coercion: None,
            value_after_coercion: None,
            returned_value: None,
            published_value: None,
            value_preview: None,
            error: None,
            child_node_ids: Vec::new(),
            prepared_call_index: None,
            prepared_argument_index: None,
        });
        self.build_argument_expr_node("left", 0, left, node_id.clone(), skipped);
        self.build_argument_expr_node("right", 1, right, node_id.clone(), skipped);
        let prepared = if skipped {
            None
        } else {
            self.take_prepared_call("OP_DIVIDE", Some(function_id))
        };
        self.apply_prepared_call_to_node(
            &node_id,
            op.label(),
            prepared.as_ref(),
            skipped,
            fallback_value,
        );
        if let Some((index, call)) = prepared.as_ref() {
            self.apply_prepared_argument_values(&node_id, *index, call);
        }
        self.record_evaluation_order(&node_id);
        node_id
    }

    fn build_leaf_node(
        &mut self,
        expr: &DrillParsedExpr,
        parent: Option<FormulaDrillNodeId>,
        skipped: bool,
    ) -> FormulaDrillNodeId {
        let value = if skipped {
            None
        } else {
            literal_eval_value(expr)
        };
        let kind = match expr.kind {
            DrillParsedExprKind::Name => FormulaDrillNodeKind::NameReference,
            _ => FormulaDrillNodeKind::Literal,
        };
        self.push_node(FormulaDrillTraceNode {
            node_id: FormulaDrillNodeId(String::new()),
            parent_node_id: parent,
            source_span: Some(expr.span),
            expression_text: Some(expr.text.clone()),
            kind,
            function_id: None,
            function_surface_name: None,
            operator_kind: None,
            argument_ordinal: None,
            argument_name: None,
            argument_role: None,
            argument_name_source: FormulaArgumentNameSource::NotApplicable,
            label_user: if skipped {
                format!("{} skipped", expr.text)
            } else if let Some(value) = value.as_ref() {
                format!("{} = {}", expr.text, eval_value_label(value))
            } else {
                expr.text.clone()
            },
            label_developer: expr.text.clone(),
            evaluation_state: if skipped {
                FormulaDrillEvaluationState::Skipped
            } else {
                value
                    .as_ref()
                    .map(evaluation_state_for_value)
                    .unwrap_or(FormulaDrillEvaluationState::Bound)
            },
            branch_disposition: None,
            value_before_coercion: value.clone(),
            value_after_coercion: value.clone(),
            returned_value: value.clone(),
            published_value: None,
            value_preview: value.as_ref().and_then(value_preview),
            error: value
                .as_ref()
                .and_then(|value| error_for_value(value, None)),
            child_node_ids: Vec::new(),
            prepared_call_index: None,
            prepared_argument_index: None,
        })
    }

    fn build_let_children(
        &mut self,
        args: &[DrillParsedArgument],
        parent: FormulaDrillNodeId,
        skipped: bool,
    ) {
        let mut index = 0;
        while index + 1 < args.len().saturating_sub(1) {
            let name = args[index].text.trim().to_string();
            let value_arg = &args[index + 1];
            let binding_id = self.push_node(FormulaDrillTraceNode {
                node_id: FormulaDrillNodeId(String::new()),
                parent_node_id: Some(parent.clone()),
                source_span: Some(TextSpan::covering(args[index].span, value_arg.span)),
                expression_text: Some(format!("{name},{}", value_arg.text)),
                kind: FormulaDrillNodeKind::LetBinding,
                function_id: None,
                function_surface_name: None,
                operator_kind: None,
                argument_ordinal: Some(index),
                argument_name: Some(name.clone()),
                argument_role: Some(FormulaArgumentRole::NameSlot),
                argument_name_source: FormulaArgumentNameSource::OxFmlSpecialForm,
                label_user: format!("bind {name}"),
                label_developer: format!("LET binding {name}"),
                evaluation_state: if skipped {
                    FormulaDrillEvaluationState::Skipped
                } else {
                    FormulaDrillEvaluationState::Bound
                },
                branch_disposition: None,
                value_before_coercion: None,
                value_after_coercion: None,
                returned_value: None,
                published_value: None,
                value_preview: None,
                error: None,
                child_node_ids: Vec::new(),
                prepared_call_index: None,
                prepared_argument_index: None,
            });
            if let Some(expr) = value_arg.expr.as_ref() {
                self.build_expr_node(expr, Some(binding_id), skipped);
            }
            index += 2;
        }
        if let Some(body) = args.last() {
            self.build_argument_expr_node(
                "body",
                args.len() - 1,
                body.expr_or_missing(),
                parent,
                skipped,
            );
        }
    }

    fn build_lambda_children(
        &mut self,
        args: &[DrillParsedArgument],
        parent: FormulaDrillNodeId,
        skipped: bool,
    ) {
        for (ordinal, arg) in args.iter().enumerate() {
            let role = if ordinal + 1 == args.len() {
                FormulaArgumentRole::BodyExpression
            } else {
                FormulaArgumentRole::NameSlot
            };
            self.build_argument_node_with_metadata(
                arg.text.trim(),
                ordinal,
                arg,
                parent.clone(),
                skipped,
                Some(role),
                FormulaArgumentNameSource::OxFmlSpecialForm,
                None,
                None,
            );
        }
    }

    fn build_if_children(
        &mut self,
        args: &[DrillParsedArgument],
        parent: FormulaDrillNodeId,
        skipped: bool,
    ) {
        let test_truth = args
            .first()
            .and_then(|arg| arg.expr.as_ref())
            .and_then(drill_literal_truth);
        for (ordinal, arg) in args.iter().enumerate() {
            let (name, role) = match ordinal {
                0 => ("logical_test", FormulaArgumentRole::LogicalTest),
                1 => ("value_if_true", FormulaArgumentRole::ValueIfTrue),
                2 => ("value_if_false", FormulaArgumentRole::ValueIfFalse),
                _ => ("arg", FormulaArgumentRole::Other("if_extra".to_string())),
            };
            let branch_disposition = if skipped {
                Some(FormulaDrillBranchDisposition::Skipped)
            } else {
                match (ordinal, test_truth) {
                    (0, _) => None,
                    (1, Some(true)) | (2, Some(false)) => {
                        Some(FormulaDrillBranchDisposition::Taken)
                    }
                    (1, Some(false)) | (2, Some(true)) => {
                        Some(FormulaDrillBranchDisposition::Skipped)
                    }
                    (1 | 2, None) => Some(FormulaDrillBranchDisposition::NotReached),
                    _ => None,
                }
            };
            let arg_skipped = skipped
                || matches!(
                    branch_disposition,
                    Some(
                        FormulaDrillBranchDisposition::Skipped
                            | FormulaDrillBranchDisposition::NotReached
                    )
                );
            self.build_argument_node_with_metadata(
                name,
                ordinal,
                arg,
                parent.clone(),
                arg_skipped,
                Some(role),
                FormulaArgumentNameSource::OxFuncMetadata,
                branch_disposition,
                None,
            );
        }
    }

    fn build_argument_node(
        &mut self,
        function_name: &str,
        ordinal: usize,
        arg: &DrillParsedArgument,
        parent: FormulaDrillNodeId,
        skipped: bool,
        branch_disposition: Option<FormulaDrillBranchDisposition>,
        prepared_argument_value: Option<EvalValue>,
    ) {
        let metadata = argument_metadata(function_name, ordinal);
        let arg_id = self.push_argument_shell(
            metadata.name,
            ordinal,
            arg,
            Some(parent),
            skipped,
            metadata.role,
            metadata.name_source,
            branch_disposition,
            prepared_argument_value,
        );
        if let Some(expr) = arg.expr.as_ref() {
            self.build_expr_node(expr, Some(arg_id), skipped);
        }
    }

    fn build_argument_node_with_metadata(
        &mut self,
        name: &str,
        ordinal: usize,
        arg: &DrillParsedArgument,
        parent: FormulaDrillNodeId,
        skipped: bool,
        role: Option<FormulaArgumentRole>,
        name_source: FormulaArgumentNameSource,
        branch_disposition: Option<FormulaDrillBranchDisposition>,
        prepared_argument_value: Option<EvalValue>,
    ) {
        let arg_id = self.push_argument_shell(
            name.to_string(),
            ordinal,
            arg,
            Some(parent),
            skipped,
            role,
            name_source,
            branch_disposition,
            prepared_argument_value,
        );
        if let Some(expr) = arg.expr.as_ref() {
            self.build_expr_node(expr, Some(arg_id), skipped);
        }
    }

    fn build_argument_expr_node(
        &mut self,
        name: &str,
        ordinal: usize,
        expr: &DrillParsedExpr,
        parent: FormulaDrillNodeId,
        skipped: bool,
    ) {
        let arg = DrillParsedArgument {
            span: expr.span,
            text: expr.text.clone(),
            expr: Some(expr.clone()),
        };
        self.build_argument_node_with_metadata(
            name,
            ordinal,
            &arg,
            parent,
            skipped,
            Some(FormulaArgumentRole::Other(name.to_string())),
            FormulaArgumentNameSource::OxFmlSpecialForm,
            None,
            None,
        );
    }

    fn push_argument_shell(
        &mut self,
        name: String,
        ordinal: usize,
        arg: &DrillParsedArgument,
        parent: Option<FormulaDrillNodeId>,
        skipped: bool,
        role: Option<FormulaArgumentRole>,
        name_source: FormulaArgumentNameSource,
        branch_disposition: Option<FormulaDrillBranchDisposition>,
        prepared_argument_value: Option<EvalValue>,
    ) -> FormulaDrillNodeId {
        let value_preview = prepared_argument_value.as_ref().and_then(value_preview);
        self.push_node(FormulaDrillTraceNode {
            node_id: FormulaDrillNodeId(String::new()),
            parent_node_id: parent,
            source_span: Some(arg.span),
            expression_text: Some(arg.text.clone()),
            kind: FormulaDrillNodeKind::Argument,
            function_id: None,
            function_surface_name: None,
            operator_kind: None,
            argument_ordinal: Some(ordinal),
            argument_name: Some(name.clone()),
            argument_role: role,
            argument_name_source: name_source,
            label_user: if skipped {
                format!("{name}: {} skipped", arg.text)
            } else {
                format!("{name}: {}", arg.text)
            },
            label_developer: format!("arg[{ordinal}] {name}: {}", arg.text),
            evaluation_state: if skipped {
                FormulaDrillEvaluationState::Skipped
            } else {
                FormulaDrillEvaluationState::Bound
            },
            branch_disposition,
            value_before_coercion: prepared_argument_value.clone(),
            value_after_coercion: prepared_argument_value,
            returned_value: None,
            published_value: None,
            value_preview,
            error: None,
            child_node_ids: Vec::new(),
            prepared_call_index: None,
            prepared_argument_index: Some(ordinal),
        })
    }

    fn push_diagnostic_placeholder(
        &mut self,
        parent: Option<FormulaDrillNodeId>,
        span: TextSpan,
        message: String,
    ) -> FormulaDrillNodeId {
        self.push_node(FormulaDrillTraceNode {
            node_id: FormulaDrillNodeId(String::new()),
            parent_node_id: parent,
            source_span: Some(span),
            expression_text: None,
            kind: FormulaDrillNodeKind::DiagnosticPlaceholder,
            function_id: None,
            function_surface_name: None,
            operator_kind: None,
            argument_ordinal: None,
            argument_name: None,
            argument_role: None,
            argument_name_source: FormulaArgumentNameSource::NotApplicable,
            label_user: message.clone(),
            label_developer: message,
            evaluation_state: FormulaDrillEvaluationState::Pending,
            branch_disposition: None,
            value_before_coercion: None,
            value_after_coercion: None,
            returned_value: None,
            published_value: None,
            value_preview: None,
            error: None,
            child_node_ids: Vec::new(),
            prepared_call_index: None,
            prepared_argument_index: None,
        })
    }

    fn take_prepared_call(
        &mut self,
        function_name: &str,
        function_id: Option<&str>,
    ) -> Option<(usize, crate::eval::PreparedCall)> {
        let normalized_name = function_name.to_ascii_uppercase();
        let found = self
            .prepared_calls
            .iter()
            .enumerate()
            .find(|(index, call)| {
                !self.consumed_prepared_calls.contains(index)
                    && (call.function_name.eq_ignore_ascii_case(&normalized_name)
                        || call.function_name.eq_ignore_ascii_case(function_name)
                        || function_id.is_some_and(|id| call.function_id == id))
            })
            .map(|(index, call)| (index, call.clone()));
        if let Some((index, _)) = &found {
            self.consumed_prepared_calls.insert(*index);
        }
        found
    }
}

fn prepared_argument_value(call: &PreparedCall, ordinal: usize) -> Option<EvalValue> {
    call.prepared_arguments
        .iter()
        .find(|argument| argument.ordinal == ordinal)
        .and_then(|argument| argument.resolved_value.clone())
}

#[derive(Debug, Clone, PartialEq)]
struct DrillParsedExpr {
    span: TextSpan,
    text: String,
    kind: DrillParsedExprKind,
}

#[derive(Debug, Clone, PartialEq)]
enum DrillParsedExprKind {
    Function {
        name: String,
        args: Vec<DrillParsedArgument>,
        closed: bool,
    },
    Binary {
        op: DrillParsedOperator,
        left: Box<DrillParsedExpr>,
        right: Box<DrillParsedExpr>,
    },
    Number,
    String,
    Logical(bool),
    Name,
    Missing,
}

#[derive(Debug, Clone, PartialEq)]
struct DrillParsedArgument {
    span: TextSpan,
    text: String,
    expr: Option<DrillParsedExpr>,
}

impl DrillParsedArgument {
    fn expr_or_missing(&self) -> &DrillParsedExpr {
        self.expr.as_ref().expect("argument expression expected")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrillParsedOperator {
    Divide,
}

impl DrillParsedOperator {
    fn label(self) -> &'static str {
        match self {
            Self::Divide => "divide",
        }
    }
}

fn parse_drill_expression_source(source_text: &str) -> Option<DrillParsedExpr> {
    let (start, text) = if let Some(rest) = source_text.strip_prefix('=') {
        (1, rest)
    } else {
        (0, source_text)
    };
    parse_drill_expr_at(text, start)
}

fn parse_drill_expr_at(text: &str, base: usize) -> Option<DrillParsedExpr> {
    let leading = text.len() - text.trim_start().len();
    let trailing = text.trim_end().len();
    let trimmed = &text[leading..trailing];
    let start = base + leading;
    if trimmed.is_empty() {
        return Some(DrillParsedExpr {
            span: TextSpan::new(start, 0),
            text: String::new(),
            kind: DrillParsedExprKind::Missing,
        });
    }
    if let Some((op_index, op)) = find_top_level_operator(trimmed) {
        let left_text = &trimmed[..op_index];
        let right_text = &trimmed[op_index + 1..];
        let left = parse_drill_expr_at(left_text, start)?;
        let right = parse_drill_expr_at(right_text, start + op_index + 1)?;
        return Some(DrillParsedExpr {
            span: TextSpan::new(start, trimmed.len()),
            text: trimmed.to_string(),
            kind: DrillParsedExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
        });
    }
    if let Some(open_index) = function_open_paren_index(trimmed) {
        let name = trimmed[..open_index].trim().to_string();
        let close_index = matching_trailing_paren_index(trimmed, open_index);
        let closed = close_index.is_some();
        let args_end = close_index.unwrap_or(trimmed.len());
        let args_text = &trimmed[open_index + 1..args_end];
        let args_base = start + open_index + 1;
        let mut args = split_drill_arguments(args_text, args_base);
        if args.is_empty() && !closed {
            args.push(DrillParsedArgument {
                span: TextSpan::new(args_base, 0),
                text: String::new(),
                expr: Some(DrillParsedExpr {
                    span: TextSpan::new(args_base, 0),
                    text: String::new(),
                    kind: DrillParsedExprKind::Missing,
                }),
            });
        }
        return Some(DrillParsedExpr {
            span: TextSpan::new(start, trimmed.len()),
            text: trimmed.to_string(),
            kind: DrillParsedExprKind::Function { name, args, closed },
        });
    }
    let kind = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        DrillParsedExprKind::String
    } else if trimmed.eq_ignore_ascii_case("TRUE") {
        DrillParsedExprKind::Logical(true)
    } else if trimmed.eq_ignore_ascii_case("FALSE") {
        DrillParsedExprKind::Logical(false)
    } else if trimmed.parse::<f64>().is_ok() {
        DrillParsedExprKind::Number
    } else {
        DrillParsedExprKind::Name
    };
    Some(DrillParsedExpr {
        span: TextSpan::new(start, trimmed.len()),
        text: trimmed.to_string(),
        kind,
    })
}

fn find_top_level_operator(text: &str) -> Option<(usize, DrillParsedOperator)> {
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '{' => depth += 1,
            ')' | '}' => depth = depth.saturating_sub(1),
            '/' if depth == 0 => return Some((index, DrillParsedOperator::Divide)),
            _ => {}
        }
    }
    None
}

fn function_open_paren_index(text: &str) -> Option<usize> {
    let open_index = text.find('(')?;
    let name = text[..open_index].trim();
    if !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
    {
        Some(open_index)
    } else {
        None
    }
}

fn matching_trailing_paren_index(text: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut last_close = None;
    for (index, ch) in text
        .char_indices()
        .skip_while(|(index, _)| *index < open_index)
    {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    last_close = Some(index);
                }
            }
            _ => {}
        }
    }
    last_close.filter(|index| text[*index + 1..].trim().is_empty())
}

fn split_drill_arguments(text: &str, base: usize) -> Vec<DrillParsedArgument> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' | '{' => depth += 1,
            ')' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(drill_argument_from_slice(text, base, start, index));
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(drill_argument_from_slice(text, base, start, text.len()));
    args
}

fn drill_argument_from_slice(
    text: &str,
    base: usize,
    start: usize,
    end: usize,
) -> DrillParsedArgument {
    let raw = &text[start..end];
    let leading = raw.len() - raw.trim_start().len();
    let trailing = raw.trim_end().len();
    let trimmed = &raw[leading..trailing];
    let arg_start = base + start + leading;
    DrillParsedArgument {
        span: TextSpan::new(arg_start, trimmed.len()),
        text: trimmed.to_string(),
        expr: parse_drill_expr_at(trimmed, arg_start),
    }
}

struct FormulaArgumentMetadata {
    name: String,
    role: Option<FormulaArgumentRole>,
    name_source: FormulaArgumentNameSource,
}

fn argument_metadata(function_name: &str, ordinal: usize) -> FormulaArgumentMetadata {
    if let Some(entry) = builtin_registry().lookup_by_surface_name(function_name) {
        if let Some(parameter) = entry.display_signature.parameters.get(ordinal).or_else(|| {
            entry
                .display_signature
                .parameters
                .last()
                .filter(|parameter| parameter.repeats)
        }) {
            let name = if parameter.repeats {
                repeated_argument_name(&parameter.name, ordinal)
            } else {
                parameter.name.clone()
            };
            return FormulaArgumentMetadata {
                role: argument_role_for_name(&name),
                name,
                name_source: FormulaArgumentNameSource::OxFuncMetadata,
            };
        }
    }
    FormulaArgumentMetadata {
        name: format!("arg[{ordinal}]"),
        role: None,
        name_source: FormulaArgumentNameSource::OrdinalFallback,
    }
}

fn repeated_argument_name(base: &str, ordinal: usize) -> String {
    let stem = base.trim_end_matches(|ch: char| ch.is_ascii_digit());
    if stem.is_empty() {
        format!("{base}{}", ordinal + 1)
    } else {
        format!("{stem}{}", ordinal + 1)
    }
}

fn argument_role_for_name(name: &str) -> Option<FormulaArgumentRole> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "logical_test" => Some(FormulaArgumentRole::LogicalTest),
        "value_if_true" => Some(FormulaArgumentRole::ValueIfTrue),
        "value_if_false" => Some(FormulaArgumentRole::ValueIfFalse),
        "text" => Some(FormulaArgumentRole::Text),
        "array" => Some(FormulaArgumentRole::Array),
        "rows" => Some(FormulaArgumentRole::Rows),
        "columns" => Some(FormulaArgumentRole::Columns),
        "step" => Some(FormulaArgumentRole::Step),
        _ if lower.starts_with("number") => Some(FormulaArgumentRole::Number),
        _ => Some(FormulaArgumentRole::Other(name.to_string())),
    }
}

fn literal_eval_value(expr: &DrillParsedExpr) -> Option<EvalValue> {
    match &expr.kind {
        DrillParsedExprKind::Number => expr.text.parse::<f64>().ok().map(EvalValue::Number),
        DrillParsedExprKind::String => Some(EvalValue::Text(
            oxfunc_core::value::ExcelText::from_interop_assignment(expr.text.trim_matches('"')),
        )),
        DrillParsedExprKind::Logical(value) => Some(EvalValue::Logical(*value)),
        DrillParsedExprKind::Name | DrillParsedExprKind::Missing => None,
        DrillParsedExprKind::Function { .. } | DrillParsedExprKind::Binary { .. } => None,
    }
}

fn drill_literal_truth(expr: &DrillParsedExpr) -> Option<bool> {
    match expr.kind {
        DrillParsedExprKind::Logical(value) => Some(value),
        _ => None,
    }
}

fn drill_call_label(name: &str, value: Option<&EvalValue>, skipped: bool) -> String {
    if skipped {
        format!("{name} skipped")
    } else if let Some(value) = value {
        format!("{name} = {}", eval_value_label(value))
    } else {
        name.to_string()
    }
}

fn evaluation_state_for_value(value: &EvalValue) -> FormulaDrillEvaluationState {
    match value {
        EvalValue::Error(_) => FormulaDrillEvaluationState::Error,
        _ => FormulaDrillEvaluationState::Evaluated,
    }
}

fn error_for_value(
    value: &EvalValue,
    causal_node_id: Option<FormulaDrillNodeId>,
) -> Option<FormulaDrillError> {
    match value {
        EvalValue::Error(code) => Some(FormulaDrillError {
            code: Some(worksheet_error_code_text(*code).to_string()),
            message: worksheet_error_message(*code).to_string(),
            causal_node_id,
        }),
        _ => None,
    }
}

fn value_preview(value: &EvalValue) -> Option<FormulaDrillValuePreview> {
    match value {
        EvalValue::Array(array) => {
            let shape = array.shape();
            let preview = array
                .iter_row_major()
                .take(4)
                .map(array_cell_label)
                .collect::<Vec<_>>();
            Some(FormulaDrillValuePreview {
                value_kind: "array".to_string(),
                array_shape: Some(FormulaDrillArrayShape {
                    rows: shape.rows,
                    cols: shape.cols,
                }),
                preview,
                truncated: shape.cell_count() > 4,
                rich_value_type_name: None,
            })
        }
        EvalValue::Lambda(lambda) => Some(FormulaDrillValuePreview {
            value_kind: "lambda".to_string(),
            array_shape: None,
            preview: vec![format!(
                "callable:{}:{}",
                lambda.callable_token, lambda.invocation_contract_ref
            )],
            truncated: false,
            rich_value_type_name: None,
        }),
        _ => None,
    }
}

fn array_cell_label(value: &ArrayCellValue) -> String {
    match value {
        ArrayCellValue::Number(number) => format_number(*number),
        ArrayCellValue::Text(text) => text.to_string_lossy(),
        ArrayCellValue::Logical(value) => value.to_string().to_ascii_uppercase(),
        ArrayCellValue::Error(code) => worksheet_error_code_text(*code).to_string(),
        ArrayCellValue::EmptyCell => String::new(),
    }
}

fn eval_value_label(value: &EvalValue) -> String {
    match value {
        EvalValue::Number(number) => format_number(*number),
        EvalValue::Text(text) => text.to_string_lossy(),
        EvalValue::Logical(value) => value.to_string().to_ascii_uppercase(),
        EvalValue::Error(code) => worksheet_error_code_text(*code).to_string(),
        EvalValue::Array(array) => {
            let shape = array.shape();
            format!("Array({}x{})", shape.rows, shape.cols)
        }
        EvalValue::Reference(reference) => reference.target.clone(),
        EvalValue::Lambda(lambda) => format!("Lambda({})", lambda.callable_token),
    }
}

fn format_number(number: f64) -> String {
    if number.fract() == 0.0 {
        format!("{}", number as i64)
    } else {
        number.to_string()
    }
}

fn worksheet_error_code_text(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "#NULL!",
        WorksheetErrorCode::Div0 => "#DIV/0!",
        WorksheetErrorCode::Value => "#VALUE!",
        WorksheetErrorCode::Ref => "#REF!",
        WorksheetErrorCode::Name => "#NAME?",
        WorksheetErrorCode::Num => "#NUM!",
        WorksheetErrorCode::NA => "#N/A",
        WorksheetErrorCode::Busy => "#BUSY!",
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn worksheet_error_message(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Div0 => "division by zero",
        WorksheetErrorCode::Null => "invalid intersection",
        WorksheetErrorCode::Value => "value error",
        WorksheetErrorCode::Ref => "invalid reference",
        WorksheetErrorCode::Name => "unknown name",
        WorksheetErrorCode::Num => "numeric error",
        WorksheetErrorCode::NA => "not available",
        WorksheetErrorCode::Busy => "busy",
        WorksheetErrorCode::GettingData => "getting data",
        WorksheetErrorCode::Spill => "spill error",
        WorksheetErrorCode::Calc => "calculation error",
        WorksheetErrorCode::Field => "field error",
        WorksheetErrorCode::Blocked => "blocked",
        WorksheetErrorCode::Connect => "connection error",
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
    pub host_formula_context: Option<RuntimeHostFormulaContext>,
    pub host_name_bind_results: Vec<RuntimeHostNameBindResult>,
    pub host_reference_bind_results: Vec<RuntimeHostReferenceBindResult>,
    pub registry_snapshot_identity: Option<String>,
    pub capability_overlay_identity: Option<String>,
    pub registry_capability_denials: Vec<RuntimeFunctionCapabilityDenial>,
    pub table_context_fingerprint: Option<String>,
    pub structured_reference_bind_records: Vec<StructuredReferenceBindRecord>,
    pub plan_template: RuntimePlanTemplateIdentity,
    pub hole_binding: RuntimeHoleBindingIdentity,
    pub formal_references: Vec<RuntimeFormalReference>,
    pub projection_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFunctionRegistryViewIdentity {
    pub registry_snapshot_identity: String,
    pub capability_overlay_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFunctionCapabilityDenial {
    pub surface_name: String,
    pub canonical_id: Option<String>,
    pub runtime_capability_state: String,
}

impl RuntimePreparedFormulaIdentity {
    pub fn prepared_formula_package(&self) -> RuntimePreparedFormulaPackage {
        RuntimePreparedFormulaPackage {
            package_key: self.prepared_formula_key.clone(),
            identity: self.clone(),
            plan_template: self.plan_template.clone(),
            hole_binding: self.hole_binding.clone(),
            formal_references: self.formal_references.clone(),
            structured_reference_bind_records: self.structured_reference_bind_records.clone(),
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
    pub structured_reference_bind_records: Vec<StructuredReferenceBindRecord>,
    pub projection_status: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeOxFuncBridgeMetadata {
    pub semantic_kernel_metadata_version: Option<String>,
    pub arg_admission_metadata_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostFormulaContext {
    pub dialect_id: String,
    pub capability_profile_id: String,
    pub resolution_rule_version: String,
    pub host_namespace_version: Option<String>,
    pub registry_snapshot_identity: Option<String>,
    pub structure_context_version: Option<String>,
    pub caller_context_identity: Option<String>,
    pub table_context_identity: Option<String>,
    pub host_reference_syntax_rules: Vec<RuntimeHostReferenceSyntaxRule>,
}

impl RuntimeHostFormulaContext {
    pub fn cache_identity_contribution(&self) -> String {
        runtime_hash_debug(self)
    }

    #[must_use]
    pub fn declared_host_reference_syntax_matches(
        &self,
        source: &FormulaSourceRecord,
    ) -> Vec<RuntimeHostReferenceSyntaxMatch> {
        self.host_reference_syntax_rules
            .iter()
            .flat_map(|rule| declared_host_reference_syntax_matches(source, self, rule))
            .collect()
    }

    #[must_use]
    pub fn project_declared_host_reference_syntax(
        &self,
        source: &FormulaSourceRecord,
    ) -> RuntimeHostReferenceSyntaxProjection {
        self.project_host_reference_syntax_matches(
            source,
            self.declared_host_reference_syntax_matches(source),
        )
    }

    #[must_use]
    pub fn project_host_reference_syntax_matches(
        &self,
        source: &FormulaSourceRecord,
        matches: impl IntoIterator<Item = RuntimeHostReferenceSyntaxMatch>,
    ) -> RuntimeHostReferenceSyntaxProjection {
        let mut matches = matches.into_iter().collect::<Vec<_>>();
        matches.sort_by_key(|syntax_match| syntax_match.source_span.start);

        let mut projected_text = String::with_capacity(source.entered_formula_text.len());
        let mut accepted_matches = Vec::new();
        let mut diagnostics = Vec::new();
        let mut cursor = 0usize;
        for syntax_match in matches {
            let start = syntax_match.source_span.start;
            let end = syntax_match.source_span.end();
            if start < cursor {
                diagnostics.push(format!(
                    "overlapping_host_reference_syntax_match:{}:{}-{end}",
                    syntax_match.source_token_text, start
                ));
                continue;
            }
            projected_text.push_str(&source.entered_formula_text[cursor..start]);
            projected_text.push_str(&syntax_match.formal_token_text());
            cursor = end;
            accepted_matches.push(syntax_match);
        }
        if accepted_matches.is_empty() {
            return RuntimeHostReferenceSyntaxProjection {
                source: source.clone(),
                matches: accepted_matches,
                diagnostics,
            };
        }
        projected_text.push_str(&source.entered_formula_text[cursor..]);
        let mut projected_source = source.clone();
        projected_source.stored_formula_text = Some(source.entered_formula_text.clone());
        projected_source.entered_formula_text = projected_text;
        RuntimeHostReferenceSyntaxProjection {
            source: projected_source,
            matches: accepted_matches,
            diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostReferenceSyntaxProjection {
    pub source: FormulaSourceRecord,
    pub matches: Vec<RuntimeHostReferenceSyntaxMatch>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostReferenceSyntaxRule {
    pub rule_id: String,
    pub rule_family: String,
    pub pattern_text: String,
    pub token_kind: String,
    pub resolution_layer: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub opaque_selector_payload: Option<String>,
}

impl RuntimeHostReferenceSyntaxRule {
    #[must_use]
    pub fn literal(
        rule_id: impl Into<String>,
        rule_family: impl Into<String>,
        pattern_text: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            rule_family: rule_family.into(),
            pattern_text: pattern_text.into(),
            token_kind: "literal".to_string(),
            resolution_layer: "explicit_host_ref".to_string(),
            shape_hint: None,
            caller_context_dependent: false,
            opaque_selector_payload: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostReferenceSyntaxMatch {
    pub syntax_match_handle: String,
    pub rule_id: String,
    pub rule_family: String,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub token_kind: String,
    pub opaque_selector_payload: Option<String>,
    pub resolution_layer: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
}

impl RuntimeHostReferenceSyntaxMatch {
    #[must_use]
    pub fn formal_token_text(&self) -> String {
        format!(
            "HOST_REF_{}_{}",
            self.source_span.start, self.source_span.len
        )
    }

    #[must_use]
    pub fn unresolved_bind_result(&self) -> RuntimeHostReferenceBindResult {
        RuntimeHostReferenceBindResult {
            reference_handle: self.syntax_match_handle.clone(),
            formal_reference_id: Some(self.formal_token_text()),
            source_span: self.source_span,
            source_token_text: self.source_token_text.clone(),
            opaque_selector_payload: self.opaque_selector_payload.clone(),
            resolution_layer: self.resolution_layer.clone(),
            shape_hint: self.shape_hint.clone(),
            caller_context_dependent: self.caller_context_dependent,
            diagnostics: self.diagnostics.clone(),
            replay_identity_contribution: format!(
                "host-reference-syntax-match:{}:{}:{}:{}",
                self.rule_id, self.rule_family, self.source_span.start, self.source_span.len
            ),
        }
    }
}

fn declared_host_reference_syntax_matches(
    source: &FormulaSourceRecord,
    context: &RuntimeHostFormulaContext,
    rule: &RuntimeHostReferenceSyntaxRule,
) -> Vec<RuntimeHostReferenceSyntaxMatch> {
    let text = source.entered_formula_text.as_str();
    if rule.pattern_text.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let mut index = 0usize;
    let mut in_string = false;
    while index < text.len() {
        let Some(current) = text[index..].chars().next() else {
            break;
        };

        if in_string {
            if current == '"' {
                let next = index + current.len_utf8();
                if text[next..].starts_with('"') {
                    index = next + 1;
                } else {
                    in_string = false;
                    index = next;
                }
            } else {
                index += current.len_utf8();
            }
            continue;
        }
        if current == '"' {
            in_string = true;
            index += current.len_utf8();
            continue;
        }

        if text[index..].starts_with(rule.pattern_text.as_str()) {
            if rule.pattern_text == "{"
                && let Some(syntax_match) =
                    declared_host_reference_literal_array_syntax_match(text, index, context, rule)
            {
                index = syntax_match.source_span.end();
                matches.push(syntax_match);
                continue;
            }
            if rule.pattern_text == "^"
                && let Some(syntax_match) =
                    declared_host_reference_repeated_prefix_syntax_match(text, index, context, rule)
            {
                index = syntax_match.source_span.end();
                matches.push(syntax_match);
                continue;
            }
            if rule.pattern_text == "["
                && let Some(syntax_match) =
                    declared_host_reference_bracket_path_syntax_match(text, index, context, rule)
            {
                index = syntax_match.source_span.end();
                matches.push(syntax_match);
                continue;
            }
            let selector_end = index + rule.pattern_text.len();
            if host_reference_literal_pattern_is_prefix_of_longer_token(
                text,
                selector_end,
                &rule.pattern_text,
            ) {
                index += current.len_utf8();
                continue;
            }
            let qualified_base = qualified_host_reference_base(text, index, &rule.pattern_text);
            let qualified_tail = qualified_host_reference_tail(text, selector_end);
            let end = qualified_tail
                .as_ref()
                .map_or(selector_end, |tail| tail.source_end);
            let unqualified_match =
                host_syntax_boundary_before(text, index) && host_syntax_boundary_after(text, end);
            let qualified_match = qualified_base.is_some() && host_syntax_boundary_after(text, end);
            if !unqualified_match && !qualified_match {
                index += current.len_utf8();
                continue;
            }

            let span_start = qualified_base
                .as_ref()
                .map_or(index, |base| base.source_start);
            let span = TextSpan::new(span_start, end - span_start);
            let source_token_text = text[span.start..span.end()].to_string();
            let opaque_selector_payload = if qualified_base.is_some() || qualified_tail.is_some() {
                Some(host_reference_selector_payload_with_qualifiers(
                    rule.opaque_selector_payload.as_deref(),
                    qualified_base.as_ref().map(|base| base.token_text.as_str()),
                    qualified_tail.as_ref().map(|tail| tail.token_text.as_str()),
                ))
            } else {
                rule.opaque_selector_payload.clone()
            };
            matches.push(RuntimeHostReferenceSyntaxMatch {
                syntax_match_handle: format!(
                    "host-reference-syntax:{}:{}:{}:{}",
                    context.dialect_id, rule.rule_id, span.start, span.len
                ),
                rule_id: rule.rule_id.clone(),
                rule_family: rule.rule_family.clone(),
                source_span: span,
                source_token_text,
                token_kind: rule.token_kind.clone(),
                opaque_selector_payload,
                resolution_layer: rule.resolution_layer.clone(),
                shape_hint: rule.shape_hint.clone(),
                caller_context_dependent: rule.caller_context_dependent,
                diagnostics: Vec::new(),
            });
            index += rule.pattern_text.len();
            continue;
        }

        index += current.len_utf8();
    }
    matches
}

fn declared_host_reference_bracket_path_syntax_match(
    text: &str,
    start: usize,
    context: &RuntimeHostFormulaContext,
    rule: &RuntimeHostReferenceSyntaxRule,
) -> Option<RuntimeHostReferenceSyntaxMatch> {
    let qualified_base = if start > 0 && text[..start].ends_with('.') {
        qualified_host_reference_base_before_dot(text, start - 1)
    } else {
        None
    };
    if qualified_base.is_none() && !host_syntax_boundary_before(text, start) {
        return None;
    }
    let end = host_reference_path_token_end(text, start)?;
    if !host_syntax_boundary_after(text, end) {
        return None;
    }
    let span_start = qualified_base
        .as_ref()
        .map_or(start, |base| base.source_start);
    let span = TextSpan::new(span_start, end - span_start);
    let source_token_text = text[span.start..span.end()].to_string();
    let mut opaque_selector_payload = rule
        .opaque_selector_payload
        .clone()
        .unwrap_or_else(|| "selector-family:escaped-path".to_string());
    opaque_selector_payload.push_str(";path_token_text=");
    opaque_selector_payload.push_str(&source_token_text);

    Some(RuntimeHostReferenceSyntaxMatch {
        syntax_match_handle: format!(
            "host-reference-syntax:{}:{}:{}:{}",
            context.dialect_id, rule.rule_id, span.start, span.len
        ),
        rule_id: rule.rule_id.clone(),
        rule_family: rule.rule_family.clone(),
        source_span: span,
        source_token_text,
        token_kind: rule.token_kind.clone(),
        opaque_selector_payload: Some(opaque_selector_payload),
        resolution_layer: rule.resolution_layer.clone(),
        shape_hint: rule.shape_hint.clone(),
        caller_context_dependent: rule.caller_context_dependent,
        diagnostics: Vec::new(),
    })
}

fn host_reference_path_token_end(text: &str, bracket_start: usize) -> Option<usize> {
    let mut end = host_reference_bracketed_segment_end(text, bracket_start)?;
    loop {
        let after = text.get(end..)?;
        if !after.starts_with('.') {
            break;
        }
        let segment_start = end + 1;
        let first = text[segment_start..].chars().next()?;
        if first == '[' {
            end = host_reference_bracketed_segment_end(text, segment_start)?;
        } else if host_reference_base_leading_char(first) {
            end = segment_start;
            for (offset, ch) in text[segment_start..].char_indices() {
                if host_reference_base_char(ch) {
                    end = segment_start + offset + ch.len_utf8();
                } else {
                    break;
                }
            }
        } else {
            break;
        }
    }
    Some(end)
}

fn host_reference_bracketed_segment_end(text: &str, start: usize) -> Option<usize> {
    if !text.get(start..)?.starts_with('[') {
        return None;
    }
    let mut cursor = start + 1;
    let mut saw_content = false;
    while cursor < text.len() {
        let current = text[cursor..].chars().next()?;
        if current == '\'' {
            cursor += current.len_utf8();
            let escaped = text[cursor..].chars().next()?;
            saw_content = true;
            cursor += escaped.len_utf8();
            continue;
        }
        if current == ']' {
            return saw_content.then_some(cursor + 1);
        }
        saw_content = true;
        cursor += current.len_utf8();
    }
    None
}

fn declared_host_reference_repeated_prefix_syntax_match(
    text: &str,
    start: usize,
    context: &RuntimeHostFormulaContext,
    rule: &RuntimeHostReferenceSyntaxRule,
) -> Option<RuntimeHostReferenceSyntaxMatch> {
    if !host_syntax_boundary_before(text, start) {
        return None;
    }
    let mut selector_end = start;
    while text[selector_end..].starts_with(rule.pattern_text.as_str()) {
        selector_end += rule.pattern_text.len();
    }
    let repeat_count = (selector_end - start) / rule.pattern_text.len();
    if repeat_count == 0 {
        return None;
    }
    let qualified_tail = qualified_host_reference_tail(text, selector_end);
    let end = qualified_tail
        .as_ref()
        .map_or(selector_end, |tail| tail.source_end);
    if !host_syntax_boundary_after(text, end) {
        return None;
    }
    let span = TextSpan::new(start, end - start);
    let source_token_text = text[span.start..span.end()].to_string();
    let mut opaque_selector_payload = rule
        .opaque_selector_payload
        .clone()
        .unwrap_or_else(|| "selector-family:repeated-prefix".to_string());
    opaque_selector_payload.push_str(";repeat_count=");
    opaque_selector_payload.push_str(&repeat_count.to_string());
    if let Some(tail) = qualified_tail.as_ref() {
        opaque_selector_payload.push_str(";tail_token_text=");
        opaque_selector_payload.push_str(&tail.token_text);
    }

    Some(RuntimeHostReferenceSyntaxMatch {
        syntax_match_handle: format!(
            "host-reference-syntax:{}:{}:{}:{}",
            context.dialect_id, rule.rule_id, span.start, span.len
        ),
        rule_id: rule.rule_id.clone(),
        rule_family: rule.rule_family.clone(),
        source_span: span,
        source_token_text,
        token_kind: rule.token_kind.clone(),
        opaque_selector_payload: Some(opaque_selector_payload),
        resolution_layer: rule.resolution_layer.clone(),
        shape_hint: rule.shape_hint.clone(),
        caller_context_dependent: rule.caller_context_dependent,
        diagnostics: Vec::new(),
    })
}

fn declared_host_reference_literal_array_syntax_match(
    text: &str,
    start: usize,
    context: &RuntimeHostFormulaContext,
    rule: &RuntimeHostReferenceSyntaxRule,
) -> Option<RuntimeHostReferenceSyntaxMatch> {
    let end = host_reference_literal_array_end(text, start)?;
    if !host_syntax_boundary_before(text, start) || !host_syntax_boundary_after(text, end) {
        return None;
    }
    let source_token_text = text[start..end].to_string();
    let element_token_texts = host_reference_literal_array_element_token_texts(&source_token_text)?;
    let span = TextSpan::new(start, end - start);
    let mut opaque_selector_payload = rule
        .opaque_selector_payload
        .clone()
        .unwrap_or_else(|| "selector-family:reference-literal-array".to_string());
    opaque_selector_payload.push_str(";element_token_texts=");
    opaque_selector_payload.push_str(&element_token_texts.join("|"));
    Some(RuntimeHostReferenceSyntaxMatch {
        syntax_match_handle: format!(
            "host-reference-syntax:{}:{}:{}:{}",
            context.dialect_id, rule.rule_id, span.start, span.len
        ),
        rule_id: rule.rule_id.clone(),
        rule_family: rule.rule_family.clone(),
        source_span: span,
        source_token_text,
        token_kind: rule.token_kind.clone(),
        opaque_selector_payload: Some(opaque_selector_payload),
        resolution_layer: rule.resolution_layer.clone(),
        shape_hint: rule.shape_hint.clone(),
        caller_context_dependent: rule.caller_context_dependent,
        diagnostics: Vec::new(),
    })
}

fn host_reference_literal_array_end(text: &str, start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    while cursor < text.len() {
        let current = text[cursor..].chars().next()?;
        match current {
            '}' => return Some(cursor + 1),
            '"' => {
                cursor += 1;
                while cursor < text.len() {
                    let string_current = text[cursor..].chars().next()?;
                    cursor += string_current.len_utf8();
                    if string_current == '"' {
                        if text[cursor..].starts_with('"') {
                            cursor += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            _ => cursor += current.len_utf8(),
        }
    }
    None
}

fn host_reference_literal_array_element_token_texts(
    source_token_text: &str,
) -> Option<Vec<String>> {
    let inner = source_token_text.strip_prefix('{')?.strip_suffix('}')?;
    let elements = inner
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!elements.is_empty() && elements.iter().all(|element| !element.is_empty())).then_some(elements)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QualifiedHostReferenceBase {
    source_start: usize,
    token_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QualifiedHostReferenceTail {
    source_end: usize,
    token_text: String,
}

fn host_reference_literal_pattern_is_prefix_of_longer_token(
    text: &str,
    selector_end: usize,
    pattern_text: &str,
) -> bool {
    pattern_text.ends_with('*') && text[selector_end..].starts_with('*')
}

fn qualified_host_reference_base(
    text: &str,
    pattern_start: usize,
    pattern_text: &str,
) -> Option<QualifiedHostReferenceBase> {
    if pattern_text.starts_with('.') {
        return qualified_host_reference_base_before_dot(text, pattern_start);
    }
    if pattern_start == 0 || !text[..pattern_start].ends_with('.') {
        return None;
    }
    qualified_host_reference_base_before_dot(text, pattern_start - 1)
}

fn qualified_host_reference_base_before_dot(
    text: &str,
    dot_index: usize,
) -> Option<QualifiedHostReferenceBase> {
    if dot_index == 0 || !text.is_char_boundary(dot_index) {
        return None;
    }
    let mut start = dot_index;
    for (index, ch) in text[..dot_index].char_indices().rev() {
        if host_reference_base_char(ch) {
            start = index;
        } else {
            break;
        }
    }
    if start == dot_index
        || !text[start..dot_index]
            .chars()
            .next()
            .is_some_and(host_reference_base_leading_char)
    {
        return None;
    }
    Some(QualifiedHostReferenceBase {
        source_start: start,
        token_text: text[start..dot_index].to_string(),
    })
}

fn qualified_host_reference_tail(
    text: &str,
    selector_end: usize,
) -> Option<QualifiedHostReferenceTail> {
    let after_selector = text.get(selector_end..)?;
    if !after_selector.starts_with('.') {
        return None;
    }
    let tail_start = selector_end + 1;
    let first = text[tail_start..].chars().next()?;
    if !host_reference_base_leading_char(first) {
        return None;
    }
    let mut end = tail_start;
    for (offset, ch) in text[tail_start..].char_indices() {
        if host_reference_base_char(ch) {
            end = tail_start + offset + ch.len_utf8();
        } else {
            break;
        }
    }
    Some(QualifiedHostReferenceTail {
        source_end: end,
        token_text: text[tail_start..end].to_string(),
    })
}

fn host_reference_base_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.')
}

fn host_reference_base_leading_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn host_reference_selector_payload_with_qualifiers(
    payload: Option<&str>,
    base_token_text: Option<&str>,
    tail_token_text: Option<&str>,
) -> String {
    let mut enriched = payload.unwrap_or("selector-family:unknown").to_string();
    if let Some(base_token_text) = base_token_text {
        enriched.push_str(";base_token_text=");
        enriched.push_str(base_token_text);
    }
    if let Some(tail_token_text) = tail_token_text {
        enriched.push_str(";tail_token_text=");
        enriched.push_str(tail_token_text);
    }
    enriched
}

fn host_syntax_boundary_before(text: &str, start: usize) -> bool {
    previous_non_whitespace_char(text, start).is_none_or(|ch| !host_syntax_identifier_char(ch))
}

fn host_syntax_boundary_after(text: &str, end: usize) -> bool {
    next_non_whitespace_char(text, end).is_none_or(|ch| !host_syntax_identifier_char(ch))
}

fn previous_non_whitespace_char(text: &str, end: usize) -> Option<char> {
    text[..end].chars().rev().find(|ch| !ch.is_whitespace())
}

fn next_non_whitespace_char(text: &str, start: usize) -> Option<char> {
    text[start..].chars().find(|ch| !ch.is_whitespace())
}

fn host_syntax_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostNameBindResult {
    pub host_name_handle: String,
    pub canonical_name: String,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub resolution_layer: String,
    pub binding_kind: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
    pub replay_identity_contribution: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeHostNameBinding {
    pub bind_result: RuntimeHostNameBindResult,
    pub binding: DefinedNameBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHostReferenceBindResult {
    pub reference_handle: String,
    pub formal_reference_id: Option<String>,
    pub source_span: TextSpan,
    pub source_token_text: String,
    pub opaque_selector_payload: Option<String>,
    pub resolution_layer: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
    pub replay_identity_contribution: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSparseReferenceCell {
    pub row: usize,
    pub col: usize,
    pub value: ArrayCellValue,
}

impl RuntimeSparseReferenceCell {
    #[must_use]
    pub fn new(row: usize, col: usize, value: ArrayCellValue) -> Self {
        Self { row, col, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSparseReferenceValuesBinding {
    pub reference: ReferenceLike,
    pub declared_rows: usize,
    pub declared_cols: usize,
    pub defined_cells: Vec<RuntimeSparseReferenceCell>,
    pub reader_identity: Option<String>,
}

impl RuntimeSparseReferenceValuesBinding {
    #[must_use]
    pub fn resolved_values(&self) -> ResolvedReferenceValues {
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(self.declared_rows, self.declared_cols),
            self.defined_cells
                .iter()
                .map(|cell| ResolvedReferenceCell::new(cell.row, cell.col, cell.value.clone()))
                .collect(),
            self.reader_identity.clone(),
        )
    }
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
    pub structured_reference_bind_record_handle: Option<String>,
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
            &self.environment.host_formula_context,
            &self.environment.host_name_bind_results(),
            &self.environment.host_reference_bind_results,
            self.environment.runtime_registry_view_identity().as_ref(),
            &prepared.registry_capability_denials,
            self.environment.table_context_fingerprint(),
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
        defined_names.extend(host_name_defined_names(
            &self.environment.host_name_bindings,
        ));
        let sparse_reference_values =
            sparse_reference_values_map(&self.environment.sparse_reference_value_bindings);
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
            sparse_reference_values,
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
        let table_context_fingerprint = self.environment.table_context_fingerprint();
        Some(runtime_managed_session_snapshot(
            record,
            &self.environment.oxfunc_bridge_metadata,
            &self.environment.host_formula_context,
            &self.environment.host_name_bind_results(),
            &self.environment.host_reference_bind_results,
            table_context_fingerprint,
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
    host_formula_context: &Option<RuntimeHostFormulaContext>,
    host_name_bind_results: &[RuntimeHostNameBindResult],
    host_reference_bind_results: &[RuntimeHostReferenceBindResult],
    registry_view_identity: Option<&RuntimeFunctionRegistryViewIdentity>,
    registry_capability_denials: &[RuntimeFunctionCapabilityDenial],
    table_context_fingerprint: Option<String>,
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
    let hole_binding_fingerprint = if bound_formula.structured_reference_bind_records.is_empty() {
        runtime_hash_debug(&(
            &bound_formula.normalized_references,
            &bound_formula.unresolved_references,
            &semantic_plan.helper_profile,
            &semantic_plan.capability_requirements,
        ))
    } else {
        runtime_hash_debug(&(
            &bound_formula.normalized_references,
            &bound_formula.unresolved_references,
            &bound_formula.structured_reference_bind_records,
            &semantic_plan.helper_profile,
            &semantic_plan.capability_requirements,
        ))
    };
    let prepared_formula_key = runtime_hash_debug(&format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
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
        host_formula_context,
        host_name_bind_results,
        host_reference_bind_results,
        registry_view_identity,
        registry_capability_denials,
        &table_context_fingerprint,
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
        host_formula_context: host_formula_context.clone(),
        host_name_bind_results: host_name_bind_results.to_vec(),
        host_reference_bind_results: host_reference_bind_results.to_vec(),
        registry_snapshot_identity: registry_view_identity
            .map(|identity| identity.registry_snapshot_identity.clone()),
        capability_overlay_identity: registry_view_identity
            .and_then(|identity| identity.capability_overlay_identity.clone()),
        registry_capability_denials: registry_capability_denials.to_vec(),
        table_context_fingerprint,
        structured_reference_bind_records: bound_formula.structured_reference_bind_records.clone(),
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

fn runtime_table_context_fingerprint(
    table_catalog: &[TableDescriptor],
    enclosing_table_ref: &Option<TableRef>,
    caller_table_region: &Option<TableCallerRegion>,
) -> Option<String> {
    if table_catalog.is_empty() && enclosing_table_ref.is_none() && caller_table_region.is_none() {
        return None;
    }
    Some(runtime_hash_debug(&(
        table_catalog,
        enclosing_table_ref,
        caller_table_region,
    )))
}

fn refresh_runtime_prepared_formula_identity_for_plan(
    identity: &mut RuntimePreparedFormulaIdentity,
    semantic_plan: &SemanticPlan,
) {
    identity.registry_capability_denials =
        runtime_registry_capability_denials(&semantic_plan.availability_summaries);
    identity.library_context_snapshot_ref = semantic_plan.library_context_snapshot_ref.clone();
    identity.plan_template.dispatch_skeleton_key = runtime_hash_debug(&(
        &semantic_plan.function_bindings,
        &semantic_plan.availability_summaries,
        &semantic_plan.oxfunc_catalog_identity,
        &semantic_plan.library_context_snapshot_ref,
    ));
    identity.plan_template.plan_template_key = semantic_plan.semantic_plan_key.clone();
    identity.prepared_formula_key = runtime_hash_debug(&format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
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
        &identity.host_formula_context,
        &identity.host_name_bind_results,
        &identity.host_reference_bind_results,
        &identity.registry_snapshot_identity,
        &identity.capability_overlay_identity,
        &identity.registry_capability_denials,
        &identity.table_context_fingerprint,
    ));
}

fn runtime_registry_capability_denials(
    summaries: &[crate::semantics::FunctionAvailabilitySummary],
) -> Vec<RuntimeFunctionCapabilityDenial> {
    summaries
        .iter()
        .filter(|summary| {
            summary.runtime_capability_state
                == Some(LibraryAvailabilityState::HostProfileUnavailable)
        })
        .map(|summary| RuntimeFunctionCapabilityDenial {
            surface_name: summary.surface_name.clone(),
            canonical_id: summary.canonical_id.clone(),
            runtime_capability_state: "HostProfileUnavailable".to_string(),
        })
        .collect()
}

fn runtime_formal_references(bound_formula: &BoundFormula) -> Vec<RuntimeFormalReference> {
    let mut structured_bind_record_index = 0usize;
    let mut references = Vec::new();
    for (index, reference) in bound_formula.normalized_references.iter().enumerate() {
        let structured_reference_bind_record_handle =
            runtime_structured_reference_record_handle_for_reference(
                reference,
                &bound_formula.structured_reference_bind_records,
                &mut structured_bind_record_index,
            );
        references.push(RuntimeFormalReference {
            reference_handle: format!(
                "formal-ref:{}:{index}",
                runtime_hash_debug(&(bound_formula.bind_hash.as_str(), reference))
            ),
            reference_descriptor: reference.to_string(),
            reference_family: runtime_reference_family(reference).to_string(),
            caller_context_dependent: runtime_reference_caller_context_dependent(reference),
            host_mappable_identity: Some(reference.to_string()),
            linked_hole_id: Some(runtime_reference_hole_id(index)),
            structured_reference_bind_record_handle,
        });
    }
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
            structured_reference_bind_record_handle:
                runtime_structured_reference_record_handle_for_unresolved(
                    unresolved,
                    &bound_formula.structured_reference_bind_records,
                ),
        },
    ));
    references
}

fn runtime_structured_reference_record_handle_for_reference(
    reference: &NormalizedReference,
    records: &[StructuredReferenceBindRecord],
    cursor: &mut usize,
) -> Option<String> {
    match reference {
        NormalizedReference::Structured(_) => {
            let handle = records
                .get(*cursor)
                .map(|record| record.bind_record_handle.clone());
            *cursor += usize::from(handle.is_some());
            handle
        }
        NormalizedReference::Error(error) => {
            let record = records
                .get(*cursor)
                .filter(|record| record.source_token_text == error.source_text)?;
            *cursor += 1;
            Some(record.bind_record_handle.clone())
        }
        NormalizedReference::Cell(_)
        | NormalizedReference::Area(_)
        | NormalizedReference::WholeRow(_)
        | NormalizedReference::WholeColumn(_)
        | NormalizedReference::Name(_)
        | NormalizedReference::External(_) => None,
    }
}

fn runtime_structured_reference_record_handle_for_unresolved(
    unresolved: &crate::binding::UnresolvedReferenceRecord,
    records: &[StructuredReferenceBindRecord],
) -> Option<String> {
    records
        .iter()
        .find(|record| {
            record.source_token_text == unresolved.source_text
                && record
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message == unresolved.reason)
        })
        .map(|record| record.bind_record_handle.clone())
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
    registry_capability_denials: Vec<RuntimeFunctionCapabilityDenial>,
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
    bind_names.extend(host_name_defined_names(&environment.host_name_bindings));
    bind_names.extend(formal_input_defined_names(
        &environment.formal_input_bindings,
    ));
    let name_caller_context_dependencies =
        host_name_caller_context_dependencies(&environment.host_name_bindings);
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
            name_caller_context_dependencies,
            table_catalog: environment.table_catalog.clone(),
            enclosing_table_ref: environment.enclosing_table_ref.clone(),
            caller_table_region: environment.caller_table_region.clone(),
            ..BindContext::default()
        },
    });
    let library_context_view = environment.library_context.pinned_view();
    let base_library_context_snapshot = library_context_view.resolve_snapshot();
    if let Some(snapshot_ref) = library_context_view.snapshot_ref() {
        if base_library_context_snapshot.is_none() {
            return Err(format!(
                "requested library context snapshot {}@{} did not resolve",
                snapshot_ref.snapshot_id, snapshot_ref.snapshot_version
            ));
        }
    }
    let library_context_snapshot =
        environment.runtime_registry_library_context_snapshot(base_library_context_snapshot);
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
    let registry_capability_denials =
        runtime_registry_capability_denials(&semantic_plan.availability_summaries);

    Ok(CompiledRuntimePrepareRequest {
        prepare_request: PrepareRequest {
            source,
            bound_formula: bind.bound_formula,
            semantic_plan,
            primary_locus: environment.primary_locus.clone(),
        },
        syntax_diagnostics,
        bind_diagnostics,
        registry_capability_denials,
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
    host_formula_context: &Option<RuntimeHostFormulaContext>,
    host_name_bind_results: &[RuntimeHostNameBindResult],
    host_reference_bind_results: &[RuntimeHostReferenceBindResult],
    table_context_fingerprint: Option<String>,
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
            host_formula_context,
            host_name_bind_results,
            host_reference_bind_results,
            None,
            &runtime_registry_capability_denials(
                &record.prepared.semantic_plan.availability_summaries,
            ),
            table_context_fingerprint,
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
