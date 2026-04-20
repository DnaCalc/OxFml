use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfunc_core::host_info::{
    CellInfoQuery, HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, InfoQuery,
    ResolvedWebImage,
};
use oxfunc_core::value::{
    CellStyleHint, EvalValue, ExcelText, NumberFormatHint, PresentationHint, ReferenceLike,
};
use serde::Deserialize;

use oxfml_core::format::en_us_context;
use oxfml_core::seam::{AcceptDecision, TraceEventKind};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::test_support::host::{EmpiricalOracleScenario, SingleFormulaHost};
use oxfml_core::{
    EvaluationBackend, FenceSnapshot, InMemoryLibraryContextProvider, LibraryContextSnapshotRef,
    ReturnedValueSurfaceKind,
};

#[test]
fn single_formula_host_recalc_updates_defined_name_inputs() {
    let mut host = SingleFormulaHost::new("host:sum", "=SUM(InputValue,2)");
    host.set_defined_name_value("InputValue", EvalValue::Number(5.0));
    let first = host
        .recalc(None, Some(&en_us_context()))
        .expect("first recalc");
    assert!(!first.artifact_reuse.green_tree_reused);
    assert!(!first.artifact_reuse.red_projection_reused);
    assert!(!first.artifact_reuse.bound_formula_reused);
    assert!(!first.artifact_reuse.semantic_plan_reused);
    match &first.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("7".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted first recalc"),
    }

    host.set_defined_name_value("InputValue", EvalValue::Number(8.0));
    let second = host
        .recalc(None, Some(&en_us_context()))
        .expect("second recalc");
    assert!(second.artifact_reuse.green_tree_reused);
    assert!(second.artifact_reuse.red_projection_reused);
    assert!(second.artifact_reuse.bound_formula_reused);
    assert!(second.artifact_reuse.semantic_plan_reused);
    match &second.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("10".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted second recalc"),
    }
}

#[test]
fn single_formula_host_invalidates_bind_reuse_when_name_kind_changes() {
    let mut host = SingleFormulaHost::new("host:reuse-invalidate", "=SUM(InputValue,2)");
    host.set_defined_name_value("InputValue", EvalValue::Number(5.0));
    host.recalc(None, Some(&en_us_context()))
        .expect("first recalc should succeed");

    host.set_defined_name_reference(
        "InputValue",
        ReferenceLike {
            kind: oxfunc_core::value::ReferenceKind::A1,
            target: "A1".to_string(),
        },
    );
    host.set_cell_value("A1", EvalValue::Number(5.0));
    let run = host
        .recalc(None, Some(&en_us_context()))
        .expect("second recalc should succeed");

    assert!(run.artifact_reuse.green_tree_reused);
    assert!(run.artifact_reuse.red_projection_reused);
    assert!(!run.artifact_reuse.bound_formula_reused);
    assert!(!run.artifact_reuse.semantic_plan_reused);
}

#[test]
fn single_formula_host_captures_candidate_and_commit_trace() {
    let mut host = SingleFormulaHost::new("host:text", "=TEXT(1234.567,\"0.00\")");
    let run = host
        .recalc(None, Some(&en_us_context()))
        .expect("recalc should succeed");
    assert_eq!(run.trace_events.len(), 2);
    assert_eq!(
        run.trace_events[0].event_kind,
        TraceEventKind::AcceptedCandidateResultBuilt
    );
    assert_eq!(
        run.trace_events[1].event_kind,
        TraceEventKind::CommitAccepted
    );
}

#[test]
fn single_formula_host_runs_host_query_formula() {
    let mut host = SingleFormulaHost::new("host:info", "=INFO(\"directory\")");
    let run = host
        .recalc(Some(&MockHostInfoProvider), Some(&en_us_context()))
        .expect("recalc should succeed");
    assert_eq!(
        run.candidate_result
            .display_delta
            .as_ref()
            .map(|delta| delta.display_effect_class.as_str()),
        Some("host_query_surface")
    );
    match &run.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Text("C:\\Work".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted host-query recalc"),
    }
}

