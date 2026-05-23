use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::binding::BindContext;
use oxfml_core::consumer::editor::{
    EditorAnalysisStage, EditorEditService, EditorEnvironment, EditorPlanOptions,
};
use oxfml_core::consumer::replay::{
    ReplayFirstHostCaptureSource, ReplayFixtureFamilySource, ReplayProjectionRequest,
    ReplayProjectionService, ReplayRetainedWitnessSource,
};
use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeHostFormulaContext,
    RuntimeHostReferenceBindResult, RuntimeSessionFacade,
};
use oxfml_core::format::{
    oxfml_current_excel_host_locale_context, oxfml_en_us_locale_context, worksheet_error_text,
};
use oxfml_core::interface::{
    InMemoryLibraryContextProvider, LibraryContextProvider, LibraryContextSnapshotRef,
};
use oxfml_core::publication::{
    VerificationConditionalFormattingRule, VerificationPublicationContext,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::FormulaSourceRecord;
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::{
    FormulaChannelKind, TableColumnDescriptor, TableDescriptor, TypedContextQueryBundle,
};
use oxfunc_core::host_info::{
    HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ResolvedWebImage,
};
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText};
use serde_json::Value;

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
    let locale_ctx = oxfml_en_us_locale_context();
    let verification_context = VerificationPublicationContext {
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
            typed_rule: None,
            font_color: Some("#FF0000".to_string()),
            fill_color: Some("#00FF00".to_string()),
            effective_display_text: None,
            applies: None,
            effective_font_color: None,
            effective_fill_color: None,
        }],
    };
    let runtime_result = environment
        .execute(
            RuntimeFormulaRequest::new(
                FormulaSourceRecord::new("replay:runtime", 1, "=SUM(1,2,3)"),
                TypedContextQueryBundle::new(None, None, Some(&locale_ctx), None, None),
            )
            .with_verification_publication_context(verification_context),
        )
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
    assert_eq!(
        runtime_result
            .verification_publication_surface
            .effective_display_text,
        "$6.00"
    );
    assert_eq!(
        runtime_projection
            .comparison_views
            .as_ref()
            .map(|views| views.len()),
        Some(6)
    );
    assert_eq!(
        runtime_projection
            .comparison_views
            .as_ref()
            .and_then(|views| {
                views
                    .iter()
                    .find(|view| view.view_family == "effective_display_text")
                    .map(|view| view.value.clone())
            }),
        Some(Value::String("$6.00".to_string()))
    );
    assert_eq!(
        runtime_projection
            .comparison_views
            .as_ref()
            .map(|views| comparison_views_json(views)),
        Some(load_expected_comparison_views_fixture())
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.effective_display_text.as_str()),
        Some("$6.00")
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .and_then(|surface| surface.number_format_code.as_deref()),
        Some("$#,##0.00")
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.published_value.clone()),
        Some(EvalValue::Number(6.0))
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.published_value_class.clone()),
        Some(oxfml_core::WorksheetValueClass::Scalar)
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.conditional_formatting_rule_kind.clone()),
        Some(vec!["Expression".to_string()])
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.conditional_formatting_target_ranges.clone()),
        Some(vec![vec!["A1".to_string()]])
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.conditional_formatting_thresholds.clone()),
        Some(vec![vec!["=A1>0".to_string()]])
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.conditional_formatting_effective_display.clone()),
        Some(vec![Some("$6.00".to_string())])
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.conditional_formatting_applies.clone()),
        Some(vec![Some(true)])
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .and_then(|surface| surface.effective_font_color.as_deref()),
        Some("#FF0000")
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
    assert_eq!(
        host_projection
            .comparison_views
            .as_ref()
            .map(|views| comparison_views_json(views)),
        Some(load_expected_comparison_views_fixture())
    );
    assert_eq!(
        host_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.effective_display_text.as_str()),
        Some("$6.00")
    );
}

