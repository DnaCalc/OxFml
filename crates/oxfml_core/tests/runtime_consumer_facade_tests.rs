use std::cell::RefCell;

use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeManagedSessionError,
    RuntimeManagedSessionPhase, RuntimeSessionFacade,
};
use oxfml_core::format::{en_us_context, worksheet_error_text};
use oxfml_core::publication::{
    VerificationConditionalFormattingRule, VerificationPublicationContext,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::{
    AcceptDecision, FormulaSourceRecord, InMemoryLibraryContextProvider, LibraryContextSnapshotRef,
    RegisteredExternalCatalogController, RegisteredExternalCatalogMutationRequest,
    RegisteredExternalCatalogMutationResult, RegisteredExternalHostRegistrationRequest,
    RegisteredExternalRegistrationChannel, TypedContextQueryBundle, TypedContextQueryFamily,
};
use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalDescriptor, RegisteredExternalOriginKind,
    RegisteredExternalProvider, RegisteredExternalProviderError, RegisteredExternalTarget,
    RegisteredProcedureSpec,
};
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::value::EvalValue;
use oxfunc_core::value::{CallArgValue, ExcelText};
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
    let locale = en_us_context();
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
    assert_eq!(result.published_worksheet_value, EvalValue::Number(7.0));
}

#[test]
fn runtime_environment_executes_registered_external_formula_through_typed_query_bundle() {
    let provider = RecordingRegisteredExternalProvider::default();
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("runtime:call-register", 1, "=CALL(4242,6,7,3)"),
        TypedContextQueryBundle::default().with_registered_external_provider(Some(&provider)),
    );

    let result = RuntimeEnvironment::new()
        .execute(request)
        .expect("runtime registered-external execution should succeed");

    assert!(
        result
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::RegisteredExternal)
    );
    assert_eq!(result.published_worksheet_value, EvalValue::Number(14.0));
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
                    CallArgValue::Eval(EvalValue::Number(6.0)),
                    CallArgValue::Eval(EvalValue::Number(7.0)),
                    CallArgValue::Eval(EvalValue::Number(3.0)),
                ]
            );
        }
        other => panic!("unexpected normalized call request: {other:?}"),
    }
}

