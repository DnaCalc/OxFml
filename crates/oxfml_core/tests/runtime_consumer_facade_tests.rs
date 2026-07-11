use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use oxfml_core::binding::{
    BinaryOp, BoundExpr, NameKind, NameRef, ReferenceExpr, StructuredReferenceSourceTokenKind,
    StructuredResolvedRef, StructuredSectionKind,
};
use oxfml_core::consumer::replay::{ReplayProjectionRequest, ReplayProjectionService};
use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormalInputBinding, RuntimeFormulaRequest,
    RuntimeHostFormulaContext, RuntimeHostNameBindResult, RuntimeHostNameBinding,
    RuntimeHostReferenceBindResult, RuntimeManagedSessionError, RuntimeManagedSessionPhase,
    RuntimeOxFuncBridgeMetadata, RuntimeSessionFacade,
};
use oxfml_core::format::{
    oxfml_en_us_format_profile, oxfml_en_us_locale_context, worksheet_error_text,
};
use oxfml_core::publication::{
    VerificationConditionalFormattingRule, VerificationPublicationContext,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::{
    AcceptDecision, CallableCaptureMode, CallableDefinedNameBinding, CallableInvocationModel,
    CallableOriginKind, CallableValueCarrier, CallableValueProfile, DefinedNameBinding,
    EvaluationTraceMode, ExecutionOutcomeKind, ExecutionOutcomeStage, FormulaChannelKind,
    FormulaSourceRecord, HostFunctionInvocation, HostFunctionProvider, HostFunctionProviderError,
    InMemoryLibraryContextProvider, LibraryContextSnapshotRef, NormalizedReference,
    RegisteredExternalCatalogController, RegisteredExternalCatalogMutationRequest,
    RegisteredExternalCatalogMutationResult, RegisteredExternalHostRegistrationRequest,
    RegisteredExternalRegistrationChannel, TableCallerRegion, TableColumnDescriptor,
    TableDescriptor, TableRef, TableRegionKind, TypedContextQueryBundle, TypedContextQueryFamily,
};
use oxfunc_core::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalDescriptor, RegisteredExternalOriginKind,
    RegisteredExternalProvider, RegisteredExternalProviderError, RegisteredExternalTarget,
    RegisteredProcedureSpec,
};
use oxfunc_core::functions::rand_fn::RandomProvider;
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::{
    FormatCodeEngine, FormatFailure, FormatProfile, LocaleFormatContext, LocaleValueParser,
    ParseFailure, WorkbookDateSystem,
};
use oxfunc_core::registry::{
    ArgAdmissionMetadata, CapabilityOverlay, FunctionEntry, FunctionRegistryMetadata,
    FunctionSource, ParameterDescriptor, RegistryFunctionMeta, RichValueUsage,
    SemanticKernelMetadata, SignatureForm, builtin_registry,
};
use oxfunc_core::resolver::{
    ReferenceEnumerationRequest, ReferenceResolutionError, ReferenceSystemProvider,
    ResolvedReferenceCell, ResolvedReferenceExtent, ResolvedReferenceValues,
};
use oxfunc_core::value::{ArrayShape, ReferenceKind, ReferenceLike, WorksheetErrorCode};
use oxfunc_core::value::{CalcArray, CalcValue};
use oxfunc_core::value::{CoreValue, ExcelText};

struct SequenceRandomProvider {
    next: Cell<u32>,
}

impl RandomProvider for SequenceRandomProvider {
    fn random_unit(&self) -> f64 {
        let next = self.next.get();
        self.next.set(next + 1);
        next as f64 / 100.0
    }
}

#[test]
fn runtime_environment_evaluates_non_formula_worksheet_entries() {
    let cases = [
        (
            "ABC",
            CalcValue::text(ExcelText::from_interop_assignment("ABC")),
        ),
        (
            "'=123",
            CalcValue::text(ExcelText::from_interop_assignment("=123")),
        ),
        (
            "12.1.1",
            CalcValue::text(ExcelText::from_interop_assignment("12.1.1")),
        ),
        (
            "x y z = 12.3",
            CalcValue::text(ExcelText::from_interop_assignment("x y z = 12.3")),
        ),
        (
            "\"ABC\"",
            CalcValue::text(ExcelText::from_interop_assignment("ABC")),
        ),
        ("123.4", CalcValue::number(123.4)),
        ("TRUE", CalcValue::logical(true)),
        ("FALSE", CalcValue::logical(false)),
    ];

    let environment = RuntimeEnvironment::new();

    for (entry_text, expected_value) in cases {
        let source =
            FormulaSourceRecord::new(format!("runtime:entry:{entry_text:?}"), 1, entry_text)
                .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);
        let result = environment
            .execute(RuntimeFormulaRequest::new(
                source,
                TypedContextQueryBundle::default(),
            ))
            .expect("literal cell entry should execute through OxFml runtime");

        assert_eq!(result.source.entered_formula_text, entry_text);
        assert!(
            result.syntax_diagnostics.is_empty(),
            "syntax diagnostics for {entry_text:?}: {:?}",
            result.syntax_diagnostics
        );
        assert!(
            result.bind_diagnostics.is_empty(),
            "bind diagnostics for {entry_text:?}: {:?}",
            result.bind_diagnostics
        );
        assert_eq!(
            result.published_worksheet_value, expected_value,
            "published value mismatch for {entry_text:?}"
        );
    }
}

#[test]
fn runtime_formula_only_without_host_provider_runs_pure_formulas_and_rejects_indirect_refs() {
    let pure = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:formula-only:pure", 1, "=1+2"),
            TypedContextQueryBundle::default(),
        ))
        .expect("pure formula should run without a host reference provider");
    assert_eq!(pure.evaluation.oxfunc_value, CalcValue::number(3.0));

    let reference = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:formula-only:indirect", 1, "=INDIRECT(\"A1\")"),
            TypedContextQueryBundle::default(),
        ))
        .expect("reference operation should return a worksheet error");
    assert_eq!(
        reference.evaluation.oxfunc_value,
        CalcValue::error(WorksheetErrorCode::Ref)
    );
}

use serde_json::Value;

#[test]
fn runtime_environment_executes_against_pinned_snapshot_ref() {
    let pinned_snapshot = runtime_snapshot_v1();
    let pinned_snapshot_ref = LibraryContextSnapshotRef::from(&pinned_snapshot);
    let current_snapshot = runtime_snapshot_v2();
    let current_snapshot_ref = LibraryContextSnapshotRef::from(&current_snapshot);
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        current_snapshot_ref,
        vec![pinned_snapshot, current_snapshot],
    );
    let environment = RuntimeEnvironment::new()
        .with_pinned_library_context(&provider, pinned_snapshot_ref.clone());
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:take", 1, "=TAKE({1,2},1)"),
        TypedContextQueryBundle::default(),
    );

    let result = environment
        .execute(request)
        .expect("runtime execution should succeed");

    assert_eq!(
        result.library_context_snapshot_ref,
        Some(pinned_snapshot_ref.clone())
    );
    assert_eq!(
        result.semantic_plan.library_context_snapshot_ref,
        Some(pinned_snapshot_ref)
    );
    let take_summary = result
        .semantic_plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "TAKE")
        .expect("TAKE availability summary");
    assert_eq!(
        take_summary.metadata_status.as_deref(),
        Some("function_meta_extracted")
    );
    assert_eq!(take_summary.surface_stable_id.as_deref(), Some("FUNC.TAKE"));
}

#[test]
fn runtime_environment_executes_against_inline_snapshot() {
    let inline_snapshot = runtime_snapshot_v2();
    let inline_snapshot_ref = LibraryContextSnapshotRef::from(&inline_snapshot);
    let environment =
        RuntimeEnvironment::new().with_inline_library_context_snapshot(inline_snapshot);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:inline-snapshot", 1, "=TAKE({1,2},1)"),
        TypedContextQueryBundle::default(),
    );

    let result = environment
        .execute(request)
        .expect("runtime execution against inline snapshot should succeed");

    assert_eq!(
        result.library_context_snapshot_ref,
        Some(inline_snapshot_ref.clone())
    );
    assert_eq!(
        result.semantic_plan.library_context_snapshot_ref,
        Some(inline_snapshot_ref)
    );
    let take_summary = result
        .semantic_plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "TAKE")
        .expect("TAKE availability summary");
    assert_eq!(
        take_summary.metadata_status.as_deref(),
        Some("runtime_snapshot")
    );
    assert_eq!(
        take_summary.surface_stable_id.as_deref(),
        Some("surface:take")
    );
}

#[test]
fn runtime_environment_rejects_unresolved_snapshot_ref_without_provider() {
    let missing_snapshot_ref = LibraryContextSnapshotRef::new("runtime-consumer", "missing");
    let environment =
        RuntimeEnvironment::new().with_library_context_snapshot_ref(missing_snapshot_ref.clone());
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:missing-snapshot", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let error = environment
        .execute(request)
        .expect_err("runtime execution should reject unresolved snapshot ref");

    assert!(error.contains("requested library context snapshot"));
    assert!(error.contains(&missing_snapshot_ref.snapshot_id));
    assert!(error.contains(&missing_snapshot_ref.snapshot_version));
}

#[test]
fn runtime_session_facade_reuses_host_artifacts_for_repeated_same_formula() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:repeat", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let first = session
        .execute(request.clone())
        .expect("first runtime execution should succeed");
    let second = session
        .execute(request)
        .expect("second runtime execution should succeed");

    assert!(!first.artifact_reuse.green_tree_reused);
    assert!(second.artifact_reuse.green_tree_reused);
    assert!(second.artifact_reuse.red_projection_reused);
    assert!(second.artifact_reuse.bound_formula_reused);
    assert!(second.artifact_reuse.semantic_plan_reused);
}

#[test]
fn runtime_session_facade_runs_managed_session_through_commit() {
    let mut defined_names = std::collections::BTreeMap::new();
    defined_names.insert(
        "InputValue".to_string(),
        oxfml_core::DefinedNameBinding::Value(CalcValue::number(5.0)),
    );
    let environment = RuntimeEnvironment::new().with_defined_names(defined_names);
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed", 1, "=SUM(InputValue,2)"),
        TypedContextQueryBundle::default(),
    );

    let open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");
    assert!(open.syntax_diagnostics.is_empty());
    assert!(open.bind_diagnostics.is_empty());

    let execution = session
        .execute_managed(request)
        .expect("managed execute should succeed");

    assert_eq!(execution.formula_stable_id, "runtime:managed");
    assert_eq!(
        open.prepared_formula_identity.prepared_formula_key,
        execution.prepared_formula_identity.prepared_formula_key
    );
    assert_eq!(
        execution.prepared_formula_identity.formula_stable_id,
        "runtime:managed"
    );
    assert_eq!(
        execution
            .prepared_formula_identity
            .plan_template
            .plan_template_key,
        open.semantic_plan.semantic_plan_key
    );
    assert!(
        execution
            .prepared_formula_identity
            .formal_references
            .iter()
            .any(|reference| reference.reference_descriptor == "name:InputValue")
    );
    assert_eq!(
        execution.typed_query_bundle_spec,
        TypedContextQueryBundle::default().freeze_candidate_spec()
    );
    assert!(execution.library_context_snapshot_ref.is_none());
    let commit = session
        .commit_managed(
            "commit:runtime-managed",
            execution.candidate_result.fence_snapshot.clone(),
        )
        .expect("managed commit should return decision");

    assert_eq!(
        commit.execution_outcome_surface.outcome_kind,
        oxfml_core::ExecutionOutcomeKind::ExecutedResult
    );
    assert_eq!(
        commit.execution_outcome_surface.outcome_stage,
        oxfml_core::ExecutionOutcomeStage::Executed
    );
    assert_eq!(commit.execution_outcome_surface.class_id, "executed_result");

    match &commit.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("7".to_string())
            );
        }
        AcceptDecision::Rejected(reject) => {
            panic!("expected accepted commit, got reject: {reject:?}");
        }
    }

    let snapshot = session
        .managed_session_snapshot()
        .expect("managed snapshot should exist");
    assert_eq!(snapshot.phase, RuntimeManagedSessionPhase::Committed);
    assert_eq!(commit.session.phase, RuntimeManagedSessionPhase::Committed);
    assert_eq!(
        snapshot.execution_outcome_surface,
        Some(commit.execution_outcome_surface.clone())
    );
    assert_eq!(
        snapshot.candidate_result_id,
        Some(execution.candidate_result.candidate_result_id)
    );
    assert_eq!(
        snapshot.prepared_formula_identity.prepared_formula_key,
        open.prepared_formula_identity.prepared_formula_key
    );
}

#[test]
fn runtime_formal_input_binding_executes_without_synthetic_cells_or_defined_names() {
    let source = FormulaSourceRecord::new("runtime:formal-input", 1, "=SUM(InputValue,2)");
    let prepare_binding = RuntimeFormalInputBinding {
        reference_handle: None,
        reference_descriptor: "InputValue".to_string(),
        binding: oxfml_core::DefinedNameBinding::Value(CalcValue::number(5.0)),
    };
    let mut prepare_session = RuntimeEnvironment::new()
        .with_formal_input_bindings(vec![prepare_binding])
        .open_session();
    let request = RuntimeFormulaRequest::new(source.clone(), TypedContextQueryBundle::default());

    let open = prepare_session
        .open_managed_session(&request)
        .expect("formal input should participate in prepare/bind");
    let formal_reference = open
        .prepared_formula_identity
        .formal_references
        .iter()
        .find(|reference| reference.reference_descriptor == "name:InputValue")
        .expect("prepared identity should expose the formal reference")
        .clone();

    let execute_binding = RuntimeFormalInputBinding {
        reference_handle: Some(formal_reference.reference_handle.clone()),
        reference_descriptor: formal_reference.reference_descriptor,
        binding: oxfml_core::DefinedNameBinding::Value(CalcValue::number(5.0)),
    };
    let result = RuntimeEnvironment::new()
        .with_formal_input_bindings(vec![execute_binding])
        .execute(RuntimeFormulaRequest::new(
            source,
            TypedContextQueryBundle::default(),
        ))
        .expect("formal input binding should execute without cell_values/defined_names");

    assert_eq!(result.published_worksheet_value, CalcValue::number(7.0));
}

