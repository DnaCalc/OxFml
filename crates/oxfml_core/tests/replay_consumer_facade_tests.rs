use oxfml_core::binding::BindContext;
use oxfml_core::consumer::editor::{
    EditorAnalysisStage, EditorEditService, EditorEnvironment, EditorPlanOptions,
};
use oxfml_core::consumer::replay::{
    ReplayFirstHostCaptureSource, ReplayFixtureFamilySource, ReplayProjectionRequest,
    ReplayProjectionService, ReplayRetainedWitnessSource,
};
use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeSessionFacade,
};
use oxfml_core::interface::{
    InMemoryLibraryContextProvider, LibraryContextProvider, LibraryContextSnapshotRef,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::FormulaSourceRecord;
use oxfml_core::{FormulaChannelKind, TypedContextQueryBundle};

#[test]
fn editor_edit_service_applies_completion_proposal_through_facade() {
    let snapshot = editor_snapshot();
    let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        snapshot_ref.clone(),
        vec![snapshot.clone()],
    );
    let environment = EditorEnvironment::new(BindContext::default())
        .with_pinned_library_context(&provider, snapshot_ref.clone())
        .with_inline_library_context_snapshot(snapshot);
    let service = EditorEditService::new(environment);
    let source = FormulaSourceRecord::new("editor:apply-completion", 1, "=")
        .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);
    let applied = service.apply_edit(
        source,
        None,
        EditorAnalysisStage::FullSemanticPlan,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: provider.snapshot_by_identity(&snapshot_ref),
        }),
    );
    let interaction = service.interact_at_cursor(&applied.document, 1);
    let proposal = interaction
        .completion_result
        .as_ref()
        .and_then(|result| result.proposals.iter().next())
        .expect("completion proposal should exist")
        .clone();

    let applied_completion = service.apply_completion_proposal(
        &applied.document,
        &proposal,
        EditorAnalysisStage::FullSemanticPlan,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: provider.snapshot_by_identity(&snapshot_ref),
        }),
    );

    assert_eq!(
        applied_completion.proposal_id.as_deref(),
        Some(proposal.proposal_id.as_str())
    );
    assert!(
        applied_completion
            .interaction_result
            .document
            .source
            .entered_formula_text
            .contains(&proposal.insert_text)
    );
}

#[test]
fn editor_edit_service_validates_manual_completion_text_through_facade() {
    let environment = EditorEnvironment::new(BindContext::default());
    let service = EditorEditService::new(environment);
    let source = FormulaSourceRecord::new("editor:validate-completion", 1, "=");
    let applied = service.apply_edit(
        source,
        None,
        EditorAnalysisStage::FullSemanticPlan,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: None,
        }),
    );

    let validated = service.validate_completion(
        &applied.document,
        None,
        "SUM(",
        EditorAnalysisStage::FullSemanticPlan,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: None,
        }),
    );

    assert_eq!(
        validated
            .interaction_result
            .document
            .source
            .entered_formula_text,
        "=SUM("
    );
    assert!(validated.interaction_result.function_help_packet.is_some());
}

#[test]
fn replay_projection_service_projects_runtime_and_host_outputs() {
    let environment = RuntimeEnvironment::new();
    let runtime_result = environment
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:runtime", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime result should execute");

    let runtime_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_result(&runtime_result)
            .with_source_case_id("case:runtime")
            .with_shared_scenario_alias("alias.runtime"),
    );

    assert_eq!(
        runtime_projection.source_artifact_family,
        "runtime_formula_result"
    );
    assert_eq!(
        runtime_projection.source_case_id.as_deref(),
        Some("case:runtime")
    );
    assert_eq!(
        runtime_projection.shared_scenario_alias.as_deref(),
        Some("alias.runtime")
    );
    assert!(
        runtime_projection
            .first_host_replay_capture_packet
            .is_some()
    );

    let first_host_capture = ReplayFirstHostCaptureSource {
        source_artifact_family: "first_host_capture_packet".to_string(),
        session_id: runtime_result.candidate_result.session_id.clone(),
        packet: runtime_result.first_host_replay_capture_packet.clone(),
    };
    let host_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::first_host_capture(&first_host_capture),
    );

    assert_eq!(
        host_projection.source_artifact_family,
        "first_host_capture_packet"
    );
    assert_eq!(
        host_projection
            .first_host_replay_capture_packet
            .as_ref()
            .map(|packet| packet.formula_stable_id.as_str()),
        Some("replay:runtime")
    );
}

