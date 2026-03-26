use oxfml_core::interface::{
    HostProviderOutcomeKind, InMemoryLibraryContextProvider, LibraryContextSnapshotRef,
    ReturnedValueSurfaceKind, TypedContextQueryBundle, TypedContextQueryFamily,
};
use oxfml_core::oxfunc_adapter::{
    OxFuncAdapterRequest, OxFuncMismatchOwnerGuess, run_oxfunc_preparation_adapter,
};
use oxfml_core::seam::Locus;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::{PreparedSourceClass, PreparedStructureClass};
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::value::EvalValue;

#[test]
fn adapter_projects_direct_scalar_and_array_like_preparation_artifacts() {
    let scalar_request = OxFuncAdapterRequest::new(
        "sum-direct-scalars",
        "formula:sum-direct-scalars",
        "=SUM(1,2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    );
    let scalar_run = run_oxfunc_preparation_adapter(scalar_request).expect("scalar adapter run");

    assert_eq!(
        scalar_run.preparation_artifact.prepared_calls[0].function_name,
        "SUM"
    );
    assert_eq!(
        scalar_run.preparation_artifact.prepared_calls[0]
            .prepared_arguments
            .iter()
            .map(|arg| arg.structure_class)
            .collect::<Vec<_>>(),
        vec![
            PreparedStructureClass::DirectScalar,
            PreparedStructureClass::DirectScalar,
        ]
    );
    assert_eq!(
        scalar_run
            .evaluation_artifact
            .evaluation_result
            .payload_summary,
        "Number(3)"
    );

    let mut array_like_request = OxFuncAdapterRequest::new(
        "sum-area-argument",
        "formula:sum-area-argument",
        "=SUM(A1:A2)",
        locus(1, 2),
        TypedContextQueryBundle::default(),
    );
    array_like_request
        .cell_fixture
        .insert("A1".to_string(), EvalValue::Number(10.0));
    array_like_request
        .cell_fixture
        .insert("A2".to_string(), EvalValue::Number(20.0));
    let array_like_run =
        run_oxfunc_preparation_adapter(array_like_request).expect("array-like adapter run");

    let prepared_argument =
        &array_like_run.preparation_artifact.prepared_calls[0].prepared_arguments[0];
    assert_eq!(
        prepared_argument.structure_class,
        PreparedStructureClass::ArrayLike
    );
    assert_eq!(
        prepared_argument.source_class,
        PreparedSourceClass::AreaReference
    );
    assert_eq!(
        array_like_run
            .evaluation_artifact
            .evaluation_result
            .payload_summary,
        "Number(30)"
    );
}

#[test]
fn adapter_respects_requested_snapshot_and_caller_anchor() {
    let current_snapshot = test_snapshot("snapshot:current", "v1", "SUM");
    let selected_snapshot = test_snapshot("snapshot:selected", "v2", "ROW");
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        LibraryContextSnapshotRef::from(&current_snapshot),
        vec![current_snapshot, selected_snapshot.clone()],
    );

    let mut request = OxFuncAdapterRequest::new(
        "row-caller-anchor",
        "formula:row-caller-anchor",
        "=ROW()",
        locus(7, 3),
        TypedContextQueryBundle::default(),
    );
    request.library_context_provider = Some(&provider);
    request.library_context_snapshot_ref =
        Some(LibraryContextSnapshotRef::from(&selected_snapshot));
    request.host_query_capability_profile = Some("caller-row-only".to_string());

    let run = run_oxfunc_preparation_adapter(request).expect("row adapter run");

    assert_eq!(
        run.preparation_artifact.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::from(&selected_snapshot))
    );
    assert_eq!(run.preparation_artifact.caller_anchor.row, 7);
    assert_eq!(run.preparation_artifact.caller_anchor.col, 3);
    assert_eq!(
        run.preparation_artifact
            .host_query_capability_profile
            .as_deref(),
        Some("caller-row-only")
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(7.0)
    );
}

#[test]
fn adapter_surfaces_typed_rtd_outcome_and_bundle_spec() {
    let provider = InMemoryLibraryContextProvider::new(test_snapshot("snapshot:rtd", "v1", "RTD"));
    let mut request = OxFuncAdapterRequest::new(
        "rtd-capability-denied",
        "formula:rtd-capability-denied",
        "=RTD(\"prog\",\"server\",\"topic\")",
        locus(1, 1),
        TypedContextQueryBundle::new(None, Some(&CapabilityDeniedRtdProvider), None, None, None),
    );
    request.library_context_provider = Some(&provider);

    let run = run_oxfunc_preparation_adapter(request).expect("rtd adapter run");

    assert!(
        run.preparation_artifact
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::Rtd)
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .map(|surface| surface.outcome_kind),
        Some(HostProviderOutcomeKind::CapabilityDenied)
    );
    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Error(Blocked)"
    );
}

#[test]
fn adapter_builds_structured_mismatch_artifact() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "mismatch-seed",
        "formula:mismatch-seed",
        "=SUM(1,2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("mismatch seed run");

    let mismatch = run.mismatch_artifact(
        "prepared_argument_family",
        "prepared_calls",
        OxFuncMismatchOwnerGuess::SharedFreezeNeeded,
        "expected DirectScalar/ArrayLike split",
        "observed narrower prepared-call field family",
        Some("narrow field name mismatch only".to_string()),
    );

    assert_eq!(mismatch.fixture_case_id, "mismatch-seed");
    assert_eq!(mismatch.expected_seam_family, "prepared_argument_family");
    assert_eq!(mismatch.failing_packet_family, "prepared_calls");
    assert_eq!(
        mismatch.owner_guess,
        OxFuncMismatchOwnerGuess::SharedFreezeNeeded
    );
    assert_eq!(
        mismatch.detail.as_deref(),
        Some("narrow field name mismatch only")
    );
}

#[test]
fn adapter_preserves_internal_lambda_but_publishes_calc_for_bare_lambda() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "lambda-publication",
        "formula:lambda-publication",
        "=LAMBDA(x,x+1)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("lambda publication adapter run");

    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Lambda(arity=1;params=x;captures=-;body=Binary)"
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Calc)
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::OrdinaryValue
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .payload_summary,
        "Error(Calc)"
    );
}

#[test]
fn adapter_rejects_duplicate_let_binding_names_as_bind_mismatch() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "duplicate-let-binding",
        "formula:duplicate-let-binding",
        "=LET(x,1,x,2,x)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("duplicate let adapter run");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(oxfml_core::RejectCode::BindMismatch)
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
    assert!(
        run.preparation_artifact
            .bind_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "duplicate LET binding name 'x'")
    );
}

fn test_snapshot(
    snapshot_id: &str,
    snapshot_version: &str,
    surface_name: &str,
) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: snapshot_id.to_string(),
        snapshot_version: snapshot_version.to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: surface_name.to_string(),
            canonical_id: Some(format!("FUNC.{surface_name}")),
            surface_stable_id: Some(format!("surface:{surface_name}")),
            name_resolution_table_ref: Some("name-table:v1".to_string()),
            semantic_trait_profile_ref: Some("traits:v1".to_string()),
            gating_profile_ref: Some("gating:v1".to_string()),
            metadata_status: Some("runtime".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("ordinary".to_string()),
            preparation_owner: Some("oxfunc".to_string()),
            runtime_boundary_kind: Some("ordinary_eval".to_string()),
            arity_shape_note: None,
            interface_contract_ref: Some("iface:v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

struct CapabilityDeniedRtdProvider;

impl RtdProvider for CapabilityDeniedRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        RtdProviderResult::CapabilityDenied
    }
}