#[test]
fn runtime_result_exposes_prepared_formula_identity_for_direct_execution_under_minimal_profile() {
    // Re-added non-grid version: drives the prepared-formula identity machinery
    // with a bare same-sheet A1 reference resolved via the auto-wired minimal test
    // reference profile (oxfml.test.minimal.v1). Asserts the abstract machinery
    // (RefOrValueHole template hole, linked formal reference, package round-trip)
    // rather than any grid-specific descriptor string.
    let mut cell_values = std::collections::BTreeMap::new();
    cell_values.insert("A1".to_string(), CalcValue::number(4.0));
    let environment = RuntimeEnvironment::new().with_cell_values(cell_values);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:prepared-identity", 1, "=SUM(A1,2)"),
        TypedContextQueryBundle::default(),
    );

    let result = environment
        .execute(request)
        .expect("runtime execution should succeed");

    assert_eq!(
        result.prepared_formula_identity.formula_stable_id,
        "runtime:prepared-identity"
    );
    assert_eq!(result.prepared_formula_identity.formula_text_version, 1);
    assert_eq!(
        result
            .prepared_formula_identity
            .plan_template
            .plan_template_key,
        result.semantic_plan.semantic_plan_key
    );
    assert!(
        result
            .prepared_formula_identity
            .plan_template
            .shape_key
            .is_none(),
        "shape key remains deferred until canonical shape abstraction exists"
    );

    // The bare A1 reference argument yields exactly one reference template-hole.
    // Under the minimal profile a bare A1 reference binds to a profile-symbolic
    // reference, so the runtime classifies its hole as "ProfileReferenceHole"
    // (the grid-agnostic analogue of the old grid "RefOrValueHole"; see note in
    // the report). The abstract machinery preserved here: a reference argument
    // produces a dedicated reference template-hole that a formal reference is
    // linked to.
    let ref_hole = result
        .prepared_formula_identity
        .plan_template
        .template_holes
        .iter()
        .find(|hole| hole.hole_id == "hole:reference:0")
        .expect("prepared identity should expose a reference template-hole for the A1 argument");

    assert_eq!(ref_hole.hole_kind, "ProfileReferenceHole");
    assert_eq!(
        ref_hole.hole_kind_key,
        "ProfileReferenceHole:profile_symbolic"
    );
    // Descriptor/path are the minimal-profile representation (profile id +
    // canonical A1 normal-form key), observed at runtime, not the old grid form.
    assert_eq!(
        ref_hole.path.as_deref(),
        Some("profile-symbolic:oxfml.test.minimal.v1:A1")
    );

    let formal_ref = result
        .prepared_formula_identity
        .formal_references
        .iter()
        .find(|reference| reference.linked_hole_id.is_some())
        .expect("formal references should include an entry with a linked_hole_id");

    assert_eq!(formal_ref.reference_family, "profile_symbolic");
    assert_eq!(
        formal_ref.reference_descriptor,
        "profile-symbolic:oxfml.test.minimal.v1:A1"
    );
    // The formal reference is linked to the reference template-hole.
    assert_eq!(
        formal_ref.linked_hole_id.as_deref(),
        Some(ref_hole.hole_id.as_str()),
        "formal reference's linked_hole_id should match the reference hole's id"
    );

    let package = result.prepared_formula_package();
    assert_eq!(
        package.package_key,
        result.prepared_formula_identity.prepared_formula_key
    );
    assert_eq!(
        package.plan_template.template_holes,
        result
            .prepared_formula_identity
            .plan_template
            .template_holes
    );
    assert!(
        result
            .prepared_formula_identity
            .hole_binding
            .projection_status
            .contains("canonical_holes_deferred")
    );
}

struct RuntimeTestReferenceSystemProvider {
    reference: ReferenceLike,
    values: ResolvedReferenceValues,
}

impl RuntimeTestReferenceSystemProvider {
    fn new(reference: ReferenceLike, values: ResolvedReferenceValues) -> Self {
        Self { reference, values }
    }
}

impl ReferenceSystemProvider for RuntimeTestReferenceSystemProvider {
    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        if request.reference == self.reference {
            Ok(Some(self.values.clone()))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn runtime_reference_system_provider_feeds_first_aggregate_group() {
    let cases = [
        ("=SUM(InputRef)", CalcValue::number(2.0)),
        ("=COUNT(InputRef)", CalcValue::number(1.0)),
        ("=COUNTA(InputRef)", CalcValue::number(3.0)),
        ("=COUNTBLANK(InputRef)", CalcValue::number(3.0)),
    ];

    for (formula, expected) in cases {
        let reference = ReferenceLike::new(ReferenceKind::Area, "host:sparse:A1:A5");
        let provider = RuntimeTestReferenceSystemProvider::new(
            reference.clone(),
            ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(5, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                    ResolvedReferenceCell::new(
                        2,
                        1,
                        CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                    ),
                    ResolvedReferenceCell::new(
                        3,
                        1,
                        CalcValue::text(ExcelText::from_utf16_code_units(
                            "x".encode_utf16().collect(),
                        )),
                    ),
                ],
                Some("reader:w051:sparse:A1:A5".to_string()),
            ),
        );
        let request = RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:w051-sparse", 1, formula),
            TypedContextQueryBundle::default().with_reference_system_provider(Some(&provider)),
        );

        let result = RuntimeEnvironment::new()
            .with_formal_input_bindings(vec![RuntimeFormalInputBinding {
                reference_handle: None,
                reference_descriptor: "name:InputRef".to_string(),
                binding: oxfml_core::DefinedNameBinding::Reference(reference.clone()),
            }])
            .execute(request)
            .expect("sparse reference aggregate should execute");

        assert_eq!(result.evaluation.oxfunc_value, expected);
    }
}

#[test]
fn runtime_structured_references_use_sparse_values_when_available() {
    let cases = [
        ("=SUM(Table1[Amount])", CalcValue::number(2.0)),
        ("=COUNT(Table1[Amount])", CalcValue::number(1.0)),
        ("=COUNTA(Table1[Amount])", CalcValue::number(2.0)),
        ("=COUNTBLANK(Table1[Amount])", CalcValue::number(2.0)),
    ];

    for (formula, expected) in cases {
        let reference = ReferenceLike::new(ReferenceKind::Area, "B2:B4");
        let provider = RuntimeTestReferenceSystemProvider::new(
            reference.clone(),
            ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(3, 1),
                vec![
                    ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                    ResolvedReferenceCell::new(
                        2,
                        1,
                        CalcValue::text(ExcelText::from_utf16_code_units(Vec::new())),
                    ),
                ],
                Some("reader:w056:table:B2:B4".to_string()),
            ),
        );
        let result = RuntimeEnvironment::new()
            .with_table_context(vec![runtime_w074_table("B2:B4")], None, None)
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new("runtime:w056-structured-sparse", 1, formula),
                TypedContextQueryBundle::default().with_reference_system_provider(Some(&provider)),
            ))
            .expect("structured sparse reference aggregate should execute");

        assert_eq!(result.evaluation.oxfunc_value, expected, "{formula}");
        assert!(
            result.structured_reference_bind_records[0]
                .resolved_reference
                .as_ref()
                .map(|resolved| format!("{resolved:?}"))
                .unwrap_or_default()
                .contains("height: 3")
        );
    }
}

#[test]
fn runtime_structured_reference_uses_formula_scope_sheet_for_sparse_values() {
    let reference = ReferenceLike::new(ReferenceKind::Area, "Sheet1!A2:A3");
    let provider = RuntimeTestReferenceSystemProvider::new(
        reference,
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(2, 1),
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(10.0)),
                ResolvedReferenceCell::new(2, 1, CalcValue::number(20.0)),
            ],
            Some("reader:treecalc:SalesTable:Amount".to_string()),
        ),
    );
    let table = TableDescriptor {
        table_id: "table:sales".to_string(),
        table_name: "SalesTable".to_string(),
        workbook_scope_ref: "Book1".to_string(),
        sheet_scope_ref: "Sheet1".to_string(),
        table_range_ref: "A1:A3".to_string(),
        row_membership_identity: Some("table:sales:rows:v1".to_string()),
        row_order_identity: Some("table:sales:row-order:v1".to_string()),
        header_region_ref: Some("A1:A1".to_string()),
        totals_region_ref: None,
        header_row_present: true,
        totals_row_present: false,
        columns: vec![TableColumnDescriptor {
            column_id: "table:sales:col:amount".to_string(),
            column_name: "Amount".to_string(),
            ordinal: 1,
            column_range_ref: "A2:A3".to_string(),
        }],
    };

    let result = RuntimeEnvironment::new()
        .with_formula_scope("Book1", "Sheet1")
        .with_table_context(vec![table], None, None)
        .execute(
            RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    "runtime:treecalc-structured-scope",
                    1,
                    "=SUM(SalesTable[Amount])",
                ),
                TypedContextQueryBundle::default().with_reference_system_provider(Some(&provider)),
            )
            .with_trace_mode(EvaluationTraceMode::PreparedCalls),
        )
        .expect("structured sparse reference should execute");

    assert_eq!(result.evaluation.oxfunc_value, CalcValue::number(30.0));
    assert_eq!(
        result.evaluation.trace.prepared_calls[0].prepared_arguments[0]
            .reference_target
            .as_deref(),
        Some("Sheet1!A2:A3")
    );
}

#[test]
fn runtime_carries_host_reference_context_without_treecalc_semantics() {
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "generic-host-v1".to_string(),
        capability_profile_id: "host-capabilities:generic-v1".to_string(),
        resolution_rule_version: "host-resolution:v1".to_string(),
        host_namespace_version: Some("host-ns:v1".to_string()),
        registry_snapshot_identity: Some("registry:snapshot:v1".to_string()),
        structure_context_version: Some("structure:v1".to_string()),
        caller_context_identity: Some("caller:sheet1-r1c1".to_string()),
        table_context_identity: Some("tables:v1".to_string()),
    };
    let bind_result = RuntimeHostReferenceBindResult {
        reference_handle: "host-ref:opaque-collection".to_string(),
        formal_reference_id: Some("formal-ref:host:opaque-collection".to_string()),
        source_span: TextSpan::new(5, 15),
        source_token_text: "HOSTREF:opaque".to_string(),
        opaque_selector_payload: Some("opaque-selector:collection".to_string()),
        resolution_layer: "explicit_host_ref".to_string(),
        shape_hint: Some("opaque_collection".to_string()),
        caller_context_dependent: true,
        diagnostics: Vec::new(),
        replay_identity_contribution: "host-ref-identity:v1".to_string(),
    };
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-host-context", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let result = RuntimeEnvironment::new()
        .with_host_formula_context(host_context.clone())
        .with_host_reference_bind_results(vec![bind_result.clone()])
        .execute(request.clone())
        .expect("host context metadata should pass through runtime execution");

    assert_eq!(result.host_formula_context, Some(host_context.clone()));
    assert_eq!(
        result.host_reference_bind_results,
        vec![bind_result.clone()]
    );
    assert_eq!(
        result.prepared_formula_identity.host_formula_context,
        Some(host_context.clone())
    );
    assert_eq!(
        result.prepared_formula_identity.host_reference_bind_results,
        vec![bind_result.clone()]
    );

    let mut changed_context = host_context.clone();
    changed_context.host_namespace_version = Some("host-ns:v2".to_string());
    let changed_result = RuntimeEnvironment::new()
        .with_host_formula_context(changed_context)
        .with_host_reference_bind_results(vec![bind_result.clone()])
        .execute(request.clone())
        .expect("changed host context metadata should pass through runtime execution");
    assert_ne!(
        result.prepared_formula_identity.prepared_formula_key,
        changed_result
            .prepared_formula_identity
            .prepared_formula_key,
        "host-context identity must contribute to prepared identity"
    );

    let mut managed = RuntimeEnvironment::new()
        .with_host_formula_context(host_context.clone())
        .with_host_reference_bind_results(vec![bind_result.clone()])
        .open_session();
    let open = managed
        .open_managed_session(&request)
        .expect("managed open should carry host context identity");
    assert_eq!(
        open.prepared_formula_identity.host_formula_context,
        Some(host_context)
    );
    let execution = managed
        .execute_managed(request)
        .expect("managed execute should carry host reference bind results");
    assert_eq!(
        execution
            .prepared_formula_identity
            .host_reference_bind_results,
        vec![bind_result]
    );
}

#[test]
fn runtime_host_namespace_version_mutation_changes_identity_without_explicit_host_reference() {
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "generic-host-v1".to_string(),
        capability_profile_id: "host-capabilities:generic-v1".to_string(),
        resolution_rule_version: "host-resolution:v1".to_string(),
        host_namespace_version: Some("host-ns:v1".to_string()),
        registry_snapshot_identity: Some("registry:snapshot:v1".to_string()),
        structure_context_version: Some("structure:v1".to_string()),
        caller_context_identity: Some("caller:sheet1-r1c1".to_string()),
        table_context_identity: None,
    };
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-host-namespace-version", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let first = RuntimeEnvironment::new()
        .with_host_formula_context(host_context.clone())
        .execute(request.clone())
        .expect("host namespace context should pass through runtime execution");

    let mut changed_context = host_context.clone();
    changed_context.host_namespace_version = Some("host-ns:v2".to_string());
    let second = RuntimeEnvironment::new()
        .with_host_formula_context(changed_context.clone())
        .execute(request.clone())
        .expect("changed host namespace context should pass through runtime execution");

    assert_eq!(first.host_formula_context, Some(host_context));
    assert_eq!(second.host_formula_context, Some(changed_context));
    assert!(first.host_reference_bind_results.is_empty());
    assert!(second.host_reference_bind_results.is_empty());
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        second.prepared_formula_identity.prepared_formula_key,
        "host namespace version must invalidate prepared identity even before an explicit host-reference bind result exists"
    );

    let mut managed = RuntimeEnvironment::new()
        .with_host_formula_context(second.host_formula_context.clone().unwrap())
        .open_session();
    let open = managed
        .open_managed_session(&request)
        .expect("managed open should carry host namespace identity");
    assert_eq!(
        open.prepared_formula_identity.host_formula_context,
        second.host_formula_context
    );
    assert!(
        open.prepared_formula_identity
            .host_reference_bind_results
            .is_empty()
    );
}

#[test]
fn runtime_bare_host_name_binding_maps_to_defined_name_lane_and_replay_identity() {
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "generic-host-v1".to_string(),
        capability_profile_id: "host-capabilities:generic-v1".to_string(),
        resolution_rule_version: "host-resolution:v1".to_string(),
        host_namespace_version: Some("host-ns:v1".to_string()),
        registry_snapshot_identity: Some("registry:snapshot:v1".to_string()),
        structure_context_version: Some("structure:v1".to_string()),
        caller_context_identity: Some("caller:node-a".to_string()),
        table_context_identity: None,
    };
    let bind_result = RuntimeHostNameBindResult {
        host_name_handle: "host-name:margin".to_string(),
        canonical_name: "HostMargin".to_string(),
        host_dependency_key: None,
        source_span: TextSpan::new(1, 10),
        source_token_text: "HostMargin".to_string(),
        resolution_layer: "defined_name_lane".to_string(),
        binding_kind: "value_like".to_string(),
        shape_hint: Some("scalar".to_string()),
        caller_context_dependent: true,
        diagnostics: vec!["host-name resolved through generic defined-name lane".to_string()],
        replay_identity_contribution: "host-name:margin:v1".to_string(),
    };
    let binding = RuntimeHostNameBinding {
        bind_result: bind_result.clone(),
        binding: DefinedNameBinding::Value(CalcValue::number(41.0)),
    };
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-bare-host-name", 1, "=HostMargin+1"),
        TypedContextQueryBundle::default(),
    );

    let first = RuntimeEnvironment::new()
        .with_host_formula_context(host_context.clone())
        .with_host_name_bindings(vec![binding.clone()])
        .execute(request.clone())
        .expect("bare host name should use the generic defined-name lane");

    assert_eq!(first.published_worksheet_value, CalcValue::number(42.0));
    assert_eq!(first.host_formula_context, Some(host_context.clone()));
    assert_eq!(first.host_name_bind_results, vec![bind_result.clone()]);
    assert!(
        first
            .prepared_formula_identity
            .formal_references
            .iter()
            .any(|reference| reference.reference_family == "relative_or_caller_sensitive"),
        "caller-context-dependent host names must be visible in prepared reference identity"
    );
    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&first));
    assert_eq!(projection.host_name_bind_results, vec![bind_result.clone()]);
    assert_eq!(
        projection
            .prepared_formula_identity
            .as_ref()
            .expect("runtime projection should carry prepared identity")
            .host_name_bind_results,
        vec![bind_result.clone()]
    );

    let mut changed_context = host_context.clone();
    changed_context.host_namespace_version = Some("host-ns:v2".to_string());
    let second = RuntimeEnvironment::new()
        .with_host_formula_context(changed_context)
        .with_host_name_bindings(vec![binding.clone()])
        .execute(request.clone())
        .expect("host namespace mutation should still execute");
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        second.prepared_formula_identity.prepared_formula_key,
        "host namespace version must invalidate bare host-name prepared identity"
    );

    let mut changed_binding = binding;
    changed_binding.bind_result.replay_identity_contribution = "host-name:margin:v2".to_string();
    let third = RuntimeEnvironment::new()
        .with_host_formula_context(host_context)
        .with_host_name_bindings(vec![changed_binding])
        .execute(request)
        .expect("host name identity mutation should still execute");
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        third.prepared_formula_identity.prepared_formula_key,
        "host-name bind-result identity must invalidate prepared identity"
    );
}

