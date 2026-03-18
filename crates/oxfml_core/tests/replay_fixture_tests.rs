use std::fs;
use std::path::PathBuf;

use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::en_us_context;
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceLike};
use serde::Deserialize;

use oxfml_core::ExecutionProfileSummary;
use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
use oxfml_core::eval::{DefinedNameBinding, EvaluationContext, evaluate_formula};
use oxfml_core::host::SingleFormulaHost;
use oxfml_core::red::project_red_view;
use oxfml_core::scheduler::{ExecutionRestriction, build_execution_contract};
use oxfml_core::seam::{
    AcceptDecision, AcceptedCandidateResult, CapabilityEffectFact, CommitRequest,
    DynamicReferenceFact, Extent, FenceSnapshot, Locus, RejectContext, ShapeDelta,
    ShapeOutcomeClass, SpillEvent, SpillEventKind, SpillFact, TopologyDelta, ValueDelta,
    ValuePayload, WorksheetValueClass, commit_candidate,
};
use oxfml_core::semantics::{
    CompileSemanticPlanRequest, EvaluationRequirement, FormulaDeterminismClass,
    FormulaThreadSafetyClass, FormulaVolatilityClass, compile_semantic_plan,
};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};

#[derive(Debug, Deserialize)]
struct SemanticPlanReplayFixture {
    case_id: String,
    formula: String,
    expected: SemanticPlanExpected,
}