#[test]
fn replay_projection_service_preserves_first_host_capture_comparison_value_for_text_date_family() {
    let locale = oxfml_current_excel_host_locale_context();
    let cases = [
        (
            "FTC-1021",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,m,15),TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"))",
            serde_json::json!({"kind": "text", "value": "15"}),
        ),
        (
            "FTC-1022",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
            serde_json::json!({"kind": "number", "value": 0.0}),
        ),
        (
            "FTC-1023",
            "=LET(baseSun,DATE(2024,1,7),headers,TEXT(baseSun+SEQUENCE(1,7,,1)-1,\"DDD\"),INDEX(headers,1,1))",
            serde_json::json!({"kind": "text", "value": "Sun"}),
        ),
        (
            "FTC-1024",
            "=LET(yr,2024,m,2,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(7,,0),dayTexts,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),TEXTJOIN(\",\",FALSE,dayTexts))",
            serde_json::json!({"kind": "text", "value": "  ,  ,  ,  ,01,02,03"}),
        ),
        (
            "FTC-1028",
            "=TEXT(DATE(2024,7,1),\"MMMM\")",
            serde_json::json!({"kind": "text", "value": "July"}),
        ),
        (
            "FTC-1040",
            "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
            serde_json::json!({"kind": "text", "value": "January|  |01|02|03|04|05|06"}),
        ),
    ];

    for (case_id, formula, expected_comparison_value) in cases {
        let mut host = oxfml_core::test_support::host::SingleFormulaHost::new(
            format!("replay:{case_id}"),
            formula,
        );
        host.now_serial = Some(46000.0);
        let output = host
            .recalc(None, Some(&locale))
            .expect("host recalc should succeed");
        let source = ReplayFirstHostCaptureSource {
            source_artifact_family: "first_host_capture_packet".to_string(),
            session_id: output.candidate_result.session_id.clone(),
            packet: output.to_first_host_replay_capture_packet(),
        };
        let projection =
            ReplayProjectionService::project(ReplayProjectionRequest::first_host_capture(&source));

        assert_eq!(
            projection
                .comparison_views
                .as_ref()
                .and_then(|views| views
                    .iter()
                    .find(|view| view.view_family == "comparison_value"))
                .map(|view| view.value.clone()),
            Some(expected_comparison_value),
            "{case_id} comparison_value"
        );
        assert_eq!(
            projection
                .verification_publication_surface
                .as_ref()
                .and_then(|surface| surface.format_profile.as_deref()),
            Some("locale-format-context"),
            "{case_id} format_profile"
        );
        assert!(
            projection
                .verification_publication_surface
                .as_ref()
                .and_then(|surface| surface.locale_format_context.as_ref())
                .is_some(),
            "{case_id} locale_format_context"
        );
    }
}