#[test]
fn managed_runtime_executes_bare_host_name_binding_with_same_prepared_identity() {
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "generic-host-v1".to_string(),
        capability_profile_id: "host-capabilities:generic-v1".to_string(),
        resolution_rule_version: "host-resolution:v1".to_string(),
        host_namespace_version: Some("host-ns:v1".to_string()),
        registry_snapshot_identity: Some("registry:snapshot:v1".to_string()),
        structure_context_version: Some("structure:v1".to_string()),
        caller_context_identity: Some("caller:node-a".to_string()),
        table_context_identity: None,
    };
    let bind_result = RuntimeHostNameBindResult {
        host_name_handle: "host-name:managed-margin".to_string(),
        canonical_name: "HostMargin".to_string(),
        host_dependency_key: None,
        source_span: TextSpan::new(1, 10),
        source_token_text: "HostMargin".to_string(),
        resolution_layer: "defined_name_lane".to_string(),
        binding_kind: "value_like".to_string(),
        shape_hint: Some("scalar".to_string()),
        caller_context_dependent: true,
        diagnostics: Vec::new(),
        replay_identity_contribution: "host-name:managed-margin:v1".to_string(),
    };
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-managed-bare-host-name", 1, "=HostMargin+1"),
        TypedContextQueryBundle::default(),
    );
    let mut managed = RuntimeEnvironment::new()
        .with_host_formula_context(host_context)
        .with_host_name_bindings(vec![RuntimeHostNameBinding {
            bind_result: bind_result.clone(),
            binding: DefinedNameBinding::Value(CalcValue::number(41.0)),
        }])
        .open_session();

    let open = managed
        .open_managed_session(&request)
        .expect("managed open should prepare host-name binding");
    assert_eq!(
        open.prepared_formula_identity.host_name_bind_results,
        vec![bind_result.clone()]
    );

    let execution = managed
        .execute_managed(request)
        .expect("managed execution should use the host-name defined-name binding");

    assert_eq!(
        execution.candidate_result.value_delta.published_payload,
        oxfml_core::ValuePayload::Number("42".to_string())
    );
    assert_eq!(
        execution.prepared_formula_identity.host_name_bind_results,
        vec![bind_result]
    );
}

#[test]
fn runtime_bare_host_callable_uses_defined_name_lambda_lane() {
    let bind_result = RuntimeHostNameBindResult {
        host_name_handle: "host-name:lambda".to_string(),
        canonical_name: "HostLambda".to_string(),
        host_dependency_key: None,
        source_span: TextSpan::new(1, 10),
        source_token_text: "HostLambda".to_string(),
        resolution_layer: "defined_name_lambda_lane".to_string(),
        binding_kind: "defined_name_lambda".to_string(),
        shape_hint: Some("callable".to_string()),
        caller_context_dependent: false,
        diagnostics: Vec::new(),
        replay_identity_contribution: "host-name:lambda:v1".to_string(),
    };
    let binding = RuntimeHostNameBinding {
        bind_result: bind_result.clone(),
        binding: DefinedNameBinding::Callable(runtime_test_callable_binding()),
    };
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-bare-host-callable", 1, "=HostLambda(2)"),
        TypedContextQueryBundle::default(),
    )
    .with_trace_mode(EvaluationTraceMode::PreparedCalls);

    let result = RuntimeEnvironment::new()
        .with_host_name_bindings(vec![binding])
        .execute(request)
        .expect("callable host name should invoke through defined-name lambda lane");

    assert_eq!(result.published_worksheet_value, CalcValue::number(3.0));
    assert_eq!(result.host_name_bind_results, vec![bind_result.clone()]);
    assert_eq!(
        result
            .evaluation
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["SPECIAL.LAMBDA_INVOKE", "FUNC.OP_ADD"]
    );
    assert!(
        result.host_reference_bind_results.is_empty(),
        "bare host-name lane must not masquerade as explicit host-reference syntax"
    );
}

#[test]
fn runtime_table_context_mutation_changes_prepared_identity_for_structured_refs() {
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-table-context", 1, "=SUM(Table1[Amount])"),
        TypedContextQueryBundle::default(),
    );
    let mut first_values = BTreeMap::new();
    first_values.insert(
        "B2:B4".to_string(),
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    );
    let first = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_table("B2:B4")], None, None)
        .with_cell_values(first_values)
        .execute(request.clone())
        .expect("first table-context execution should succeed");
    assert_eq!(first.evaluation.oxfunc_value, CalcValue::number(12.0));

    let mut second_values = BTreeMap::new();
    second_values.insert(
        "D2:D4".to_string(),
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    );
    let second = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_table("D2:D4")], None, None)
        .with_cell_values(second_values)
        .execute(request)
        .expect("mutated table-context execution should succeed");

    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        second.prepared_formula_identity.prepared_formula_key,
        "table-context changes that alter structured-reference binding must change prepared identity"
    );
    assert_ne!(
        first.prepared_formula_identity.formal_references,
        second.prepared_formula_identity.formal_references,
        "structured-reference formal references should carry the changed table column target"
    );
}

#[test]
fn runtime_projects_structured_reference_bind_packets_for_downstream_consumers() {
    let mut explicit_values = BTreeMap::new();
    explicit_values.insert(
        "B2:B4".to_string(),
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    );
    let explicit = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_table("B2:B4")], None, None)
        .with_cell_values(explicit_values)
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:w074-structured-packet-explicit",
                1,
                "=SUM(Table1[Amount])",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("explicit structured reference should execute");
    assert_eq!(
        explicit.structured_reference_bind_records,
        explicit
            .prepared_formula_identity
            .structured_reference_bind_records
    );
    let explicit_record = &explicit.structured_reference_bind_records[0];
    assert_eq!(explicit_record.source_token_text, "Table1[Amount]");
    assert_eq!(
        explicit_record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(
        explicit_record.explicit_table_name.as_deref(),
        Some("Table1")
    );
    assert!(!explicit_record.omitted_table_name);
    assert_eq!(
        explicit_record.effective_table_id.as_deref(),
        Some("table:w074")
    );
    assert_eq!(
        explicit_record.selected_sections,
        vec![StructuredSectionKind::Data]
    );
    assert_eq!(explicit_record.selected_column_ids, vec!["column:amount"]);
    assert!(explicit_record.resolved_reference.is_some());
    assert!(explicit_record.diagnostics.is_empty());
    assert_eq!(
        explicit.prepared_formula_identity.formal_references[0]
            .structured_reference_bind_record_handle
            .as_deref(),
        Some(explicit_record.bind_record_handle.as_str())
    );

    let mut omitted_values = BTreeMap::new();
    omitted_values.insert("B3".to_string(), CalcValue::number(7.0));
    let omitted = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table("B2:B4")],
            Some(TableRef {
                table_id: "table:w074".to_string(),
            }),
            Some(TableCallerRegion {
                table_id: "table:w074".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(1),
            }),
        )
        .with_cell_values(omitted_values)
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:w074-structured-packet-omitted", 1, "=[@Amount]"),
            TypedContextQueryBundle::default(),
        ))
        .expect("omitted structured reference should execute");
    let omitted_record = &omitted.structured_reference_bind_records[0];
    assert_eq!(omitted_record.source_token_text, "[@Amount]");
    assert_eq!(
        omitted_record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(omitted_record.explicit_table_name, None);
    assert!(omitted_record.omitted_table_name);
    assert_eq!(
        omitted_record.selected_sections,
        vec![StructuredSectionKind::ThisRow]
    );
    assert!(omitted_record.uses_this_row);
    assert!(omitted_record.caller_context_dependent);
}

#[test]
fn runtime_links_qualified_structured_reference_failure_and_following_success_packets() {
    let mut table = runtime_w074_table("B2:B4");
    table.sheet_scope_ref = "Sheet1".to_string();
    let result = RuntimeEnvironment::new()
        .with_table_context(vec![table], None, None)
        .with_cell_values(BTreeMap::from([(
            "Sheet1!B2:B4".to_string(),
            CalcValue::array(
                CalcArray::from_rows(vec![vec![
                    CalcValue::number(3.0),
                    CalcValue::number(4.0),
                    CalcValue::number(5.0),
                ]])
                .expect("array fixture should be valid"),
            ),
        )]))
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:w074-structured-packet-qualified-failure",
                1,
                "=SUM(Sheet1!Missing[Amount],Sheet1!Table1[Amount])",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("qualified structured failure should still project runtime identity");

    assert_eq!(result.structured_reference_bind_records.len(), 2);
    let missing_record = &result.structured_reference_bind_records[0];
    let valid_record = &result.structured_reference_bind_records[1];
    assert_eq!(missing_record.source_token_text, "Sheet1!Missing[Amount]");
    assert_eq!(
        missing_record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(
        missing_record.explicit_table_name.as_deref(),
        Some("Missing")
    );
    assert_eq!(missing_record.effective_table_id, None);
    assert_eq!(missing_record.diagnostics.len(), 1);
    assert_eq!(valid_record.source_token_text, "Sheet1!Table1[Amount]");
    assert_eq!(
        valid_record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(valid_record.explicit_table_name.as_deref(), Some("Table1"));
    assert_eq!(
        valid_record.effective_table_id.as_deref(),
        Some("table:w074")
    );
    assert!(valid_record.diagnostics.is_empty());

    let formal_references = &result.prepared_formula_identity.formal_references;
    assert_eq!(
        formal_references[0]
            .structured_reference_bind_record_handle
            .as_deref(),
        Some(missing_record.bind_record_handle.as_str())
    );
    assert_eq!(
        formal_references[1]
            .structured_reference_bind_record_handle
            .as_deref(),
        Some(valid_record.bind_record_handle.as_str())
    );
    assert_eq!(
        formal_references[2]
            .structured_reference_bind_record_handle
            .as_deref(),
        Some(missing_record.bind_record_handle.as_str())
    );
}

#[test]
fn runtime_stable_table_fact_mutation_changes_prepared_identity_for_structured_refs() {
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-table-stable-facts", 1, "=SUM(Table1[Amount])"),
        TypedContextQueryBundle::default(),
    );
    let mut values = BTreeMap::new();
    values.insert(
        "B2:B4".to_string(),
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    );

    let first = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table_with_stable_facts(
                "B2:B4",
                "rows:v1",
                "row-order:v1",
                "A1:D1",
                "A5:D5",
            )],
            None,
            None,
        )
        .with_cell_values(values.clone())
        .execute(request.clone())
        .expect("first table stable-fact execution should succeed");
    let second = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table_with_stable_facts(
                "B2:B4",
                "rows:v2",
                "row-order:v2",
                "A1:D1",
                "A5:D5",
            )],
            None,
            None,
        )
        .with_cell_values(values)
        .execute(request)
        .expect("stable table fact mutation should succeed");

    assert_eq!(first.evaluation.oxfunc_value, CalcValue::number(12.0));
    assert_eq!(second.evaluation.oxfunc_value, CalcValue::number(12.0));
    assert_eq!(
        first.prepared_formula_identity.formal_references,
        second.prepared_formula_identity.formal_references,
        "row membership/order identities should not alter the already-resolved structured reference"
    );
    assert_ne!(
        first.prepared_formula_identity.table_context_fingerprint,
        second.prepared_formula_identity.table_context_fingerprint,
        "stable row membership/order identities must update the public table-context fingerprint"
    );
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        second.prepared_formula_identity.prepared_formula_key,
        "stable row membership/order identities must contribute to prepared identity"
    );
}

