use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use oxfml_core::binding::{BindContext, BindRequest, NameKind, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::scheduler::{
    ExecutionRestriction, ReplaySensitivityClass, SchedulerLaneClass, build_execution_contract,
};
use oxfml_core::seam::{
    AcceptDecision, AcceptedCandidateResult, CommitRequest, Extent, FenceSnapshot, Locus,
    ShapeDelta, ShapeOutcomeClass, SpillEvent, SpillEventKind, TopologyDelta, ValueDelta,
    ValuePayload, WorksheetValueClass, commit_candidate,
};
use oxfml_core::session::{CapabilityViewSpec, PrepareRequest, SessionService};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{CompileSemanticPlanRequest, compile_semantic_plan};

#[derive(Debug, Deserialize)]
struct AdapterCapabilityManifest {
    adapter_id: String,
    adapter_version: String,
    lane_id: String,
    supported_source_schema_ids: Vec<String>,
    supported_replay_bundle_schema_versions: Vec<String>,
    claimed_capability_levels: Vec<String>,
    scaffolded_capability_levels: Vec<String>,
    known_limits: Vec<String>,
    conformance_artifact_refs: Vec<String>,
    registry_version_refs: Vec<RegistryVersionRef>,
    rollout_notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersionRef {
    registry_family: String,
    registry_version: String,
}

#[derive(Debug, Deserialize)]
struct ReductionManifest {
    reduction_id: String,
    source_bundle_ref: String,
    source_scope_ref: String,
    predicate_ref: PredicateRef,
    strategy_id: String,
    unit_kinds: Vec<String>,
    retained_units: Vec<String>,
    removed_units: Vec<String>,
    rewritten_units: Vec<String>,
    closure_rules_applied: Vec<String>,
    iteration_count: u64,
    final_status: String,
    witness_bundle_ref: String,
}

#[derive(Debug, Deserialize)]
struct PredicateRef {
    predicate_id: String,
    predicate_kind: String,
    required_reject_code: Option<String>,
    required_mismatch_member_kind: Option<String>,
    required_publication_value_class: Option<String>,
    required_execution_restriction: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WitnessBundle<T> {
    witness_id: String,
    source_fixture_family: String,
    reduced_role: String,
    bundle_state: String,
    source_case_ids: Vec<String>,
    scenario_cases: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct WitnessLifecycleRecord {
    witness_id: String,
    source_bundle_ref: String,
    reduction_manifest_ref: String,
    lifecycle_state: String,
    retention_policy_id: String,
    promotion_refs: Vec<String>,
    supersedes: Vec<String>,
    quarantine_reason: Option<String>,
    gc_eligibility: String,
    notes: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct FecCommitReplayFixture {
    case_id: String,
    observed_fence: FenceFixture,
    expected: FecCommitExpected,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct FenceFixture {
    formula_token: String,
    snapshot_epoch: String,
    bind_hash: String,
    profile_version: String,
    capability_view_key: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct FecCommitExpected {
    decision: String,
    published_payload: Option<String>,
    shape_outcome_class: Option<String>,
    spill_event_kind: Option<String>,
    reject_code: Option<String>,
    mismatch_member_kind: Option<String>,
    mismatch_class: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct SessionReplayFixture {
    case_id: String,
    formula: String,
    with_input_name: bool,
    capability_spec: CapabilitySpecFixture,
    action: String,
    expected: SessionReplayExpected,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct CapabilitySpecFixture {
    host_query_enabled: bool,
    locale_format_enabled: bool,
    caller_context_enabled: bool,
    external_provider_enabled: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct SessionReplayExpected {
    phase: String,
    decision: String,
    reject_code: Option<String>,
    published_payload: Option<String>,
    trace_event_kinds: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct ExecutionContractReplayFixture {
    case_id: String,
    formula: String,
    expected: ExecutionContractExpected,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct ExecutionContractExpected {
    lane_class: String,
    replay_sensitivity: String,
    single_flight_advisable: bool,
    restrictions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct NormalizedPackCandidateBundle {
    bundle_id: String,
    bundle_schema_version: String,
    lane_id: String,
    adapter_id: String,
    source_schema_id: String,
    source_fixture_family: String,
    bundle_role: String,
    pack_eligibility_state: String,
    required_capability_level: String,
    registry_pin: String,
    authorized_transform_family: String,
    rewrite_authorizations: Vec<String>,
    witness_lifecycle_refs: Vec<String>,
    normalized_event_families: Vec<String>,
    source_artifact_refs: Vec<String>,
    promotion_blockers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackCandidateIndex {
    index_id: String,
    bundle_schema_version: String,
    lane_id: String,
    adapter_id: String,
    index_state: String,
    pack_eligibility_state: String,
    required_capability_level: String,
    bundle_refs: Vec<String>,
    promotion_blockers: Vec<String>,
    notes: Vec<String>,
}

#[test]
fn replay_adapter_manifest_stays_within_documented_capability_floor() {
    let manifest_path = repo_root()
        .join("docs")
        .join("spec")
        .join("OXFML_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json");
    let content = fs::read_to_string(&manifest_path).expect("manifest should exist");
    let manifest: AdapterCapabilityManifest =
        serde_json::from_str(&content).expect("manifest should deserialize");

    assert_eq!(manifest.adapter_id, "oxfml.replay_adapter.v1");
    assert_eq!(manifest.adapter_version, "v1-draft");
    assert_eq!(manifest.lane_id, "oxfml");
    assert_eq!(
        manifest.supported_replay_bundle_schema_versions,
        vec!["dna-replay-bundle/v1".to_string()]
    );
    assert_eq!(
        manifest.claimed_capability_levels,
        vec![
            "cap.C0.ingest_valid".to_string(),
            "cap.C1.replay_valid".to_string(),
            "cap.C2.diff_valid".to_string(),
            "cap.C3.explain_valid".to_string(),
        ]
    );
    assert_eq!(
        manifest.scaffolded_capability_levels,
        vec!["cap.C4.distill_valid".to_string()]
    );
    assert!(
        !manifest
            .claimed_capability_levels
            .iter()
            .any(|level| level == "cap.C4.distill_valid" || level == "cap.C5.pack_valid")
    );
    assert!(
        manifest
            .known_limits
            .iter()
            .any(|entry| entry.contains("cap.C5.pack_valid"))
    );
    assert!(
        manifest
            .rollout_notes
            .iter()
            .any(|entry| entry.contains("OxFml remains authoritative"))
    );
    assert_eq!(manifest.supported_source_schema_ids.len(), 8);

    for registry_ref in &manifest.registry_version_refs {
        assert_eq!(
            registry_ref.registry_version,
            "oxfml.local.registry_pin.foundation_handoff_20260315_pass01"
        );
        assert!(!registry_ref.registry_family.is_empty());
    }

    for reference in &manifest.conformance_artifact_refs {
        assert!(
            repo_root().join(reference).exists(),
            "conformance artifact ref should exist: {reference}"
        );
    }
}

#[test]
fn reduced_witness_and_pack_candidate_artifacts_validate() {
    validate_reduced_reject_witness();
    validate_reduced_accept_witness();
    validate_reduced_session_witness();
    validate_reduced_execution_contract_witness();
    validate_quarantined_contention_witness();
    validate_local_pack_candidate_bundles();
}

fn validate_reduced_reject_witness() {
    let manifest: ReductionManifest =
        load_json_fixture("witness_distillation/fec_reject_formula_token_reduction_manifest.json");
    let witness_bundle: WitnessBundle<FecCommitReplayFixture> =
        load_json_fixture("witness_distillation/fec_reject_formula_token_witness_bundle.json");
    let lifecycle: WitnessLifecycleRecord =
        load_json_fixture("witness_distillation/fec_reject_formula_token_lifecycle.json");
    let source_cases: Vec<FecCommitReplayFixture> =
        load_json_fixture("fec_commit_replay_cases.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.fec_reject_formula_token.v1"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_kind,
        "pred.reject.family_present"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.fec_reject_formula_token.v1"
    );
    assert_eq!(
        manifest.predicate_ref.required_reject_code.as_deref(),
        Some("FenceMismatch")
    );
    assert_eq!(
        manifest.source_scope_ref,
        "fixture_family:fec_commit_replay_cases"
    );
    assert_eq!(manifest.strategy_id, "hierarchy_first");
    assert_eq!(
        manifest.unit_kinds,
        vec!["oxfml.local.reduction_unit.fixture_case"]
    );
    assert_eq!(
        manifest.retained_units,
        vec!["fec_002_formula_token_reject"]
    );
    assert_eq!(manifest.removed_units.len(), 2);
    assert!(manifest.rewritten_units.is_empty());
    assert_eq!(manifest.iteration_count, 1);
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "identity_closure")
    );
    assert_eq!(
        manifest
            .predicate_ref
            .required_mismatch_member_kind
            .as_deref(),
        Some("formula_token")
    );
    assert_eq!(manifest.final_status, "red.preserved");
    assert_existing_manifest_refs(&manifest);
    assert_eq!(
        witness_bundle.source_fixture_family,
        "fec_commit_replay_cases"
    );
    assert_eq!(witness_bundle.reduced_role, "replay_closed_witness");
    assert_eq!(witness_bundle.bundle_state, "replay_valid");
    assert_eq!(
        witness_bundle.source_case_ids,
        vec!["fec_002_formula_token_reject"]
    );
    assert_eq!(witness_bundle.scenario_cases.len(), 1);
    assert!(witness_bundle.scenario_cases.len() < source_cases.len());
    assert_retained_local_lifecycle(
        &lifecycle,
        &witness_bundle.witness_id,
        &manifest.source_bundle_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/fec_reject_formula_token_reduction_manifest.json",
    );
    let decision = replay_fec_commit_case(&witness_bundle.scenario_cases[0]);
    let AcceptDecision::Rejected(reject) = decision else {
        panic!("reduced witness should preserve rejected outcome");
    };
    assert_eq!(reject_code_name(reject.reject_code), "FenceMismatch");
}

fn validate_reduced_accept_witness() {
    let manifest: ReductionManifest =
        load_json_fixture("witness_distillation/fec_accept_publication_reduction_manifest.json");
    let witness_bundle: WitnessBundle<FecCommitReplayFixture> =
        load_json_fixture("witness_distillation/fec_accept_publication_witness_bundle.json");
    let lifecycle: WitnessLifecycleRecord =
        load_json_fixture("witness_distillation/fec_accept_publication_lifecycle.json");
    let source_cases: Vec<FecCommitReplayFixture> =
        load_json_fixture("fec_commit_replay_cases.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.fec_accept_publication.v1"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_kind,
        "pred.publication.accepted_payload_present"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.fec_accept_publication.v1"
    );
    assert_eq!(
        manifest
            .predicate_ref
            .required_publication_value_class
            .as_deref(),
        Some("Number")
    );
    assert_eq!(
        manifest.source_scope_ref,
        "fixture_family:fec_commit_replay_cases"
    );
    assert_eq!(manifest.strategy_id, "hierarchy_first");
    assert_eq!(
        manifest.unit_kinds,
        vec!["oxfml.local.reduction_unit.fixture_case"]
    );
    assert_eq!(manifest.retained_units, vec!["fec_001_accept"]);
    assert_eq!(manifest.removed_units.len(), 2);
    assert!(manifest.rewritten_units.is_empty());
    assert_eq!(manifest.iteration_count, 1);
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "candidate_commit_lineage")
    );
    assert_existing_manifest_refs(&manifest);
    assert_eq!(
        witness_bundle.source_fixture_family,
        "fec_commit_replay_cases"
    );
    assert_eq!(witness_bundle.reduced_role, "replay_closed_witness");
    assert_eq!(witness_bundle.bundle_state, "replay_valid");
    assert_eq!(witness_bundle.source_case_ids, vec!["fec_001_accept"]);
    assert_eq!(witness_bundle.scenario_cases.len(), 1);
    assert!(witness_bundle.scenario_cases.len() < source_cases.len());
    assert_retained_local_lifecycle(
        &lifecycle,
        &witness_bundle.witness_id,
        &manifest.source_bundle_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/fec_accept_publication_reduction_manifest.json",
    );
    let decision = replay_fec_commit_case(&witness_bundle.scenario_cases[0]);
    let AcceptDecision::Accepted(bundle) = decision else {
        panic!("reduced witness should preserve accepted outcome");
    };
    assert_eq!(
        value_payload_name(&bundle.value_delta.published_payload),
        "Number(42)"
    );
}

fn validate_reduced_session_witness() {
    let manifest: ReductionManifest =
        load_json_fixture("witness_distillation/session_capability_denied_reduction_manifest.json");
    let witness_bundle: WitnessBundle<SessionReplayFixture> =
        load_json_fixture("witness_distillation/session_capability_denied_witness_bundle.json");
    let lifecycle: WitnessLifecycleRecord =
        load_json_fixture("witness_distillation/session_capability_denied_lifecycle.json");
    let source_cases: Vec<SessionReplayFixture> =
        load_json_fixture("session_lifecycle_replay_cases.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.session_capability_denied.v1"
    );
    assert_eq!(
        manifest.predicate_ref.required_reject_code.as_deref(),
        Some("CapabilityDenied")
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.session_capability_denied.v1"
    );
    assert_eq!(
        manifest.source_scope_ref,
        "fixture_family:session_lifecycle_replay_cases"
    );
    assert_eq!(manifest.strategy_id, "hierarchy_first");
    assert_eq!(
        manifest.unit_kinds,
        vec!["oxfml.local.reduction_unit.fixture_case"]
    );
    assert_eq!(
        manifest.retained_units,
        vec!["session_002_capability_denied"]
    );
    assert_eq!(manifest.removed_units.len(), 6);
    assert!(manifest.rewritten_units.is_empty());
    assert_eq!(manifest.iteration_count, 1);
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "session_phase_closure")
    );
    assert_existing_manifest_refs(&manifest);
    assert_eq!(
        witness_bundle.source_fixture_family,
        "session_lifecycle_replay_cases"
    );
    assert_eq!(witness_bundle.reduced_role, "replay_closed_witness");
    assert_eq!(witness_bundle.bundle_state, "replay_valid");
    assert_eq!(
        witness_bundle.source_case_ids,
        vec!["session_002_capability_denied"]
    );
    assert_eq!(witness_bundle.scenario_cases.len(), 1);
    assert!(witness_bundle.scenario_cases.len() < source_cases.len());
    assert_retained_local_lifecycle(
        &lifecycle,
        &witness_bundle.witness_id,
        &manifest.source_bundle_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/session_capability_denied_reduction_manifest.json",
    );
    let reject = replay_session_capability_denied_case(&witness_bundle.scenario_cases[0]);
    assert_eq!(reject_code_name(reject.reject_code), "CapabilityDenied");
}

fn validate_reduced_execution_contract_witness() {
    let manifest: ReductionManifest = load_json_fixture(
        "witness_distillation/execution_contract_host_query_reduction_manifest.json",
    );
    let witness_bundle: WitnessBundle<ExecutionContractReplayFixture> =
        load_json_fixture("witness_distillation/execution_contract_host_query_witness_bundle.json");
    let lifecycle: WitnessLifecycleRecord =
        load_json_fixture("witness_distillation/execution_contract_host_query_lifecycle.json");
    let source_cases: Vec<ExecutionContractReplayFixture> =
        load_json_fixture("execution_contract_replay_cases.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.execution_contract_host_query.v1"
    );
    assert_eq!(
        manifest
            .predicate_ref
            .required_execution_restriction
            .as_deref(),
        Some("HostQuery")
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.execution_contract_host_query.v1"
    );
    assert_eq!(
        manifest.source_scope_ref,
        "fixture_family:execution_contract_replay_cases"
    );
    assert_eq!(manifest.strategy_id, "hierarchy_first");
    assert_eq!(
        manifest.unit_kinds,
        vec!["oxfml.local.reduction_unit.fixture_case"]
    );
    assert_eq!(manifest.retained_units, vec!["contract_002_cell"]);
    assert_eq!(manifest.removed_units.len(), 2);
    assert!(manifest.rewritten_units.is_empty());
    assert_eq!(manifest.iteration_count, 1);
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "execution_profile_closure")
    );
    assert_existing_manifest_refs(&manifest);
    assert_eq!(
        witness_bundle.source_fixture_family,
        "execution_contract_replay_cases"
    );
    assert_eq!(witness_bundle.reduced_role, "replay_closed_witness");
    assert_eq!(witness_bundle.bundle_state, "replay_valid");
    assert_eq!(witness_bundle.source_case_ids, vec!["contract_002_cell"]);
    assert_eq!(witness_bundle.scenario_cases.len(), 1);
    assert!(witness_bundle.scenario_cases.len() < source_cases.len());
    assert_retained_local_lifecycle(
        &lifecycle,
        &witness_bundle.witness_id,
        &manifest.source_bundle_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/execution_contract_host_query_reduction_manifest.json",
    );
    let contract = replay_execution_contract_case(&witness_bundle.scenario_cases[0]);
    assert_eq!(scheduler_lane_name(contract.lane_class), "Serialized");
    assert_eq!(
        replay_sensitivity_name(contract.replay_sensitivity),
        "Stable"
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::HostQuery)
    );
}

fn validate_quarantined_contention_witness() {
    let manifest: ReductionManifest = load_json_fixture(
        "witness_distillation/session_contention_quarantine_reduction_manifest.json",
    );
    let witness_bundle: WitnessBundle<SessionReplayFixture> =
        load_json_fixture("witness_distillation/session_contention_quarantine_witness_bundle.json");
    let lifecycle: WitnessLifecycleRecord =
        load_json_fixture("witness_distillation/session_contention_quarantine_lifecycle.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.session_contention_quarantine.v1"
    );
    assert_eq!(
        manifest.predicate_ref.required_reject_code.as_deref(),
        Some("StructuralConflict")
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.session_contention_conflict.v1"
    );
    assert_eq!(manifest.final_status, "red.unsupported");
    assert_eq!(
        manifest.retained_units,
        vec!["session_008_contention_execute_rejected"]
    );
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "contention_interleaving_closure")
    );
    assert_existing_manifest_refs(&manifest);

    assert_eq!(
        witness_bundle.source_case_ids,
        vec!["session_008_contention_execute_rejected"]
    );
    assert_eq!(witness_bundle.reduced_role, "explanatory_only_witness");
    assert_eq!(witness_bundle.bundle_state, "quarantined_local");
    assert_eq!(witness_bundle.scenario_cases.len(), 1);

    assert_eq!(lifecycle.lifecycle_state, "wit.quarantined_local");
    assert_eq!(
        lifecycle.reduction_manifest_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/session_contention_quarantine_reduction_manifest.json"
    );
    assert_eq!(
        lifecycle.quarantine_reason.as_deref(),
        Some("contention_interleaving_not_replay_safe_rewrite")
    );
    assert_eq!(lifecycle.gc_eligibility, "manual_review");
    assert!(lifecycle.promotion_refs.is_empty());
    assert!(lifecycle.supersedes.is_empty());
}

fn validate_local_pack_candidate_bundles() {
    let fec_bundle: NormalizedPackCandidateBundle =
        load_json_fixture("replay_bundle_normalization/fec_commit_pack_candidate_bundle.json");
    let session_bundle: NormalizedPackCandidateBundle = load_json_fixture(
        "replay_bundle_normalization/session_lifecycle_pack_candidate_bundle.json",
    );
    let index: PackCandidateIndex =
        load_json_fixture("replay_bundle_normalization/pack_candidate_index.json");

    for bundle in [&fec_bundle, &session_bundle] {
        assert_eq!(bundle.bundle_schema_version, "dna-replay-bundle/v1");
        assert_eq!(bundle.lane_id, "oxfml");
        assert_eq!(bundle.adapter_id, "oxfml.replay_adapter.v1");
        assert_eq!(bundle.bundle_role, "local_pack_candidate");
        assert_eq!(bundle.pack_eligibility_state, "not_pack_eligible");
        assert_eq!(bundle.required_capability_level, "cap.C3.explain_valid");
        assert_eq!(
            bundle.registry_pin,
            "oxfml.local.registry_pin.foundation_handoff_20260315_pass01"
        );
        assert_eq!(bundle.authorized_transform_family, "subset_projection_only");
        assert!(bundle.rewrite_authorizations.is_empty());
        assert!(!bundle.witness_lifecycle_refs.is_empty());
        assert!(!bundle.normalized_event_families.is_empty());
        assert!(!bundle.source_artifact_refs.is_empty());
        assert!(!bundle.bundle_id.is_empty());
        assert!(
            bundle
                .source_schema_id
                .starts_with("oxfml.local.source_schema.")
        );
        assert!(
            bundle
                .promotion_blockers
                .iter()
                .any(|entry| entry == "local_witness_tier_only")
        );
        for reference in &bundle.witness_lifecycle_refs {
            assert!(
                repo_root().join(reference).exists(),
                "witness ref should exist: {reference}"
            );
        }
        for reference in &bundle.source_artifact_refs {
            assert!(
                repo_root().join(reference).exists(),
                "source ref should exist: {reference}"
            );
        }
    }

    assert_eq!(fec_bundle.source_fixture_family, "fec_commit_replay_cases");
    assert_eq!(
        session_bundle.source_fixture_family,
        "session_lifecycle_replay_cases"
    );

    assert_eq!(index.bundle_schema_version, "dna-replay-bundle/v1");
    assert_eq!(index.lane_id, "oxfml");
    assert_eq!(index.adapter_id, "oxfml.replay_adapter.v1");
    assert_eq!(index.index_state, "local_candidate_only");
    assert_eq!(index.pack_eligibility_state, "not_pack_eligible");
    assert_eq!(index.required_capability_level, "cap.C3.explain_valid");
    assert_eq!(index.bundle_refs.len(), 2);
    assert_eq!(index.index_id, "oxfml.local.pack_candidate_index.v1");
    assert!(
        index
            .promotion_blockers
            .iter()
            .any(|entry| entry == "pack_grade_bundle_governance_not_satisfied")
    );
    assert!(
        index
            .notes
            .iter()
            .any(|entry| entry.contains("non-pack-eligible"))
    );
    for reference in &index.bundle_refs {
        assert!(
            repo_root().join(reference).exists(),
            "bundle ref should exist: {reference}"
        );
    }
}

fn replay_fec_commit_case(fixture: &FecCommitReplayFixture) -> AcceptDecision {
    commit_candidate(CommitRequest {
        candidate_result: sample_candidate(),
        commit_attempt_id: format!("commit:{}", fixture.case_id),
        observed_fence: FenceSnapshot {
            formula_token: fixture.observed_fence.formula_token.clone(),
            snapshot_epoch: fixture.observed_fence.snapshot_epoch.clone(),
            bind_hash: fixture.observed_fence.bind_hash.clone(),
            profile_version: fixture.observed_fence.profile_version.clone(),
            capability_view_key: Some(fixture.observed_fence.capability_view_key.clone()),
        },
    })
}

fn replay_session_capability_denied_case(
    fixture: &SessionReplayFixture,
) -> oxfml_core::RejectRecord {
    let prepared = compile_prepare_request(&fixture.formula, fixture.with_input_name);
    let mut service = SessionService::new();
    let prepared = service.prepare(prepared).expect("prepare should succeed");
    let open = service.open_session(prepared);
    service
        .establish_capability_view(
            &open.session_id,
            into_capability_spec(&fixture.capability_spec),
        )
        .expect_err("capability view should reject")
}

fn replay_execution_contract_case(
    fixture: &ExecutionContractReplayFixture,
) -> oxfml_core::ExecutionContract {
    let source = FormulaSourceRecord::new("execution-contract-witness", 1, fixture.formula.clone());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion(
                "execution-contract-reduction-v1".to_string(),
            ),
            ..BindContext::default()
        },
    });
    let plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula,
        oxfunc_catalog_identity: "oxfunc:execution-contract-reduction".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan;
    build_execution_contract(&plan)
}