#[test]
fn single_formula_host_preserves_hyperlink_publication_intent() {
    let mut host = SingleFormulaHost::new(
        "host:hyperlink",
        "=HYPERLINK(\"https://example.com\",\"Go\")",
    );
    let run = host
        .recalc(None, Some(&en_us_context()))
        .expect("recalc should succeed");

    assert_eq!(
        run.returned_value_surface.kind,
        ReturnedValueSurfaceKind::ValueWithPresentation
    );
    assert_eq!(
        run.returned_value_surface.presentation_hint,
        Some(PresentationHint::style(CellStyleHint::Hyperlink))
    );
    assert_eq!(
        run.candidate_result.returned_value_surface.kind,
        ReturnedValueSurfaceKind::ValueWithPresentation
    );
    match &run.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Text("Go".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted hyperlink recalc"),
    }
}

#[test]
fn single_formula_host_preserves_today_presentation_hint() {
    let mut host = SingleFormulaHost::new("host:today", "=TODAY()");
    let run = host
        .recalc(None, Some(&en_us_context()))
        .expect("recalc should succeed");

    assert_eq!(
        run.returned_value_surface.kind,
        ReturnedValueSurfaceKind::ValueWithPresentation
    );
    assert_eq!(
        run.returned_value_surface.presentation_hint,
        Some(PresentationHint::number_format(NumberFormatHint::DateLike))
    );
    assert_eq!(
        run.candidate_result.returned_value_surface.kind,
        ReturnedValueSurfaceKind::ValueWithPresentation
    );
    match &run.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.returned_value_surface.kind,
                ReturnedValueSurfaceKind::ValueWithPresentation
            );
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("46000".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted today recalc"),
    }
}

#[test]
fn single_formula_host_preserves_image_rich_value_surface() {
    let mut host = SingleFormulaHost::new(
        "host:image",
        "=IMAGE(\"https://example.com/sphere.png\",\"Sphere\")",
    );
    let run = host
        .recalc(Some(&ImageHostInfoProvider), Some(&en_us_context()))
        .expect("recalc should succeed");

    assert_eq!(
        run.returned_value_surface.kind,
        ReturnedValueSurfaceKind::RichValue
    );
    assert_eq!(
        run.returned_value_surface.rich_value_type_name.as_deref(),
        Some("_webimage")
    );
    assert_eq!(
        run.candidate_result.returned_value_surface.kind,
        ReturnedValueSurfaceKind::RichValue
    );
    match &run.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.returned_value_surface.kind,
                ReturnedValueSurfaceKind::RichValue
            );
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Text("-2146826273".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted image recalc"),
    }
}

#[test]
fn single_formula_host_supports_local_bootstrap_backend_for_basic_formulae() {
    let mut host = SingleFormulaHost::new("host:bootstrap", "=InputValue+2");
    host.set_defined_name_value("InputValue", EvalValue::Number(5.0));
    let run = host
        .recalc_with_backend(
            EvaluationBackend::LocalBootstrap,
            None,
            Some(&en_us_context()),
        )
        .expect("bootstrap recalc should succeed");
    match &run.commit_decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("7".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted bootstrap recalc"),
    }
}

#[test]
fn single_formula_host_rejects_execution_when_syntax_diagnostics_exist() {
    let mut host = SingleFormulaHost::new("host:syntax-reject", "=1~2");
    let error = host
        .recalc(None, Some(&en_us_context()))
        .expect_err("unsupported syntax should reject execution");

    assert!(error.contains("syntax diagnostics"));
    assert!(error.contains("Unknown"));
}

#[test]
fn single_formula_host_can_capture_commit_reject_trace() {
    let mut host = SingleFormulaHost::new("host:reject", "=SUM(InputValue,2)");
    host.set_defined_name_value("InputValue", EvalValue::Number(5.0));
    let run = host
        .recalc_with_observed_fence_override(
            None,
            Some(&en_us_context()),
            FenceSnapshot {
                formula_token: "mismatch".to_string(),
                snapshot_epoch: "epoch:1".to_string(),
                bind_hash: "bind:override".to_string(),
                profile_version: "profile:override".to_string(),
                capability_view_key: Some("cap:override".to_string()),
            },
        )
        .expect("reject recalc should still produce output");
    match &run.commit_decision {
        AcceptDecision::Rejected(reject) => {
            assert_eq!(reject.reject_code, oxfml_core::RejectCode::FenceMismatch);
            assert_eq!(run.trace_events.len(), 3);
            assert_eq!(
                run.trace_events[1].event_kind,
                TraceEventKind::CommitRejected
            );
            assert_eq!(run.trace_events[2].event_kind, TraceEventKind::RejectIssued);
        }
        AcceptDecision::Accepted(_) => panic!("expected rejected override recalc"),
    }
}