#[test]
fn runtime_table_descriptor_fact_mutations_change_prepared_identity() {
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new(
            "runtime:w074-table-descriptor-facts",
            1,
            "=SUM(Table1[Amount])",
        ),
        TypedContextQueryBundle::default(),
    );
    let base_values = runtime_w074_range_values("B2:B4");
    let base = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_table("B2:B4")], None, None)
        .with_cell_values(base_values.clone())
        .execute(request.clone())
        .expect("base table descriptor execution should succeed");
    assert_eq!(base.evaluation.oxfunc_value, CalcValue::number(12.0));

    let mut changed_table_id = runtime_w074_table("B2:B4");
    changed_table_id.table_id = "table:w074:renamed-id".to_string();
    let changed_table_id_result = RuntimeEnvironment::new()
        .with_table_context(vec![changed_table_id], None, None)
        .with_cell_values(base_values.clone())
        .execute(request.clone())
        .expect("changed table id execution should succeed");
    assert_eq!(
        changed_table_id_result.evaluation.oxfunc_value,
        CalcValue::number(12.0)
    );
    assert_ne!(
        base.structured_reference_bind_records,
        changed_table_id_result.structured_reference_bind_records,
        "table_id must be visible in the structured-reference bind packet"
    );
    assert_ne!(
        base.prepared_formula_identity.prepared_formula_key,
        changed_table_id_result
            .prepared_formula_identity
            .prepared_formula_key,
        "table_id changes must invalidate prepared identity"
    );

    let mut changed_table_range = runtime_w074_table("B2:B4");
    changed_table_range.table_range_ref = "A10:D14".to_string();
    let changed_table_range_result = RuntimeEnvironment::new()
        .with_table_context(vec![changed_table_range], None, None)
        .with_cell_values(base_values.clone())
        .execute(request.clone())
        .expect("changed table range execution should succeed");
    assert_eq!(
        changed_table_range_result.evaluation.oxfunc_value,
        CalcValue::number(12.0)
    );
    assert_eq!(
        base.prepared_formula_identity.formal_references,
        changed_table_range_result
            .prepared_formula_identity
            .formal_references,
        "data-column semantics should stay stable when only the enclosing table range changes"
    );
    assert_ne!(
        base.prepared_formula_identity.table_context_fingerprint,
        changed_table_range_result
            .prepared_formula_identity
            .table_context_fingerprint,
        "table_range_ref must be represented in the table-context fingerprint"
    );
    assert_ne!(
        base.prepared_formula_identity.prepared_formula_key,
        changed_table_range_result
            .prepared_formula_identity
            .prepared_formula_key,
        "table_range_ref changes must invalidate prepared identity conservatively"
    );

    let mut changed_column_id = runtime_w074_table("B2:B4");
    changed_column_id.columns[0].column_id = "column:amount:v2".to_string();
    let changed_column_id_result = RuntimeEnvironment::new()
        .with_table_context(vec![changed_column_id], None, None)
        .with_cell_values(base_values.clone())
        .execute(request.clone())
        .expect("changed column id execution should succeed");
    assert_eq!(
        changed_column_id_result.evaluation.oxfunc_value,
        CalcValue::number(12.0)
    );
    assert_ne!(
        base.structured_reference_bind_records,
        changed_column_id_result.structured_reference_bind_records,
        "column_id must be visible in the structured-reference bind packet"
    );
    assert_ne!(
        base.prepared_formula_identity.prepared_formula_key,
        changed_column_id_result
            .prepared_formula_identity
            .prepared_formula_key,
        "column_id changes must invalidate prepared identity"
    );

    let mut changed_column_ordinal = runtime_w074_table("B2:B4");
    changed_column_ordinal.columns[0].ordinal = 3;
    let changed_column_ordinal_result = RuntimeEnvironment::new()
        .with_table_context(vec![changed_column_ordinal], None, None)
        .with_cell_values(base_values.clone())
        .execute(request.clone())
        .expect("changed column ordinal execution should succeed");
    assert_eq!(
        changed_column_ordinal_result.evaluation.oxfunc_value,
        CalcValue::number(12.0)
    );
    assert_eq!(
        base.structured_reference_bind_records,
        changed_column_ordinal_result.structured_reference_bind_records,
        "a single selected column keeps the same bind record when ordinal does not alter selection"
    );
    assert_ne!(
        base.prepared_formula_identity.table_context_fingerprint,
        changed_column_ordinal_result
            .prepared_formula_identity
            .table_context_fingerprint,
        "column ordinal remains a conservative table-context identity input"
    );
    assert_ne!(
        base.prepared_formula_identity.prepared_formula_key,
        changed_column_ordinal_result
            .prepared_formula_identity
            .prepared_formula_key,
        "column ordinal changes must invalidate prepared identity conservatively"
    );

    let changed_column_range_result = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_table("D2:D4")], None, None)
        .with_cell_values(runtime_w074_range_values("D2:D4"))
        .execute(request)
        .expect("changed column range execution should succeed");
    assert_eq!(
        changed_column_range_result.evaluation.oxfunc_value,
        CalcValue::number(12.0)
    );
    assert_ne!(
        base.structured_reference_bind_records,
        changed_column_range_result.structured_reference_bind_records,
        "column_range_ref must be visible in selected regions and resolved references"
    );
    assert_ne!(
        base.prepared_formula_identity.prepared_formula_key,
        changed_column_range_result
            .prepared_formula_identity
            .prepared_formula_key,
        "column_range_ref changes must invalidate prepared identity"
    );
}

#[test]
fn runtime_unrelated_table_catalog_mutation_is_identity_only_for_referenced_table() {
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new(
            "runtime:w074-table-unrelated-catalog",
            1,
            "=SUM(Table1[Amount])",
        ),
        TypedContextQueryBundle::default(),
    );
    let mut unrelated = runtime_w074_table("F2:F4");
    unrelated.table_id = "table:unrelated".to_string();
    unrelated.table_name = "OtherTable".to_string();
    unrelated.columns[0].column_id = "column:other:amount".to_string();

    let mut mutated_unrelated = unrelated.clone();
    mutated_unrelated.row_membership_identity = Some("table:unrelated:rows:v2".to_string());
    mutated_unrelated.row_order_identity = Some("table:unrelated:row-order:v2".to_string());
    mutated_unrelated.columns[0].ordinal = 7;
    mutated_unrelated.columns[0].column_range_ref = "H2:H4".to_string();

    let first = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_table("B2:B4"), unrelated], None, None)
        .with_cell_values(runtime_w074_range_values("B2:B4"))
        .execute(request.clone())
        .expect("first unrelated table catalog execution should succeed");
    let second = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table("B2:B4"), mutated_unrelated],
            None,
            None,
        )
        .with_cell_values(runtime_w074_range_values("B2:B4"))
        .execute(request)
        .expect("mutated unrelated table catalog execution should succeed");

    assert_eq!(first.evaluation.oxfunc_value, CalcValue::number(12.0));
    assert_eq!(second.evaluation.oxfunc_value, CalcValue::number(12.0));
    assert_eq!(
        first.structured_reference_bind_records, second.structured_reference_bind_records,
        "unrelated table catalog entries must not change the referenced structured bind packet"
    );
    assert_eq!(
        first.prepared_formula_identity.formal_references,
        second.prepared_formula_identity.formal_references,
        "unrelated table catalog entries must not change referenced formal references"
    );
    assert_ne!(
        first.prepared_formula_identity.table_context_fingerprint,
        second.prepared_formula_identity.table_context_fingerprint,
        "full table catalog identity remains a conservative prepared-identity input"
    );
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        second.prepared_formula_identity.prepared_formula_key,
        "unrelated table catalog mutation should still invalidate conservative prepared identity"
    );
}

#[test]
fn runtime_omitted_structured_refs_include_enclosing_table_and_caller_row_identity() {
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-table-omitted-caller", 1, "=[@Amount]"),
        TypedContextQueryBundle::default(),
    );
    let mut other_table = runtime_w074_table("D2:D4");
    other_table.table_id = "table:w074:other".to_string();
    other_table.table_name = "OtherTable".to_string();

    let first = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table("B2:B4"), other_table.clone()],
            Some(TableRef {
                table_id: "table:w074".to_string(),
            }),
            Some(TableCallerRegion {
                table_id: "table:w074".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(1),
            }),
        )
        .with_cell_values(BTreeMap::from([("B3".to_string(), CalcValue::number(7.0))]))
        .execute(request.clone())
        .expect("first omitted structured reference should execute");
    let second = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table("B2:B4"), other_table],
            Some(TableRef {
                table_id: "table:w074:other".to_string(),
            }),
            Some(TableCallerRegion {
                table_id: "table:w074:other".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(1),
            }),
        )
        .with_cell_values(BTreeMap::from([("D3".to_string(), CalcValue::number(7.0))]))
        .execute(request.clone())
        .expect("changed enclosing table should execute");
    let third = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table("B2:B4")],
            Some(TableRef {
                table_id: "table:w074".to_string(),
            }),
            Some(TableCallerRegion {
                table_id: "table:w074".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(2),
            }),
        )
        .with_cell_values(BTreeMap::from([("B4".to_string(), CalcValue::number(7.0))]))
        .execute(request)
        .expect("changed caller row should execute");

    assert_eq!(first.evaluation.oxfunc_value, CalcValue::number(7.0));
    assert_eq!(second.evaluation.oxfunc_value, CalcValue::number(7.0));
    assert_eq!(third.evaluation.oxfunc_value, CalcValue::number(7.0));
    assert_ne!(
        first.structured_reference_bind_records, second.structured_reference_bind_records,
        "enclosing_table_ref changes must change omitted structured-reference binding"
    );
    assert_ne!(
        first.prepared_formula_identity.formal_references,
        third.prepared_formula_identity.formal_references,
        "caller_table_region data_row_offset must change current-row formal references"
    );
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        second.prepared_formula_identity.prepared_formula_key,
        "enclosing_table_ref must participate in prepared identity"
    );
    assert_ne!(
        first.prepared_formula_identity.prepared_formula_key,
        third.prepared_formula_identity.prepared_formula_key,
        "caller_table_region data_row_offset must participate in prepared identity"
    );
}

#[test]
fn runtime_exact_header_and_totals_region_refs_change_structured_identity() {
    let header_request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-table-header-ref", 1, "=Table1[#Headers]"),
        TypedContextQueryBundle::default(),
    );
    let first_header = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table_with_stable_facts(
                "B2:B4",
                "rows:v1",
                "row-order:v1",
                "A1:D1",
                "A5:D5",
            )],
            None,
            None,
        )
        .execute(header_request.clone())
        .expect("first header ref should prepare");
    let second_header = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table_with_stable_facts(
                "B2:B4",
                "rows:v1",
                "row-order:v1",
                "A10:D10",
                "A20:D20",
            )],
            None,
            None,
        )
        .execute(header_request)
        .expect("changed exact header ref should prepare");
    assert_ne!(
        first_header.prepared_formula_identity.formal_references,
        second_header.prepared_formula_identity.formal_references,
        "exact header region refs should change structured-reference identity"
    );
    assert_ne!(
        first_header.prepared_formula_identity.prepared_formula_key,
        second_header.prepared_formula_identity.prepared_formula_key,
        "exact header region refs must contribute to prepared identity"
    );
    let first_header_packet = &first_header.structured_reference_bind_records[0];
    assert_eq!(first_header_packet.source_token_text, "Table1[#Headers]");
    assert_eq!(
        first_header_packet.selected_sections,
        vec![StructuredSectionKind::Headers]
    );
    assert_eq!(
        first_header_packet.selected_regions[0]
            .region_ref
            .as_deref(),
        Some("A1:D1")
    );

    let totals_request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:w074-table-totals-ref", 1, "=Table1[#Totals]"),
        TypedContextQueryBundle::default(),
    );
    let first_totals = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table_with_stable_facts(
                "B2:B4",
                "rows:v1",
                "row-order:v1",
                "A1:D1",
                "A5:D5",
            )],
            None,
            None,
        )
        .execute(totals_request.clone())
        .expect("first totals ref should prepare");
    let second_totals = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_table_with_stable_facts(
                "B2:B4",
                "rows:v1",
                "row-order:v1",
                "A10:D10",
                "A20:D20",
            )],
            None,
            None,
        )
        .execute(totals_request)
        .expect("changed exact totals ref should prepare");
    assert_ne!(
        first_totals.prepared_formula_identity.formal_references,
        second_totals.prepared_formula_identity.formal_references,
        "exact totals region refs should change structured-reference identity"
    );
    assert_ne!(
        first_totals.prepared_formula_identity.prepared_formula_key,
        second_totals.prepared_formula_identity.prepared_formula_key,
        "exact totals region refs must contribute to prepared identity"
    );
    let first_totals_packet = &first_totals.structured_reference_bind_records[0];
    assert_eq!(first_totals_packet.source_token_text, "Table1[#Totals]");
    assert_eq!(
        first_totals_packet.selected_sections,
        vec![StructuredSectionKind::Totals]
    );
    assert_eq!(
        first_totals_packet.selected_regions[0]
            .region_ref
            .as_deref(),
        Some("A5:D5")
    );
}

#[test]
fn runtime_carries_zero_row_structured_data_packet_without_data_a1_area() {
    let result = RuntimeEnvironment::new()
        .with_table_context(vec![runtime_w074_zero_row_table()], None, None)
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:w074-zero-row-data-packet",
                1,
                "=IF(FALSE,Table1[Amount],0)",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("lazy zero-row structured reference should execute");

    assert_eq!(result.evaluation.oxfunc_value, CalcValue::number(0.0));
    assert!(result.bind_diagnostics.is_empty());
    assert!(
        result
            .prepared_formula_identity
            .table_context_fingerprint
            .is_some()
    );
    assert_eq!(
        result.structured_reference_bind_records,
        result
            .prepared_formula_identity
            .structured_reference_bind_records
    );
    let record = &result.structured_reference_bind_records[0];
    assert_eq!(record.source_token_text, "Table1[Amount]");
    assert_eq!(
        record.effective_table_id.as_deref(),
        Some("table:w074:zero")
    );
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
    assert_eq!(record.selected_sections, vec![StructuredSectionKind::Data]);
    assert!(record.selected_regions[0].is_empty);
    assert!(record.selected_regions[0].column_range_refs.is_empty());
    let Some(StructuredResolvedRef::EmptyArea(empty)) = &record.resolved_reference else {
        panic!("expected empty structured data body reference");
    };
    assert_eq!(empty.section_kind, StructuredSectionKind::Data);
    assert_eq!(empty.selected_column_ids, vec!["column:amount"]);
    assert_eq!(empty.column_count, 1);
}

#[test]
fn runtime_reports_zero_row_this_row_diagnostic_with_packet_identity() {
    let result = RuntimeEnvironment::new()
        .with_table_context(
            vec![runtime_w074_zero_row_table()],
            Some(TableRef {
                table_id: "table:w074:zero".to_string(),
            }),
            Some(TableCallerRegion {
                table_id: "table:w074:zero".to_string(),
                region_kind: TableRegionKind::Data,
                data_row_offset: Some(0),
            }),
        )
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:w074-zero-row-this-row-diagnostic",
                1,
                "=IF(FALSE,[@Amount],0)",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("lazy zero-row current-row diagnostic should execute");

    assert_eq!(result.evaluation.oxfunc_value, CalcValue::number(0.0));
    assert_eq!(result.bind_diagnostics.len(), 1);
    assert!(
        result.bind_diagnostics[0]
            .message
            .contains("no table data row")
    );
    let record = &result.structured_reference_bind_records[0];
    assert_eq!(record.source_token_text, "[@Amount]");
    assert_eq!(
        record.effective_table_id.as_deref(),
        Some("table:w074:zero")
    );
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
    assert_eq!(
        record.selected_sections,
        vec![StructuredSectionKind::ThisRow]
    );
    assert!(record.uses_this_row);
    assert!(record.caller_context_dependent);
    assert_eq!(record.diagnostics.len(), 1);
    assert!(record.selected_regions[0].is_empty);
    assert_eq!(record.resolved_reference, None);
}

#[test]
fn runtime_preserves_lexical_callables_without_host_namespace() {
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:w074-lexical-no-host",
                1,
                "=LET(base,100,adder,LAMBDA(n,LAMBDA(x,x+n+base)),add5,adder(5),add5(10))",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("lexical returned lambda should execute without host namespace");

    assert_eq!(result.evaluation.oxfunc_value, CalcValue::number(115.0));
    assert_eq!(result.host_formula_context, None);
    assert!(result.host_reference_bind_results.is_empty());
    assert_eq!(result.prepared_formula_identity.host_formula_context, None);
    assert!(
        result
            .prepared_formula_identity
            .host_reference_bind_results
            .is_empty()
    );
}

fn runtime_w074_table(amount_range_ref: &str) -> TableDescriptor {
    TableDescriptor {
        table_id: "table:w074".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "A1:D5".to_string(),
        row_membership_identity: Some("table:w074:rows:v1".to_string()),
        row_order_identity: Some("table:w074:row-order:v1".to_string()),
        header_region_ref: Some("A1:D1".to_string()),
        totals_region_ref: Some("A5:D5".to_string()),
        header_row_present: true,
        totals_row_present: true,
        columns: vec![TableColumnDescriptor {
            column_id: "column:amount".to_string(),
            column_name: "Amount".to_string(),
            ordinal: 1,
            column_range_ref: amount_range_ref.to_string(),
        }],
    }
}

fn runtime_w074_zero_row_table() -> TableDescriptor {
    TableDescriptor {
        table_id: "table:w074:zero".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "A1:D2".to_string(),
        row_membership_identity: Some("table:w074:zero:rows:empty".to_string()),
        row_order_identity: Some("table:w074:zero:row-order:empty".to_string()),
        header_region_ref: Some("A1:D1".to_string()),
        totals_region_ref: Some("A2:D2".to_string()),
        header_row_present: true,
        totals_row_present: true,
        columns: vec![TableColumnDescriptor {
            column_id: "column:amount".to_string(),
            column_name: "Amount".to_string(),
            ordinal: 2,
            column_range_ref: String::new(),
        }],
    }
}

