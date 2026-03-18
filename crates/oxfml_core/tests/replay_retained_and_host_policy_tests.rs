use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::en_us_context;
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike};
use serde::Deserialize;

use oxfml_core::seam::AcceptDecision;
use oxfml_core::{EmpiricalOracleScenario, EvaluationBackend, SingleFormulaHost};

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
    required_publication_value_class: Option<String>,
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

#[derive(Debug, Deserialize)]
struct RetainedWitnessSetIndex {
    index_id: String,
    lane_id: String,
    adapter_id: String,
    index_state: String,
    retained_witness_refs: Vec<String>,
    quarantined_witness_refs: Vec<String>,
    family_counts: BTreeMap<String, u64>,
    capability_residuals: Vec<String>,
    promotion_blockers: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct HostReplayFixture {
    case_id: String,
    formula: String,
    backend: String,
    defined_names: BTreeMap<String, String>,
    #[serde(default)]
    cell_bindings: BTreeMap<String, String>,
    host_query_profile: Option<String>,
    expected: HostReplayExpected,
}

#[derive(Debug, Deserialize, Clone)]
struct HostReplayExpected {
    payload_summary: String,
    commit_decision: String,
    #[serde(default)]
    trace_event_kinds: Vec<String>,
    #[serde(default)]
    capability_effect_kinds: Vec<String>,
    #[serde(default)]
    format_dependency_tokens: Vec<String>,
    #[serde(default)]
    spill_event_kinds: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct EmpiricalOracleScenarioWire {
    scenario_id: String,
    formula: String,
    entered_formula_text: String,
    stored_formula_text: Option<String>,
    input_bindings: BTreeMap<String, String>,
    #[serde(default)]
    cell_bindings: BTreeMap<String, String>,
    expected_result_summary: String,
    locale_profile: Option<String>,
    date_system: Option<String>,
    host_query_profile: Option<String>,
    #[serde(default)]
    expected_trace_event_kinds: Vec<String>,
    #[serde(default)]
    expected_capability_effect_kinds: Vec<String>,
    #[serde(default)]
    expected_format_dependency_tokens: Vec<String>,
    #[serde(default)]
    expected_spill_event_kinds: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct HostPolicyProfile {
    profile_id: String,
    host_case_ids: Vec<String>,
    empirical_oracle_scenario_ids: Vec<String>,
    requires_direct_cell_bindings: bool,
    host_query_profile_required: bool,
    allowed_backends: Vec<String>,
    pack_state: String,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmpiricalPackCandidateGroup {
    group_id: String,
    host_case_ids: Vec<String>,
    empirical_oracle_scenario_ids: Vec<String>,
    requires_direct_cell_bindings: bool,
    primary_semantic_lanes: Vec<String>,
    pack_state: String,
    promotion_blockers: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PromotionReadinessIndex {
    index_id: String,
    lane_id: String,
    adapter_id: String,
    index_state: String,
    claimed_capability_ceiling: String,
    family_refs: Vec<String>,
    nearest_promotion_families: Vec<String>,
    promotion_blockers: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PromotionCandidateFamily {
    family_id: String,
    source_fixture_family: String,
    current_evidence_tier: String,
    promotion_readiness: String,
    required_capability_level: String,
    required_predicates: Vec<String>,
    promotion_blockers: Vec<String>,
}

#[test]
fn retained_witness_set_index_spans_host_and_oracle_families() {
    let index: RetainedWitnessSetIndex =
        load_json_fixture("witness_distillation/retained_witness_set_index.json");

    assert_eq!(index.index_id, "oxfml.local.retained_witness_set_index.v1");
    assert_eq!(index.lane_id, "oxfml");
    assert_eq!(index.adapter_id, "oxfml.replay_adapter.v1");
    assert_eq!(index.index_state, "retained_local_floor");
    assert_eq!(index.retained_witness_refs.len(), 6);
    assert_eq!(index.quarantined_witness_refs.len(), 1);
    assert_eq!(index.family_counts.get("fec_commit"), Some(&2));
    assert_eq!(index.family_counts.get("session_lifecycle"), Some(&1));
    assert_eq!(index.family_counts.get("execution_contract"), Some(&1));
    assert_eq!(index.family_counts.get("single_formula_host"), Some(&1));
    assert_eq!(index.family_counts.get("empirical_oracle"), Some(&1));
    assert!(
        index
            .capability_residuals
            .iter()
            .any(|entry| entry.contains("cap.C4.distill_valid"))
    );
    assert!(
        index
            .promotion_blockers
            .iter()
            .any(|entry| entry == "cap_c4_breadth_not_satisfied")
    );
    assert!(
        index
            .notes
            .iter()
            .any(|entry| entry.contains("Retained-local witness breadth"))
    );

    for reference in index
        .retained_witness_refs
        .iter()
        .chain(index.quarantined_witness_refs.iter())
    {
        assert!(
            repo_root().join(reference).exists(),
            "witness index ref should exist: {reference}"
        );
    }

    validate_reduced_host_scalarization_witness();
    validate_reduced_empirical_oracle_host_query_witness();
}

#[test]
fn dna_onecalc_host_policy_profiles_and_empirical_pack_groups_reference_exercised_cases() {
    let profiles: Vec<HostPolicyProfile> =
        load_json_fixture("empirical_pack_planning/dna_onecalc_host_policy_profiles.json");
    let groups: Vec<EmpiricalPackCandidateGroup> =
        load_json_fixture("empirical_pack_planning/empirical_pack_candidate_groups.json");
    let host_cases: Vec<HostReplayFixture> =
        load_json_fixture("single_formula_host_replay_cases.json");
    let oracle_cases: Vec<EmpiricalOracleScenarioWire> =
        load_json_fixture("empirical_oracle_scenarios.json");

    let host_case_ids: BTreeSet<String> =
        host_cases.iter().map(|case| case.case_id.clone()).collect();
    let oracle_case_ids: BTreeSet<String> = oracle_cases
        .iter()
        .map(|case| case.scenario_id.clone())
        .collect();

    assert_eq!(profiles.len(), 6);
    assert_eq!(groups.len(), 6);

    for profile in &profiles {
        assert!(
            profile
                .profile_id
                .starts_with("dna_onecalc.single_formula.")
        );
        assert_eq!(profile.pack_state, "planning_only");
        assert!(!profile.allowed_backends.is_empty());
        assert!(!profile.notes.is_empty());

        for case_id in &profile.host_case_ids {
            assert!(
                host_case_ids.contains(case_id),
                "unknown host case id in host policy profile: {case_id}"
            );
        }
        for scenario_id in &profile.empirical_oracle_scenario_ids {
            assert!(
                oracle_case_ids.contains(scenario_id),
                "unknown oracle scenario id in host policy profile: {scenario_id}"
            );
        }

        if profile.requires_direct_cell_bindings {
            for case_id in &profile.host_case_ids {
                let host_case = host_cases
                    .iter()
                    .find(|case| &case.case_id == case_id)
                    .expect("host case should exist");
                assert!(
                    !host_case.cell_bindings.is_empty(),
                    "profile requires direct cell bindings but host case does not: {case_id}"
                );
            }
            for scenario_id in &profile.empirical_oracle_scenario_ids {
                let oracle_case = oracle_cases
                    .iter()
                    .find(|case| &case.scenario_id == scenario_id)
                    .expect("oracle case should exist");
                assert!(
                    !oracle_case.cell_bindings.is_empty(),
                    "profile requires direct cell bindings but oracle case does not: {scenario_id}"
                );
            }
        }

        if profile.host_query_profile_required {
            for case_id in &profile.host_case_ids {
                let host_case = host_cases
                    .iter()
                    .find(|case| &case.case_id == case_id)
                    .expect("host case should exist");
                assert!(
                    host_case.host_query_profile.is_some(),
                    "profile requires host-query profile but host case does not: {case_id}"
                );
            }
            for scenario_id in &profile.empirical_oracle_scenario_ids {
                let oracle_case = oracle_cases
                    .iter()
                    .find(|case| &case.scenario_id == scenario_id)
                    .expect("oracle case should exist");
                assert!(
                    oracle_case.host_query_profile.is_some(),
                    "profile requires host-query profile but oracle case does not: {scenario_id}"
                );
            }
        }
    }

    for group in &groups {
        assert!(group.group_id.starts_with("emp_pack."));
        assert_eq!(group.pack_state, "planning_only");
        assert!(!group.primary_semantic_lanes.is_empty());
        assert!(!group.promotion_blockers.is_empty());
        assert!(!group.notes.is_empty());

        for case_id in &group.host_case_ids {
            assert!(
                host_case_ids.contains(case_id),
                "unknown host case id in empirical pack group: {case_id}"
            );
        }
        for scenario_id in &group.empirical_oracle_scenario_ids {
            assert!(
                oracle_case_ids.contains(scenario_id),
                "unknown oracle scenario id in empirical pack group: {scenario_id}"
            );
        }

        if group.requires_direct_cell_bindings {
            for case_id in &group.host_case_ids {
                let host_case = host_cases
                    .iter()
                    .find(|case| &case.case_id == case_id)
                    .expect("host case should exist");
                assert!(
                    !host_case.cell_bindings.is_empty(),
                    "group requires direct cell bindings but host case does not: {case_id}"
                );
            }
            for scenario_id in &group.empirical_oracle_scenario_ids {
                let oracle_case = oracle_cases
                    .iter()
                    .find(|case| &case.scenario_id == scenario_id)
                    .expect("oracle case should exist");
                assert!(
                    !oracle_case.cell_bindings.is_empty(),
                    "group requires direct cell bindings but oracle case does not: {scenario_id}"
                );
            }
        }
    }
}

#[test]
fn replay_promotion_readiness_index_classifies_current_local_families() {
    let index: PromotionReadinessIndex =
        load_json_fixture("replay_bundle_normalization/promotion_readiness_index.json");
    let families: Vec<PromotionCandidateFamily> =
        load_json_fixture("replay_bundle_normalization/promotion_candidate_families.json");

    assert_eq!(
        index.index_id,
        "oxfml.local.replay_promotion_readiness_index.v1"
    );
    assert_eq!(index.lane_id, "oxfml");
    assert_eq!(index.adapter_id, "oxfml.replay_adapter.v1");
    assert_eq!(index.index_state, "promotion_baseline_only");
    assert_eq!(index.claimed_capability_ceiling, "cap.C3.explain_valid");
    assert_eq!(index.family_refs.len(), 1);
    assert_eq!(
        index.nearest_promotion_families,
        vec!["fec_commit", "session_lifecycle"]
    );
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
            .any(|entry| entry.contains("does not claim cap.C4.distill_valid"))
    );
    for reference in &index.family_refs {
        assert!(repo_root().join(reference).exists());
    }

    assert_eq!(families.len(), 5);
    for family in &families {
        assert_eq!(family.current_evidence_tier, "retained_local");
        assert_eq!(family.required_capability_level, "cap.C3.explain_valid");
        assert!(!family.required_predicates.is_empty());
        assert!(!family.promotion_blockers.is_empty());
    }

    let fec_commit = families
        .iter()
        .find(|item| item.family_id == "fec_commit")
        .expect("fec_commit family should exist");
    assert_eq!(
        fec_commit.promotion_readiness,
        "candidate_for_pack_grade_baseline"
    );
    assert_eq!(fec_commit.source_fixture_family, "fec_commit_replay_cases");

    let empirical_oracle = families
        .iter()
        .find(|item| item.family_id == "empirical_oracle")
        .expect("empirical_oracle family should exist");
    assert_eq!(
        empirical_oracle.promotion_readiness,
        "blocked_empirical_pack_policy"
    );
}

fn validate_reduced_host_scalarization_witness() {
    let manifest: ReductionManifest = load_json_fixture(
        "witness_distillation/single_formula_host_scalarization_reduction_manifest.json",
    );
    let witness_bundle: WitnessBundle<HostReplayFixture> = load_json_fixture(
        "witness_distillation/single_formula_host_scalarization_witness_bundle.json",
    );
    let lifecycle: WitnessLifecycleRecord =
        load_json_fixture("witness_distillation/single_formula_host_scalarization_lifecycle.json");
    let source_cases: Vec<HostReplayFixture> =
        load_json_fixture("single_formula_host_replay_cases.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.single_formula_host_scalarization.v1"
    );
    assert_eq!(
        manifest.source_scope_ref,
        "fixture_family:single_formula_host_replay_cases"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_kind,
        "pred.publication.accepted_payload_present"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.single_formula_host_scalarization.v1"
    );
    assert_eq!(
        manifest
            .predicate_ref
            .required_publication_value_class
            .as_deref(),
        Some("Number")
    );
    assert_eq!(manifest.strategy_id, "hierarchy_first");
    assert_eq!(
        manifest.unit_kinds,
        vec!["oxfml.local.reduction_unit.fixture_case"]
    );
    assert_eq!(
        manifest.retained_units,
        vec!["host_004_implicit_intersection"]
    );
    assert_eq!(manifest.removed_units.len(), 9);
    assert!(manifest.rewritten_units.is_empty());
    assert_eq!(manifest.iteration_count, 1);
    assert_eq!(manifest.final_status, "red.preserved");
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "host_cell_binding_closure")
    );
    assert_existing_manifest_refs(&manifest);

    assert_eq!(
        witness_bundle.source_fixture_family,
        "single_formula_host_replay_cases"
    );
    assert_eq!(witness_bundle.reduced_role, "replay_closed_witness");
    assert_eq!(witness_bundle.bundle_state, "replay_valid");
    assert_eq!(
        witness_bundle.source_case_ids,
        vec!["host_004_implicit_intersection"]
    );
    assert_eq!(witness_bundle.scenario_cases.len(), 1);
    assert!(witness_bundle.scenario_cases.len() < source_cases.len());
    assert_retained_local_lifecycle(
        &lifecycle,
        &witness_bundle.witness_id,
        &manifest.source_bundle_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/single_formula_host_scalarization_reduction_manifest.json",
    );

    let run = replay_host_case(&witness_bundle.scenario_cases[0]);
    assert_eq!(
        run.evaluation.result.payload_summary,
        witness_bundle.scenario_cases[0].expected.payload_summary
    );
    assert_eq!(
        witness_bundle.scenario_cases[0].expected.commit_decision,
        "accepted"
    );
    assert_commit_accepted(&run.commit_decision);
    assert_host_trace_and_effects(&run, &witness_bundle.scenario_cases[0].expected);
}

fn validate_reduced_empirical_oracle_host_query_witness() {
    let manifest: ReductionManifest = load_json_fixture(
        "witness_distillation/empirical_oracle_host_query_reference_reduction_manifest.json",
    );
    let witness_bundle: WitnessBundle<EmpiricalOracleScenarioWire> = load_json_fixture(
        "witness_distillation/empirical_oracle_host_query_reference_witness_bundle.json",
    );
    let lifecycle: WitnessLifecycleRecord = load_json_fixture(
        "witness_distillation/empirical_oracle_host_query_reference_lifecycle.json",
    );
    let source_cases: Vec<EmpiricalOracleScenarioWire> =
        load_json_fixture("empirical_oracle_scenarios.json");

    assert_eq!(
        manifest.reduction_id,
        "oxfml.local.reduction.empirical_oracle_host_query_reference.v1"
    );
    assert_eq!(
        manifest.source_scope_ref,
        "fixture_family:empirical_oracle_scenarios"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_kind,
        "pred.publication.accepted_payload_present"
    );
    assert_eq!(
        manifest.predicate_ref.predicate_id,
        "oxfml.local.predicate.empirical_oracle_host_query_reference.v1"
    );
    assert_eq!(
        manifest
            .predicate_ref
            .required_publication_value_class
            .as_deref(),
        Some("Text")
    );
    assert_eq!(manifest.strategy_id, "hierarchy_first");
    assert_eq!(
        manifest.unit_kinds,
        vec!["oxfml.local.reduction_unit.fixture_case"]
    );
    assert_eq!(manifest.retained_units, vec!["oracle_007_cell_filename"]);
    assert_eq!(manifest.removed_units.len(), 6);
    assert!(manifest.rewritten_units.is_empty());
    assert_eq!(manifest.iteration_count, 1);
    assert_eq!(manifest.final_status, "red.preserved");
    assert!(
        manifest
            .closure_rules_applied
            .iter()
            .any(|rule| rule == "host_query_capability_closure")
    );
    assert_existing_manifest_refs(&manifest);

    assert_eq!(
        witness_bundle.source_fixture_family,
        "empirical_oracle_scenarios"
    );
    assert_eq!(witness_bundle.reduced_role, "replay_closed_witness");
    assert_eq!(witness_bundle.bundle_state, "replay_valid");
    assert_eq!(
        witness_bundle.source_case_ids,
        vec!["oracle_007_cell_filename"]
    );
    assert_eq!(witness_bundle.scenario_cases.len(), 1);
    assert!(witness_bundle.scenario_cases.len() < source_cases.len());
    assert_retained_local_lifecycle(
        &lifecycle,
        &witness_bundle.witness_id,
        &manifest.source_bundle_ref,
        "crates/oxfml_core/tests/fixtures/witness_distillation/empirical_oracle_host_query_reference_reduction_manifest.json",
    );

    let run = replay_empirical_oracle_case(&witness_bundle.scenario_cases[0]);
    assert_eq!(
        run.evaluation.result.payload_summary,
        witness_bundle.scenario_cases[0].expected_result_summary
    );
    let actual_trace_kinds = run
        .trace_events
        .iter()
        .map(|event| format!("{:?}", event.event_kind))
        .collect::<Vec<_>>();
    assert_eq!(
        actual_trace_kinds,
        witness_bundle.scenario_cases[0].expected_trace_event_kinds
    );
    let actual_capability_effect_kinds = run
        .candidate_result
        .topology_delta
        .capability_effect_facts
        .iter()
        .map(|fact| fact.capability_kind.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_capability_effect_kinds,
        witness_bundle.scenario_cases[0].expected_capability_effect_kinds
    );
    let actual_format_dependency_tokens = run
        .candidate_result
        .topology_delta
        .format_dependency_facts
        .iter()
        .map(|fact| fact.dependency_token.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_format_dependency_tokens,
        witness_bundle.scenario_cases[0].expected_format_dependency_tokens
    );
    let actual_spill_event_kinds = run
        .candidate_result
        .spill_events
        .iter()
        .map(|event| format!("{:?}", event.spill_event_kind))
        .collect::<Vec<_>>();
    assert_eq!(
        actual_spill_event_kinds,
        witness_bundle.scenario_cases[0].expected_spill_event_kinds
    );
}

fn replay_host_case(fixture: &HostReplayFixture) -> oxfml_core::HostRecalcOutput {
    let mut host = SingleFormulaHost::new(&fixture.case_id, &fixture.formula);

    for (name, wire_value) in &fixture.defined_names {
        if let Some(target) = wire_value
            .strip_prefix("Reference(")
            .and_then(|value| value.strip_suffix(')'))
        {
            host.set_defined_name_reference(
                name,
                ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: target.to_string(),
                },
            );
        } else {
            host.set_defined_name_value(name, parse_eval_value_wire(wire_value));
        }
    }

    for (target, wire_value) in &fixture.cell_bindings {
        host.set_cell_value(target, parse_eval_value_wire(wire_value));
    }

    let host_info = fixture
        .host_query_profile
        .as_deref()
        .map(|_| &ReplayHostInfoProvider as &dyn HostInfoProvider);
    let backend = match fixture.backend.as_str() {
        "LocalBootstrap" => EvaluationBackend::LocalBootstrap,
        "OxFuncBacked" => EvaluationBackend::OxFuncBacked,
        other => panic!("unexpected backend: {other}"),
    };

    host.recalc_with_backend(backend, host_info, Some(&en_us_context()))
        .expect("replayed host case should execute")
}

fn replay_empirical_oracle_case(
    scenario: &EmpiricalOracleScenarioWire,
) -> oxfml_core::HostRecalcOutput {
    let host_info = scenario
        .host_query_profile
        .as_deref()
        .map(|_| &ReplayHostInfoProvider as &dyn HostInfoProvider);
    SingleFormulaHost::run_empirical_oracle_scenario(
        &EmpiricalOracleScenario {
            scenario_id: scenario.scenario_id.clone(),
            formula: scenario.formula.clone(),
            entered_formula_text: scenario.entered_formula_text.clone(),
            stored_formula_text: scenario.stored_formula_text.clone(),
            input_bindings: scenario.input_bindings.clone(),
            cell_bindings: scenario.cell_bindings.clone(),
            expected_result_summary: scenario.expected_result_summary.clone(),
            locale_profile: scenario.locale_profile.clone(),
            date_system: scenario.date_system.clone(),
            host_query_profile: scenario.host_query_profile.clone(),
        },
        host_info,
        Some(&en_us_context()),
    )
    .expect("replayed empirical oracle case should execute")
}

fn assert_host_trace_and_effects(
    run: &oxfml_core::HostRecalcOutput,
    expected: &HostReplayExpected,
) {
    let actual_trace_kinds = run
        .trace_events
        .iter()
        .map(|event| format!("{:?}", event.event_kind))
        .collect::<Vec<_>>();
    assert_eq!(actual_trace_kinds, expected.trace_event_kinds);

    let actual_capability_effect_kinds = run
        .candidate_result
        .topology_delta
        .capability_effect_facts
        .iter()
        .map(|fact| fact.capability_kind.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_capability_effect_kinds,
        expected.capability_effect_kinds
    );

    let actual_format_dependency_tokens = run
        .candidate_result
        .topology_delta
        .format_dependency_facts
        .iter()
        .map(|fact| fact.dependency_token.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_format_dependency_tokens,
        expected.format_dependency_tokens
    );

    let actual_spill_event_kinds = run
        .candidate_result
        .spill_events
        .iter()
        .map(|event| format!("{:?}", event.spill_event_kind))
        .collect::<Vec<_>>();
    assert_eq!(actual_spill_event_kinds, expected.spill_event_kinds);
}

fn assert_commit_accepted(decision: &AcceptDecision) {
    match decision {
        AcceptDecision::Accepted(_) => {}
        AcceptDecision::Rejected(reject) => {
            panic!(
                "expected accepted host witness decision, got reject: {:?}",
                reject.reject_code
            )
        }
    }
}

fn parse_eval_value_wire(wire_value: &str) -> EvalValue {
    if let Some(inner) = wire_value
        .strip_prefix("Number(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return EvalValue::Number(inner.parse::<f64>().expect("number should parse"));
    }

    if let Some(inner) = wire_value
        .strip_prefix("Text(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return EvalValue::Text(ExcelText::from_utf16_code_units(
            inner.encode_utf16().collect(),
        ));
    }

    panic!("unsupported eval value wire format: {wire_value}");
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

fn load_json_fixture<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative);
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root should resolve")
}

struct ReplayHostInfoProvider;

impl HostInfoProvider for ReplayHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        match query {
            CellInfoQuery::Filename => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "[Book1]Sheet1".encode_utf16().collect(),
            ))),
            _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        match query {
            InfoQuery::Directory => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "C:\\Work".encode_utf16().collect(),
            ))),
            _ => Err(HostInfoError::UnsupportedInfoQuery(query)),
        }
    }
}