#[test]
fn empirical_oracle_scenarios_deserialize_in_expected_shape() {
    let scenarios = load_empirical_scenarios();
    assert_eq!(scenarios.len(), 7);
    assert_eq!(scenarios[0].scenario_id, "oracle_001_text");
    assert_eq!(scenarios[6].scenario_id, "oracle_007_cell_filename");
}

#[test]
fn empirical_oracle_scenarios_execute_through_single_formula_host() {
    let scenarios = load_empirical_scenarios();
    for scenario in &scenarios {
        let host_info = scenario
            .host_query_profile
            .as_deref()
            .map(|_| &MockHostInfoProvider as &dyn HostInfoProvider);
        let run = SingleFormulaHost::run_empirical_oracle_scenario(
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
        .expect("empirical scenario should execute");
        assert_eq!(
            run.evaluation.result.payload_summary, scenario.expected_result_summary,
            "unexpected empirical result for {}",
            scenario.scenario_id
        );
        let actual_trace_kinds = run
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_trace_kinds, scenario.expected_trace_event_kinds,
            "unexpected empirical trace kinds for {}",
            scenario.scenario_id
        );
        let actual_capability_effect_kinds = run
            .candidate_result
            .topology_delta
            .capability_effect_facts
            .iter()
            .map(|fact| fact.capability_kind.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_capability_effect_kinds, scenario.expected_capability_effect_kinds,
            "unexpected empirical capability effects for {}",
            scenario.scenario_id
        );
        let actual_format_dependency_tokens = run
            .candidate_result
            .topology_delta
            .format_dependency_facts
            .iter()
            .map(|fact| fact.dependency_token.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_format_dependency_tokens, scenario.expected_format_dependency_tokens,
            "unexpected empirical format dependencies for {}",
            scenario.scenario_id
        );
        let actual_spill_event_kinds = run
            .candidate_result
            .spill_events
            .iter()
            .map(|event| format!("{:?}", event.spill_event_kind))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_spill_event_kinds, scenario.expected_spill_event_kinds,
            "unexpected empirical spill events for {}",
            scenario.scenario_id
        );
        let actual_format_delta_classes = run
            .candidate_result
            .format_delta
            .iter()
            .map(|delta| delta.format_effect_class.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_format_delta_classes, scenario.expected_format_delta_classes,
            "unexpected empirical format deltas for {}",
            scenario.scenario_id
        );
        let actual_display_delta_classes = run
            .candidate_result
            .display_delta
            .iter()
            .map(|delta| delta.display_effect_class.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_display_delta_classes, scenario.expected_display_delta_classes,
            "unexpected empirical display deltas for {}",
            scenario.scenario_id
        );
    }
}

#[test]
fn locale_sensitive_host_run_surfaces_format_dependency_fact() {
    let mut host = SingleFormulaHost::new("host:locale", "=TEXT(InputValue,\"0.00\")");
    host.set_defined_name_value("InputValue", EvalValue::Number(12.5));
    let run = host
        .recalc(None, Some(&en_us_context()))
        .expect("recalc should succeed");
    assert_eq!(
        run.candidate_result
            .topology_delta
            .format_dependency_facts
            .len(),
        1
    );
    assert_eq!(
        run.candidate_result.topology_delta.format_dependency_facts[0].dependency_token,
        "locale_format_context"
    );
    assert_eq!(
        run.candidate_result
            .format_delta
            .as_ref()
            .map(|delta| delta.format_effect_class.as_str()),
        Some("locale_format_semantics")
    );
}