fn compile_prepare_request(formula: &str, with_input_name: bool) -> PrepareRequest {
    let source = FormulaSourceRecord::new("session-reduction-witness", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let mut names = BTreeMap::new();
    if with_input_name {
        names.insert("InputValue".to_string(), NameKind::ValueLike);
    }
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("session-reduction-v1".to_string()),
            names,
            ..BindContext::default()
        },
    });
    let plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:session-reduction".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan;
    PrepareRequest {
        source,
        bound_formula: bind.bound_formula,
        semantic_plan: plan,
        primary_locus: Locus {
            sheet_id: "sheet:default".to_string(),
            row: 1,
            col: 1,
        },
    }
}

fn into_capability_spec(fixture: &CapabilitySpecFixture) -> CapabilityViewSpec {
    CapabilityViewSpec {
        host_query_enabled: fixture.host_query_enabled,
        locale_format_enabled: fixture.locale_format_enabled,
        caller_context_enabled: fixture.caller_context_enabled,
        external_provider_enabled: fixture.external_provider_enabled,
    }
}

fn assert_existing_manifest_refs(manifest: &ReductionManifest) {
    assert!(repo_root().join(&manifest.source_bundle_ref).exists());
    assert!(repo_root().join(&manifest.witness_bundle_ref).exists());
}