fn runtime_w074_range_values(range_ref: &str) -> BTreeMap<String, CalcValue> {
    BTreeMap::from([(
        range_ref.to_string(),
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    )])
}

fn runtime_w074_table_with_stable_facts(
    amount_range_ref: &str,
    row_membership_identity: &str,
    row_order_identity: &str,
    header_region_ref: &str,
    totals_region_ref: &str,
) -> TableDescriptor {
    let mut table = runtime_w074_table(amount_range_ref);
    table.row_membership_identity = Some(row_membership_identity.to_string());
    table.row_order_identity = Some(row_order_identity.to_string());
    table.header_region_ref = Some(header_region_ref.to_string());
    table.totals_region_ref = Some(totals_region_ref.to_string());
    table
}

#[test]
fn runtime_prepared_identity_carries_oxfunc_bridge_versions_without_enforcement() {
    let result = RuntimeEnvironment::new()
        .with_oxfunc_bridge_metadata(RuntimeOxFuncBridgeMetadata {
            semantic_kernel_metadata_version: Some("sem-kernel:v1".to_string()),
            arg_admission_metadata_version: Some("arg-admission:v1".to_string()),
        })
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:oxfunc-bridge", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    assert_eq!(
        result
            .prepared_formula_identity
            .semantic_kernel_metadata_version
            .as_deref(),
        Some("sem-kernel:v1")
    );
    assert_eq!(
        result
            .prepared_formula_identity
            .arg_admission_metadata_version
            .as_deref(),
        Some("arg-admission:v1")
    );

    let changed_semantic_version = RuntimeEnvironment::new()
        .with_semantic_kernel_metadata_version("sem-kernel:v2")
        .with_arg_admission_metadata_version("arg-admission:v1")
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:oxfunc-bridge", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    assert_ne!(
        result.prepared_formula_identity.prepared_formula_key,
        changed_semantic_version
            .prepared_formula_identity
            .prepared_formula_key,
        "semantic kernel metadata version participates in prepared-package invalidation"
    );
}

#[test]
fn runtime_prepared_identity_derives_oxfunc_bridge_versions_from_registry_metadata() {
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:registry-bridge", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    assert!(
        result
            .prepared_formula_identity
            .semantic_kernel_metadata_version
            .as_deref()
            .is_some_and(
                |version| version.contains("numerical_reduction_policy=SequentialLeftFold")
            ),
        "SUM should carry OxFunc registry semantic kernel metadata version"
    );
    assert_eq!(
        result
            .prepared_formula_identity
            .arg_admission_metadata_version
            .as_deref(),
        Some("arg_admission_metadata.v1;existing_arg_preparation=values_only_pre_adapter")
    );
}

#[test]
fn runtime_registry_view_admits_udf_without_unknown_function_freeze() {
    let source = FormulaSourceRecord::new("runtime:registry-udf-admission", 1, "=MYFUNC(10,\"x\")");
    let default_result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            source.clone(),
            TypedContextQueryBundle::default(),
        ))
        .expect("default runtime should still classify unknown UDF calls as worksheet results");
    assert!(
        default_result
            .semantic_plan
            .diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "unknown_function"
                    && diagnostic.function_name.as_deref() == Some("MYFUNC")
            })
    );
    assert_eq!(
        default_result.published_worksheet_value,
        CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Name)
    );

    let mut registry = builtin_registry().clone();
    registry
        .register_udf(runtime_test_udf_entry())
        .expect("UDF registration should be accepted by OxFunc registry");
    let registered_result = RuntimeEnvironment::new()
        .with_function_registry(&registry)
        .execute(RuntimeFormulaRequest::new(
            source.clone(),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime should classify registered UDF calls deterministically");

    assert!(
        registered_result
            .semantic_plan
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "unknown_function"),
        "registered UDF calls should be registry-present rather than unknown"
    );
    let summary = registered_result
        .semantic_plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "MYFUNC")
        .expect("registered UDF should be represented in semantic availability summaries");
    assert_eq!(summary.canonical_id.as_deref(), Some("FUNC.UDF.MYFUNC"));
    assert_eq!(
        summary.parse_bind_state,
        LibraryAvailabilityState::CatalogKnown
    );
    assert!(
        registered_result
            .prepared_formula_identity
            .registry_snapshot_identity
            .is_some()
    );
    assert_ne!(
        default_result
            .prepared_formula_identity
            .prepared_formula_key,
        registered_result
            .prepared_formula_identity
            .prepared_formula_key,
        "registry snapshot identity must participate in prepared identity"
    );
    let registered_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_result(&registered_result),
    );
    assert_eq!(
        registered_projection.registry_pin,
        registered_result
            .prepared_formula_identity
            .registry_snapshot_identity
            .clone(),
        "registered UDF runtime replay must preserve registry snapshot identity"
    );

    registry
        .unregister_udf("FUNC.UDF.MYFUNC")
        .expect("UDF unregister should be accepted by OxFunc registry");
    let unregistered_result = RuntimeEnvironment::new()
        .with_function_registry(&registry)
        .execute(RuntimeFormulaRequest::new(
            source,
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime should return to unknown classification after unregister");
    assert!(
        unregistered_result
            .semantic_plan
            .diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic.code == "unknown_function"
                    && diagnostic.function_name.as_deref() == Some("MYFUNC")
            })
    );
    assert_eq!(
        unregistered_result.published_worksheet_value,
        CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Name)
    );
    assert_ne!(
        registered_result
            .prepared_formula_identity
            .prepared_formula_key,
        unregistered_result
            .prepared_formula_identity
            .prepared_formula_key,
        "UDF unregister/default-registry transition must invalidate prepared identity"
    );
    let unregistered_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_result(&unregistered_result),
    );
    assert_eq!(
        unregistered_projection.registry_pin,
        unregistered_result
            .prepared_formula_identity
            .registry_snapshot_identity
            .clone(),
        "unregistered/default runtime replay must preserve registry snapshot identity"
    );
}

#[test]
fn runtime_capability_overlay_blocks_registry_present_call_before_dispatch_and_replays_identity() {
    let mut overlay = CapabilityOverlay::new();
    overlay.deny_function_id("FUNC.SUM", "W074 capability denial probe");
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:registry-capability-denied", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );
    let allowed_result = RuntimeEnvironment::new()
        .execute(request.clone())
        .expect("baseline SUM should execute without capability overlay");
    assert_eq!(
        allowed_result.published_worksheet_value,
        CalcValue::number(3.0)
    );
    assert!(
        allowed_result
            .prepared_formula_identity
            .registry_capability_denials
            .is_empty()
    );

    let result = RuntimeEnvironment::new()
        .with_capability_overlay(&overlay)
        .execute(request)
        .expect("capability denied registry view should classify as worksheet result");

    assert_eq!(
        result.published_worksheet_value,
        CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Blocked)
    );
    let summary = result
        .semantic_plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "SUM")
        .expect("SUM should remain registry-present under capability denial");
    assert_eq!(summary.canonical_id.as_deref(), Some("FUNC.SUM"));
    assert_eq!(
        summary.runtime_capability_state,
        Some(LibraryAvailabilityState::HostProfileUnavailable)
    );
    assert_eq!(
        result
            .prepared_formula_identity
            .registry_capability_denials
            .iter()
            .map(|denial| denial.surface_name.as_str())
            .collect::<Vec<_>>(),
        vec!["SUM"]
    );
    assert!(
        result
            .prepared_formula_identity
            .capability_overlay_identity
            .is_some(),
        "capability overlay identity must be present in prepared identity"
    );
    assert_ne!(
        allowed_result
            .prepared_formula_identity
            .prepared_formula_key,
        result.prepared_formula_identity.prepared_formula_key,
        "capability overlay denial must invalidate prepared identity"
    );

    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&result));
    assert_eq!(
        projection.registry_pin,
        result
            .prepared_formula_identity
            .registry_snapshot_identity
            .clone()
    );
    assert_eq!(
        projection
            .prepared_formula_identity
            .as_ref()
            .expect("runtime projection should carry prepared identity")
            .registry_capability_denials[0]
            .canonical_id
            .as_deref(),
        Some("FUNC.SUM")
    );
}

#[test]
fn managed_runtime_snapshots_carry_oxfunc_bridge_versions() {
    let environment = RuntimeEnvironment::new()
        .with_semantic_kernel_metadata_version("sem-kernel:managed:v1")
        .with_arg_admission_metadata_version("arg-admission:managed:v1");
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-oxfunc-bridge", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");
    let execution = session
        .execute_managed(request)
        .expect("managed execution should succeed");
    let snapshot = session
        .managed_session_snapshot()
        .expect("managed snapshot should exist");

    for identity in [
        &open.prepared_formula_identity,
        &execution.prepared_formula_identity,
        &snapshot.prepared_formula_identity,
    ] {
        assert_eq!(
            identity.semantic_kernel_metadata_version.as_deref(),
            Some("sem-kernel:managed:v1")
        );
        assert_eq!(
            identity.arg_admission_metadata_version.as_deref(),
            Some("arg-admission:managed:v1")
        );
    }
}

#[test]
fn runtime_session_facade_reports_managed_abort_with_session_snapshot() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-abort", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let _open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");
    let termination = session
        .abort_managed(Some("user_cancelled".to_string()))
        .expect("managed abort should produce a structured result");

    assert_eq!(
        termination.session.phase,
        RuntimeManagedSessionPhase::Aborted
    );
    assert_eq!(
        termination.reject_record.reject_code,
        oxfml_core::RejectCode::SessionTerminated
    );
    assert_eq!(
        termination.execution_outcome_surface.outcome_kind,
        oxfml_core::ExecutionOutcomeKind::Rejected
    );
    assert_eq!(
        termination.execution_outcome_surface.outcome_stage,
        oxfml_core::ExecutionOutcomeStage::CommitBoundary
    );
    assert_eq!(
        termination.execution_outcome_surface.class_id,
        "commit_boundary_reject"
    );
    assert_eq!(
        termination
            .execution_outcome_surface
            .lane_reason_code
            .as_deref(),
        Some("SessionTerminated")
    );
    assert_eq!(
        termination.session.execution_outcome_surface,
        Some(termination.execution_outcome_surface.clone())
    );
    assert!(
        termination
            .session
            .trace_events
            .iter()
            .any(|event| matches!(event.event_kind, oxfml_core::TraceEventKind::SessionAborted))
    );
}

#[test]
fn runtime_session_facade_managed_session_uses_inline_snapshot() {
    let inline_snapshot = runtime_snapshot_v2();
    let inline_snapshot_ref = LibraryContextSnapshotRef::from(&inline_snapshot);
    let environment =
        RuntimeEnvironment::new().with_inline_library_context_snapshot(inline_snapshot);
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-inline", 1, "=TAKE({1,2},1)"),
        TypedContextQueryBundle::default(),
    );

    let open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");

    assert_eq!(
        open.library_context_snapshot_ref,
        Some(inline_snapshot_ref.clone())
    );
    assert_eq!(
        open.semantic_plan.library_context_snapshot_ref,
        Some(inline_snapshot_ref)
    );
}

#[test]
fn runtime_session_facade_invalidates_cache_when_formula_source_changes() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let first_request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:change", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );
    let second_request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:change", 2, "=SUM(1,3)")
            .with_stored_formula_text("=SUM(1,3)"),
        TypedContextQueryBundle::default(),
    );

    let _first = session
        .execute(first_request)
        .expect("first runtime execution should succeed");
    let second = session
        .execute(second_request)
        .expect("second runtime execution should succeed");

    assert!(!second.artifact_reuse.green_tree_reused);
    assert!(!second.artifact_reuse.red_projection_reused);
    assert!(!second.artifact_reuse.bound_formula_reused);
    assert!(!second.artifact_reuse.semantic_plan_reused);
    assert_eq!(
        second.source.stored_formula_text.as_deref(),
        Some("=SUM(1,3)")
    );
}

#[test]
fn runtime_environment_builder_applies_caller_context() {
    let environment = RuntimeEnvironment::new().with_caller_position(5, 3);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:caller", 1, "=ROW()+COLUMN()"),
        TypedContextQueryBundle::default(),
    );

    let result = environment
        .execute(request)
        .expect("runtime execution with caller context should succeed");

    assert_eq!(result.published_worksheet_value, CalcValue::number(8.0));
}

#[test]
fn runtime_environment_rejects_execution_when_syntax_diagnostics_exist() {
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:syntax-reject", 1, "=1~2"),
        TypedContextQueryBundle::default(),
    );

    let error = RuntimeEnvironment::new()
        .execute(request)
        .expect_err("runtime execution should reject unsupported syntax");

    assert!(error.contains("syntax diagnostics"));
    assert!(error.contains("Unknown"));
}

#[test]
fn runtime_session_facade_reports_missing_managed_snapshot_as_structured_error() {
    let environment = RuntimeEnvironment::new().with_library_context_snapshot_ref(
        LibraryContextSnapshotRef::new("runtime-consumer", "missing"),
    );
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-missing", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let error = session
        .open_managed_session(&request)
        .expect_err("managed open should reject unresolved snapshot ref");

    match error {
        RuntimeManagedSessionError::Preparation(message) => {
            assert!(message.contains("requested library context snapshot"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn runtime_session_facade_rejects_managed_execute_when_syntax_diagnostics_exist() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-syntax-reject", 1, "=1~2"),
        TypedContextQueryBundle::default(),
    );

    let open = session
        .open_managed_session(&request)
        .expect("managed open should preserve diagnostics for analysis");
    assert!(!open.syntax_diagnostics.is_empty());

    let error = session
        .execute_managed(request)
        .expect_err("managed execute should reject unsupported syntax");

    match error {
        RuntimeManagedSessionError::Preparation(message) => {
            assert!(message.contains("syntax diagnostics"));
            assert!(message.contains("Unknown"));
        }
        other => panic!("unexpected error kind: {other:?}"),
    }
}

#[test]
fn runtime_session_facade_executes_and_commits_managed_in_one_step() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-one-step", 1, "=SUM(4,5)"),
        TypedContextQueryBundle::default(),
    );

    let commit = session
        .execute_and_commit_managed(request, "commit:runtime-managed-one-step")
        .expect("one-step managed execution should succeed");

    assert_eq!(commit.session.phase, RuntimeManagedSessionPhase::Committed);
    assert_eq!(
        commit.execution_outcome_surface.outcome_kind,
        oxfml_core::ExecutionOutcomeKind::ExecutedResult
    );
    assert_eq!(
        commit.execution_outcome_surface.outcome_stage,
        oxfml_core::ExecutionOutcomeStage::Executed
    );
    match &commit.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("9".to_string())
            );
        }
        AcceptDecision::Rejected(reject) => {
            panic!("expected accepted commit, got reject: {reject:?}");
        }
    }
}