#[test]
fn replay_projection_service_matches_dnaonecalc_exact_request_shape_for_runtime_text_date_family() {
    let locale = oxfml_en_us_locale_context();
    let verification_context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: None,
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: Vec::new(),
    };
    let cases = [
        (
            "FTC-1021",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,m,15),TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"))",
            serde_json::json!({"kind": "text", "value": "15"}),
        ),
        (
            "FTC-1022",
            "=LET(yr,2024,m,3,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),testDate,DATE(yr,2,28),result,TEXT(testDate,\"[<\"&firstDay&\"] ;[>\"&lastDay&\"] ;dd\"),LEN(TRIM(result)))",
            serde_json::json!({"kind": "number", "value": 0.0}),
        ),
        (
            "FTC-1023",
            "=LET(baseSun,DATE(2024,1,7),headers,TEXT(baseSun+SEQUENCE(1,7,,1)-1,\"DDD\"),INDEX(headers,1,1))",
            serde_json::json!({"kind": "text", "value": "Sun"}),
        ),
        (
            "FTC-1024",
            "=LET(yr,2024,m,2,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(7,,0),dayTexts,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),TEXTJOIN(\",\",FALSE,dayTexts))",
            serde_json::json!({"kind": "text", "value": "  ,  ,  ,  ,01,02,03"}),
        ),
        (
            "FTC-1028",
            "=TEXT(DATE(2024,7,1),\"MMMM\")",
            serde_json::json!({"kind": "text", "value": "July"}),
        ),
        (
            "FTC-1040",
            "=LET(yr,2024,m,1,firstDay,DATE(yr,m,1),lastDay,EOMONTH(firstDay,0),gridStart,firstDay-WEEKDAY(firstDay,1)+1,dates,gridStart+SEQUENCE(42,,0),dayStrs,MAP(dates,LAMBDA(d,IF(AND(d>=firstDay,d<=lastDay),TEXT(DAY(d),\"00\"),\"  \"))),monthName,TEXT(firstDay,\"MMMM\"),TEXTJOIN(\"|\",FALSE,monthName,INDEX(dayStrs,1),INDEX(dayStrs,2),INDEX(dayStrs,3),INDEX(dayStrs,4),INDEX(dayStrs,5),INDEX(dayStrs,6),INDEX(dayStrs,7)))",
            serde_json::json!({"kind": "text", "value": "January|  |01|02|03|04|05|06"}),
        ),
    ];

    for (case_id, formula, expected_comparison_value) in cases {
        let runtime_result = RuntimeEnvironment::new()
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new(
                        &format!("replay:verification-context:{case_id}"),
                        1,
                        formula,
                    )
                    .with_formula_channel_kind(FormulaChannelKind::WorksheetA1),
                    TypedContextQueryBundle::new(
                        None,
                        None,
                        Some(&locale),
                        Some(46000.0),
                        Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
                    ),
                )
                .with_verification_publication_context(verification_context.clone()),
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{case_id} runtime execution with verification context should succeed: {error}"
                )
            });

        assert_eq!(
            runtime_result.source.formula_channel_kind,
            FormulaChannelKind::WorksheetA1,
            "{case_id} formula channel"
        );
        let projection = ReplayProjectionService::project(
            ReplayProjectionRequest::runtime_result(&runtime_result).with_source_case_id(case_id),
        );

        assert_eq!(
            projection
                .comparison_views
                .as_ref()
                .and_then(|views| views
                    .iter()
                    .find(|view| view.view_family == "comparison_value"))
                .map(|view| view.value.clone()),
            Some(expected_comparison_value),
            "{case_id} comparison_value"
        );
        assert_eq!(
            projection
                .verification_publication_surface
                .as_ref()
                .and_then(|surface| surface.format_profile.as_deref()),
            Some("en-US"),
            "{case_id} format_profile"
        );
        assert!(
            projection
                .verification_publication_surface
                .as_ref()
                .and_then(|surface| surface.locale_format_context.as_ref())
                .is_some(),
            "{case_id} locale_format_context"
        );
    }
}

#[test]
fn replay_projection_service_prefers_first_host_capture_publication_surface_for_runtime_results() {
    let locale = oxfml_current_excel_host_locale_context();
    let runtime_result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "replay:runtime-prefers-host-capture",
                1,
                "=TEXT(DATE(2024,7,1),\"MMMM\")",
            ),
            TypedContextQueryBundle::new(
                None,
                None,
                Some(&locale),
                Some(46000.0),
                Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
            ),
        ))
        .expect("runtime result should execute");

    let mut mutated = runtime_result.clone();
    mutated.verification_publication_surface.format_profile = None;
    mutated
        .verification_publication_surface
        .locale_format_context = None;
    mutated.verification_publication_surface.published_value =
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value);
    mutated.verification_publication_surface.visible_value_text = "#VALUE!".to_string();
    mutated
        .verification_publication_surface
        .effective_display_text = "#VALUE!".to_string();

    let projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_result(&mutated)
            .with_source_case_id("case:runtime-prefers-host-capture"),
    );

    assert_eq!(
        projection
            .comparison_views
            .as_ref()
            .and_then(|views| views
                .iter()
                .find(|view| view.view_family == "comparison_value"))
            .map(|view| view.value.clone()),
        Some(serde_json::json!({"kind": "text", "value": "July"}))
    );
    assert_eq!(
        projection
            .verification_publication_surface
            .as_ref()
            .and_then(|surface| surface.format_profile.as_deref()),
        Some("locale-format-context")
    );
    assert!(
        projection
            .verification_publication_surface
            .as_ref()
            .and_then(|surface| surface.locale_format_context.as_ref())
            .is_some()
    );
    assert_eq!(
        projection.verification_publication_surface,
        Some(
            mutated
                .first_host_replay_capture_packet
                .verification_publication_surface
                .clone()
        )
    );
}