fn assert_retained_local_lifecycle(
    lifecycle: &WitnessLifecycleRecord,
    witness_id: &str,
    source_bundle_ref: &str,
    reduction_manifest_ref: &str,
) {
    assert_eq!(lifecycle.witness_id, witness_id);
    assert_eq!(lifecycle.lifecycle_state, "wit.retained_local");
    assert_eq!(lifecycle.source_bundle_ref, source_bundle_ref);
    assert_eq!(lifecycle.reduction_manifest_ref, reduction_manifest_ref);
    assert_eq!(
        lifecycle.retention_policy_id,
        "oxfml.local.retention.manual_review"
    );
    assert!(lifecycle.promotion_refs.is_empty());
    assert!(lifecycle.supersedes.is_empty());
    assert!(lifecycle.quarantine_reason.is_none());
    assert_eq!(lifecycle.gc_eligibility, "not_eligible");
    assert!(!lifecycle.notes.is_empty());
    assert!(repo_root().join(&lifecycle.reduction_manifest_ref).exists());
}

fn sample_candidate() -> AcceptedCandidateResult {
    AcceptedCandidateResult {
        formula_stable_id: "formula:fec".to_string(),
        session_id: Some("session:0001".to_string()),
        candidate_result_id: "candidate:fec".to_string(),
        fence_snapshot: FenceSnapshot {
            formula_token: "token:v1".to_string(),
            snapshot_epoch: "epoch:10".to_string(),
            bind_hash: "bind:abc".to_string(),
            profile_version: "profile:v1".to_string(),
            capability_view_key: Some("cap:view:v1".to_string()),
        },
        value_delta: ValueDelta {
            formula_stable_id: "formula:fec".to_string(),
            primary_locus: Locus {
                sheet_id: "sheet:1".to_string(),
                row: 1,
                col: 1,
            },
            affected_value_loci: vec![Locus {
                sheet_id: "sheet:1".to_string(),
                row: 1,
                col: 1,
            }],
            published_value_class: WorksheetValueClass::Scalar,
            published_payload: ValuePayload::Number("42".to_string()),
            result_extent: Some(Extent { rows: 1, cols: 1 }),
            candidate_result_id: Some("candidate:fec".to_string()),
        },
        shape_delta: ShapeDelta {
            formula_stable_id: "formula:fec".to_string(),
            anchor_locus: Locus {
                sheet_id: "sheet:1".to_string(),
                row: 1,
                col: 1,
            },
            intended_extent: Extent { rows: 1, cols: 1 },
            published_extent: Some(Extent { rows: 1, cols: 1 }),
            blocked_loci: Vec::new(),
            shape_outcome_class: ShapeOutcomeClass::Established,
            candidate_result_id: Some("candidate:fec".to_string()),
        },
        topology_delta: TopologyDelta {
            formula_stable_id: "formula:fec".to_string(),
            dependency_additions: Vec::new(),
            dependency_removals: Vec::new(),
            dependency_reclassifications: Vec::new(),
            dependency_consequence_facts: Vec::new(),
            dynamic_reference_facts: Vec::new(),
            spill_facts: Vec::new(),
            format_dependency_facts: Vec::new(),
            capability_effect_facts: Vec::new(),
            candidate_result_id: Some("candidate:fec".to_string()),
        },
        format_delta: None,
        display_delta: None,
        spill_events: vec![SpillEvent {
            spill_event_kind: SpillEventKind::SpillTakeover,
            formula_stable_id: "formula:fec".to_string(),
            anchor_locus: Locus {
                sheet_id: "sheet:1".to_string(),
                row: 1,
                col: 1,
            },
            intended_extent: Extent { rows: 1, cols: 1 },
            affected_extent: Some(Extent { rows: 1, cols: 1 }),
            blocking_loci: Vec::new(),
            blocking_reason_class: None,
            correlation_id: "candidate:fec".to_string(),
        }],
        execution_profile: None,
        trace_correlation_id: "trace:candidate:fec".to_string(),
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative)
}