#[test]
fn runtime_environment_executes_rtd_formula_through_typed_query_bundle() {
    let locale = oxfml_en_us_locale_context();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:rtd", 1, "=RTD(\"prog\",\"server\",\"topic\")"),
        TypedContextQueryBundle::new(None, Some(&ValueRtdProvider), Some(&locale), None, None),
    );

    let result = RuntimeEnvironment::new()
        .execute(request)
        .expect("runtime RTD execution should succeed");

    assert!(
        result
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::Rtd)
    );
    assert_eq!(result.published_worksheet_value, CalcValue::number(7.0));
}

#[test]
fn runtime_environment_executes_registered_external_formula_through_typed_query_bundle() {
    let provider = RecordingRegisteredExternalProvider::default();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:call-register", 1, "=CALL(4242,6,7,3)"),
        TypedContextQueryBundle::default().with_registered_external_provider(Some(&provider)),
    )
    .with_trace_mode(EvaluationTraceMode::PreparedCalls);

    let result = RuntimeEnvironment::new()
        .execute(request)
        .expect("runtime registered-external execution should succeed");

    assert!(
        result
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::RegisteredExternal)
    );
    assert_eq!(result.published_worksheet_value, CalcValue::number(14.0));
    assert_eq!(provider.last_lookup.borrow().as_ref(), Some(&4242.0));
    match result.evaluation.trace.prepared_calls[0]
        .registered_external_call_request
        .as_ref()
        .expect("normalized call request")
    {
        oxfml_core::RegisteredExternalCallRequest {
            target: RegisteredExternalTarget::RegisterId(register_id),
            invocation_args,
        } => {
            assert_eq!(*register_id, 4242.0);
            assert_eq!(
                *invocation_args,
                vec![
                    CalcValue::number(6.0),
                    CalcValue::number(7.0),
                    CalcValue::number(3.0),
                ]
            );
        }
        other => panic!("unexpected normalized call request: {other:?}"),
    }
}

#[test]
fn runtime_environment_executes_bind_visible_host_function_through_typed_query_bundle() {
    let provider = RecordingHostFunctionProvider::default();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:vba-addthem", 1, "=AddThem(2,3)"),
        TypedContextQueryBundle::default().with_host_function_provider(Some(&provider)),
    )
    .with_trace_mode(EvaluationTraceMode::PreparedCalls);
    let environment =
        RuntimeEnvironment::new().with_inline_library_context_snapshot(vba_udf_snapshot());

    let result = environment
        .execute(request)
        .expect("runtime host function execution should succeed");

    assert!(
        result
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::HostFunction)
    );
    assert!(result.semantic_plan.diagnostics.is_empty());
    assert_eq!(result.published_worksheet_value, CalcValue::number(5.0));
    assert_eq!(
        provider.last_invocation.borrow().as_ref(),
        Some(&HostFunctionInvocation {
            function_name: "ADDTHEM".to_string(),
            args: vec![CalcValue::number(2.0), CalcValue::number(3.0)],
        })
    );
    let prepared_call = &result.evaluation.trace.prepared_calls[0];
    assert_eq!(prepared_call.function_name, "ADDTHEM");
    assert_eq!(prepared_call.function_id, "FUNC.HOST_CALLBACK");
    assert_eq!(prepared_call.returned_value, Some(CalcValue::number(5.0)));
}

#[test]
fn runtime_environment_keeps_unknown_function_as_name_error_with_host_provider() {
    let provider = RecordingHostFunctionProvider::default();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:vba-unknown", 1, "=NotRegistered(2,3)"),
        TypedContextQueryBundle::default().with_host_function_provider(Some(&provider)),
    );

    let result = RuntimeEnvironment::new()
        .execute(request)
        .expect("runtime unknown function execution should still produce worksheet value");

    assert_eq!(
        result.published_worksheet_value,
        CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Name)
    );
    assert!(provider.last_invocation.borrow().is_none());
}

#[test]
fn runtime_environment_emits_effective_display_text_comparison_view_for_verification_context() {
    let locale = oxfml_en_us_locale_context();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:verification-comparison-views", 1, "=SUM(1,2,3)"),
        TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
    )
    .with_verification_publication_context(VerificationPublicationContext {
        format_profile: Some("excel-spreadsheetml-2003-default".to_string()),
        number_format_code: Some("$#,##0.00".to_string()),
        style_id: Some("calc".to_string()),
        style_hierarchy: vec!["calcBase".to_string(), "calc".to_string()],
        font_color: Some("#112233".to_string()),
        fill_color: Some("#445566".to_string()),
        conditional_formatting_rules: vec![VerificationConditionalFormattingRule {
            target_ranges: vec!["A1".to_string()],
            rule_kind: "Expression".to_string(),
            operator: None,
            thresholds: vec!["=A1>0".to_string()],
            typed_rule: None,
            font_color: Some("#FF0000".to_string()),
            fill_color: Some("#00FF00".to_string()),
            effective_display_text: None,
            applies: None,
            effective_font_color: None,
            effective_fill_color: None,
        }],
    });

    let result = RuntimeEnvironment::new()
        .execute(request)
        .expect("runtime verification execution should succeed");

    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        "$6.00"
    );
    assert_eq!(result.comparison_views.len(), 5);
    assert_eq!(
        result
            .comparison_views
            .iter()
            .find(|view| view.view_family == "comparison_value")
            .map(|view| view.value.clone()),
        Some(serde_json::json!({
            "kind": "number",
            "value": 6.0
        }))
    );
    assert_eq!(
        result
            .comparison_views
            .iter()
            .find(|view| view.view_family == "effective_display_text")
            .map(|view| view.value.clone()),
        Some(Value::String("$6.00".to_string()))
    );
}

#[test]
fn runtime_environment_emits_effective_display_text_comparison_view_for_programmatic_verification_cases()
 {
    let locale = oxfml_en_us_locale_context();
    let cases = [
        ("FTC-0703", "=DATEDIF(\"2020-01-15\",\"2024-03-20\",\"Y\")"),
        (
            "FTC-0761",
            "=LET(n,10,result,REDUCE({0,1},SEQUENCE(n-1),LAMBDA(pair,_,LET(a,INDEX(pair,1),b,INDEX(pair,2),HSTACK(b,a+b)))),INDEX(result,2))",
        ),
        (
            "FTC-0940",
            "=SUM(FILTER({1,2,3,4,5},ISNUMBER(XMATCH({1,2,3,4,5},{2,4,6,8}))))",
        ),
        (
            "FTC-1030",
            "=LET(data,CHOOSE(SEQUENCE(1,3),{1;2;3},{10;20;30},{100;200;300}),result,TRANSPOSE(data),INDEX(result,1,2))",
        ),
    ];

    for (case_id, formula) in cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    format!("runtime:programmatic-verification:{case_id}"),
                    1,
                    formula,
                ),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .unwrap_or_else(|error| panic!("{case_id} runtime execution should succeed: {error}"));

        assert!(
            !result
                .verification_publication_surface
                .has_publication_context,
            "{case_id} should remain a no-explicit-publication-context control"
        );
        let expected_value =
            expected_programmatic_comparison_value(&result.published_worksheet_value);
        let expected_text = result
            .verification_publication_surface
            .effective_display_text
            .clone();
        assert_eq!(
            result.verification_publication_surface.visible_value_text,
            result
                .verification_publication_surface
                .effective_display_text,
            "{case_id} visible value text"
        );
        assert_eq!(
            result.comparison_views.len(),
            3,
            "{case_id} comparison view count"
        );
        assert_eq!(
            result
                .comparison_views
                .iter()
                .find(|view| view.view_family == "comparison_value")
                .map(|view| view.value.clone()),
            Some(expected_value.clone()),
            "{case_id} comparison_value view"
        );
        assert_eq!(
            result
                .comparison_views
                .iter()
                .find(|view| view.view_family == "visible_value_text")
                .map(|view| view.value.clone()),
            Some(Value::String(expected_text.clone())),
            "{case_id} visible_value_text view"
        );
        assert_eq!(
            result
                .comparison_views
                .iter()
                .find(|view| view.view_family == "effective_display_text")
                .map(|view| view.value.clone()),
            Some(Value::String(expected_text)),
            "{case_id} effective_display_text view"
        );
    }
}

#[test]
fn runtime_environment_preserves_randarray_width_for_columns_ftc_0505_with_random_provider() {
    let locale = oxfml_en_us_locale_context();
    let random_provider = SequenceRandomProvider { next: Cell::new(1) };
    let result = RuntimeEnvironment::new()
        .execute(
            RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    "runtime:foundation:FTC-0505",
                    1,
                    "=COLUMNS(RANDARRAY(5,3))",
                ),
                TypedContextQueryBundle::new(
                    None,
                    None,
                    Some(&locale),
                    None,
                    Some(&random_provider),
                ),
            )
            .with_trace_mode(EvaluationTraceMode::PreparedCalls),
        )
        .expect("FTC-0505 runtime execution should succeed");

    assert_eq!(result.published_worksheet_value, CalcValue::number(3.0));
    assert_eq!(
        result.execution_outcome_surface.outcome_kind,
        ExecutionOutcomeKind::ExecutedResult
    );
    assert_eq!(
        result
            .comparison_views
            .iter()
            .find(|view| view.view_family == "comparison_value")
            .map(|view| view.value.clone()),
        Some(serde_json::json!({
            "kind": "number",
            "value": 3.0
        }))
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "3"
    );
    assert!(
        result
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::RandomProvider)
    );
    assert_eq!(
        result.evaluation.trace.prepared_calls[1].prepared_arguments[0].structure_class,
        oxfml_core::PreparedStructureClass::ArrayLike
    );
}

#[test]
fn runtime_environment_randarray_consumes_distinct_provider_draws() {
    let locale = oxfml_en_us_locale_context();
    let random_provider = SequenceRandomProvider { next: Cell::new(1) };
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:randarray-5x5", 1, "=RANDARRAY(5,5)"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, Some(&random_provider)),
        ))
        .expect("RANDARRAY runtime execution should succeed");

    let CoreValue::Array(array) = result.published_worksheet_value.core() else {
        panic!("expected array result");
    };
    assert_eq!(array.shape(), ArrayShape { rows: 5, cols: 5 });
    let values = array.iter_row_major().cloned().collect::<Vec<_>>();
    assert_eq!(values.first(), Some(&CalcValue::number(0.01)));
    assert_eq!(values.get(12), Some(&CalcValue::number(0.13)));
    assert_eq!(values.last(), Some(&CalcValue::number(0.25)));
    assert_eq!(random_provider.next.get(), 26);
}

#[test]
fn runtime_environment_matches_excel_lambda_array_authoring_frontier_cases() {
    let locale = oxfml_en_us_locale_context();
    let cases = [
        (
            "FTC-0446",
            "=LET(dict,{\"key1\",LAMBDA({10,20,30});\"key2\",LAMBDA({40,50,60})},keys,INDEX(dict,0,1),INDEX(keys,1,1))",
        ),
        (
            "FTC-0447",
            "=LET(dict,{\"a\",LAMBDA(1);\"b\",LAMBDA(2);\"c\",LAMBDA(3)},keys,TAKE(dict,,1),ROWS(keys))",
        ),
    ];

    for (case_id, formula) in cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:foundation:{case_id}"), 1, formula),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .unwrap_or_else(|error| {
                panic!("{case_id} runtime execution should classify bind mismatch: {error:?}")
            });

        assert_eq!(
            result.published_worksheet_value,
            CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Value),
            "{case_id} published worksheet value"
        );
        assert_eq!(
            result.execution_outcome_surface.outcome_stage,
            ExecutionOutcomeStage::BindBoundary,
            "{case_id} outcome stage"
        );
        assert!(result.bind_diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "LAMBDA cannot appear inside array constants"
        }));
    }
}

#[test]
fn runtime_environment_matches_excel_builtin_collision_arity_authoring_frontier_ftc_0444() {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:foundation:FTC-0444",
                1,
                "=LET(THUNK,LAMBDA(x,LAMBDA(x)),t,THUNK(42),t())",
            ),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("FTC-0444 runtime execution should classify bind mismatch");

    assert_eq!(
        result.published_worksheet_value,
        CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "#VALUE!"
    );
    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        "#VALUE!"
    );
    assert_eq!(
        result.execution_outcome_surface.outcome_kind,
        ExecutionOutcomeKind::Rejected
    );
    assert_eq!(
        result.execution_outcome_surface.outcome_stage,
        ExecutionOutcomeStage::BindBoundary
    );
    assert_eq!(
        result.execution_outcome_surface.class_id,
        "bind_boundary_reject"
    );
    assert_eq!(
        result.execution_outcome_surface.lane_reason_code.as_deref(),
        Some("BindMismatch")
    );
    assert!(result.bind_diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .starts_with("built-in function call 'T' rejects 0 arguments at the authoring boundary")
    }));
    assert!(result.evaluation.trace.prepared_calls.is_empty());
}

#[test]
fn runtime_environment_mirrors_builtin_frontier_for_colliding_let_call_shapes() {
    let locale = oxfml_en_us_locale_context();
    let reject_cases = [
        ("plain-t-zero", "=T()", "T"),
        ("colliding-t-zero", "=LET(t,LAMBDA(42),t())", "T"),
        ("plain-gcd-zero", "=GCD()", "GCD"),
        ("colliding-gcd-zero", "=LET(gcd,LAMBDA(42),gcd())", "GCD"),
    ];

    for (case_id, formula, builtin_name) in reject_cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:{case_id}"), 1, formula),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .unwrap_or_else(|error| {
                panic!("{case_id} runtime execution should classify bind mismatch: {error:?}")
            });

        assert_eq!(
            result.execution_outcome_surface.outcome_kind,
            ExecutionOutcomeKind::Rejected,
            "{case_id} outcome kind"
        );
        assert_eq!(
            result.execution_outcome_surface.outcome_stage,
            ExecutionOutcomeStage::BindBoundary,
            "{case_id} outcome stage"
        );
        assert!(result.bind_diagnostics.iter().any(|diagnostic| {
            diagnostic.message.starts_with(&format!(
                "built-in function call '{builtin_name}' rejects 0 arguments at the authoring boundary"
            ))
        }));
    }

    let accepted_cases = [
        ("plain-t-one", "=T(\"x\")", text_eval_value("x"), "x"),
        (
            "colliding-t-one",
            "=LET(t,LAMBDA(42),t(\"x\"))",
            text_eval_value("x"),
            "x",
        ),
        (
            "plain-gcd-two",
            "=GCD(48,36)",
            CalcValue::number(12.0),
            "12",
        ),
        (
            "colliding-gcd-two",
            "=LET(gcd,LAMBDA(42),gcd(48,36))",
            CalcValue::number(12.0),
            "12",
        ),
        (
            "plain-gcd-value",
            "=GCD(\"x\",48)",
            CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Value),
            "#VALUE!",
        ),
    ];

    for (case_id, formula, expected_value, expected_text) in accepted_cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:{case_id}"), 1, formula),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .unwrap_or_else(|error| {
                panic!("{case_id} runtime execution should succeed: {error:?}")
            });

        assert_eq!(
            result.execution_outcome_surface.outcome_kind,
            ExecutionOutcomeKind::ExecutedResult,
            "{case_id} outcome kind"
        );
        assert_eq!(
            result.published_worksheet_value, expected_value,
            "{case_id} value"
        );
        assert_eq!(
            result.verification_publication_surface.visible_value_text, expected_text,
            "{case_id} visible text"
        );
    }
}