#[derive(Debug, Deserialize)]
struct SemanticPlanExpected {
    function_ids: Vec<String>,
    evaluation_requirements: Vec<String>,
    execution_profile: SemanticExecutionExpected,
    helper_profile: SemanticHelperExpected,
    capability_requirements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SemanticExecutionExpected {
    thread_safety: String,
    volatility: String,
    determinism: String,
    requires_host_query: bool,
    requires_caller_context: bool,
    requires_reference_preservation: bool,
    uses_implicit_intersection: bool,
    uses_spill_reference: bool,
    requires_branch_laziness: bool,
    requires_fallback_laziness: bool,
    requires_serial_scheduler_lane: bool,
}

#[derive(Debug, Deserialize)]
struct SemanticHelperExpected {
    contains_let: bool,
    contains_lambda: bool,
    contains_lambda_invocation: bool,
    lambda_literal_count: usize,
    lambda_invocation_count: usize,
    max_lambda_arity: usize,
    lexical_capture_required: bool,
}

#[derive(Debug, Deserialize)]
struct FecCommitReplayFixture {
    case_id: String,
    observed_fence: FenceFixture,
    expected: FecCommitExpected,
}

#[derive(Debug, Deserialize)]
struct FenceFixture {
    formula_token: String,
    snapshot_epoch: String,
    bind_hash: String,
    profile_version: String,
    capability_view_key: String,
}

#[derive(Debug, Deserialize)]
struct FecCommitExpected {
    decision: String,
    published_payload: Option<String>,
    shape_outcome_class: Option<String>,
    spill_event_kind: Option<String>,
    reject_code: Option<String>,
    mismatch_member_kind: Option<String>,
    mismatch_class: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PreparedCallReplayFixture {
    case_id: String,
    formula: String,
    defined_names: std::collections::BTreeMap<String, String>,
    host_query_profile: Option<String>,
    expected: PreparedCallExpected,
}

#[derive(Debug, Deserialize)]
struct PreparedCallExpected {
    prepared_calls: Vec<PreparedCallExpectedCall>,
    result_class: String,
    result_structure_class: String,
    payload_summary: String,
    blankness_class: String,
    callable_profile: Option<String>,
    callable_profile_detail: Option<CallableProfileExpected>,
    deferred_reason: Option<String>,
    format_hint: Option<String>,
    publication_hint: Option<String>,
    capability_dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CallableProfileExpected {
    arity: usize,
    parameter_names: Vec<String>,
    capture_names: Vec<String>,
    body_kind: String,
}

#[derive(Debug, Deserialize)]
struct PreparedCallExpectedCall {
    function_id: String,
    prepared_arguments: Vec<PreparedArgumentExpected>,
}

#[derive(Debug, Deserialize)]
struct PreparedArgumentExpected {
    structure_class: String,
    source_class: String,
    evaluation_mode: String,
    blankness_class: String,
    caller_context_sensitive: bool,
    reference_target: Option<String>,
    opaque_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExecutionContractReplayFixture {
    case_id: String,
    formula: String,
    expected: ExecutionContractExpected,
}

#[derive(Debug, Deserialize)]
struct ExecutionContractExpected {
    lane_class: String,
    replay_sensitivity: String,
    single_flight_advisable: bool,
    restrictions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SingleFormulaHostReplayFixture {
    case_id: String,
    formula: String,
    backend: String,
    #[serde(default)]
    defined_names: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    cell_bindings: std::collections::BTreeMap<String, String>,
    host_query_profile: Option<String>,
    expected: SingleFormulaHostExpected,
}

#[derive(Debug, Deserialize)]
struct SingleFormulaHostExpected {
    payload_summary: String,
    commit_decision: String,
    trace_event_kinds: Vec<String>,
    #[serde(default)]
    capability_effect_kinds: Vec<String>,
    #[serde(default)]
    format_dependency_tokens: Vec<String>,
    #[serde(default)]
    spill_event_kinds: Vec<String>,
}

#[test]
fn semantic_plan_replay_fixtures_match_expected_snapshots() {
    let fixtures = load_semantic_plan_fixtures();
    for fixture in fixtures {
        let plan = compile(&fixture.formula);
        let mut actual_function_ids = plan
            .function_bindings
            .iter()
            .map(|binding| binding.function_id.to_string())
            .collect::<Vec<_>>();
        actual_function_ids.sort();
        let mut expected_function_ids = fixture.expected.function_ids.clone();
        expected_function_ids.sort();
        assert_eq!(
            actual_function_ids, expected_function_ids,
            "function id mismatch for {}",
            fixture.case_id
        );
        let mut actual_requirements = plan
            .evaluation_requirements
            .iter()
            .map(eval_requirement_name)
            .collect::<Vec<_>>();
        actual_requirements.sort();
        let mut expected_requirements = fixture.expected.evaluation_requirements.clone();
        expected_requirements.sort();
        assert_eq!(
            actual_requirements, expected_requirements,
            "evaluation requirement mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            profile_thread_safety_name(plan.execution_profile.thread_safety),
            fixture.expected.execution_profile.thread_safety,
            "thread safety mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            profile_volatility_name(plan.execution_profile.volatility),
            fixture.expected.execution_profile.volatility,
            "volatility mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            profile_determinism_name(plan.execution_profile.determinism),
            fixture.expected.execution_profile.determinism,
            "determinism mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.requires_host_query,
            fixture.expected.execution_profile.requires_host_query,
            "host-query mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.requires_caller_context,
            fixture.expected.execution_profile.requires_caller_context,
            "caller-context mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.requires_reference_preservation,
            fixture
                .expected
                .execution_profile
                .requires_reference_preservation,
            "reference-preservation mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.uses_implicit_intersection,
            fixture
                .expected
                .execution_profile
                .uses_implicit_intersection,
            "implicit-intersection mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.uses_spill_reference,
            fixture.expected.execution_profile.uses_spill_reference,
            "spill-reference mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.requires_branch_laziness,
            fixture.expected.execution_profile.requires_branch_laziness,
            "branch-lazy mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.requires_fallback_laziness,
            fixture
                .expected
                .execution_profile
                .requires_fallback_laziness,
            "fallback-lazy mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.execution_profile.requires_serial_scheduler_lane,
            fixture
                .expected
                .execution_profile
                .requires_serial_scheduler_lane,
            "serial-lane mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.contains_let, fixture.expected.helper_profile.contains_let,
            "helper contains-let mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.contains_lambda, fixture.expected.helper_profile.contains_lambda,
            "helper contains-lambda mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.contains_lambda_invocation,
            fixture.expected.helper_profile.contains_lambda_invocation,
            "helper contains-lambda-invocation mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.lambda_literal_count,
            fixture.expected.helper_profile.lambda_literal_count,
            "helper lambda-literal-count mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.lambda_invocation_count,
            fixture.expected.helper_profile.lambda_invocation_count,
            "helper lambda-invocation-count mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.max_lambda_arity, fixture.expected.helper_profile.max_lambda_arity,
            "helper max-lambda-arity mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.helper_profile.lexical_capture_required,
            fixture.expected.helper_profile.lexical_capture_required,
            "helper lexical-capture mismatch for {}",
            fixture.case_id
        );
        let mut actual_capability_requirements = plan.capability_requirements.clone();
        actual_capability_requirements.sort();
        let mut expected_capability_requirements = fixture.expected.capability_requirements.clone();
        expected_capability_requirements.sort();
        assert_eq!(
            actual_capability_requirements, expected_capability_requirements,
            "capability requirements mismatch for {}",
            fixture.case_id
        );
    }
}

#[test]
fn fec_commit_replay_fixtures_match_expected_snapshots() {
    let fixtures = load_fec_commit_fixtures();
    for fixture in fixtures {
        let candidate = sample_candidate();
        let decision = commit_candidate(CommitRequest {
            candidate_result: candidate,
            commit_attempt_id: format!("commit:{}", fixture.case_id),
            observed_fence: FenceSnapshot {
                formula_token: fixture.observed_fence.formula_token,
                snapshot_epoch: fixture.observed_fence.snapshot_epoch,
                bind_hash: fixture.observed_fence.bind_hash,
                profile_version: fixture.observed_fence.profile_version,
                capability_view_key: Some(fixture.observed_fence.capability_view_key),
            },
        });

        match decision {
            AcceptDecision::Accepted(bundle) => {
                assert_eq!(fixture.expected.decision, "accepted");
                assert_eq!(
                    Some(value_payload_name(&bundle.value_delta.published_payload)),
                    fixture.expected.published_payload,
                    "published payload mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    Some(shape_outcome_name(bundle.shape_delta.shape_outcome_class)),
                    fixture.expected.shape_outcome_class,
                    "shape outcome mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    bundle
                        .spill_events
                        .first()
                        .map(|event| spill_event_name(event.spill_event_kind)),
                    fixture.expected.spill_event_kind,
                    "spill event mismatch for {}",
                    fixture.case_id
                );
            }
            AcceptDecision::Rejected(reject) => {
                assert_eq!(fixture.expected.decision, "rejected");
                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code,
                    "reject code mismatch for {}",
                    fixture.case_id
                );
                let RejectContext::FenceMismatch(context) = reject.context else {
                    panic!("expected fence mismatch context for {}", fixture.case_id);
                };
                assert_eq!(
                    Some(context.mismatch_member_kind),
                    fixture.expected.mismatch_member_kind,
                    "mismatch member mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    Some(context.mismatch_class),
                    fixture.expected.mismatch_class,
                    "mismatch class mismatch for {}",
                    fixture.case_id
                );
            }
        }
    }
}

#[test]
fn prepared_call_replay_fixtures_match_expected_snapshots() {
    let fixtures = load_prepared_call_fixtures();
    for fixture in fixtures {
        let output = evaluate_fixture_formula(
            &fixture.formula,
            &fixture.defined_names,
            fixture.host_query_profile.as_deref(),
        );
        assert_eq!(
            prepared_result_class_name(output.result.result_class),
            fixture.expected.result_class,
            "result class mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            prepared_structure_class_name(output.result.structure_class),
            fixture.expected.result_structure_class,
            "result structure mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.result.payload_summary, fixture.expected.payload_summary,
            "payload summary mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            prepared_blankness_class_name(output.result.blankness_class),
            fixture.expected.blankness_class,
            "result blankness mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.result.callable_profile, fixture.expected.callable_profile,
            "callable profile mismatch for {}",
            fixture.case_id
        );
        match (
            output.result.callable_profile_detail.as_ref(),
            fixture.expected.callable_profile_detail.as_ref(),
        ) {
            (Some(actual), Some(expected)) => {
                assert_eq!(
                    actual.arity, expected.arity,
                    "callable arity mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    actual.parameter_names, expected.parameter_names,
                    "callable parameter names mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    actual.capture_names, expected.capture_names,
                    "callable capture names mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    actual.body_kind, expected.body_kind,
                    "callable body kind mismatch for {}",
                    fixture.case_id
                );
            }
            (None, None) => {}
            _ => panic!(
                "callable profile detail presence mismatch for {}",
                fixture.case_id
            ),
        }
        assert_eq!(
            output.result.deferred_reason, fixture.expected.deferred_reason,
            "deferred reason mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.result.format_hint, fixture.expected.format_hint,
            "format hint mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.result.publication_hint, fixture.expected.publication_hint,
            "publication hint mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.result.capability_dependencies, fixture.expected.capability_dependencies,
            "result capability dependency mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.trace.prepared_calls.len(),
            fixture.expected.prepared_calls.len(),
            "prepared call count mismatch for {}",
            fixture.case_id
        );
        for (actual_call, expected_call) in output
            .trace
            .prepared_calls
            .iter()
            .zip(fixture.expected.prepared_calls.iter())
        {
            assert_eq!(
                actual_call.function_id, expected_call.function_id,
                "function id mismatch for {}",
                fixture.case_id
            );
            assert_eq!(
                actual_call.prepared_arguments.len(),
                expected_call.prepared_arguments.len(),
                "prepared arg count mismatch for {}",
                fixture.case_id
            );
            for (actual_arg, expected_arg) in actual_call
                .prepared_arguments
                .iter()
                .zip(expected_call.prepared_arguments.iter())
            {
                assert_eq!(
                    prepared_structure_class_name(actual_arg.structure_class),
                    expected_arg.structure_class,
                    "prepared arg structure mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    prepared_source_class_name(actual_arg.source_class),
                    expected_arg.source_class,
                    "prepared arg source mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    prepared_evaluation_mode_name(actual_arg.evaluation_mode),
                    expected_arg.evaluation_mode,
                    "prepared arg mode mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    prepared_blankness_class_name(actual_arg.blankness_class),
                    expected_arg.blankness_class,
                    "prepared arg blankness mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    actual_arg.caller_context_sensitive, expected_arg.caller_context_sensitive,
                    "prepared arg caller-context sensitivity mismatch for {}",
                    fixture.case_id
                );
                if expected_arg.reference_target.is_some() {
                    assert_eq!(
                        actual_arg.reference_target, expected_arg.reference_target,
                        "prepared arg reference target mismatch for {}",
                        fixture.case_id
                    );
                }
                if expected_arg.opaque_reason.is_some() {
                    assert_eq!(
                        actual_arg.opaque_reason, expected_arg.opaque_reason,
                        "prepared arg opaque reason mismatch for {}",
                        fixture.case_id
                    );
                }
            }
        }
    }
}

#[test]
fn execution_contract_replay_fixtures_match_expected_snapshots() {
    let fixtures = load_execution_contract_fixtures();
    for fixture in fixtures {
        let plan = compile(&fixture.formula);
        let contract = build_execution_contract(&plan);
        let mut actual_restrictions = contract
            .restrictions
            .iter()
            .map(execution_restriction_name)
            .collect::<Vec<_>>();
        actual_restrictions.sort();
        let mut expected_restrictions = fixture.expected.restrictions.clone();
        expected_restrictions.sort();
        assert_eq!(
            scheduler_lane_name(contract.lane_class),
            fixture.expected.lane_class,
            "lane class mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            replay_sensitivity_name(contract.replay_sensitivity),
            fixture.expected.replay_sensitivity,
            "replay sensitivity mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            contract.single_flight_advisable, fixture.expected.single_flight_advisable,
            "single-flight mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            actual_restrictions, expected_restrictions,
            "restriction mismatch for {}",
            fixture.case_id
        );
    }
}

#[test]
fn single_formula_host_replay_fixtures_match_expected_snapshots() {
    let fixtures = load_single_formula_host_fixtures();
    for fixture in fixtures {
        let mut host =
            SingleFormulaHost::new(format!("host:{}", fixture.case_id), &fixture.formula);
        for (name, value) in &fixture.defined_names {
            apply_defined_name_summary(&mut host, name, value);
        }
        for (target, value) in &fixture.cell_bindings {
            host.set_cell_value(target, parse_eval_value_summary(value));
        }
        let host_info = fixture
            .host_query_profile
            .as_deref()
            .map(|_| &ReplayHostInfoProvider as &dyn HostInfoProvider);
        let output = match fixture.backend.as_str() {
            "OxFuncBacked" => host.recalc(host_info, Some(&en_us_context())),
            "LocalBootstrap" => host.recalc_with_backend(
                oxfml_core::EvaluationBackend::LocalBootstrap,
                host_info,
                Some(&en_us_context()),
            ),
            other => panic!("unexpected backend {other}"),
        }
        .expect("host replay fixture should execute");

        assert_eq!(
            output.evaluation.result.payload_summary, fixture.expected.payload_summary,
            "payload mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            accept_decision_name(&output.commit_decision),
            fixture.expected.commit_decision,
            "decision mismatch for {}",
            fixture.case_id
        );
        let actual_trace_kinds = output
            .trace_events
            .iter()
            .map(|event| trace_event_name(event.event_kind))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_trace_kinds, fixture.expected.trace_event_kinds,
            "trace kind mismatch for {}",
            fixture.case_id
        );
        let actual_capability_effect_kinds = output
            .candidate_result
            .topology_delta
            .capability_effect_facts
            .iter()
            .map(|fact| fact.capability_kind.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_capability_effect_kinds, fixture.expected.capability_effect_kinds,
            "capability effect mismatch for {}",
            fixture.case_id
        );
        let actual_format_dependency_tokens = output
            .candidate_result
            .topology_delta
            .format_dependency_facts
            .iter()
            .map(|fact| fact.dependency_token.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_format_dependency_tokens, fixture.expected.format_dependency_tokens,
            "format dependency mismatch for {}",
            fixture.case_id
        );
        let actual_spill_event_kinds = output
            .candidate_result
            .spill_events
            .iter()
            .map(|event| spill_event_name(event.spill_event_kind))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_spill_event_kinds, fixture.expected.spill_event_kinds,
            "spill event mismatch for {}",
            fixture.case_id
        );
    }
}

fn compile(formula: &str) -> oxfml_core::SemanticPlan {
    let source = FormulaSourceRecord::new("replay-fixture", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("replay-struct-v1".to_string()),
            ..BindContext::default()
        },
    });

    compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula,
        oxfunc_catalog_identity: "oxfunc:fixture".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan
}

fn sample_candidate() -> AcceptedCandidateResult {
    let primary_locus = Locus {
        sheet_id: "sheet:default".to_string(),
        row: 1,
        col: 1,
    };

    AcceptedCandidateResult {
        formula_stable_id: "formula:001".to_string(),
        session_id: Some("session:001".to_string()),
        candidate_result_id: "candidate:001".to_string(),
        fence_snapshot: FenceSnapshot {
            formula_token: "token:v1".to_string(),
            snapshot_epoch: "epoch:10".to_string(),
            bind_hash: "bind:abc".to_string(),
            profile_version: "profile:v1".to_string(),
            capability_view_key: Some("cap:view:v1".to_string()),
        },
        value_delta: ValueDelta {
            formula_stable_id: "formula:001".to_string(),
            primary_locus: primary_locus.clone(),
            affected_value_loci: vec![primary_locus.clone()],
            published_value_class: WorksheetValueClass::Scalar,
            published_payload: ValuePayload::Number("42".to_string()),
            result_extent: Some(Extent { rows: 1, cols: 1 }),
            candidate_result_id: Some("candidate:001".to_string()),
        },
        shape_delta: ShapeDelta {
            formula_stable_id: "formula:001".to_string(),
            anchor_locus: primary_locus.clone(),
            intended_extent: Extent { rows: 1, cols: 1 },
            published_extent: Some(Extent { rows: 1, cols: 1 }),
            blocked_loci: Vec::new(),
            shape_outcome_class: ShapeOutcomeClass::Established,
            candidate_result_id: Some("candidate:001".to_string()),
        },
        topology_delta: TopologyDelta {
            formula_stable_id: "formula:001".to_string(),
            dependency_additions: vec!["name:InputA".to_string()],
            dependency_removals: Vec::new(),
            dependency_reclassifications: Vec::new(),
            dependency_consequence_facts: Vec::new(),
            dynamic_reference_facts: vec![DynamicReferenceFact {
                formula_stable_id: "formula:001".to_string(),
                discovery_site: "OFFSET".to_string(),
                reference_identity: Some("ref:A1:B2".to_string()),
                target_extent: Some(Extent { rows: 2, cols: 2 }),
                resolution_failure_class: None,
            }],
            spill_facts: vec![SpillFact {
                formula_stable_id: "formula:001".to_string(),
                anchor_locus: primary_locus.clone(),
                intended_extent: Extent { rows: 1, cols: 1 },
                published_extent: Some(Extent { rows: 1, cols: 1 }),
                blocked_loci: Vec::new(),
                blocked_reason_class: None,
            }],
            format_dependency_facts: Vec::new(),
            capability_effect_facts: vec![CapabilityEffectFact {
                formula_stable_id: "formula:001".to_string(),
                capability_kind: "host_query".to_string(),
                phase_kind: "execute".to_string(),
                effect_class: "admitted".to_string(),
                fallback_class: None,
            }],
            candidate_result_id: Some("candidate:001".to_string()),
        },
        format_delta: None,
        display_delta: None,
        spill_events: vec![SpillEvent {
            spill_event_kind: SpillEventKind::SpillTakeover,
            formula_stable_id: "formula:001".to_string(),
            anchor_locus: primary_locus,
            intended_extent: Extent { rows: 1, cols: 1 },
            affected_extent: Some(Extent { rows: 1, cols: 1 }),
            blocking_loci: Vec::new(),
            blocking_reason_class: None,
            correlation_id: "candidate:001".to_string(),
        }],
        execution_profile: Some(ExecutionProfileSummary::default()),
        trace_correlation_id: "trace:001".to_string(),
    }
}