#[test]
fn replay_projection_service_emits_comparison_value_and_visible_text_without_publication_context() {
    let environment = RuntimeEnvironment::new();
    let runtime_result = environment
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:no-publication-context", 1, "=1+2+3"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime result should execute");

    let runtime_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_result(&runtime_result)
            .with_source_case_id("case:no-publication-context")
            .with_shared_scenario_alias("alias.no-publication-context"),
    );

    assert_eq!(
        runtime_projection
            .comparison_views
            .as_ref()
            .map(|views| comparison_views_json(views)),
        Some(serde_json::json!([
            {
                "view_family": "comparison_value",
                "value": {
                    "kind": "number",
                    "value": 6.0
                }
            },
            {
                "view_family": "visible_value_text",
                "value": "6"
            },
            {
                "view_family": "effective_display_text",
                "value": "6"
            },
            {
                "view_family": "execution_outcome",
                "value": {
                    "outcome_kind": "executed_result",
                    "outcome_stage": "executed",
                    "class_id": "executed_result",
                    "lane_reason_code": null,
                    "raw_detail": null
                }
            }
        ]))
    );
    assert_eq!(
        runtime_projection
            .verification_publication_surface
            .as_ref()
            .map(|surface| surface.has_publication_context),
        Some(false)
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
        host_projection
            .comparison_views
            .as_ref()
            .map(|views| comparison_views_json(views)),
        Some(serde_json::json!([
            {
                "view_family": "comparison_value",
                "value": {
                    "kind": "number",
                    "value": 6.0
                }
            },
            {
                "view_family": "visible_value_text",
                "value": "6"
            },
            {
                "view_family": "effective_display_text",
                "value": "6"
            },
            {
                "view_family": "execution_outcome",
                "value": {
                    "outcome_kind": "executed_result",
                    "outcome_stage": "executed",
                    "class_id": "executed_result",
                    "lane_reason_code": null,
                    "raw_detail": null
                }
            }
        ]))
    );
}

#[test]
fn replay_projection_service_surfaces_bind_boundary_execution_outcome() {
    let locale = oxfml_en_us_locale_context();
    let runtime_result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:bind-boundary", 1, "={\"x\",LAMBDA(100)}"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("bind-boundary runtime result should project");

    let runtime_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_result(&runtime_result)
            .with_source_case_id("case:bind-boundary")
            .with_shared_scenario_alias("alias.bind-boundary"),
    );

    assert_eq!(
        runtime_projection
            .execution_outcome_surface
            .as_ref()
            .map(|surface| surface.class_id.as_str()),
        Some("bind_boundary_reject")
    );
    assert_eq!(
        runtime_projection
            .comparison_views
            .as_ref()
            .and_then(|views| views
                .iter()
                .find(|view| view.view_family == "execution_outcome"))
            .map(|view| view.value.clone()),
        Some(serde_json::json!({
            "outcome_kind": "rejected",
            "outcome_stage": "bind_boundary",
            "class_id": "bind_boundary_reject",
            "lane_reason_code": "BindMismatch",
            "raw_detail": "LAMBDA cannot appear inside array constants"
        }))
    );
}