#[test]
fn replay_projection_service_projects_runtime_managed_session_results() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("replay:session", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");
    let open_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_open(&open)
            .with_source_case_id("case:session-open")
            .with_shared_scenario_alias("alias.session.open"),
    );
    assert_eq!(
        open_projection.source_artifact_family,
        "runtime_managed_open"
    );
    assert_eq!(open_projection.phase.as_deref(), Some("Open"));

    let execution = session
        .execute_managed(request)
        .expect("managed execute should succeed");
    let execution_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_execution(&execution)
            .with_source_case_id("case:session")
            .with_shared_scenario_alias("alias.session"),
    );

    assert_eq!(
        execution_projection.source_artifact_family,
        "runtime_managed_execution"
    );
    assert_eq!(execution_projection.phase.as_deref(), Some("Executed"));
    assert_eq!(
        execution_projection.formula_stable_id,
        "replay:session".to_string()
    );
    assert_eq!(
        execution_projection.source_case_id.as_deref(),
        Some("case:session")
    );
    assert_eq!(
        execution_projection.shared_scenario_alias.as_deref(),
        Some("alias.session")
    );
    assert!(execution_projection.candidate_result_id.is_some());
    assert!(execution_projection.library_context_snapshot_ref.is_none());

    let session_snapshot = session
        .managed_session_snapshot()
        .expect("managed session snapshot should exist");
    let session_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_session(&session_snapshot),
    );
    assert_eq!(
        session_projection.source_artifact_family,
        "runtime_managed_session"
    );
    assert_eq!(session_projection.phase.as_deref(), Some("Executed"));
    assert!(session_projection.candidate_result_id.is_some());

    let commit = session
        .commit_managed(
            "commit:replay-session",
            execution.candidate_result.fence_snapshot.clone(),
        )
        .expect("managed commit should succeed");
    let commit_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_commit(&commit)
            .with_source_case_id("case:session-commit")
            .with_shared_scenario_alias("alias.session.commit"),
    );
    assert_eq!(
        commit_projection.source_artifact_family,
        "runtime_managed_commit"
    );
    assert_eq!(commit_projection.phase.as_deref(), Some("Committed"));
    assert_eq!(
        commit_projection.commit_decision_kind.as_deref(),
        Some("accepted")
    );
    assert_eq!(
        commit_projection.source_case_id.as_deref(),
        Some("case:session-commit")
    );
}

#[test]
fn replay_projection_service_projects_runtime_managed_termination_results() {
    let environment = RuntimeEnvironment::new();
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("replay:session-abort", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let _open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");
    let termination = session
        .abort_managed(Some("test_abort".to_string()))
        .expect("managed abort should succeed");
    let termination_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_termination(&termination)
            .with_source_case_id("case:session-abort")
            .with_shared_scenario_alias("alias.session.abort"),
    );

    assert_eq!(
        termination_projection.source_artifact_family,
        "runtime_managed_termination"
    );
    assert_eq!(termination_projection.phase.as_deref(), Some("Aborted"));
    assert_eq!(
        termination_projection.commit_decision_kind.as_deref(),
        Some("rejected")
    );
}

#[test]
fn replay_projection_service_projects_fixture_family_metadata() {
    let source = ReplayFixtureFamilySource {
        source_schema_id: "oxfml.local.source_schema.session_lifecycle_replay_cases.v1".to_string(),
        source_fixture_family: "session_lifecycle_replay_cases".to_string(),
        source_case_ids: vec![
            "session_capability_denied".to_string(),
            "session_execute_commit".to_string(),
        ],
        registry_pin: Some(
            "oxfml.local.registry_pin.foundation_handoff_20260315_pass01".to_string(),
        ),
    };

    let projection = ReplayProjectionService::project(
        ReplayProjectionRequest::fixture_family(&source)
            .with_source_case_id("session_capability_denied")
            .with_shared_scenario_alias("alias.session.capability_denied"),
    );

    assert_eq!(projection.source_artifact_family, "fixture_family");
    assert_eq!(
        projection.source_fixture_family.as_deref(),
        Some("session_lifecycle_replay_cases")
    );
    assert_eq!(
        projection.source_schema_id.as_deref(),
        Some("oxfml.local.source_schema.session_lifecycle_replay_cases.v1")
    );
    assert_eq!(projection.source_case_ids.len(), 2);
    assert_eq!(
        projection.registry_pin.as_deref(),
        Some("oxfml.local.registry_pin.foundation_handoff_20260315_pass01")
    );
}

#[test]
fn replay_projection_service_projects_retained_witness_metadata() {
    let source = ReplayRetainedWitnessSource {
        witness_id: "oxfml.local.witness.session_capability_denied.v1".to_string(),
        source_fixture_family: "session_lifecycle_replay_cases".to_string(),
        source_case_ids: vec!["session_capability_denied".to_string()],
        witness_lifecycle_state: "wit.retained_local".to_string(),
        retention_policy_id: Some("retain.local.replay_valid".to_string()),
        registry_pin: Some("oxfml.local.registry_pin.foundation_handoff_20260315_pass01".to_string()),
        source_bundle_ref: Some("crates/oxfml_core/tests/fixtures/session_lifecycle_replay_cases.json".to_string()),
        reduction_manifest_ref: Some(
            "crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_reduction_manifest.json".to_string(),
        ),
    };

    let projection = ReplayProjectionService::project(
        ReplayProjectionRequest::retained_witness(&source)
            .with_source_case_id("session_capability_denied")
            .with_shared_scenario_alias("alias.session.capability_denied"),
    );

    assert_eq!(projection.source_artifact_family, "retained_witness");
    assert_eq!(
        projection.witness_id.as_deref(),
        Some("oxfml.local.witness.session_capability_denied.v1")
    );
    assert_eq!(
        projection.witness_lifecycle_state.as_deref(),
        Some("wit.retained_local")
    );
    assert_eq!(
        projection.source_fixture_family.as_deref(),
        Some("session_lifecycle_replay_cases")
    );
    assert_eq!(
        projection.reduction_manifest_ref.as_deref(),
        Some(
            "crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_reduction_manifest.json"
        )
    );
}

fn editor_snapshot() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "replay-editor".to_string(),
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
