use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfunc_core::locale_format::en_us_context;
use oxfunc_core::value::EvalValue;
use serde::Deserialize;

use oxfml_core::binding::{BindContext, BindRequest, NameKind, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::session::{
    CapabilityViewSpec, ExecuteRequest, PrepareRequest, SessionPhase, SessionService,
};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{
    AcceptDecision, DefinedNameBinding, EvaluationBackend, Locus, RejectCode, TraceEventKind,
    ValuePayload, compile_semantic_plan,
};

#[derive(Debug, Deserialize)]
struct SessionReplayFixture {
    case_id: String,
    formula: String,
    with_input_name: bool,
    capability_spec: CapabilitySpecFixture,
    action: String,
    expected: SessionReplayExpected,
}

#[derive(Debug, Deserialize)]
struct CapabilitySpecFixture {
    host_query_enabled: bool,
    locale_format_enabled: bool,
    caller_context_enabled: bool,
    external_provider_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct SessionReplayExpected {
    phase: String,
    decision: String,
    reject_code: Option<String>,
    published_payload: Option<String>,
    trace_event_kinds: Vec<String>,
}

#[test]
fn session_lifecycle_replay_fixtures_match_expected_snapshots() {
    for fixture in load_session_fixtures() {
        let prepared = compile_prepared(&fixture.formula, fixture.with_input_name);
        let mut service = SessionService::new();
        let prepared = service.prepare(prepared).expect("prepare should succeed");
        let open = service.open_session(prepared);

        match fixture.action.as_str() {
            "execute_commit" => {
                service
                    .establish_capability_view(&open.session_id, into_capability_spec(&fixture))
                    .expect("capability view should succeed");

                let mut defined_names = BTreeMap::new();
                if fixture.with_input_name {
                    defined_names.insert(
                        "InputValue".to_string(),
                        DefinedNameBinding::Value(EvalValue::Number(5.0)),
                    );
                }

                let candidate = service
                    .execute(ExecuteRequest {
                        session_id: open.session_id.clone(),
                        backend: EvaluationBackend::OxFuncBacked,
                        caller_row: 1,
                        caller_col: 1,
                        cell_values: BTreeMap::new(),
                        defined_names,
                        locale_ctx: Some(&en_us_context()),
                        host_info: None,
                        now_serial: Some(46000.0),
                        random_value: Some(0.25),
                    })
                    .expect("execute should succeed");

                let decision =
                    service.commit(&open.session_id, "commit:fixture", candidate.fence_snapshot);
                assert_eq!(
                    accept_decision_name(&decision),
                    fixture.expected.decision,
                    "decision mismatch for {}",
                    fixture.case_id
                );

                match decision {
                    AcceptDecision::Accepted(bundle) => assert_eq!(
                        fixture.expected.published_payload,
                        Some(value_payload_name(&bundle.value_delta.published_payload)),
                        "payload mismatch for {}",
                        fixture.case_id
                    ),
                    AcceptDecision::Rejected(reject) => panic!(
                        "expected accepted decision for {} but got {:?}",
                        fixture.case_id, reject.reject_code
                    ),
                }
            }
            "capability_only" => {
                let reject = service
                    .establish_capability_view(&open.session_id, into_capability_spec(&fixture))
                    .expect_err("capability view should reject");
                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code,
                    "reject code mismatch for {}",
                    fixture.case_id
                );
            }
            "abort_before_execute" => {
                let reject =
                    service.abort_session(&open.session_id, Some("manual_abort".to_string()));
                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code,
                    "reject code mismatch for {}",
                    fixture.case_id
                );
            }
            "execute_twice" => {
                service
                    .establish_capability_view(&open.session_id, into_capability_spec(&fixture))
                    .expect("capability view should succeed");
                let mut defined_names = BTreeMap::new();
                if fixture.with_input_name {
                    defined_names.insert(
                        "InputValue".to_string(),
                        DefinedNameBinding::Value(EvalValue::Number(5.0)),
                    );
                }
                service
                    .execute(ExecuteRequest {
                        session_id: open.session_id.clone(),
                        backend: EvaluationBackend::OxFuncBacked,
                        caller_row: 1,
                        caller_col: 1,
                        cell_values: BTreeMap::new(),
                        defined_names: defined_names.clone(),
                        locale_ctx: Some(&en_us_context()),
                        host_info: None,
                        now_serial: Some(46000.0),
                        random_value: Some(0.25),
                    })
                    .expect("first execute should succeed");
                let reject = service
                    .execute(ExecuteRequest {
                        session_id: open.session_id.clone(),
                        backend: EvaluationBackend::OxFuncBacked,
                        caller_row: 1,
                        caller_col: 1,
                        cell_values: BTreeMap::new(),
                        defined_names,
                        locale_ctx: Some(&en_us_context()),
                        host_info: None,
                        now_serial: Some(46000.0),
                        random_value: Some(0.25),
                    })
                    .expect_err("second execute should reject");
                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code,
                    "reject code mismatch for {}",
                    fixture.case_id
                );
            }
            "commit_without_execute" => {
                service
                    .establish_capability_view(&open.session_id, into_capability_spec(&fixture))
                    .expect("capability view should succeed");
                let decision = service.commit(
                    &open.session_id,
                    "commit_without_execute",
                    open.fence_snapshot,
                );
                assert_eq!(
                    accept_decision_name(&decision),
                    fixture.expected.decision,
                    "decision mismatch for {}",
                    fixture.case_id
                );
                let AcceptDecision::Rejected(reject) = decision else {
                    panic!("expected rejected decision for {}", fixture.case_id);
                };
                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code,
                    "reject code mismatch for {}",
                    fixture.case_id
                );
            }
            "expire_before_execute" => {
                let reject =
                    service.expire_session(&open.session_id, Some("ttl_expired".to_string()));
                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code,
                    "reject code mismatch for {}",
                    fixture.case_id
                );
            }
            other => panic!("unsupported action {other} for {}", fixture.case_id),
        }

        let session = service
            .session(&open.session_id)
            .expect("session should exist");
        assert_eq!(
            session_phase_name(&session.phase),
            fixture.expected.phase,
            "phase mismatch for {}",
            fixture.case_id
        );
        let actual_trace = session
            .trace_events
            .iter()
            .map(|event| trace_event_name(event.event_kind))
            .collect::<Vec<_>>();
        assert_eq!(
            actual_trace, fixture.expected.trace_event_kinds,
            "trace event mismatch for {}",
            fixture.case_id
        );
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load_session_fixtures() -> Vec<SessionReplayFixture> {
    let path = fixture_path("session_lifecycle_replay_cases.json");
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn into_capability_spec(fixture: &SessionReplayFixture) -> CapabilityViewSpec {
    CapabilityViewSpec {
        host_query_enabled: fixture.capability_spec.host_query_enabled,
        locale_format_enabled: fixture.capability_spec.locale_format_enabled,
        caller_context_enabled: fixture.capability_spec.caller_context_enabled,
        external_provider_enabled: fixture.capability_spec.external_provider_enabled,
    }
}

fn compile_prepared(formula: &str, with_input_name: bool) -> PrepareRequest {
    let source = FormulaSourceRecord::new("session-replay-fixture", 1, formula.to_string());
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
            structure_context_version: StructureContextVersion("session-replay-v1".to_string()),
            names,
            ..BindContext::default()
        },
    });
    let plan = compile_semantic_plan(oxfml_core::CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:session-replay".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
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

fn session_phase_name(phase: &SessionPhase) -> String {
    match phase {
        SessionPhase::Open => "Open",
        SessionPhase::CapabilityViewEstablished => "CapabilityViewEstablished",
        SessionPhase::Executed => "Executed",
        SessionPhase::Committed => "Committed",
        SessionPhase::Rejected => "Rejected",
        SessionPhase::Aborted => "Aborted",
        SessionPhase::Expired => "Expired",
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

fn value_payload_name(payload: &ValuePayload) -> String {
    match payload {
        ValuePayload::Number(value) => format!("Number({value})"),
        ValuePayload::Text(value) => format!("Text({value})"),
        ValuePayload::Logical(value) => format!("Logical({value})"),
        ValuePayload::ErrorCode(value) => format!("ErrorCode({value})"),
        ValuePayload::Blank => "Blank".to_string(),
    }
}

fn reject_code_name(code: RejectCode) -> String {
    match code {
        RejectCode::FenceMismatch => "FenceMismatch",
        RejectCode::CapabilityDenied => "CapabilityDenied",
        RejectCode::SessionTerminated => "SessionTerminated",
        RejectCode::BindMismatch => "BindMismatch",
        RejectCode::StructuralConflict => "StructuralConflict",
        RejectCode::DynamicReferenceFailure => "DynamicReferenceFailure",
        RejectCode::ResourceInvariantFailure => "ResourceInvariantFailure",
    }
    .to_string()
}

fn trace_event_name(kind: TraceEventKind) -> String {
    match kind {
        TraceEventKind::SessionOpened => "SessionOpened",
        TraceEventKind::CapabilityViewEstablished => "CapabilityViewEstablished",
        TraceEventKind::AcceptedCandidateResultBuilt => "AcceptedCandidateResultBuilt",
        TraceEventKind::CommitAccepted => "CommitAccepted",
        TraceEventKind::CommitRejected => "CommitRejected",
        TraceEventKind::RejectIssued => "RejectIssued",
        TraceEventKind::SessionAborted => "SessionAborted",
        TraceEventKind::SessionExpired => "SessionExpired",
    }
    .to_string()
}