#[test]
fn replay_projection_service_emits_effective_display_text_for_programmatic_verification_cases() {
    let locale = oxfml_en_us_locale_context();
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

    let environment = RuntimeEnvironment::new();

    for (case_id, formula) in cases {
        let runtime_result = environment
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(
                    &format!("replay:programmatic-verification:{case_id}"),
                    1,
                    formula,
                ),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .unwrap_or_else(|error| panic!("{case_id} runtime result should execute: {error}"));
        let expected_value =
            expected_programmatic_comparison_value(&runtime_result.published_worksheet_value);
        let expected_text = runtime_result
            .verification_publication_surface
            .effective_display_text
            .clone();

        let runtime_projection = ReplayProjectionService::project(
            ReplayProjectionRequest::runtime_result(&runtime_result),
        );

        assert_eq!(
            runtime_projection
                .comparison_views
                .as_ref()
                .map(|views| views.len()),
            Some(4),
            "{case_id} runtime projection view count"
        );
        assert_eq!(
            runtime_projection
                .comparison_views
                .as_ref()
                .and_then(|views| {
                    views
                        .iter()
                        .find(|view| view.view_family == "comparison_value")
                        .map(|view| view.value.clone())
                }),
            Some(expected_value.clone()),
            "{case_id} runtime projection comparison_value"
        );
        assert_eq!(
            runtime_projection
                .comparison_views
                .as_ref()
                .and_then(|views| {
                    views
                        .iter()
                        .find(|view| view.view_family == "effective_display_text")
                        .map(|view| view.value.clone())
                }),
            Some(Value::String(expected_text.clone())),
            "{case_id} runtime projection effective_display_text"
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
            host_projection
                .comparison_views
                .as_ref()
                .map(|views| views.len()),
            Some(4),
            "{case_id} first-host projection view count"
        );
        assert_eq!(
            host_projection.comparison_views.as_ref().and_then(|views| {
                views
                    .iter()
                    .find(|view| view.view_family == "effective_display_text")
                    .map(|view| view.value.clone())
            }),
            Some(Value::String(expected_text)),
            "{case_id} first-host projection effective_display_text"
        );
    }
}

#[test]
fn replay_projection_service_preserves_full_array_contents_in_comparison_value() {
    let environment = RuntimeEnvironment::new();
    let runtime_result = environment
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:array", 1, "=SEQUENCE(2,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime result should execute");

    let runtime_projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));

    let comparison_value = runtime_projection
        .comparison_views
        .as_ref()
        .and_then(|views| {
            views
                .iter()
                .find(|view| view.view_family == "comparison_value")
                .map(|view| view.value.clone())
        })
        .expect("comparison_value view should exist");

    assert_eq!(
        comparison_value,
        serde_json::json!({
            "kind": "array",
            "shape": {
                "rows": 2,
                "cols": 2
            },
            "cells": [
                { "kind": "number", "value": 1.0 },
                { "kind": "number", "value": 2.0 },
                { "kind": "number", "value": 3.0 },
                { "kind": "number", "value": 4.0 }
            ]
        })
    );
    assert_eq!(
        runtime_result
            .verification_publication_surface
            .visible_value_text,
        "1"
    );
}

fn comparison_views_json(views: &[oxfml_core::consumer::replay::ReplayComparisonView]) -> Value {
    Value::Array(
        views
            .iter()
            .map(|view| {
                serde_json::json!({
                    "view_family": view.view_family,
                    "value": view.value
                })
            })
            .collect(),
    )
}