fn load_semantic_plan_fixtures() -> Vec<SemanticPlanReplayFixture> {
    load_fixture("semantic_plan_replay_cases.json")
}

fn load_fec_commit_fixtures() -> Vec<FecCommitReplayFixture> {
    load_fixture("fec_commit_replay_cases.json")
}

fn load_prepared_call_fixtures() -> Vec<PreparedCallReplayFixture> {
    load_fixture("prepared_call_replay_cases.json")
}

fn load_execution_contract_fixtures() -> Vec<ExecutionContractReplayFixture> {
    load_fixture("execution_contract_replay_cases.json")
}

fn load_single_formula_host_fixtures() -> Vec<SingleFormulaHostReplayFixture> {
    load_fixture("single_formula_host_replay_cases.json")
}

fn load_fixture<T: for<'de> Deserialize<'de>>(file_name: &str) -> T {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(file_name);
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn eval_requirement_name(requirement: &EvaluationRequirement) -> String {
    match requirement {
        EvaluationRequirement::BranchLazy => "BranchLazy",
        EvaluationRequirement::FallbackLazy => "FallbackLazy",
        EvaluationRequirement::ReferencePreservedCalls => "ReferencePreservedCalls",
        EvaluationRequirement::CallerContextSensitive => "CallerContextSensitive",
        EvaluationRequirement::HostQueryCapability => "HostQueryCapability",
        EvaluationRequirement::LocaleAware => "LocaleAware",
        EvaluationRequirement::ImplicitIntersection => "ImplicitIntersection",
        EvaluationRequirement::SpillReference => "SpillReference",
        EvaluationRequirement::ReferenceExpression => "ReferenceExpression",
        EvaluationRequirement::ExternalReferenceDeferred => "ExternalReferenceDeferred",
        EvaluationRequirement::LegacySingleCompat => "LegacySingleCompat",
        EvaluationRequirement::HelperEnvironment => "HelperEnvironment",
    }
    .to_string()
}

fn profile_thread_safety_name(class: FormulaThreadSafetyClass) -> String {
    match class {
        FormulaThreadSafetyClass::SafeConcurrent => "SafeConcurrent",
        FormulaThreadSafetyClass::HostSerialized => "HostSerialized",
        FormulaThreadSafetyClass::NotThreadSafe => "NotThreadSafe",
    }
    .to_string()
}

fn profile_volatility_name(class: FormulaVolatilityClass) -> String {
    match class {
        FormulaVolatilityClass::Stable => "Stable",
        FormulaVolatilityClass::Contextual => "Contextual",
        FormulaVolatilityClass::Volatile => "Volatile",
    }
    .to_string()
}

fn profile_determinism_name(class: FormulaDeterminismClass) -> String {
    match class {
        FormulaDeterminismClass::Deterministic => "Deterministic",
        FormulaDeterminismClass::Contextual => "Contextual",
        FormulaDeterminismClass::NonDeterministic => "NonDeterministic",
    }
    .to_string()
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

fn prepared_result_class_name(class: oxfml_core::PreparedResultClass) -> String {
    match class {
        oxfml_core::PreparedResultClass::Scalar => "Scalar",
        oxfml_core::PreparedResultClass::Array => "Array",
        oxfml_core::PreparedResultClass::Reference => "Reference",
        oxfml_core::PreparedResultClass::Error => "Error",
    }
    .to_string()
}

fn prepared_structure_class_name(class: oxfml_core::PreparedStructureClass) -> String {
    match class {
        oxfml_core::PreparedStructureClass::DirectScalar => "DirectScalar",
        oxfml_core::PreparedStructureClass::ArrayLike => "ArrayLike",
        oxfml_core::PreparedStructureClass::ReferenceVisible => "ReferenceVisible",
        oxfml_core::PreparedStructureClass::Omitted => "Omitted",
    }
    .to_string()
}

fn prepared_source_class_name(class: oxfml_core::PreparedSourceClass) -> String {
    match class {
        oxfml_core::PreparedSourceClass::Literal => "Literal",
        oxfml_core::PreparedSourceClass::HelperParameter => "HelperParameter",
        oxfml_core::PreparedSourceClass::FunctionCall => "FunctionCall",
        oxfml_core::PreparedSourceClass::CellReference => "CellReference",
        oxfml_core::PreparedSourceClass::AreaReference => "AreaReference",
        oxfml_core::PreparedSourceClass::WholeRowReference => "WholeRowReference",
        oxfml_core::PreparedSourceClass::WholeColumnReference => "WholeColumnReference",
        oxfml_core::PreparedSourceClass::NameReference => "NameReference",
        oxfml_core::PreparedSourceClass::ExternalReference => "ExternalReference",
        oxfml_core::PreparedSourceClass::SpillReference => "SpillReference",
        oxfml_core::PreparedSourceClass::ImplicitIntersection => "ImplicitIntersection",
        oxfml_core::PreparedSourceClass::BinaryExpression => "BinaryExpression",
    }
    .to_string()
}

fn prepared_evaluation_mode_name(mode: oxfml_core::PreparedEvaluationMode) -> String {
    match mode {
        oxfml_core::PreparedEvaluationMode::EagerValue => "EagerValue",
        oxfml_core::PreparedEvaluationMode::ReferencePreserved => "ReferencePreserved",
        oxfml_core::PreparedEvaluationMode::CallerContextScalarized => "CallerContextScalarized",
    }
    .to_string()
}

fn prepared_blankness_class_name(class: oxfml_core::PreparedBlanknessClass) -> String {
    match class {
        oxfml_core::PreparedBlanknessClass::NonBlank => "NonBlank",
        oxfml_core::PreparedBlanknessClass::Omitted => "Omitted",
        oxfml_core::PreparedBlanknessClass::EmptyCell => "EmptyCell",
        oxfml_core::PreparedBlanknessClass::EmptyText => "EmptyText",
    }
    .to_string()
}

fn scheduler_lane_name(class: oxfml_core::SchedulerLaneClass) -> String {
    match class {
        oxfml_core::SchedulerLaneClass::ConcurrentSafe => "ConcurrentSafe",
        oxfml_core::SchedulerLaneClass::Serialized => "Serialized",
    }
    .to_string()
}

fn replay_sensitivity_name(class: oxfml_core::ReplaySensitivityClass) -> String {
    match class {
        oxfml_core::ReplaySensitivityClass::Stable => "Stable",
        oxfml_core::ReplaySensitivityClass::ContextSensitive => "ContextSensitive",
        oxfml_core::ReplaySensitivityClass::NonDeterministic => "NonDeterministic",
    }
    .to_string()
}

fn execution_restriction_name(restriction: &ExecutionRestriction) -> String {
    match restriction {
        ExecutionRestriction::HostSerialized => "HostSerialized",
        ExecutionRestriction::NotThreadSafe => "NotThreadSafe",
        ExecutionRestriction::ThreadAffine => "ThreadAffine",
        ExecutionRestriction::AsyncCoupled => "AsyncCoupled",
        ExecutionRestriction::SerialOnly => "SerialOnly",
        ExecutionRestriction::SingleFlightAdvisable => "SingleFlightAdvisable",
        ExecutionRestriction::Volatile => "Volatile",
        ExecutionRestriction::ContextualVolatility => "ContextualVolatility",
        ExecutionRestriction::HostQuery => "HostQuery",
        ExecutionRestriction::CallerContext => "CallerContext",
        ExecutionRestriction::ReferencePreserved => "ReferencePreserved",
        ExecutionRestriction::BranchLazy => "BranchLazy",
        ExecutionRestriction::FallbackLazy => "FallbackLazy",
        ExecutionRestriction::LocaleSensitive => "LocaleSensitive",
        ExecutionRestriction::PseudoRandom => "PseudoRandom",
        ExecutionRestriction::TimeDependent => "TimeDependent",
        ExecutionRestriction::ExternalEventDependent => "ExternalEventDependent",
    }
    .to_string()
}

fn accept_decision_name(decision: &AcceptDecision) -> String {
    match decision {
        AcceptDecision::Accepted(_) => "accepted",
        AcceptDecision::Rejected(_) => "rejected",
    }
    .to_string()
}

fn trace_event_name(kind: oxfml_core::TraceEventKind) -> String {
    match kind {
        oxfml_core::TraceEventKind::SessionOpened => "SessionOpened",
        oxfml_core::TraceEventKind::CapabilityViewEstablished => "CapabilityViewEstablished",
        oxfml_core::TraceEventKind::AcceptedCandidateResultBuilt => "AcceptedCandidateResultBuilt",
        oxfml_core::TraceEventKind::CommitAccepted => "CommitAccepted",
        oxfml_core::TraceEventKind::CommitRejected => "CommitRejected",
        oxfml_core::TraceEventKind::RejectIssued => "RejectIssued",
        oxfml_core::TraceEventKind::SessionAborted => "SessionAborted",
        oxfml_core::TraceEventKind::SessionExpired => "SessionExpired",
    }
    .to_string()
}

fn evaluate_fixture_formula(
    formula: &str,
    defined_names: &std::collections::BTreeMap<String, String>,
    host_query_profile: Option<&str>,
) -> oxfml_core::EvaluationOutput {
    let source = FormulaSourceRecord::new("prepared-replay", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);

    let mut name_kinds = std::collections::BTreeMap::new();
    let mut binding_map = std::collections::BTreeMap::new();
    for (name, value) in defined_names {
        name_kinds.insert(name.clone(), oxfml_core::binding::NameKind::ValueLike);
        binding_map.insert(name.clone(), parse_defined_name_summary(value));
    }

    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("prepared-replay-v1".to_string()),
            names: name_kinds,
            ..BindContext::default()
        },
    });

    let plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:fixture".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan;

    let mut context = EvaluationContext::new(&bind.bound_formula, &plan);
    let locale_ctx = en_us_context();
    context.locale_ctx = Some(&locale_ctx);
    context.defined_names = binding_map;
    context
        .cell_values
        .insert("A1".to_string(), EvalValue::Number(7.0));
    context
        .cell_values
        .insert("A2".to_string(), EvalValue::Number(11.0));
    context
        .cell_values
        .insert("B2".to_string(), EvalValue::Number(13.0));
    if host_query_profile.is_some() {
        context.host_info = Some(&ReplayHostInfoProvider);
    }
    context.now_serial = Some(46000.0);
    context.random_value = Some(0.25);

    evaluate_formula(context).expect("fixture evaluation should succeed")
}