#[test]
fn runtime_environment_preserves_non_colliding_zero_arg_lambda_thunk_call_ftc_0444_control() {
    assert_runtime_foundation_case(
        "FTC-0444-CONTROL",
        "=LET(THUNK,LAMBDA(x,LAMBDA(x)),tt,THUNK(42),tt())",
        CalcValue::number(42.0),
        "42",
    );
}

#[test]
fn runtime_environment_executes_foundation_array_lambda_carrier_case_ftc_0455() {
    assert_runtime_foundation_case(
        "FTC-0455",
        "=LET(THUNK,LAMBDA(x,LAMBDA(x)),vals,MAP({1;2;3},LAMBDA(v,THUNK(v*10))),INDEX(vals,2,1)())",
        CalcValue::number(20.0),
        "20",
    );
}

#[test]
fn runtime_environment_matches_excel_builtin_colliding_let_recursive_name_frontier_ftc_0443() {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:foundation:FTC-0443",
                1,
                "=LET(gcd,LAMBDA(self,a,b,IF(b=0,a,self(self,b,MOD(a,b)))),gcd(gcd,48,36))",
            ),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("FTC-0443 runtime execution should succeed");

    assert_eq!(
        result.execution_outcome_surface.outcome_kind,
        ExecutionOutcomeKind::ExecutedResult
    );
    assert_eq!(
        result.execution_outcome_surface.outcome_stage,
        ExecutionOutcomeStage::Executed
    );
    assert_eq!(
        result.published_worksheet_value,
        CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "#VALUE!"
    );
    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        "#VALUE!"
    );
}

#[test]
fn runtime_environment_preserves_non_builtin_recursive_self_application() {
    assert_runtime_foundation_case(
        "FTC-0443-NON-BUILTIN",
        "=LET(zzgcd,LAMBDA(self,a,b,IF(b=0,a,self(self,b,MOD(a,b)))),zzgcd(zzgcd,48,36))",
        CalcValue::number(12.0),
        "12",
    );
}

#[test]
fn runtime_environment_preserves_generic_recursive_self_application_baseline() {
    assert_runtime_foundation_case(
        "FTC-0443-BASELINE",
        "=LET(f,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),f(f,3))",
        CalcValue::number(3.0),
        "3",
    );
}

#[test]
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1021() {
    assert_runtime_foundation_text_case(
        "FTC-1021",
        "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,m,15),TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"))",
        text_eval_value("15"),
        "15",
    );
}

#[test]
fn runtime_environment_executes_text_with_pinned_grouping_separator_context() {
    assert_runtime_foundation_text_case(
        "TEXT-PINNED-GROUPING-SEPARATORS",
        "=TEXT(1234567.89,\"#,##0.00\")",
        text_eval_value("1,234,567.89"),
        "1,234,567.89",
    );
}

#[test]
fn runtime_environment_executes_text_with_scientific_format_pattern_ftc_0655() {
    assert_runtime_foundation_text_case(
        "FTC-0655",
        "=TEXT(12345.6789,\"0.00E+00\")",
        text_eval_value("1.23E+04"),
        "1.23E+04",
    );
}

#[test]
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1022() {
    assert_runtime_foundation_text_case(
        "FTC-1022",
        "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
        CalcValue::number(0.0),
        "0",
    );
}

#[test]
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1023() {
    assert_runtime_foundation_text_case(
        "FTC-1023",
        "=LET(baseSun,DATE(2024,1,7),headers,TEXT(baseSun+SEQUENCE(1,7,,1)-1,\"DDD\"),INDEX(headers,1,1))",
        text_eval_value("Sun"),
        "Sun",
    );
}

#[test]
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1024() {
    assert_runtime_foundation_text_case(
        "FTC-1024",
        "=LET(yr,2024,m,2,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(7,,0),dayTexts,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),TEXTJOIN(\",\",FALSE,dayTexts))",
        text_eval_value("  ,  ,  ,  ,01,02,03"),
        "  ,  ,  ,  ,01,02,03",
    );
}

#[test]
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1028() {
    assert_runtime_foundation_text_case(
        "FTC-1028",
        "=TEXT(DATE(2024,7,1),\"MMMM\")",
        text_eval_value("July"),
        "July",
    );
}

#[test]
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1040() {
    assert_runtime_foundation_text_case(
        "FTC-1040",
        "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
        text_eval_value("January|  |01|02|03|04|05|06"),
        "January|  |01|02|03|04|05|06",
    );
}

#[test]
fn runtime_environment_matches_dnaonecalc_exact_request_shape_for_text_date_family() {
    let locale = oxfml_en_us_locale_context();
    let verification_context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: None,
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: Vec::new(),
    };
    let cases = [
        (
            "FTC-1021",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,m,15),TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"))",
            text_eval_value("15"),
            serde_json::json!({"kind": "text", "value": "15"}),
        ),
        (
            "FTC-1022",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
            CalcValue::number(0.0),
            serde_json::json!({"kind": "number", "value": 0.0}),
        ),
        (
            "FTC-1023",
            "=LET(baseSun,DATE(2024,1,7),headers,TEXT(baseSun+SEQUENCE(1,7,,1)-1,\"DDD\"),INDEX(headers,1,1))",
            text_eval_value("Sun"),
            serde_json::json!({"kind": "text", "value": "Sun"}),
        ),
        (
            "FTC-1024",
            "=LET(yr,2024,m,2,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(7,,0),dayTexts,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),TEXTJOIN(\",\",FALSE,dayTexts))",
            text_eval_value("  ,  ,  ,  ,01,02,03"),
            serde_json::json!({"kind": "text", "value": "  ,  ,  ,  ,01,02,03"}),
        ),
        (
            "FTC-1028",
            "=TEXT(DATE(2024,7,1),\"MMMM\")",
            text_eval_value("July"),
            serde_json::json!({"kind": "text", "value": "July"}),
        ),
        (
            "FTC-1040",
            "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
            text_eval_value("January|  |01|02|03|04|05|06"),
            serde_json::json!({"kind": "text", "value": "January|  |01|02|03|04|05|06"}),
        ),
    ];

    for (case_id, formula, expected_value, expected_comparison_value) in cases {
        let result = RuntimeEnvironment::new()
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new(
                        format!("runtime:verification-context:{case_id}"),
                        1,
                        formula,
                    )
                    .with_formula_channel_kind(FormulaChannelKind::WorksheetA1),
                    TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
                )
                .with_verification_publication_context(verification_context.clone()),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{case_id} runtime execution with verification context should succeed: {error}"
                )
            });

        assert_eq!(
            result.source.formula_channel_kind,
            FormulaChannelKind::WorksheetA1,
            "{case_id} formula channel"
        );
        assert_eq!(
            result.published_worksheet_value, expected_value,
            "{case_id} published worksheet value"
        );
        assert_eq!(
            result
                .comparison_views
                .iter()
                .find(|view| view.view_family == "comparison_value")
                .map(|view| view.value.clone()),
            Some(expected_comparison_value),
            "{case_id} comparison_value"
        );
        assert_eq!(
            result
                .verification_publication_surface
                .format_profile
                .as_deref(),
            Some("en-US"),
            "{case_id} format_profile"
        );
        assert!(
            result
                .verification_publication_surface
                .locale_format_context
                .is_some(),
            "{case_id} locale_format_context"
        );
        assert_eq!(
            result.verification_publication_surface.published_value, expected_value,
            "{case_id} verification surface published_value"
        );
        assert_eq!(
            result
                .first_host_replay_capture_packet
                .verification_publication_surface
                .published_value,
            expected_value,
            "{case_id} first-host capture published_value"
        );
        assert_eq!(
            result
                .first_host_replay_capture_packet
                .verification_publication_surface,
            result.verification_publication_surface,
            "{case_id} first-host capture surface"
        );
    }
}

#[test]
fn runtime_environment_canonicalizes_en_us_locale_context_engines_for_text_date_family() {
    let locale = foreign_en_us_context_with_rejecting_formatter();
    let verification_context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: None,
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: Vec::new(),
    };
    let cases = [
        (
            "FTC-1021",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,m,15),TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"))",
            text_eval_value("15"),
        ),
        (
            "FTC-1022",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
            CalcValue::number(0.0),
        ),
        (
            "FTC-1023",
            "=LET(baseSun,DATE(2024,1,7),headers,TEXT(baseSun+SEQUENCE(1,7,,1)-1,\"DDD\"),INDEX(headers,1,1))",
            text_eval_value("Sun"),
        ),
        (
            "FTC-1024",
            "=LET(yr,2024,m,2,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(7,,0),dayTexts,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),TEXTJOIN(\",\",FALSE,dayTexts))",
            text_eval_value("  ,  ,  ,  ,01,02,03"),
        ),
        (
            "FTC-1028",
            "=TEXT(DATE(2024,7,1),\"MMMM\")",
            text_eval_value("July"),
        ),
        (
            "FTC-1040",
            "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
            text_eval_value("January|  |01|02|03|04|05|06"),
        ),
    ];

    for (case_id, formula, expected_value) in cases {
        let result = RuntimeEnvironment::new()
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new(
                        format!("runtime:foreign-locale:{case_id}"),
                        1,
                        formula,
                    )
                    .with_formula_channel_kind(FormulaChannelKind::WorksheetA1),
                    TypedContextQueryBundle::new(
                        None,
                        None,
                        Some(&locale),
                        Some(46000.0),
                        Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
                    ),
                )
                .with_verification_publication_context(verification_context.clone()),
            )
            .unwrap_or_else(|error| {
                panic!("{case_id} runtime execution with foreign locale engines should succeed: {error}")
            });

        assert_eq!(
            result.published_worksheet_value, expected_value,
            "{case_id} published worksheet value"
        );
        assert_eq!(
            result.verification_publication_surface.published_value, expected_value,
            "{case_id} verification surface published_value"
        );
        assert_eq!(
            result
                .first_host_replay_capture_packet
                .verification_publication_surface
                .published_value,
            expected_value,
            "{case_id} first-host capture published_value"
        );
    }
}

#[test]
fn runtime_environment_executes_if_text_true_condition_ftc_0541() {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "runtime:foundation:FTC-0541",
                1,
                "=IF(\"TRUE\",\"yes\",\"no\")",
            ),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("FTC-0541 runtime execution should succeed");

    assert_eq!(result.published_worksheet_value, text_eval_value("yes"));
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "yes"
    );
    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        "yes"
    );
}

#[test]
fn runtime_environment_executes_foundation_helper_array_case_ftc_1031() {
    let locale = oxfml_en_us_locale_context();
    let formula = "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,week1,gridStart+SEQUENCE(1,7,,1)-1,dayNums,MAP(week1,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),DAY(d),0))),SUM(dayNums))";

    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:foundation:FTC-1031", 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("FTC-1031 runtime execution should succeed");

    assert_eq!(result.published_worksheet_value, CalcValue::number(21.0));
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "21"
    );
    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        "21"
    );
}

#[test]
fn runtime_environment_blocks_no_locale_text_verification_cases() {
    let cases = [
        (
            "FTC-1021",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,m,15),TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"))",
        ),
        (
            "FTC-1022",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
        ),
        (
            "FTC-1023",
            "=LET(baseSun,DATE(2024,1,7),headers,TEXT(baseSun+SEQUENCE(1,7,,1)-1,\"DDD\"),INDEX(headers,1,1))",
        ),
        (
            "FTC-1024",
            "=LET(yr,2024,m,2,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(7,,0),dayTexts,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),TEXTJOIN(\",\",FALSE,dayTexts))",
        ),
        ("FTC-1028", "=TEXT(DATE(2024,7,1),\"MMMM\")"),
        (
            "FTC-1040",
            "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
        ),
        // Separator-sensitive TEXT formatting remains blocked without an explicit locale
        // context; OxFml should not silently guess a machine-specific Excel separator profile.
        ("TEXT-SEPARATORS-UNPINNED", "=TEXT(1234567.89,\"#,##0.00\")"),
    ];

    for (case_id, formula) in cases {
        let error = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    format!("runtime:no-locale-foundation:{case_id}"),
                    1,
                    formula,
                ),
                TypedContextQueryBundle::default(),
            ))
            .expect_err("no-locale runtime execution should be blocked");

        assert!(
            error.contains("capability denied"),
            "{case_id} capability denial marker"
        );
        assert!(
            error.contains("locale_format_context"),
            "{case_id} missing locale capability marker"
        );
    }
}

#[test]
fn runtime_environment_applies_registered_external_catalog_mutation() {
    let controller = RecordingCatalogController::default();
    let environment = RuntimeEnvironment::new();
    let request = RegisteredExternalCatalogMutationRequest::Register(
        RegisteredExternalHostRegistrationRequest {
            registration_channel: RegisteredExternalRegistrationChannel::HostApiRegistration,
            register_id_request: sample_register_id_request("User32", "MessageBoxW", Some("JJCCJ")),
            stable_registration_id_hint: Some("REG.messagebox".to_string()),
            display_name_hint: Some("MessageBoxW".to_string()),
            help_text_hint: Some("Displays a modal message box.".to_string()),
            source_project_ref: None,
            source_module_ref: None,
            source_procedure_ref: None,
            host_execution_profile: Some("desktop-trusted".to_string()),
        },
    );

    let result = environment
        .apply_registered_external_catalog_mutation(&controller, &request)
        .expect("runtime mutation should succeed");

    assert_eq!(controller.recorded.borrow().len(), 1);
    match result {
        RegisteredExternalCatalogMutationResult::RegisterApplied {
            descriptor,
            host_execution_profile,
        } => {
            assert_eq!(
                descriptor.origin_kind,
                RegisteredExternalOriginKind::HostRegisteredExternal
            );
            assert_eq!(host_execution_profile.as_deref(), Some("desktop-trusted"));
        }
        other => panic!("unexpected runtime mutation result: {other:?}"),
    }
}

#[test]
fn runtime_session_facade_reports_managed_diagnostics_for_overlay_and_claim_owner() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let locale = oxfml_en_us_locale_context();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:managed-diagnostics", 1, "=INFO(\"directory\")"),
        TypedContextQueryBundle::new(
            Some(&ClaimingHostInfoProvider),
            None,
            Some(&locale),
            None,
            None,
        ),
    );

    let execution = session
        .execute_managed(request)
        .expect("managed execute should succeed");
    let diagnostics = session
        .managed_session_diagnostics()
        .expect("managed diagnostics should exist");

    assert_eq!(diagnostics.phase, RuntimeManagedSessionPhase::Executed);
    assert_eq!(
        diagnostics.active_locus_claim_owner.as_deref(),
        Some(diagnostics.session_id.as_str())
    );
    assert!(!diagnostics.overlay_entries.is_empty());
    assert!(
        diagnostics
            .overlay_entries
            .iter()
            .all(|entry| entry.formula_stable_id == "runtime:managed-diagnostics")
    );
    assert_eq!(
        session
            .managed_session_snapshot()
            .expect("managed snapshot")
            .candidate_result_id,
        Some(execution.candidate_result.candidate_result_id)
    );
}

fn assert_runtime_foundation_case(
    case_id: &str,
    formula: &str,
    expected_value: CalcValue,
    expected_text: &str,
) {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(format!("runtime:foundation:{case_id}"), 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .unwrap_or_else(|error| panic!("{case_id} runtime execution should succeed: {error}"));

    assert_eq!(
        result.published_worksheet_value, expected_value,
        "{case_id} published worksheet value"
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text, expected_text,
        "{case_id} visible value text"
    );
    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        expected_text,
        "{case_id} effective display text"
    );
}