fn load_expected_comparison_views_fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("xml_verification_comparison_views_projection.json");
    let content = fs::read_to_string(path).expect("comparison-view fixture file should exist");
    let fixture: Value =
        serde_json::from_str(&content).expect("comparison-view fixture should deserialize");
    fixture["comparison_views"].clone()
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
    assert_eq!(
        execution_projection
            .prepared_formula_identity
            .as_ref()
            .map(|identity| identity.prepared_formula_key.as_str()),
        Some(
            execution
                .prepared_formula_identity
                .prepared_formula_key
                .as_str()
        )
    );

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
    assert_eq!(session_projection.execution_outcome_surface, None);
    assert_eq!(
        session_projection
            .prepared_formula_identity
            .as_ref()
            .map(|identity| identity.prepared_formula_key.as_str()),
        Some(
            execution
                .prepared_formula_identity
                .prepared_formula_key
                .as_str()
        )
    );

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
    assert_eq!(commit.execution_outcome_surface.class_id, "executed_result");
    assert_eq!(
        commit_projection.execution_outcome_surface,
        Some(commit.execution_outcome_surface.clone())
    );
    assert_eq!(
        commit_projection
            .prepared_formula_identity
            .as_ref()
            .map(|identity| identity.prepared_formula_key.as_str()),
        Some(
            execution
                .prepared_formula_identity
                .prepared_formula_key
                .as_str()
        )
    );
    let committed_session_snapshot = session
        .managed_session_snapshot()
        .expect("managed committed session snapshot should exist");
    let committed_session_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_session(&committed_session_snapshot),
    );
    assert_eq!(
        committed_session_projection.execution_outcome_surface,
        Some(commit.execution_outcome_surface.clone())
    );
    assert_eq!(
        commit_projection.source_case_id.as_deref(),
        Some("case:session-commit")
    );
}

#[test]
fn replay_projection_carries_oxfunc_bridge_versions_from_runtime_artifacts() {
    let runtime_result = RuntimeEnvironment::new()
        .with_semantic_kernel_metadata_version("sem-kernel:replay:v1")
        .with_arg_admission_metadata_version("arg-admission:replay:v1")
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:oxfunc-bridge", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    let runtime_projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));
    let runtime_identity = runtime_projection
        .prepared_formula_identity
        .as_ref()
        .expect("runtime replay projection should carry prepared identity");
    assert_eq!(
        runtime_identity.semantic_kernel_metadata_version.as_deref(),
        Some("sem-kernel:replay:v1")
    );
    assert_eq!(
        runtime_projection
            .semantic_kernel_metadata_version
            .as_deref(),
        Some("sem-kernel:replay:v1")
    );
    assert_eq!(
        runtime_identity.arg_admission_metadata_version.as_deref(),
        Some("arg-admission:replay:v1")
    );
    assert_eq!(
        runtime_projection.arg_admission_metadata_version.as_deref(),
        Some("arg-admission:replay:v1")
    );

    let environment = RuntimeEnvironment::new()
        .with_semantic_kernel_metadata_version("sem-kernel:managed-replay:v1")
        .with_arg_admission_metadata_version("arg-admission:managed-replay:v1");
    let mut session = RuntimeSessionFacade::new(environment);
    let request = RuntimeFormulaRequest::new(
        FormulaSourceRecord::new("replay:managed-oxfunc-bridge", 1, "=SUM(1,2)"),
        TypedContextQueryBundle::default(),
    );

    let _open = session
        .open_managed_session(&request)
        .expect("managed open should succeed");
    let execution = session
        .execute_managed(request)
        .expect("managed execution should succeed");
    let execution_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_execution(&execution),
    );
    let execution_identity = execution_projection
        .prepared_formula_identity
        .as_ref()
        .expect("managed replay projection should carry prepared identity");
    assert_eq!(
        execution_identity
            .semantic_kernel_metadata_version
            .as_deref(),
        Some("sem-kernel:managed-replay:v1")
    );
    assert_eq!(
        execution_projection
            .semantic_kernel_metadata_version
            .as_deref(),
        Some("sem-kernel:managed-replay:v1")
    );
    assert_eq!(
        execution_identity.arg_admission_metadata_version.as_deref(),
        Some("arg-admission:managed-replay:v1")
    );
    assert_eq!(
        execution_projection
            .arg_admission_metadata_version
            .as_deref(),
        Some("arg-admission:managed-replay:v1")
    );
}