#[test]
fn single_formula_host_executes_text_with_scientific_format_pattern_ftc_0655() {
    let mut host = SingleFormulaHost::new(
        "host:scientific-text",
        "=TEXT(12345.6789,\"0.00E+00\")",
    );
    let run = host
        .recalc(None, Some(&en_us_context()))
        .expect("recalc should succeed");

    assert_eq!(
        run.published_worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("1.23E+04"))
    );
    assert_eq!(run.evaluation.result.payload_summary, "Text(1.23E+04)");
    assert_eq!(run.returned_value_surface.payload_summary, "Text");
}

#[test]
fn single_formula_host_uses_pinned_snapshot_ref_over_provider_current_snapshot() {
    let pinned_snapshot = host_library_context_snapshot_v1();
    let selected_snapshot_ref = LibraryContextSnapshotRef::from(&pinned_snapshot);
    let current_snapshot = host_library_context_snapshot_v2();
    let current_snapshot_ref = LibraryContextSnapshotRef::from(&current_snapshot);
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        current_snapshot_ref,
        vec![pinned_snapshot, current_snapshot],
    );
    let locale = en_us_context();
    let bundle = oxfml_core::TypedContextQueryBundle::new(
        None,
        None,
        Some(&locale),
        Some(46000.0),
        Some(0.25),
    );
    let mut host = SingleFormulaHost::new("host:pinned-snapshot", "=TAKE({1,2},1)");

    let first = host
        .recalc_with_interfaces_and_snapshot_ref(
            EvaluationBackend::OxFuncBacked,
            bundle,
            Some(&provider),
            Some(&selected_snapshot_ref),
        )
        .expect("first recalc should succeed");

    assert_eq!(
        first.library_context_snapshot_ref,
        Some(selected_snapshot_ref.clone())
    );
    assert_eq!(
        first.semantic_plan.library_context_snapshot_ref,
        Some(selected_snapshot_ref.clone())
    );
    let take_first = first
        .semantic_plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "TAKE")
        .expect("TAKE availability summary");
    assert_eq!(take_first.metadata_status, None);
    assert_eq!(take_first.surface_stable_id.as_deref(), Some("FUNC.TAKE"));

    let current_snapshot_ref = LibraryContextSnapshotRef::new("host-runtime", "v2");
    let second = host
        .recalc_with_interfaces_and_snapshot_ref(
            EvaluationBackend::OxFuncBacked,
            bundle,
            Some(&provider),
            Some(&current_snapshot_ref),
        )
        .expect("second recalc should succeed");

    assert!(second.artifact_reuse.green_tree_reused);
    assert!(second.artifact_reuse.red_projection_reused);
    assert!(second.artifact_reuse.bound_formula_reused);
    assert!(!second.artifact_reuse.semantic_plan_reused);
    assert_eq!(
        second.library_context_snapshot_ref,
        Some(current_snapshot_ref.clone())
    );
    let take_second = second
        .semantic_plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "TAKE")
        .expect("TAKE availability summary");
    assert_eq!(
        take_second.metadata_status.as_deref(),
        Some("runtime_snapshot")
    );
    assert_eq!(
        take_second.surface_stable_id.as_deref(),
        Some("surface:take")
    );
}

fn load_empirical_scenarios() -> Vec<EmpiricalOracleScenarioWire> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("empirical_oracle_scenarios.json");
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

struct MockHostInfoProvider;

impl HostInfoProvider for MockHostInfoProvider {
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

struct ImageHostInfoProvider;

impl HostInfoProvider for ImageHostInfoProvider {
    fn query_image(&self, request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        assert_eq!(
            request.source.to_string_lossy(),
            "https://example.com/sphere.png"
        );
        Ok(ImageProviderResult::Image(ResolvedWebImage {
            web_image_identifier: "img-1".to_string(),
            published_fallback: ExcelText::from_interop_assignment("-2146826273"),
        }))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
    #[serde(default)]
    expected_format_delta_classes: Vec<String>,
    #[serde(default)]
    expected_display_delta_classes: Vec<String>,
}

fn host_library_context_snapshot_v1() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "host-runtime".to_string(),
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

fn host_library_context_snapshot_v2() -> LibraryContextSnapshot {
    let mut snapshot = host_library_context_snapshot_v1();
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
