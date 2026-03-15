use std::collections::BTreeMap;

use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::en_us_context;
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceLike};

use oxfml_core::binding::{BindContext, BindRequest, NameKind, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::session::{
    CapabilityViewSpec, ExecuteRequest, PrepareRequest, SessionPhase, SessionService,
};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{
    AcceptDecision, DefinedNameBinding, EvaluationBackend, Locus, RejectCode, TraceEventKind,
    compile_semantic_plan,
};

#[test]
fn managed_session_happy_path_runs_through_commit() {
    let prepared = compile_prepared("=SUM(InputValue,2)", true);
    let mut service = SessionService::new();
    let prepared = service.prepare(prepared).expect("prepare should succeed");
    let open = service.open_session(prepared);
    service
        .establish_capability_view(&open.session_id, CapabilityViewSpec::default())
        .expect("capability view should succeed");

    let mut defined_names = BTreeMap::new();
    defined_names.insert(
        "InputValue".to_string(),
        DefinedNameBinding::Value(EvalValue::Number(5.0)),
    );

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

    let decision = service.commit(
        &open.session_id,
        "commit:test",
        candidate.fence_snapshot.clone(),
    );

    match decision {
        AcceptDecision::Accepted(bundle) => {
            assert_eq!(
                bundle.value_delta.published_payload,
                oxfml_core::ValuePayload::Number("7".to_string())
            );
        }
        AcceptDecision::Rejected(_) => panic!("expected accepted commit"),
    }

    let session = service
        .session(&open.session_id)
        .expect("session should exist");
    assert_eq!(session.phase, SessionPhase::Committed);
    assert_eq!(
        session.trace_events[0].event_kind,
        TraceEventKind::SessionOpened
    );
    assert_eq!(
        session.trace_events[1].event_kind,
        TraceEventKind::CapabilityViewEstablished
    );
    assert_eq!(
        session.trace_events[2].event_kind,
        TraceEventKind::AcceptedCandidateResultBuilt
    );
    assert_eq!(
        session.trace_events[3].event_kind,
        TraceEventKind::CommitAccepted
    );
}

#[test]
fn managed_session_rejects_missing_host_query_capability() {
    let prepared = compile_prepared("=INFO(\"directory\")", false);
    let mut service = SessionService::new();
    let prepared = service.prepare(prepared).expect("prepare should succeed");
    let open = service.open_session(prepared);

    let reject = service
        .establish_capability_view(
            &open.session_id,
            CapabilityViewSpec {
                host_query_enabled: false,
                locale_format_enabled: true,
                caller_context_enabled: true,
                external_provider_enabled: false,
            },
        )
        .expect_err("capability view should reject");

    assert_eq!(reject.reject_code, RejectCode::CapabilityDenied);
    let session = service
        .session(&open.session_id)
        .expect("session should exist");
    assert_eq!(session.phase, SessionPhase::Rejected);
}

#[test]
fn managed_session_abort_prevents_execute() {
    let prepared = compile_prepared("=SUM(InputValue,2)", true);
    let mut service = SessionService::new();
    let prepared = service.prepare(prepared).expect("prepare should succeed");
    let open = service.open_session(prepared);
    let reject = service.abort_session(&open.session_id, Some("manual_abort".to_string()));
    assert_eq!(reject.reject_code, RejectCode::SessionTerminated);

    let execute_reject = service
        .execute(ExecuteRequest {
            session_id: open.session_id.clone(),
            backend: EvaluationBackend::OxFuncBacked,
            caller_row: 1,
            caller_col: 1,
            cell_values: BTreeMap::new(),
            defined_names: BTreeMap::new(),
            locale_ctx: Some(&en_us_context()),
            host_info: None,
            now_serial: Some(46000.0),
            random_value: Some(0.25),
        })
        .expect_err("execute should reject");

    assert_eq!(execute_reject.reject_code, RejectCode::SessionTerminated);
    let session = service
        .session(&open.session_id)
        .expect("session should exist");
    assert_eq!(session.phase, SessionPhase::Aborted);
    assert_eq!(
        session.trace_events.last().expect("abort trace").event_kind,
        TraceEventKind::SessionAborted
    );
}

#[test]
fn managed_session_rejects_second_execute_as_structural_conflict() {
    let prepared = compile_prepared("=SUM(InputValue,2)", true);
    let mut service = SessionService::new();
    let prepared = service.prepare(prepared).expect("prepare should succeed");
    let open = service.open_session(prepared);
    service
        .establish_capability_view(&open.session_id, CapabilityViewSpec::default())
        .expect("capability view should succeed");

    let mut defined_names = BTreeMap::new();
    defined_names.insert(
        "InputValue".to_string(),
        DefinedNameBinding::Value(EvalValue::Number(5.0)),
    );

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

    assert_eq!(reject.reject_code, RejectCode::StructuralConflict);
}

#[test]
fn managed_session_surfaces_execution_restriction_effects() {
    let prepared = compile_prepared("=CELL(\"filename\",A1)", false);
    let mut service = SessionService::new();
    let prepared = service.prepare(prepared).expect("prepare should succeed");
    let open = service.open_session(prepared);
    service
        .establish_capability_view(
            &open.session_id,
            CapabilityViewSpec {
                host_query_enabled: true,
                locale_format_enabled: true,
                caller_context_enabled: true,
                external_provider_enabled: false,
            },
        )
        .expect("capability view should succeed");

    let candidate = service
        .execute(ExecuteRequest {
            session_id: open.session_id.clone(),
            backend: EvaluationBackend::OxFuncBacked,
            caller_row: 1,
            caller_col: 1,
            cell_values: BTreeMap::new(),
            defined_names: BTreeMap::new(),
            locale_ctx: Some(&en_us_context()),
            host_info: Some(&SessionMockHostInfoProvider),
            now_serial: Some(46000.0),
            random_value: Some(0.25),
        })
        .expect("execute should succeed");

    let facts = &candidate.topology_delta.capability_effect_facts;
    assert!(
        facts
            .iter()
            .any(|fact| fact.capability_kind == "host_query")
    );
    assert!(
        facts
            .iter()
            .any(|fact| fact.capability_kind == "caller_context")
    );
    assert!(
        facts
            .iter()
            .any(|fact| fact.capability_kind == "thread_affinity")
    );
    assert!(
        facts
            .iter()
            .any(|fact| fact.capability_kind == "serial_scheduler_lane")
    );
}

fn compile_prepared(formula: &str, with_input_name: bool) -> PrepareRequest {
    let source = FormulaSourceRecord::new("session-fixture", 1, formula.to_string());
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
            structure_context_version: StructureContextVersion("session-struct-v1".to_string()),
            names,
            ..BindContext::default()
        },
    });
    let plan = compile_semantic_plan(oxfml_core::CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:session".to_string(),
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

struct SessionMockHostInfoProvider;

impl HostInfoProvider for SessionMockHostInfoProvider {
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
        Err(HostInfoError::UnsupportedInfoQuery(query))
    }
}