#[test]
fn replay_projection_carries_w074_host_reference_context() {
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "generic-host-v1".to_string(),
        capability_profile_id: "host-capabilities:generic-v1".to_string(),
        resolution_rule_version: "host-resolution:v1".to_string(),
        host_namespace_version: Some("host-ns:v1".to_string()),
        registry_snapshot_identity: Some("registry:snapshot:v1".to_string()),
        structure_context_version: Some("structure:v1".to_string()),
        caller_context_identity: Some("caller:sheet1-r1c1".to_string()),
        table_context_identity: Some("tables:v1".to_string()),
    };
    let bind_result = RuntimeHostReferenceBindResult {
        reference_handle: "host-ref:opaque-collection".to_string(),
        formal_reference_id: Some("formal-ref:host:opaque-collection".to_string()),
        source_span: TextSpan::new(5, 15),
        source_token_text: "HOSTREF:opaque".to_string(),
        opaque_selector_payload: Some("opaque-selector:collection".to_string()),
        resolution_layer: "explicit_host_ref".to_string(),
        shape_hint: Some("opaque_collection".to_string()),
        caller_context_dependent: true,
        diagnostics: vec!["diagnostic:host-reference-observed".to_string()],
        replay_identity_contribution: "host-ref-identity:v1".to_string(),
    };
    let runtime_result = RuntimeEnvironment::new()
        .with_host_formula_context(host_context.clone())
        .with_host_reference_bind_results(vec![bind_result.clone()])
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:w074-host-context", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));

    assert_eq!(projection.host_formula_context, Some(host_context.clone()));
    assert_eq!(
        projection.host_reference_bind_results,
        vec![bind_result.clone()]
    );
    let identity = projection
        .prepared_formula_identity
        .expect("runtime projection should preserve prepared identity");
    assert_eq!(identity.host_formula_context, Some(host_context));
    assert_eq!(identity.host_reference_bind_results, vec![bind_result]);
}

#[test]
fn replay_projection_carries_host_namespace_version_without_explicit_host_reference() {
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "generic-host-v1".to_string(),
        capability_profile_id: "host-capabilities:generic-v1".to_string(),
        resolution_rule_version: "host-resolution:v1".to_string(),
        host_namespace_version: Some("host-ns:v2".to_string()),
        registry_snapshot_identity: Some("registry:snapshot:v1".to_string()),
        structure_context_version: Some("structure:v1".to_string()),
        caller_context_identity: Some("caller:sheet1-r1c1".to_string()),
        table_context_identity: None,
    };
    let runtime_result = RuntimeEnvironment::new()
        .with_host_formula_context(host_context.clone())
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("replay:w074-host-namespace-version", 1, "=SUM(1,2)"),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));

    assert_eq!(projection.host_formula_context, Some(host_context.clone()));
    assert!(projection.host_reference_bind_results.is_empty());
    let identity = projection
        .prepared_formula_identity
        .expect("runtime projection should preserve prepared identity");
    assert_eq!(identity.host_formula_context, Some(host_context));
    assert!(identity.host_reference_bind_results.is_empty());
}

#[test]
fn replay_projection_carries_w074_structured_table_identity() {
    let runtime_result = RuntimeEnvironment::new()
        .with_table_context(vec![replay_w074_table("B2:B4")], None, None)
        .with_cell_values(BTreeMap::from([(
            "B2:B4".to_string(),
            EvalValue::Array(
                EvalArray::from_rows(vec![vec![
                    ArrayCellValue::Number(3.0),
                    ArrayCellValue::Number(4.0),
                    ArrayCellValue::Number(5.0),
                ]])
                .expect("array fixture should be valid"),
            ),
        )]))
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "replay:w074-structured-table-identity",
                1,
                "=SUM(Table1[Amount])",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("structured table runtime execution should succeed");

    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));
    let identity = projection
        .prepared_formula_identity
        .expect("runtime projection should preserve prepared identity");

    assert_eq!(
        identity.table_context_fingerprint,
        runtime_result
            .prepared_formula_identity
            .table_context_fingerprint
    );
    assert!(identity.table_context_fingerprint.is_some());
    assert_eq!(
        identity.structured_reference_bind_records,
        runtime_result.structured_reference_bind_records
    );
    assert_eq!(
        identity.formal_references,
        runtime_result.prepared_formula_identity.formal_references
    );
    assert_eq!(
        identity.structured_reference_bind_records[0].source_token_text,
        "Table1[Amount]"
    );
}