fn load_json_fixture<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    let path = fixture_path(relative);
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn value_payload_name(payload: &ValuePayload) -> String {
    match payload {
        ValuePayload::Number(value) => format!("Number({value})"),
        ValuePayload::Text(value) => format!("Text({value})"),
        ValuePayload::Logical(value) => format!("Logical({value})"),
        ValuePayload::ErrorCode(value) => format!("ErrorCode({value})"),
        ValuePayload::Blank => "Blank".to_string(),
    }
}

fn reject_code_name(code: oxfml_core::RejectCode) -> &'static str {
    match code {
        oxfml_core::RejectCode::FenceMismatch => "FenceMismatch",
        oxfml_core::RejectCode::CapabilityDenied => "CapabilityDenied",
        oxfml_core::RejectCode::SessionTerminated => "SessionTerminated",
        oxfml_core::RejectCode::BindMismatch => "BindMismatch",
        oxfml_core::RejectCode::StructuralConflict => "StructuralConflict",
        oxfml_core::RejectCode::DynamicReferenceFailure => "DynamicReferenceFailure",
        oxfml_core::RejectCode::ResourceInvariantFailure => "ResourceInvariantFailure",
    }
}

fn scheduler_lane_name(lane: SchedulerLaneClass) -> &'static str {
    match lane {
        SchedulerLaneClass::ConcurrentSafe => "ConcurrentSafe",
        SchedulerLaneClass::Serialized => "Serialized",
    }
}

fn replay_sensitivity_name(value: ReplaySensitivityClass) -> &'static str {
    match value {
        ReplaySensitivityClass::Stable => "Stable",
        ReplaySensitivityClass::ContextSensitive => "ContextSensitive",
        ReplaySensitivityClass::NonDeterministic => "NonDeterministic",
    }
}