fn parse_defined_name_summary(summary: &str) -> DefinedNameBinding {
    if let Some(target) = summary
        .strip_prefix("Reference(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return DefinedNameBinding::Reference(ReferenceLike {
            kind: oxfunc_core::value::ReferenceKind::A1,
            target: target.to_string(),
        });
    }

    DefinedNameBinding::Value(parse_eval_value_summary(summary))
}

fn parse_eval_value_summary(summary: &str) -> EvalValue {
    if let Some(number) = summary
        .strip_prefix("Number(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Number(number.parse::<f64>().expect("numeric fixture binding"));
    }

    if let Some(text) = summary
        .strip_prefix("Text(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ));
    }

    if let Some(logical) = summary
        .strip_prefix("Logical(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return match logical {
            "true" | "True" | "TRUE" => EvalValue::Logical(true),
            "false" | "False" | "FALSE" => EvalValue::Logical(false),
            _ => panic!("unsupported logical fixture binding {summary}"),
        };
    }

    panic!("unsupported eval-value summary {summary}");
}

fn apply_defined_name_summary(host: &mut SingleFormulaHost, name: &str, summary: &str) {
    match parse_defined_name_summary(summary) {
        DefinedNameBinding::Value(value) => host.set_defined_name_value(name, value),
        DefinedNameBinding::Reference(reference) => {
            host.set_defined_name_reference(name, reference)
        }
    }
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

fn shape_outcome_name(class: ShapeOutcomeClass) -> String {
    match class {
        ShapeOutcomeClass::Established => "Established",
        ShapeOutcomeClass::Reconfigured => "Reconfigured",
        ShapeOutcomeClass::Cleared => "Cleared",
        ShapeOutcomeClass::Blocked => "Blocked",
    }
    .to_string()
}

fn spill_event_name(kind: SpillEventKind) -> String {
    match kind {
        SpillEventKind::SpillTakeover => "SpillTakeover",
        SpillEventKind::SpillClearance => "SpillClearance",
        SpillEventKind::SpillBlocked => "SpillBlocked",
    }
    .to_string()
}

fn reject_code_name(code: oxfml_core::RejectCode) -> String {
    match code {
        oxfml_core::RejectCode::FenceMismatch => "FenceMismatch",
        oxfml_core::RejectCode::CapabilityDenied => "CapabilityDenied",
        oxfml_core::RejectCode::SessionTerminated => "SessionTerminated",
        oxfml_core::RejectCode::BindMismatch => "BindMismatch",
        oxfml_core::RejectCode::StructuralConflict => "StructuralConflict",
        oxfml_core::RejectCode::DynamicReferenceFailure => "DynamicReferenceFailure",
        oxfml_core::RejectCode::ResourceInvariantFailure => "ResourceInvariantFailure",
    }
    .to_string()
}