#[test]
fn replay_projection_preserves_no_host_namespace_lexical_guardrail() {
    let runtime_result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "replay:w074-lexical-no-host",
                1,
                "=LET(base,100,adder,LAMBDA(n,LAMBDA(x,x+n+base)),add5,adder(5),add5(10))",
            ),
            TypedContextQueryBundle::default(),
        ))
        .expect("lexical returned lambda should execute without host namespace");

    assert_eq!(
        runtime_result.evaluation.oxfunc_value,
        EvalValue::Number(115.0)
    );
    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));

    assert_eq!(projection.host_formula_context, None);
    assert!(projection.host_reference_bind_results.is_empty());
    let identity = projection
        .prepared_formula_identity
        .expect("runtime projection should preserve prepared identity");
    assert_eq!(identity.host_formula_context, None);
    assert!(identity.host_reference_bind_results.is_empty());
}

#[test]
fn replay_projection_carries_image_producer_capability_columns() {
    let runtime_result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(
                "replay:image-capability-columns",
                1,
                "=IMAGE(\"https://example.com/sphere.png\",\"Sphere\")",
            ),
            TypedContextQueryBundle::new(
                Some(&ReplayImageHostInfoProvider),
                None,
                None,
                None,
                None,
            ),
        ))
        .expect("IMAGE runtime execution should succeed");

    let projection =
        ReplayProjectionService::project(ReplayProjectionRequest::runtime_result(&runtime_result));

    assert!(
        projection
            .producer_capability_set_keys
            .iter()
            .any(|key| key.starts_with("Materialisable(")),
        "runtime replay projection should expose IMAGE/_webimage producer capability keys"
    );
    assert!(
        projection
            .exercised_capability_keys
            .iter()
            .any(|key| key.starts_with("Materialisable(")),
        "runtime replay projection should expose successful IMAGE/_webimage exercised capability keys"
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
    assert_eq!(
        termination.execution_outcome_surface.class_id,
        "commit_boundary_reject"
    );
    assert_eq!(
        termination
            .execution_outcome_surface
            .lane_reason_code
            .as_deref(),
        Some("SessionTerminated")
    );
    assert_eq!(
        termination_projection.execution_outcome_surface,
        Some(termination.execution_outcome_surface.clone())
    );
    let terminated_session_snapshot = session
        .managed_session_snapshot()
        .expect("managed terminated session snapshot should exist");
    let terminated_session_projection = ReplayProjectionService::project(
        ReplayProjectionRequest::runtime_managed_session(&terminated_session_snapshot),
    );
    assert_eq!(
        terminated_session_projection.execution_outcome_surface,
        Some(termination.execution_outcome_surface.clone())
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

fn replay_w074_table(amount_range_ref: &str) -> TableDescriptor {
    TableDescriptor {
        table_id: "table:w074:replay".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "A1:D5".to_string(),
        row_membership_identity: Some("table:w074:replay:rows:v1".to_string()),
        row_order_identity: Some("table:w074:replay:row-order:v1".to_string()),
        header_region_ref: Some("A1:D1".to_string()),
        totals_region_ref: Some("A5:D5".to_string()),
        header_row_present: true,
        totals_row_present: true,
        columns: vec![TableColumnDescriptor {
            column_id: "column:amount".to_string(),
            column_name: "Amount".to_string(),
            ordinal: 1,
            column_range_ref: amount_range_ref.to_string(),
        }],
    }
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
            interface_contract_ref: Some("contract:sum".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: None,
        }],
    }
}

struct ReplayImageHostInfoProvider;

impl HostInfoProvider for ReplayImageHostInfoProvider {
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