#[test]
fn runtime_environment_emits_effective_display_text_comparison_view_for_verification_context() {
    let locale = en_us_context();
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
    let locale = en_us_context();
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
                    &format!("runtime:programmatic-verification:{case_id}"),
                    1,
                    formula,
                ),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .unwrap_or_else(|error| panic!("{case_id} runtime execution should succeed: {error}"));

        assert!(
            !result.verification_publication_surface.has_publication_context,
            "{case_id} should remain a no-explicit-publication-context control"
        );
        let expected_value =
            expected_programmatic_comparison_value(&result.published_worksheet_value);
        let expected_text = result.verification_publication_surface.effective_display_text.clone();
        assert_eq!(
            result.verification_publication_surface.visible_value_text,
            result.verification_publication_surface.effective_display_text,
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
fn runtime_environment_executes_foundation_let_lambda_success_cases() {
    let cases = [
        (
            "FTC-0444",
            "=LET(THUNK,LAMBDA(x,LAMBDA(x)),t,THUNK(42),t())",
            EvalValue::Number(42.0),
            "42",
        ),
        (
            "FTC-0446",
            "=LET(dict,{\"key1\",LAMBDA({10,20,30});\"key2\",LAMBDA({40,50,60})},keys,INDEX(dict,0,1),INDEX(keys,1,1))",
            text_eval_value("key1"),
            "key1",
        ),
        (
            "FTC-0447",
            "=LET(dict,{\"a\",LAMBDA(1);\"b\",LAMBDA(2);\"c\",LAMBDA(3)},keys,TAKE(dict,,1),ROWS(keys))",
            EvalValue::Number(3.0),
            "3",
        ),
    ];

    for (case_id, formula, expected_value, expected_text) in cases {
        assert_runtime_foundation_case(case_id, formula, expected_value, expected_text);
    }
}

#[test]
fn runtime_environment_executes_foundation_array_lambda_carrier_cases() {
    let cases = [
        (
            "FTC-0448",
            "=LET(dict,{\"x\",LAMBDA(100);\"y\",LAMBDA(200)},GETlambda,LAMBDA(d,LAMBDA(key,LET(keys,TAKE(d,,1),objects,DROP(d,,1),obj,XLOOKUP(key,keys,objects,\"not found\"),obj()))),getter,GETlambda(dict),getter(\"y\"))",
            EvalValue::Number(200.0),
            "200",
        ),
        (
            "FTC-0455",
            "=LET(THUNK,LAMBDA(x,LAMBDA(x)),vals,MAP({1;2;3},LAMBDA(v,THUNK(v*10))),INDEX(vals,2,1)())",
            EvalValue::Number(20.0),
            "20",
        ),
    ];

    for (case_id, formula, expected_value, expected_text) in cases {
        assert_runtime_foundation_case(case_id, formula, expected_value, expected_text);
    }
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
fn runtime_environment_executes_foundation_text_date_format_case_ftc_1022() {
    assert_runtime_foundation_text_case(
        "FTC-1022",
        "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
        EvalValue::Number(0.0),
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
fn runtime_environment_executes_if_text_true_condition_ftc_0541() {
    let locale = en_us_context();
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

    assert_eq!(
        result.published_worksheet_value,
        text_eval_value("yes")
    );
    assert_eq!(result.verification_publication_surface.visible_value_text, "yes");
    assert_eq!(result.verification_publication_surface.effective_display_text, "yes");
}

#[test]
fn runtime_environment_executes_foundation_helper_array_case_ftc_1031() {
    let locale = en_us_context();
    let formula = "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,week1,gridStart+SEQUENCE(1,7,,1)-1,dayNums,MAP(week1,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),DAY(d),0))),SUM(dayNums))";

    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:foundation:FTC-1031", 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("FTC-1031 runtime execution should succeed");

    assert_eq!(result.published_worksheet_value, EvalValue::Number(21.0));
    assert_eq!(result.verification_publication_surface.visible_value_text, "21");
    assert_eq!(result.verification_publication_surface.effective_display_text, "21");
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
        (
            "FTC-1028",
            "=TEXT(DATE(2024,7,1),\"MMMM\")",
        ),
        (
            "FTC-1040",
            "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
        ),
        // Separator-sensitive TEXT formatting remains blocked without an explicit locale
        // context; OxFml should not silently guess a machine-specific Excel separator profile.
        (
            "TEXT-SEPARATORS-UNPINNED",
            "=TEXT(1234567.89,\"#,##0.00\")",
        ),
    ];

    for (case_id, formula) in cases {
        let error = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    &format!("runtime:no-locale-foundation:{case_id}"),
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
    let locale = en_us_context();
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
    expected_value: EvalValue,
    expected_text: &str,
) {
    let locale = en_us_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(&format!("runtime:foundation:{case_id}"), 1, formula),
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
        result.verification_publication_surface.effective_display_text, expected_text,
        "{case_id} effective display text"
    );
}

fn assert_runtime_foundation_text_case(
    case_id: &str,
    formula: &str,
    expected_value: EvalValue,
    expected_text: &str,
) {
    let locale = en_us_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(&format!("runtime:foundation:{case_id}"), 1, formula),
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
        result.verification_publication_surface.effective_display_text, expected_text,
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

fn expected_programmatic_comparison_value(value: &EvalValue) -> Value {
    match value {
        EvalValue::Number(number) => serde_json::json!({
            "kind": "number",
            "value": number
        }),
        EvalValue::Error(code) => serde_json::json!({
            "kind": "error",
            "code": format!("{code:?}"),
            "display": worksheet_error_text(*code)
        }),
        other => panic!("unexpected programmatic verification value shape: {other:?}"),
    }
}

fn text_eval_value(text: &str) -> EvalValue {
    EvalValue::Text(ExcelText::from_interop_assignment(text))
}

struct ValueRtdProvider;

impl RtdProvider for ValueRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        RtdProviderResult::Value(EvalValue::Number(7.0))
    }
}

struct ClaimingHostInfoProvider;

impl HostInfoProvider for ClaimingHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&oxfunc_core::value::ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedCellInfoQuery(query))
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        match query {
            InfoQuery::Directory => Ok(EvalValue::Text(ExcelText::from_interop_assignment(
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
        args: &[CallArgValue],
    ) -> Result<EvalValue, RegisteredExternalProviderError> {
        match &descriptor.procedure {
            RegisteredProcedureSpec::Name(name) if name.to_string_lossy() == "MulDiv" => match args
            {
                [
                    CallArgValue::Eval(EvalValue::Number(a)),
                    CallArgValue::Eval(EvalValue::Number(b)),
                    CallArgValue::Eval(EvalValue::Number(c)),
                ] => Ok(EvalValue::Number((a * b) / c)),
                _ => Err(RegisteredExternalProviderError::WorksheetError(
                    oxfunc_core::value::WorksheetErrorCode::Value,
                )),
            },
            _ => Ok(EvalValue::Number(descriptor.register_id)),
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
