use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeManagedSessionError,
    RuntimeManagedSessionPhase, RuntimeSessionFacade,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::{
    AcceptDecision, FormulaSourceRecord, InMemoryLibraryContextProvider, LibraryContextSnapshotRef,
    TypedContextQueryBundle,
};
use oxfunc_core::value::EvalValue;

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
    assert_eq!(take_summary.metadata_status, None);
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
        oxfml_core::DefinedNameBinding::Value(EvalValue::Number(5.0)),
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
        snapshot.candidate_result_id,
        Some(execution.candidate_result.candidate_result_id)
    );
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

    assert_eq!(result.published_worksheet_value, EvalValue::Number(8.0));
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
            arity_shape_note: None,
            interface_contract_ref: Some("contract:sum".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: None,
        }],
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
        arity_shape_note: None,
        interface_contract_ref: Some("contract:take".to_string()),
        registration_source_kind: RegistrationSourceKind::BuiltIn,
        parse_bind_state: LibraryAvailabilityState::CatalogKnown,
        semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
        runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
        post_dispatch_state: None,
    });
    snapshot
}