fn assert_runtime_foundation_text_case(
    case_id: &str,
    formula: &str,
    expected_value: CalcValue,
    expected_text: &str,
) {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(format!("runtime:foundation:{case_id}"), 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .unwrap_or_else(|error| panic!("{case_id} runtime execution should succeed: {error}"));

    assert_eq!(
        result.published_worksheet_value, expected_value,
        "{case_id} published worksheet value"
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text, expected_text,
        "{case_id} visible value text"
    );
    assert_eq!(
        result
            .verification_publication_surface
            .effective_display_text,
        expected_text,
        "{case_id} effective display text"
    );
}

fn runtime_snapshot_v1() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "runtime-consumer".to_string(),
        snapshot_version: "v1".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: "SUM".to_string(),
            canonical_id: Some("FUNC.SUM".to_string()),
            surface_stable_id: Some("surface:sum".to_string()),
            name_resolution_table_ref: None,
            semantic_trait_profile_ref: None,
            gating_profile_ref: None,
            metadata_status: Some("runtime_snapshot".to_string()),
            special_interface_kind: None,
            admission_interface_kind: None,
            preparation_owner: None,
            runtime_boundary_kind: None,
            interface_contract_ref: Some("contract:sum".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: None,
        }],
    }
}

struct ForeignLocaleValueParser;
struct RejectingFormatCodeEngine;

static FOREIGN_LOCALE_VALUE_PARSER: ForeignLocaleValueParser = ForeignLocaleValueParser;
static REJECTING_FORMAT_CODE_ENGINE: RejectingFormatCodeEngine = RejectingFormatCodeEngine;

impl LocaleValueParser for ForeignLocaleValueParser {
    fn parse_value_text(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        text: &str,
    ) -> Result<f64, ParseFailure> {
        oxfml_core::format::parse_value_text(profile, date_system, text)
    }
}

impl FormatCodeEngine for RejectingFormatCodeEngine {
    fn render_with_code(
        &self,
        _profile: &FormatProfile,
        _date_system: WorkbookDateSystem,
        _value: f64,
        code: &str,
    ) -> Result<ExcelText, FormatFailure> {
        Err(FormatFailure::UnsupportedCode(code.to_string()))
    }

    fn render_currency(
        &self,
        _profile: &FormatProfile,
        _value: f64,
        _decimals: i32,
    ) -> Result<ExcelText, FormatFailure> {
        Err(FormatFailure::UnsupportedCode("currency".to_string()))
    }

    fn render_fixed(
        &self,
        _profile: &FormatProfile,
        _value: f64,
        _decimals: i32,
        _no_commas: bool,
    ) -> Result<ExcelText, FormatFailure> {
        Err(FormatFailure::UnsupportedCode("fixed".to_string()))
    }
}

fn foreign_en_us_context_with_rejecting_formatter() -> LocaleFormatContext<'static> {
    LocaleFormatContext {
        profile: oxfml_en_us_format_profile(),
        date_system: WorkbookDateSystem::System1900,
        parser: &FOREIGN_LOCALE_VALUE_PARSER,
        formatter: &REJECTING_FORMAT_CODE_ENGINE,
    }
}

fn expected_programmatic_comparison_value(value: &CalcValue) -> Value {
    match value.core() {
        CoreValue::Number(number) => serde_json::json!({
            "kind": "number",
            "value": number
        }),
        CoreValue::Error(code) => serde_json::json!({
            "kind": "error",
            "code": format!("{code:?}"),
            "display": worksheet_error_text(*code)
        }),
        other => panic!("unexpected programmatic verification value shape: {other:?}"),
    }
}

fn text_eval_value(text: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_interop_assignment(text))
}

struct ValueRtdProvider;

impl RtdProvider for ValueRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        RtdProviderResult::Value(CalcValue::number(7.0))
    }
}

struct ClaimingHostInfoProvider;

impl HostInfoProvider for ClaimingHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&oxfunc_core::value::ReferenceLike>,
    ) -> Result<CalcValue, HostInfoError> {
        Err(HostInfoError::UnsupportedCellInfoQuery(query))
    }

    fn query_info(&self, query: InfoQuery) -> Result<CalcValue, HostInfoError> {
        match query {
            InfoQuery::Directory => Ok(CalcValue::text(ExcelText::from_interop_assignment(
                "C:\\Work",
            ))),
            _ => Err(HostInfoError::UnsupportedInfoQuery(query)),
        }
    }
}

#[derive(Default)]
struct RecordingRegisteredExternalProvider {
    last_lookup: RefCell<Option<f64>>,
}

impl RegisteredExternalProvider for RecordingRegisteredExternalProvider {
    fn resolve_register_id(
        &self,
        request: &RegisterIdRequest,
    ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        Ok(RegisteredExternalDescriptor {
            stable_registration_id: format!(
                "REG.{}",
                request.library_name.to_string_lossy().to_ascii_lowercase()
            ),
            register_id: 4242.0,
            origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
            display_name: Some(ExcelText::from_interop_assignment(
                &request_procedure_display_name(request),
            )),
            library_name: request.library_name.clone(),
            procedure: request.procedure.clone(),
            declared_type_text: request.declared_type_text.clone(),
        })
    }

    fn lookup_registered_external(
        &self,
        register_id: f64,
    ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        self.last_lookup.replace(Some(register_id));
        Ok(RegisteredExternalDescriptor {
            stable_registration_id: "REG.by-id".to_string(),
            register_id,
            origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
            display_name: Some(ExcelText::from_interop_assignment("LookupById")),
            library_name: ExcelText::from_interop_assignment("Kernel32"),
            procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment("MulDiv")),
            declared_type_text: Some(ExcelText::from_interop_assignment("JJJJ")),
        })
    }

    fn invoke_registered_external(
        &self,
        descriptor: &RegisteredExternalDescriptor,
        args: &[CalcValue],
    ) -> Result<CalcValue, RegisteredExternalProviderError> {
        match &descriptor.procedure {
            RegisteredProcedureSpec::Name(name) if name.to_string_lossy() == "MulDiv" => match args
            {
                [a, b, c] => match (a.core(), b.core(), c.core()) {
                    (CoreValue::Number(a), CoreValue::Number(b), CoreValue::Number(c)) => {
                        Ok(CalcValue::number((a * b) / c))
                    }
                    _ => Err(RegisteredExternalProviderError::WorksheetError(
                        oxfunc_core::value::WorksheetErrorCode::Value,
                    )),
                },
                _ => Err(RegisteredExternalProviderError::WorksheetError(
                    oxfunc_core::value::WorksheetErrorCode::Value,
                )),
            },
            _ => Ok(CalcValue::number(descriptor.register_id)),
        }
    }
}

#[derive(Default)]
struct RecordingHostFunctionProvider {
    last_invocation: RefCell<Option<HostFunctionInvocation>>,
}

impl HostFunctionProvider for RecordingHostFunctionProvider {
    fn invoke_host_function(
        &self,
        invocation: &HostFunctionInvocation,
    ) -> Result<CalcValue, HostFunctionProviderError> {
        self.last_invocation.replace(Some(invocation.clone()));
        match invocation.args.as_slice() {
            [a, b] if invocation.function_name.eq_ignore_ascii_case("AddThem") => {
                let Some(a) = a.as_number() else {
                    return Err(HostFunctionProviderError::new("unsupported host function"));
                };
                let Some(b) = b.as_number() else {
                    return Err(HostFunctionProviderError::new("unsupported host function"));
                };
                Ok(CalcValue::number(a + b))
            }
            _ => Err(HostFunctionProviderError::new("unsupported host function")),
        }
    }
}

#[derive(Default)]
struct RecordingCatalogController {
    recorded: RefCell<Vec<RegisteredExternalCatalogMutationRequest>>,
}

impl RegisteredExternalCatalogController for RecordingCatalogController {
    fn apply_mutation(
        &self,
        request: &RegisteredExternalCatalogMutationRequest,
    ) -> Result<RegisteredExternalCatalogMutationResult, RegisteredExternalProviderError> {
        self.recorded.borrow_mut().push(request.clone());
        match request {
            RegisteredExternalCatalogMutationRequest::Register(register) => {
                Ok(RegisteredExternalCatalogMutationResult::RegisterApplied {
                    descriptor: RegisteredExternalDescriptor {
                        stable_registration_id: register
                            .stable_registration_id_hint
                            .clone()
                            .unwrap_or_else(|| "REG.synthetic".to_string()),
                        register_id: 5000.0,
                        origin_kind: RegisteredExternalOriginKind::HostRegisteredExternal,
                        display_name: register
                            .display_name_hint
                            .as_ref()
                            .map(|text| ExcelText::from_interop_assignment(text)),
                        library_name: register.register_id_request.library_name.clone(),
                        procedure: register.register_id_request.procedure.clone(),
                        declared_type_text: register.register_id_request.declared_type_text.clone(),
                    },
                    host_execution_profile: register.host_execution_profile.clone(),
                })
            }
            RegisteredExternalCatalogMutationRequest::Unregister(unregister) => {
                Ok(RegisteredExternalCatalogMutationResult::UnregisterApplied {
                    stable_registration_id: unregister.stable_registration_id.clone(),
                    host_execution_profile: unregister.host_execution_profile.clone(),
                })
            }
        }
    }
}

fn request_procedure_display_name(request: &RegisterIdRequest) -> String {
    match &request.procedure {
        RegisteredProcedureSpec::Name(name) => name.to_string_lossy(),
        RegisteredProcedureSpec::Ordinal(ordinal) => ordinal.to_string(),
    }
}

fn sample_register_id_request(
    library_name: &str,
    procedure_name: &str,
    type_text: Option<&str>,
) -> RegisterIdRequest {
    RegisterIdRequest {
        library_name: ExcelText::from_interop_assignment(library_name),
        procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment(
            procedure_name,
        )),
        declared_type_text: type_text.map(ExcelText::from_interop_assignment),
    }
}

fn runtime_snapshot_v2() -> LibraryContextSnapshot {
    let mut snapshot = runtime_snapshot_v1();
    snapshot.snapshot_version = "v2".to_string();
    snapshot.entries.push(LibraryContextSnapshotEntry {
        surface_name: "TAKE".to_string(),
        canonical_id: Some("FUNC.TAKE".to_string()),
        surface_stable_id: Some("surface:take".to_string()),
        name_resolution_table_ref: None,
        semantic_trait_profile_ref: None,
        gating_profile_ref: None,
        metadata_status: Some("runtime_snapshot".to_string()),
        special_interface_kind: None,
        admission_interface_kind: None,
        preparation_owner: None,
        runtime_boundary_kind: None,
        interface_contract_ref: Some("contract:take".to_string()),
        registration_source_kind: RegistrationSourceKind::BuiltIn,
        parse_bind_state: LibraryAvailabilityState::CatalogKnown,
        semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
        runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
        post_dispatch_state: None,
    });
    snapshot
}

fn vba_udf_snapshot() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "runtime-vba-udf".to_string(),
        snapshot_version: "v1".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: "AddThem".to_string(),
            canonical_id: Some("FUNC.VBA.ADDTHEM".to_string()),
            surface_stable_id: Some("vba:project:main:addthem".to_string()),
            name_resolution_table_ref: Some("vba:project:main".to_string()),
            semantic_trait_profile_ref: Some("vba-udf-double.v1".to_string()),
            gating_profile_ref: None,
            metadata_status: Some("host_registered".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("excel_observed_double_first_slice".to_string()),
            preparation_owner: Some("OxFml".to_string()),
            runtime_boundary_kind: Some("vba_host_callback".to_string()),
            interface_contract_ref: Some("dnaonecalc-vba-udf-first-slice".to_string()),
            registration_source_kind: RegistrationSourceKind::Vba,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: None,
        }],
    }
}

fn runtime_test_callable_binding() -> CallableDefinedNameBinding {
    CallableDefinedNameBinding {
        summary: "arity=1;params=x;captures=-;body=Binary".to_string(),
        carrier: CallableValueCarrier {
            origin_kind: CallableOriginKind::DefinedNameCallable,
            invocation_model: CallableInvocationModel::TypedInvocationOnly,
            capture_mode: CallableCaptureMode::NoCapture,
            arity: 1,
        },
        profile: CallableValueProfile {
            arity: 1,
            required_arity: 1,
            parameter_names: vec!["x".to_string()],
            optional_parameter_names: Vec::new(),
            capture_names: Vec::new(),
            body_kind: "Binary".to_string(),
        },
        params: vec!["x".to_string()],
        optional_parameter_names: Vec::new(),
        body: BoundExpr::Binary {
            op: BinaryOp::Add,
            left: Box::new(runtime_name_ref_expr("x", NameKind::HelperLocal)),
            right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
        },
        closure: BTreeMap::new(),
    }
}

fn runtime_name_ref_expr(name: &str, kind: NameKind) -> BoundExpr {
    BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(NameRef {
        name: name.to_string(),
        workbook_id: "book:default".to_string(),
        sheet_id: "sheet:default".to_string(),
        kind,
        caller_context_dependent: false,
    })))
}

fn runtime_test_udf_entry() -> FunctionEntry {
    FunctionEntry {
        meta: RegistryFunctionMeta {
            function_id: "FUNC.UDF.MYFUNC".to_string(),
            function_spec_axes_metadata:
                oxfunc_core::registry::FunctionSpecAxesMetadata::default_axes(),
            function_spec_axes_metadata_version:
                oxfunc_core::registry::FunctionSpecAxesMetadata::default_axes().version_key(),
            arity: Arity::exact(2),
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::None,
            semantic_kernel_metadata: SemanticKernelMetadata {
                reduction_sensitive: false,
                error_collapse_sensitive: false,
                numerical_reduction_policy: None,
                error_algebra: None,
            },
            semantic_kernel_metadata_version:
                "semantic_kernel_metadata.v1;reduction_sensitive=false;error_collapse_sensitive=false;numerical_reduction_policy=none;error_algebra=none"
                    .to_string(),
            arg_admission_metadata: ArgAdmissionMetadata::ExistingArgPreparation {
                profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            },
            arg_admission_metadata_version:
                "arg_admission_metadata.v1;existing_arg_preparation=values_only_pre_adapter"
                    .to_string(),
            rich_value_usage: RichValueUsage::RichBlind,
            producer_capability_set_keys: Vec::new(),
        },
        surface_name: "MYFUNC".to_string(),
        display_signature: SignatureForm {
            signature_display: "MYFUNC(value, label)".to_string(),
            parameters: vec![
                ParameterDescriptor {
                    name: "value".to_string(),
                    optional: false,
                    repeats: false,
                    short_description: None,
                },
                ParameterDescriptor {
                    name: "label".to_string(),
                    optional: false,
                    repeats: false,
                    short_description: None,
                },
            ],
            trailing_repeats: false,
        },
        registry_metadata: FunctionRegistryMetadata::default(),
        short_description: None,
        long_description: None,
        source: FunctionSource::Udf {
            provenance: Some("runtime_consumer_facade_tests".to_string()),
            replaces_builtin: false,
        },
    }
}
