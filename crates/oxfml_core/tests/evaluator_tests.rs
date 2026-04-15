use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

mod common;

use oxfml_core::format::en_us_context;
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult};
use oxfunc_core::host_info::{
    CellInfoQuery, HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest,
    ImageSizingMode, InfoQuery, ResolvedWebImage,
};
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::value::{
    ArrayCellValue, CellStyleHint, EvalValue, ExcelText, NumberFormatHint, PresentationHint,
    ReferenceKind, ReferenceLike, WorksheetErrorCode,
};
use serde::Deserialize;

use oxfml_core::binding::{
    BinaryOp, BoundExpr, NameKind, NameRef, NormalizedReference, ReferenceExpr,
};
use oxfml_core::eval::{
    CallableDefinedNameBinding, CallableValueCarrier, CallableValueProfile, DefinedNameBinding,
    EvaluationContext, evaluate_formula,
};
use oxfml_core::interface::TypedContextQueryBundle;

#[test]
fn evaluator_runs_text_with_locale_format_context() {
    let output = evaluate(
        "=TEXT(1234.567,\"0.00\")",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Text(1234.57)");
    assert_eq!(
        output.result.format_hint.as_deref(),
        Some("locale_format_semantics")
    );
    assert_eq!(output.trace.prepared_calls.len(), 1);
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.TEXT");
    assert_eq!(
        output.result.capability_dependencies,
        vec!["locale_format_context".to_string()]
    );
}

#[test]
fn evaluator_runs_value_with_locale_parser() {
    let output = evaluate("=VALUE(\"12%\")", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(0.12)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.VALUE");
}

#[test]
fn evaluator_runs_cell_with_host_info_provider() {
    let output = evaluate(
        "=CELL(\"filename\",A1)",
        None,
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Text([Book1]Sheet1)");
    assert_eq!(
        output.result.publication_hint.as_deref(),
        Some("host_query_surface")
    );
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.CELL");
    assert!(output.trace.prepared_calls[0].host_query_enabled);
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[1].structure_class,
        oxfml_core::PreparedStructureClass::ReferenceVisible
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[1].evaluation_mode,
        oxfml_core::PreparedEvaluationMode::ReferencePreserved
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[1].blankness_class,
        oxfml_core::PreparedBlanknessClass::NonBlank
    );
    assert_eq!(
        output.result.capability_dependencies,
        vec!["caller_context".to_string(), "host_query".to_string()]
    );
}

#[test]
fn evaluator_runs_info_with_host_info_provider() {
    let output = evaluate(
        "=INFO(\"directory\")",
        None,
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Text(C:\\Work)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.INFO");
}

#[test]
fn evaluator_preserves_hyperlink_publication_intent() {
    let output = evaluate(
        "=HYPERLINK(\"https://example.com\",\"Go\")",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Text(ExcelText::from_interop_assignment("Go"))
    );
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::ValueWithPresentation
    );
    assert_eq!(
        output.returned_value_surface.presentation_hint,
        Some(PresentationHint::style(CellStyleHint::Hyperlink))
    );
}

#[test]
fn evaluator_preserves_today_presentation_hint_through_generic_extended_path() {
    let output = evaluate("=TODAY()", None, None, Some(&en_us_context()));
    assert_eq!(output.oxfunc_value, EvalValue::Number(46000.0));
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::ValueWithPresentation
    );
    assert_eq!(
        output.returned_value_surface.presentation_hint,
        Some(PresentationHint::number_format(NumberFormatHint::DateLike))
    );
}

#[test]
fn evaluator_preserves_image_rich_value_surface_through_host_query_lane() {
    let output = evaluate(
        "=IMAGE(\"https://example.com/sphere.png\",\"Sphere\")",
        None,
        Some(&ImageHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Text(ExcelText::from_interop_assignment("-2146826273"))
    );
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::RichValue
    );
    assert_eq!(
        output
            .returned_value_surface
            .rich_value_type_name
            .as_deref(),
        Some("_webimage")
    );
}

#[test]
fn evaluator_maps_image_provider_denial_to_blocked_error() {
    let output = evaluate(
        "=IMAGE(\"https://example.com/sphere.png\")",
        None,
        Some(&BlockedImageHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Blocked)
    );
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::OrdinaryValue
    );
    assert_eq!(
        output.returned_value_surface.payload_summary,
        "Error(Blocked)"
    );
}

#[test]
fn evaluator_projects_info_unsupported_query_as_typed_host_provider_outcome() {
    let output = evaluate(
        "=INFO(\"system\")",
        None,
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Error(Value)");
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        output
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .expect("typed host/provider outcome should exist")
            .outcome_kind,
        oxfml_core::HostProviderOutcomeKind::UnsupportedQuery
    );
}

#[test]
fn evaluator_projects_cell_provider_failure_as_typed_host_provider_outcome() {
    let output = evaluate(
        "=CELL(\"filename\",A1)",
        None,
        Some(&FailingHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Error(Value)");
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        output
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .expect("typed host/provider outcome should exist")
            .outcome_kind,
        oxfml_core::HostProviderOutcomeKind::ProviderFailure
    );
}

#[test]
fn evaluator_projects_rtd_value_as_typed_host_provider_outcome() {
    let output = evaluate_with_rtd_provider(
        "=RTD(\"prog\",\"server\",\"topic\")",
        None,
        None,
        Some(&ValueRtdProvider),
        Some(&en_us_context()),
    )
    .expect("evaluation should succeed");
    assert_eq!(output.result.payload_summary, "Number(7)");
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        output
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .expect("typed host/provider outcome should exist")
            .outcome_kind,
        oxfml_core::HostProviderOutcomeKind::Value
    );
}

#[test]
fn evaluator_projects_rtd_capability_denied_as_typed_host_provider_outcome() {
    let output = evaluate_with_rtd_provider(
        "=RTD(\"prog\",\"server\",\"topic\")",
        None,
        None,
        Some(&CapabilityDeniedRtdProvider),
        Some(&en_us_context()),
    )
    .expect("evaluation should succeed");
    assert_eq!(output.result.payload_summary, "Error(Blocked)");
    assert_eq!(
        output.returned_value_surface.kind,
        oxfml_core::ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        output
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .expect("typed host/provider outcome should exist")
            .outcome_kind,
        oxfml_core::HostProviderOutcomeKind::CapabilityDenied
    );
}

#[test]
fn evaluator_runs_row_and_column_with_caller_context() {
    let row_output = evaluate("=ROW()", None, None, Some(&en_us_context()));
    assert_eq!(row_output.result.payload_summary, "Number(1)");
    assert_eq!(row_output.trace.prepared_calls[0].function_id, "FUNC.ROW");
    assert_eq!(
        row_output.result.capability_dependencies,
        vec!["caller_context".to_string()]
    );

    let column_output = evaluate("=COLUMN(A1:B2)", None, None, Some(&en_us_context()));
    assert_eq!(column_output.result.payload_summary, "Array(1x2)");
    assert_eq!(
        column_output.trace.prepared_calls[0].function_id,
        "FUNC.COLUMN"
    );
}

#[test]
fn evaluator_runs_unary_negative_literal_through_function_calls() {
    let sign_output = evaluate("=SIGN(-5)", None, None, Some(&en_us_context()));
    assert_eq!(sign_output.oxfunc_value, EvalValue::Number(-1.0));

    let pv_output = evaluate("=PV(0.05,10,-100)", None, None, Some(&en_us_context()));
    match pv_output.oxfunc_value {
        EvalValue::Number(value) => assert!((value - 772.1734929184818).abs() < 1e-9),
        other => panic!("expected numeric PV result, got {other:?}"),
    }

    let fv_output = evaluate("=FV(0.05,10,-100)", None, None, Some(&en_us_context()));
    match fv_output.oxfunc_value {
        EvalValue::Number(value) => assert!((value - 1257.789253554884).abs() < 1e-9),
        other => panic!("expected numeric FV result, got {other:?}"),
    }
}

#[test]
fn evaluator_matches_current_exponentiation_empirical_baseline() {
    let chained = evaluate("=2^3^2", None, None, Some(&en_us_context()));
    assert_eq!(chained.oxfunc_value, EvalValue::Number(64.0));
    assert_eq!(
        chained
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_POWER", "FUNC.OP_POWER"]
    );

    let unary = evaluate("=-2^2", None, None, Some(&en_us_context()));
    assert_eq!(unary.oxfunc_value, EvalValue::Number(4.0));
    assert_eq!(
        unary
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_NEGATE", "FUNC.OP_POWER"]
    );

    let multiplied = evaluate("=2^2*3", None, None, Some(&en_us_context()));
    assert_eq!(multiplied.oxfunc_value, EvalValue::Number(12.0));
    assert_eq!(
        multiplied
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_POWER", "FUNC.OP_MULTIPLY"]
    );

    let additive = evaluate("=1+2*3^2", None, None, Some(&en_us_context()));
    assert_eq!(additive.oxfunc_value, EvalValue::Number(19.0));
    assert_eq!(
        additive
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_POWER", "FUNC.OP_MULTIPLY", "FUNC.OP_ADD"]
    );
}

#[test]
fn evaluator_dispatches_percent_concat_and_comparison_operators_to_oxfunc() {
    let percent = evaluate("=50%", None, None, Some(&en_us_context()));
    assert_eq!(percent.oxfunc_value, EvalValue::Number(0.5));
    assert_eq!(
        percent
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_PERCENT"]
    );

    let concat = evaluate("=1&2", None, None, Some(&en_us_context()));
    assert_eq!(
        concat.oxfunc_value,
        EvalValue::Text(ExcelText::from_interop_assignment("12"))
    );
    assert_eq!(
        concat
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_CONCAT"]
    );

    let compared = evaluate("=\"1\"&\"2\"=\"12\"", None, None, Some(&en_us_context()));
    assert_eq!(compared.oxfunc_value, EvalValue::Logical(true));
    assert_eq!(
        compared
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_CONCAT", "FUNC.OP_EQUAL"]
    );

    let ordered = evaluate("=1<2", None, None, Some(&en_us_context()));
    assert_eq!(ordered.oxfunc_value, EvalValue::Logical(true));
    assert_eq!(
        ordered
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_LESS_THAN"]
    );

    let not_equal = evaluate("=1<>2", None, None, Some(&en_us_context()));
    assert_eq!(not_equal.oxfunc_value, EvalValue::Logical(true));
    assert_eq!(
        not_equal
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_NOT_EQUAL"]
    );

    let greater_equal = evaluate("=2>=1", None, None, Some(&en_us_context()));
    assert_eq!(greater_equal.oxfunc_value, EvalValue::Logical(true));
    assert_eq!(
        greater_equal
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_GREATER_EQUAL"]
    );

    let bool_concat = evaluate("=TRUE&FALSE", None, None, Some(&en_us_context()));
    assert_eq!(
        bool_concat.oxfunc_value,
        EvalValue::Text(ExcelText::from_interop_assignment("TRUEFALSE"))
    );
}

#[test]
fn evaluator_supports_scientific_numeric_literals() {
    let literal = evaluate("=1E+3+2", None, None, Some(&en_us_context()));
    assert_eq!(literal.oxfunc_value, EvalValue::Number(1002.0));
    assert_eq!(
        literal
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_ADD"]
    );

    let leading_decimal = evaluate("=.5E+1", None, None, Some(&en_us_context()));
    assert_eq!(leading_decimal.oxfunc_value, EvalValue::Number(5.0));
}

#[test]
fn evaluator_maps_negative_fractional_power_to_num_error() {
    let output = evaluate("=(-1)^0.5", None, None, Some(&en_us_context()));
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Num)
    );
    assert_eq!(output.result.payload_summary, "Error(Num)");
}

#[test]
fn evaluator_projects_ordinary_worksheet_errors_as_values() {
    let output = evaluate("=ABS(\"x\")", None, None, Some(&en_us_context()));
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Value)
    );
    assert_eq!(output.result.payload_summary, "Error(Value)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.ABS");
}

#[test]
fn evaluator_matches_current_if_empty_text_excel_outcome() {
    let output = evaluate("=IF(\"\",1,2)", None, None, Some(&en_us_context()));
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Value)
    );
    assert_eq!(output.result.payload_summary, "Error(Value)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.IF");
}

#[test]
fn evaluator_consumes_broader_float_comparison_family_split() {
    let tolerant_operator = evaluate("=0.1+0.2=0.3", None, None, Some(&en_us_context()));
    assert_eq!(tolerant_operator.oxfunc_value, EvalValue::Logical(true));
    assert_eq!(
        tolerant_operator
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_ADD", "FUNC.OP_EQUAL"]
    );

    let tolerant_switch = evaluate(
        "=SWITCH(((123456789012345*10)+5)/1E25,((123456789012345*10)+4)/1E25,1,0)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(tolerant_switch.oxfunc_value, EvalValue::Number(1.0));

    let exact_delta = evaluate(
        "=DELTA(((123456789012345*10)+5)/1E25,((123456789012345*10)+4)/1E25)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(exact_delta.oxfunc_value, EvalValue::Number(0.0));
}

#[test]
fn evaluator_dispatches_range_and_intersection_reference_operators_to_oxfunc() {
    let range = evaluate("=SUM(A1:B2)", None, None, Some(&en_us_context()));
    assert_eq!(range.oxfunc_value, EvalValue::Number(31.0));
    assert_eq!(
        range
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.SUM"]
    );

    let intersection = evaluate("=SUM((A1:B2 A2:B2))", None, None, Some(&en_us_context()));
    assert_eq!(intersection.oxfunc_value, EvalValue::Number(24.0));
    assert_eq!(
        intersection
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_INTERSECTION_REF", "FUNC.SUM"]
    );

    let union = evaluate("=SUM((A1:A2,B2))", None, None, Some(&en_us_context()));
    assert_eq!(union.oxfunc_value, EvalValue::Number(31.0));
    assert_eq!(
        union
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_UNION_REF", "FUNC.SUM"]
    );
}

#[test]
fn evaluator_materializes_same_sheet_prefixed_multi_area_references() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!A2".to_string(), EvalValue::Number(11.0));
    extra_cells.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));

    let output = evaluate_with_cells("=SUM((Alpha!A1:A2,Alpha!B2))", extra_cells);
    assert_eq!(output.oxfunc_value, EvalValue::Number(31.0));
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_UNION_REF", "FUNC.SUM"]
    );
}

#[test]
fn evaluator_resolves_direct_sheet_qualified_cell_reference_end_to_end() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(17.0));

    let output = evaluate_with_cells("=Alpha!A1", extra_cells);
    assert_eq!(output.oxfunc_value, EvalValue::Number(17.0));
}

#[test]
fn evaluator_resolves_direct_sheet_qualified_area_reference_end_to_end() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!A2".to_string(), EvalValue::Number(11.0));

    let output = evaluate_with_cells("=SUM(Alpha!A1:A2)", extra_cells);
    assert_eq!(output.oxfunc_value, EvalValue::Number(18.0));
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.SUM"]
    );
}

#[test]
fn evaluator_rejects_mixed_sheet_multi_area_end_to_end() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!A2".to_string(), EvalValue::Number(11.0));
    extra_cells.insert("Beta!B2".to_string(), EvalValue::Number(13.0));

    let got = evaluate_with_cells_result("=SUM((Alpha!A1:A2,Beta!B2))", extra_cells);
    let err = got.expect_err("mixed-sheet multi-area should reject");
    assert!(
        err.message.contains("mixed_sheet_multi_area"),
        "expected mixed-sheet multi-area failure, got {}",
        err.message
    );
}

#[test]
fn evaluator_preserves_sheet_qualified_whole_row_reference_for_reference_visible_function() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));
    extra_cells.insert("Alpha!C3".to_string(), EvalValue::Number(17.0));

    let output = evaluate_with_cells("=ROWS(Alpha!1:3)", extra_cells);
    assert_eq!(output.oxfunc_value, EvalValue::Number(3.0));
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.ROWS");
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[0].evaluation_mode,
        oxfml_core::PreparedEvaluationMode::ReferencePreserved
    );
}

#[test]
fn evaluator_preserves_sheet_qualified_whole_column_reference_for_reference_visible_function() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!A2".to_string(), EvalValue::Number(11.0));
    extra_cells.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));

    let output = evaluate_with_cells("=COLUMNS(Alpha!A:B)", extra_cells);
    assert_eq!(output.oxfunc_value, EvalValue::Number(2.0));
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.COLUMNS");
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[0].evaluation_mode,
        oxfml_core::PreparedEvaluationMode::ReferencePreserved
    );
}

#[test]
fn evaluator_rejects_sheet_qualified_whole_row_reference_in_local_value_only_lane() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));
    extra_cells.insert("Alpha!C3".to_string(), EvalValue::Number(17.0));

    let got = evaluate_with_cells_result("=SUM(Alpha!1:3)", extra_cells);
    let err = got.expect_err("whole-row local value-only deref should reject honestly");
    assert!(
        err.message.contains("UnresolvedReference") && err.message.contains("Alpha!1:3"),
        "expected unresolved whole-row reference failure, got {}",
        err.message
    );
}

#[test]
fn evaluator_rejects_sheet_qualified_whole_column_reference_in_local_value_only_lane() {
    let mut extra_cells = BTreeMap::new();
    extra_cells.insert("Alpha!A1".to_string(), EvalValue::Number(7.0));
    extra_cells.insert("Alpha!A2".to_string(), EvalValue::Number(11.0));
    extra_cells.insert("Alpha!B2".to_string(), EvalValue::Number(13.0));

    let got = evaluate_with_cells_result("=SUM(Alpha!A:B)", extra_cells);
    let err = got.expect_err("whole-column local value-only deref should reject honestly");
    assert!(
        err.message.contains("UnresolvedReference") && err.message.contains("Alpha!A:B"),
        "expected unresolved whole-column reference failure, got {}",
        err.message
    );
}

#[test]
fn evaluator_lifts_binary_arithmetic_over_array_literals_and_scalar_negation() {
    let output = evaluate("={1,2,3;2,3,4}*-1", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Array(2x3)");
    assert_eq!(
        array_numbers(&output.oxfunc_value),
        vec![-1.0, -2.0, -3.0, -2.0, -3.0, -4.0]
    );
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["FUNC.OP_NEGATE", "FUNC.OP_MULTIPLY"]
    );
}

#[test]
fn evaluator_lifts_unary_minus_over_array_literals_via_unary_dispatch() {
    let output = evaluate("=-{1,2,3;2,3,4}", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Array(2x3)");
    assert_eq!(
        array_numbers(&output.oxfunc_value),
        vec![-1.0, -2.0, -3.0, -2.0, -3.0, -4.0]
    );
}

#[test]
fn evaluator_lifts_binary_arithmetic_over_same_shape_arrays() {
    let output = evaluate(
        "={1,2;3,4}+{10,20;30,40}",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(2x2)");
    assert_eq!(
        array_numbers(&output.oxfunc_value),
        vec![11.0, 22.0, 33.0, 44.0]
    );
}

#[test]
fn evaluator_lifts_division_over_arrays_and_preserves_element_errors() {
    let output = evaluate("={8,6;4,2}/{2,0;1,2}", None, None, Some(&en_us_context()));
    let EvalValue::Array(array) = &output.oxfunc_value else {
        panic!("expected array result, got {:?}", output.oxfunc_value);
    };
    assert_eq!(output.result.payload_summary, "Array(2x2)");
    assert_eq!(array.get(0, 0), Some(&ArrayCellValue::Number(4.0)));
    assert_eq!(
        array.get(0, 1),
        Some(&ArrayCellValue::Error(
            oxfunc_core::value::WorksheetErrorCode::Div0
        ))
    );
    assert_eq!(array.get(1, 0), Some(&ArrayCellValue::Number(4.0)));
    assert_eq!(array.get(1, 1), Some(&ArrayCellValue::Number(1.0)));
}

#[derive(Debug, Deserialize)]
struct OperatorArrayArithmeticFixture {
    case_id: String,
    formula: String,
    expected_payload_summary: String,
    expected_value_summary: String,
}

#[test]
fn evaluator_operator_array_arithmetic_fixture_corpus_matches_expected_values() {
    let fixtures: Vec<OperatorArrayArithmeticFixture> =
        load_json_fixture("operator_array_arithmetic_cases.json");

    for fixture in fixtures {
        let output = evaluate(&fixture.formula, None, None, Some(&en_us_context()));
        assert_eq!(
            output.result.payload_summary, fixture.expected_payload_summary,
            "payload summary mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            eval_value_summary(&output.oxfunc_value),
            fixture.expected_value_summary,
            "value summary mismatch for {}",
            fixture.case_id
        );
    }
}

#[test]
fn evaluator_treats_absent_single_cell_reference_as_true_blank() {
    let isblank_output = evaluate("=ISBLANK(A9)", None, None, Some(&en_us_context()));
    assert_eq!(isblank_output.oxfunc_value, EvalValue::Logical(true));

    let n_output = evaluate("=N(A9)", None, None, Some(&en_us_context()));
    assert_eq!(n_output.oxfunc_value, EvalValue::Number(0.0));

    let type_output = evaluate("=TYPE(A9)", None, None, Some(&en_us_context()));
    assert_eq!(type_output.oxfunc_value, EvalValue::Number(1.0));
}

#[test]
fn evaluator_runs_indirect_offset_and_iferror() {
    let indirect_output = evaluate("=INDIRECT(\"A1\")", None, None, Some(&en_us_context()));
    assert_eq!(
        indirect_output.trace.prepared_calls[0].function_id,
        "FUNC.INDIRECT"
    );

    let offset_output = evaluate("=OFFSET(A1,0,0)", None, None, Some(&en_us_context()));
    assert_eq!(
        offset_output.trace.prepared_calls[0].function_id,
        "FUNC.OFFSET"
    );

    let iferror_output = evaluate(
        "=IFERROR(UnknownName,2)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(iferror_output.result.payload_summary, "Number(2)");
    assert_eq!(
        iferror_output.trace.prepared_calls[0].function_id,
        "FUNC.IFERROR"
    );
}

#[test]
fn evaluator_preserves_if_branch_laziness_locally() {
    let output = evaluate("=IF(TRUE,1,1/0)", None, None, Some(&en_us_context()));
    assert_eq!(output.oxfunc_value, EvalValue::Number(1.0));
}

#[test]
fn evaluator_preserves_iferror_fallback_laziness_locally() {
    let output = evaluate("=IFERROR(1,1/0)", None, None, Some(&en_us_context()));
    assert_eq!(output.oxfunc_value, EvalValue::Number(1.0));
}

#[test]
fn evaluator_runs_now_and_today_with_supplied_serial() {
    let now_output = evaluate("=NOW()", None, None, Some(&en_us_context()));
    assert_eq!(now_output.result.payload_summary, "Number(46000)");

    let today_output = evaluate("=TODAY()", None, None, Some(&en_us_context()));
    assert_eq!(today_output.result.payload_summary, "Number(46000)");
}

#[test]
fn evaluator_uses_defined_name_bindings_for_sum() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "InputValue".to_string(),
        DefinedNameBinding::Value(EvalValue::Number(5.0)),
    );

    let output = evaluate(
        "=SUM(InputValue,2)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(7)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.SUM");
}

#[test]
fn evaluator_uses_defined_name_reference_for_cell_contents() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "InputRef".to_string(),
        DefinedNameBinding::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        }),
    );

    let output = evaluate(
        "=CELL(\"contents\",InputRef)",
        Some(bindings),
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(7)");
}

#[test]
fn evaluator_runs_let_with_helper_bindings() {
    let output = evaluate("=LET(x,1,x+2)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(3)");
}

#[test]
fn evaluator_runs_let_with_reference_preserved_binding() {
    let output = evaluate("=LET(r,A1,SUM(r,2))", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(9)");
}

#[test]
fn evaluator_runs_legacy_single_compat() {
    let output = evaluate("=_xlfn.SINGLE(A1)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(7)");
}

#[test]
fn evaluator_returns_lambda_value_summary() {
    let output = evaluate("=LAMBDA(x,x+1)", None, None, Some(&en_us_context()));
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=x;optional_params=-;captures=-;body=Binary)"
    );
    assert_eq!(
        output.result.callable_profile.as_deref(),
        Some("arity=1;required_arity=1;params=x;optional_params=-;captures=-;body=Binary")
    );
    let carrier = output
        .result
        .callable_carrier
        .as_ref()
        .expect("callable carrier should exist");
    assert_eq!(
        carrier.origin_kind,
        oxfml_core::CallableOriginKind::HelperLambda
    );
    assert_eq!(
        carrier.invocation_model,
        oxfml_core::CallableInvocationModel::TypedInvocationOnly
    );
    assert_eq!(
        carrier.capture_mode,
        oxfml_core::CallableCaptureMode::NoCapture
    );
    assert_eq!(carrier.arity, 1);
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert_eq!(detail.arity, 1);
    assert_eq!(detail.required_arity, 1);
    assert_eq!(detail.parameter_names, vec!["x".to_string()]);
    assert!(detail.optional_parameter_names.is_empty());
    assert!(detail.capture_names.is_empty());
    assert_eq!(detail.body_kind, "Binary");
}

#[test]
fn evaluator_returns_lambda_value_summary_with_lexical_capture_metadata() {
    let output = evaluate(
        "=LET(x,10,LAMBDA(y,x+y))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=y;optional_params=-;captures=x;body=Binary)"
    );
    assert_eq!(
        output.result.callable_profile.as_deref(),
        Some("arity=1;required_arity=1;params=y;optional_params=-;captures=x;body=Binary")
    );
    let carrier = output
        .result
        .callable_carrier
        .as_ref()
        .expect("callable carrier should exist");
    assert_eq!(
        carrier.origin_kind,
        oxfml_core::CallableOriginKind::HelperLambda
    );
    assert_eq!(
        carrier.invocation_model,
        oxfml_core::CallableInvocationModel::TypedInvocationOnly
    );
    assert_eq!(
        carrier.capture_mode,
        oxfml_core::CallableCaptureMode::LexicalCapture
    );
    assert_eq!(carrier.arity, 1);
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert_eq!(detail.arity, 1);
    assert_eq!(detail.required_arity, 1);
    assert_eq!(detail.parameter_names, vec!["y".to_string()]);
    assert!(detail.optional_parameter_names.is_empty());
    assert_eq!(detail.capture_names, vec!["x".to_string()]);
    assert_eq!(detail.body_kind, "Binary");
}

#[test]
fn evaluator_runs_immediate_lambda_invocation() {
    let output = evaluate("=LAMBDA(x,x+1)(2)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(3)");
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["SPECIAL.LAMBDA_INVOKE", "FUNC.OP_ADD"]
    );
}

#[test]
fn evaluator_runs_helper_bound_lambda_invocation() {
    let output = evaluate(
        "=LET(f,LAMBDA(x,x+1),f(2))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(3)");
    let function_ids = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id)
        .collect::<Vec<_>>();
    assert_eq!(
        function_ids,
        vec![
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "FUNC.OP_ADD",
            "SPECIAL.LET"
        ]
    );
}

#[test]
fn evaluator_runs_helper_bound_lambda_power_invocation() {
    let output = evaluate(
        "=LET(f,LAMBDA(x,x^2),f(3))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(9)");
    let function_ids = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id)
        .collect::<Vec<_>>();
    assert_eq!(
        function_ids,
        vec![
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "FUNC.OP_POWER",
            "SPECIAL.LET"
        ]
    );
}

#[test]
fn evaluator_resolves_helper_bound_lambda_arguments_in_caller_scope() {
    let output = evaluate(
        "=LET(a,3,f,LAMBDA(x,x^2),f(a))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(9)");
    let function_ids = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id)
        .collect::<Vec<_>>();
    assert_eq!(
        function_ids,
        vec![
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "FUNC.OP_POWER",
            "SPECIAL.LET"
        ]
    );
}

#[test]
fn evaluator_uses_lexical_not_dynamic_scope_for_helper_bound_lambda() {
    let output = evaluate(
        "=LET(x,10,f,LAMBDA(y,x+y),LET(x,20,f(2)))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(12)");
}

#[test]
fn evaluator_invokes_defined_name_callable_binding() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "NamedLambda".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=x;captures=-;body=Binary",
            vec!["x"],
            BoundExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(name_ref_expr("x", NameKind::HelperLocal)),
                right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate(
        "=NamedLambda(2)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(3)");
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec!["SPECIAL.LAMBDA_INVOKE", "FUNC.OP_ADD"]
    );
}

#[test]
fn evaluator_preserves_defined_name_callable_as_first_class_value() {
    let mut bindings = BTreeMap::new();
    let mut closure = BTreeMap::new();
    closure.insert(
        "x".to_string(),
        DefinedNameBinding::Value(EvalValue::Number(10.0)),
    );
    bindings.insert(
        "NamedLambda".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=y;captures=x;body=Binary",
            vec!["y"],
            BoundExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(name_ref_expr("x", NameKind::ValueLike)),
                right: Box::new(name_ref_expr("y", NameKind::HelperLocal)),
            },
            closure,
        )),
    );

    let value_output = evaluate(
        "=NamedLambda",
        Some(bindings.clone()),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        value_output.result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=y;optional_params=-;captures=x;body=Binary)"
    );
    assert_eq!(
        value_output
            .result
            .callable_carrier
            .as_ref()
            .expect("callable carrier should exist")
            .origin_kind,
        oxfml_core::CallableOriginKind::DefinedNameCallable
    );
    assert_eq!(
        value_output
            .result
            .callable_carrier
            .as_ref()
            .expect("callable carrier should exist")
            .capture_mode,
        oxfml_core::CallableCaptureMode::LexicalCapture
    );

    let invoke_output = evaluate(
        "=NamedLambda(2)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(invoke_output.result.payload_summary, "Number(12)");
}

#[test]
fn evaluator_lambda_summary_ignores_unused_helper_bindings() {
    let output = evaluate(
        "=LET(x,10,unused,99,LAMBDA(y,x+y))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=y;optional_params=-;captures=x;body=Binary)"
    );
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert_eq!(detail.capture_names, vec!["x".to_string()]);
}

#[test]
fn evaluator_lambda_summary_respects_parameter_shadowing() {
    let output = evaluate(
        "=LET(x,10,LAMBDA(x,x+1))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=x;optional_params=-;captures=-;body=Binary)"
    );
    let carrier = output
        .result
        .callable_carrier
        .as_ref()
        .expect("callable carrier should exist");
    assert_eq!(
        carrier.capture_mode,
        oxfml_core::CallableCaptureMode::NoCapture
    );
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert!(detail.capture_names.is_empty());
}

#[test]
fn evaluator_surfaces_typed_external_reference_deferment() {
    let output = evaluate("=[Book.xlsx]Sheet2!A1", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Error(Ref)");
    assert_eq!(
        output.result.deferred_reason.as_deref(),
        Some("external_reference_deferred")
    );
    assert_eq!(
        output.result.capability_dependencies,
        vec!["external_reference".to_string()]
    );
    assert_eq!(output.trace.prepared_calls.len(), 1);
    assert_eq!(
        output.trace.prepared_calls[0].function_id,
        "SPECIAL.EXTERNAL_REFERENCE_DEFERRED"
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[0].source_class,
        oxfml_core::PreparedSourceClass::ExternalReference
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[0]
            .opaque_reason
            .as_deref(),
        Some("external_reference_deferred")
    );
}

#[test]
fn evaluator_runs_index_and_xmatch_catalog_lanes() {
    let index_output = evaluate("=INDEX(SEQUENCE(3),2)", None, None, Some(&en_us_context()));
    assert_eq!(index_output.result.payload_summary, "Number(2)");
    assert_eq!(
        index_output.trace.prepared_calls[0].function_id,
        "FUNC.SEQUENCE"
    );
    assert_eq!(
        index_output.trace.prepared_calls[1].function_id,
        "FUNC.INDEX"
    );

    let xmatch_output = evaluate("=XMATCH(3,SEQUENCE(5))", None, None, Some(&en_us_context()));
    assert_eq!(xmatch_output.result.payload_summary, "Number(3)");
    assert_eq!(
        xmatch_output.trace.prepared_calls[0].function_id,
        "FUNC.SEQUENCE"
    );
    assert_eq!(
        xmatch_output.trace.prepared_calls[1].function_id,
        "FUNC.XMATCH"
    );
}

#[test]
fn evaluator_executes_map_with_local_lambda_callable() {
    let output = evaluate(
        "=MAP(SEQUENCE(3),LAMBDA(x,x+1))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(3x1)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![2.0, 3.0, 4.0]);
    let function_ids = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id)
        .collect::<Vec<_>>();
    assert_eq!(
        function_ids,
        vec!["FUNC.SEQUENCE", "SPECIAL.LAMBDA", "FUNC.MAP"]
    );
}

#[test]
fn evaluator_executes_reduce_with_local_lambda_callable() {
    let output = evaluate(
        "=REDUCE(0,SEQUENCE(3),LAMBDA(a,b,a+b))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(6)");
    assert_eq!(output.oxfunc_value, EvalValue::Number(6.0));
}

#[test]
fn evaluator_executes_scan_with_local_lambda_callable() {
    let output = evaluate(
        "=SCAN(0,SEQUENCE(3),LAMBDA(a,b,a+b))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(3x1)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![1.0, 3.0, 6.0]);
}

#[test]
fn evaluator_executes_map_with_helper_bound_lambda_callable() {
    let output = evaluate(
        "=LET(f,LAMBDA(x,x+1),MAP(SEQUENCE(3),f))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(3x1)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![2.0, 3.0, 4.0]);
    let function_ids = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id)
        .collect::<Vec<_>>();
    assert_eq!(
        function_ids,
        vec!["SPECIAL.LAMBDA", "FUNC.SEQUENCE", "FUNC.MAP", "SPECIAL.LET"]
    );
}

#[test]
fn evaluator_executes_map_with_defined_name_callable() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "NamedLambda".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=x;captures=-;body=Binary",
            vec!["x"],
            BoundExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(name_ref_expr("x", NameKind::HelperLocal)),
                right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate(
        "=MAP(SEQUENCE(3),NamedLambda)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(3x1)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![2.0, 3.0, 4.0]);
}

#[test]
fn evaluator_executes_byrow_with_local_lambda_callable() {
    let output = evaluate(
        "=BYROW(SEQUENCE(2,2),LAMBDA(r,SUM(r)))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(2x1)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![3.0, 7.0]);
}

#[test]
fn evaluator_executes_bycol_with_local_lambda_callable() {
    let output = evaluate(
        "=BYCOL(SEQUENCE(2,2),LAMBDA(c,SUM(c)))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(1x2)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![4.0, 6.0]);
}

#[test]
fn evaluator_executes_makearray_with_local_lambda_callable() {
    let output = evaluate(
        "=MAKEARRAY(2,3,LAMBDA(r,c,r+c))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(2x3)");
    assert_eq!(
        array_numbers(&output.oxfunc_value),
        vec![2.0, 3.0, 4.0, 3.0, 4.0, 5.0]
    );
}

#[test]
fn evaluator_runs_isomitted_with_present_arg() {
    let output = evaluate("=ISOMITTED(1)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Logical(false)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.ISOMITTED");
}

#[test]
fn evaluator_distinguishes_lambda_underapplication_from_isomitted() {
    let present = evaluate(
        "=LAMBDA(a,ISOMITTED(a))(3)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(present.result.payload_summary, "Logical(false)");

    let underapplied = evaluate_with_rtd_provider(
        "=LAMBDA(a,ISOMITTED(a))()",
        None,
        None,
        None,
        Some(&en_us_context()),
    );
    let error = underapplied.expect_err("underapplication should fail before ISOMITTED is useful");
    assert!(error.message.contains("lambda invocation arity mismatch"));
}

#[test]
fn evaluator_executes_map_with_isomitted_for_present_args() {
    let output = evaluate(
        "=MAP(SEQUENCE(2),LAMBDA(a,ISOMITTED(a)))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(2x1)");
    assert_eq!(array_logicals(&output.oxfunc_value), vec![false, false]);
}

#[test]
fn evaluator_preserves_explicit_omitted_placeholder_for_plain_lambda_params() {
    let output = evaluate(
        "=LAMBDA(a,b,ISOMITTED(b))(1,)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Logical(true)");
}

#[test]
fn evaluator_executes_direct_lambda_with_optional_bracket_parameter() {
    let omitted = evaluate(
        "=LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(omitted.result.payload_summary, "Number(10)");
    assert_eq!(omitted.oxfunc_value, EvalValue::Number(10.0));

    let present = evaluate(
        "=LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5,3)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(present.result.payload_summary, "Number(8)");
    assert_eq!(present.oxfunc_value, EvalValue::Number(8.0));
}

#[test]
fn evaluator_executes_helper_bound_lambda_with_optional_bracket_parameter() {
    let omitted = evaluate(
        "=LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(omitted.result.payload_summary, "Number(10)");
    assert_eq!(omitted.oxfunc_value, EvalValue::Number(10.0));

    let present = evaluate(
        "=LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5,3))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(present.result.payload_summary, "Number(8)");
    assert_eq!(present.oxfunc_value, EvalValue::Number(8.0));
}

#[test]
fn evaluator_executes_map_with_optional_lambda_parameter_omitted_by_helper() {
    let output = evaluate(
        "=MAP(SEQUENCE(2),LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Array(2x1)");
    assert_eq!(array_numbers(&output.oxfunc_value), vec![2.0, 4.0]);
}

#[test]
fn evaluator_executes_helper_bound_returned_lambda_invocation() {
    let output = evaluate(
        "=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.oxfunc_value, EvalValue::Number(15.0));
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec![
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "FUNC.OP_ADD",
            "SPECIAL.LET",
        ]
    );
}

#[test]
fn evaluator_returns_lambda_value_from_helper_bound_returned_lambda() {
    let output = evaluate(
        "=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),adder(5))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert!(matches!(output.oxfunc_value, EvalValue::Lambda(_)));
    assert_eq!(
        output
            .result
            .callable_carrier
            .as_ref()
            .map(|carrier| carrier.arity),
        Some(1)
    );
}

#[test]
fn evaluator_executes_nested_returned_lambda_invocation() {
    let output = evaluate(
        "=LAMBDA(n,LAMBDA(x,x+n))(5)(10)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.oxfunc_value, EvalValue::Number(15.0));
    assert_eq!(
        output
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec![
            "SPECIAL.LAMBDA_INVOKE",
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "FUNC.OP_ADD",
        ]
    );
}

#[test]
fn evaluator_executes_returned_lambda_with_lexical_capture() {
    let output = evaluate(
        "=LET(base,100,adder,LAMBDA(n,LAMBDA(x,x+n+base)),add5,adder(5),add5(10))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.oxfunc_value, EvalValue::Number(115.0));
}

#[test]
fn evaluator_projects_runaway_recursive_defined_name_callable_as_num_error() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "Loop".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=0;params=-;captures=-;body=Invocation",
            vec![],
            BoundExpr::Invocation {
                callee: Box::new(name_ref_expr("Loop", NameKind::ValueLike)),
                args: vec![],
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate("=Loop()", Some(bindings), None, Some(&en_us_context()));
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Num)
    );
}

#[test]
fn evaluator_executes_bounded_recursive_defined_name_callable() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "Fact".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=n;captures=-;body=FunctionCall",
            vec!["n"],
            BoundExpr::FunctionCall {
                function_name: "IF".to_string(),
                args: vec![
                    BoundExpr::Binary {
                        op: BinaryOp::LessEqual,
                        left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                        right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
                    },
                    BoundExpr::NumberLiteral("1".to_string()),
                    BoundExpr::Binary {
                        op: BinaryOp::Multiply,
                        left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                        right: Box::new(BoundExpr::Invocation {
                            callee: Box::new(name_ref_expr("Fact", NameKind::ValueLike)),
                            args: vec![BoundExpr::Binary {
                                op: BinaryOp::Subtract,
                                left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                                right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
                            }],
                        }),
                    },
                ],
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate("=Fact(5)", Some(bindings), None, Some(&en_us_context()));
    assert_eq!(output.oxfunc_value, EvalValue::Number(120.0));
}

#[test]
fn evaluator_matches_excel_named_recursion_success_boundary() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "CountDown".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=n;captures=-;body=FunctionCall",
            vec!["n"],
            BoundExpr::FunctionCall {
                function_name: "IF".to_string(),
                args: vec![
                    BoundExpr::Binary {
                        op: BinaryOp::LessEqual,
                        left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                        right: Box::new(BoundExpr::NumberLiteral("0".to_string())),
                    },
                    BoundExpr::NumberLiteral("0".to_string()),
                    BoundExpr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(BoundExpr::NumberLiteral("1".to_string())),
                        right: Box::new(BoundExpr::Invocation {
                            callee: Box::new(name_ref_expr("CountDown", NameKind::ValueLike)),
                            args: vec![BoundExpr::Binary {
                                op: BinaryOp::Subtract,
                                left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                                right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
                            }],
                        }),
                    },
                ],
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate(
        "=CountDown(5460)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.oxfunc_value, EvalValue::Number(5460.0));
}

#[test]
fn evaluator_matches_excel_named_recursion_failure_boundary() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "CountDown".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=n;captures=-;body=FunctionCall",
            vec!["n"],
            BoundExpr::FunctionCall {
                function_name: "IF".to_string(),
                args: vec![
                    BoundExpr::Binary {
                        op: BinaryOp::LessEqual,
                        left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                        right: Box::new(BoundExpr::NumberLiteral("0".to_string())),
                    },
                    BoundExpr::NumberLiteral("0".to_string()),
                    BoundExpr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(BoundExpr::NumberLiteral("1".to_string())),
                        right: Box::new(BoundExpr::Invocation {
                            callee: Box::new(name_ref_expr("CountDown", NameKind::ValueLike)),
                            args: vec![BoundExpr::Binary {
                                op: BinaryOp::Subtract,
                                left: Box::new(name_ref_expr("n", NameKind::HelperLocal)),
                                right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
                            }],
                        }),
                    },
                ],
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate(
        "=CountDown(5461)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Num)
    );
}

#[test]
fn evaluator_matches_excel_let_self_application_recursion_success_boundary() {
    let output = evaluate(
        "=LET(F,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),F(F,4094))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.oxfunc_value, EvalValue::Number(4094.0));
}

#[test]
fn evaluator_matches_excel_let_self_application_recursion_failure_boundary() {
    let output = evaluate(
        "=LET(F,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),F(F,4095))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Num)
    );
}

#[test]
fn evaluator_projects_direct_helper_local_self_recursion_as_name_error() {
    let output = evaluate(
        "=LET(F,LAMBDA(n,IF(n<=0,0,1+F(n-1))),F(5))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Name)
    );
}

fn evaluate(
    formula: &str,
    defined_names: Option<BTreeMap<String, DefinedNameBinding>>,
    host_info: Option<&dyn HostInfoProvider>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> oxfml_core::EvaluationOutput {
    evaluate_with_rtd_provider(formula, defined_names, host_info, None, locale_ctx)
        .expect("evaluation should succeed")
}

fn evaluate_with_cells(
    formula: &str,
    extra_cells: BTreeMap<String, EvalValue>,
) -> oxfml_core::EvaluationOutput {
    evaluate_with_cells_result(formula, extra_cells).expect("evaluation should succeed")
}

fn evaluate_with_cells_result(
    formula: &str,
    extra_cells: BTreeMap<String, EvalValue>,
) -> Result<oxfml_core::EvaluationOutput, oxfml_core::eval::EvaluationError> {
    let compiled = common::compile_formula(
        "eval-fixture",
        formula,
        BTreeMap::new(),
        "eval-struct-v1",
        "oxfunc:test",
    );

    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context
        .cell_values
        .insert("A1".to_string(), EvalValue::Number(7.0));
    context
        .cell_values
        .insert("A2".to_string(), EvalValue::Number(11.0));
    context
        .cell_values
        .insert("B2".to_string(), EvalValue::Number(13.0));
    context.cell_values.extend(extra_cells);
    let locale = en_us_context();
    context.apply_typed_context_query_bundle(TypedContextQueryBundle::new(
        None,
        None,
        Some(&locale),
        Some(46000.0),
        Some(0.25),
    ));

    evaluate_formula(context)
}

fn evaluate_with_rtd_provider(
    formula: &str,
    defined_names: Option<BTreeMap<String, DefinedNameBinding>>,
    host_info: Option<&dyn HostInfoProvider>,
    rtd_provider: Option<&dyn RtdProvider>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> Result<oxfml_core::EvaluationOutput, oxfml_core::eval::EvaluationError> {
    let mut names = BTreeMap::new();
    if let Some(bindings) = &defined_names {
        for (name, binding) in bindings {
            names.insert(
                name.clone(),
                match binding {
                    DefinedNameBinding::Value(_) => NameKind::ValueLike,
                    DefinedNameBinding::Reference(_) => NameKind::ReferenceLike,
                    DefinedNameBinding::Callable(_) => NameKind::ValueLike,
                },
            );
        }
    }

    let compiled = common::compile_formula(
        "eval-fixture",
        formula,
        names,
        "eval-struct-v1",
        "oxfunc:test",
    );

    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context
        .cell_values
        .insert("A1".to_string(), EvalValue::Number(7.0));
    context
        .cell_values
        .insert("A2".to_string(), EvalValue::Number(11.0));
    context
        .cell_values
        .insert("B2".to_string(), EvalValue::Number(13.0));
    context.defined_names = defined_names.unwrap_or_default();
    context.apply_typed_context_query_bundle(TypedContextQueryBundle::new(
        host_info,
        rtd_provider,
        locale_ctx,
        Some(46000.0),
        Some(0.25),
    ));

    evaluate_formula(context)
}

fn local_callable_binding(
    summary: &str,
    params: Vec<&str>,
    body: BoundExpr,
    closure: BTreeMap<String, DefinedNameBinding>,
) -> CallableDefinedNameBinding {
    let profile = callable_profile_from_summary(summary);
    CallableDefinedNameBinding {
        summary: summary.to_string(),
        carrier: CallableValueCarrier {
            origin_kind: oxfml_core::CallableOriginKind::HelperLambda,
            invocation_model: oxfml_core::CallableInvocationModel::TypedInvocationOnly,
            capture_mode: if profile.capture_names.is_empty() {
                oxfml_core::CallableCaptureMode::NoCapture
            } else {
                oxfml_core::CallableCaptureMode::LexicalCapture
            },
            arity: profile.arity,
        },
        profile,
        params: params.into_iter().map(|value| value.to_string()).collect(),
        optional_parameter_names: Vec::new(),
        body,
        closure,
    }
}

fn callable_profile_from_summary(summary: &str) -> CallableValueProfile {
    let mut arity = None;
    let mut required_arity = None;
    let mut parameter_names = None;
    let mut optional_parameter_names = None;
    let mut capture_names = None;
    let mut body_kind = None;

    for part in summary.split(';') {
        let (key, value) = part
            .split_once('=')
            .expect("callable summary entries should be key=value");
        match key {
            "arity" => arity = Some(value.parse::<usize>().expect("callable arity should parse")),
            "required_arity" => {
                required_arity = Some(
                    value
                        .parse::<usize>()
                        .expect("callable required arity should parse"),
                )
            }
            "params" => parameter_names = Some(split_profile_list(value)),
            "optional_params" => optional_parameter_names = Some(split_profile_list(value)),
            "captures" => capture_names = Some(split_profile_list(value)),
            "body" => body_kind = Some(value.to_string()),
            _ => {}
        }
    }

    let arity = arity.expect("callable arity should exist");
    CallableValueProfile {
        arity,
        required_arity: required_arity.unwrap_or(arity),
        parameter_names: parameter_names.unwrap_or_default(),
        optional_parameter_names: optional_parameter_names.unwrap_or_default(),
        capture_names: capture_names.unwrap_or_default(),
        body_kind: body_kind.expect("callable body kind should exist"),
    }
}

fn split_profile_list(value: &str) -> Vec<String> {
    if value == "-" || value.is_empty() {
        Vec::new()
    } else if value.contains('|') {
        value.split('|').map(|item| item.to_string()).collect()
    } else {
        value.split(',').map(|item| item.to_string()).collect()
    }
}

fn load_json_fixture<T: for<'de> Deserialize<'de>>(file_name: &str) -> T {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(file_name);
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn eval_value_summary(value: &EvalValue) -> String {
    match value {
        EvalValue::Number(number) => format!("Number({})", number_summary(*number)),
        EvalValue::Text(text) => format!("Text({})", text.to_string_lossy()),
        EvalValue::Logical(value) => format!("Logical({value})"),
        EvalValue::Error(code) => format!("Error({code:?})"),
        EvalValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(array_cell_summary)
                .collect::<Vec<_>>()
                .join(",");
            format!("Array({cells})")
        }
        EvalValue::Reference(reference) => format!("Reference({})", reference.target),
        EvalValue::Lambda(lambda) => format!("Lambda({})", lambda.callable_token),
    }
}

fn array_cell_summary(cell: &ArrayCellValue) -> String {
    match cell {
        ArrayCellValue::Number(number) => format!("Number({})", number_summary(*number)),
        ArrayCellValue::Text(text) => format!("Text({})", text.to_string_lossy()),
        ArrayCellValue::Logical(value) => format!("Logical({value})"),
        ArrayCellValue::Error(code) => format!("Error({code:?})"),
        ArrayCellValue::EmptyCell => "EmptyCell".to_string(),
    }
}

fn number_summary(number: f64) -> String {
    if number.fract() == 0.0 {
        format!("{number:.0}")
    } else {
        format!("{number:?}")
    }
}

fn array_numbers(value: &EvalValue) -> Vec<f64> {
    let EvalValue::Array(array) = value else {
        panic!("expected array result, got {value:?}");
    };
    array
        .iter_row_major()
        .map(|cell| match cell {
            ArrayCellValue::Number(number) => *number,
            other => panic!("expected numeric array cell, got {other:?}"),
        })
        .collect()
}

fn array_logicals(value: &EvalValue) -> Vec<bool> {
    let EvalValue::Array(array) = value else {
        panic!("expected array result, got {value:?}");
    };
    array
        .iter_row_major()
        .map(|cell| match cell {
            ArrayCellValue::Logical(value) => *value,
            other => panic!("expected logical array cell, got {other:?}"),
        })
        .collect()
}

fn name_ref_expr(name: &str, kind: NameKind) -> BoundExpr {
    BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(NameRef {
        name: name.to_string(),
        workbook_id: "book:default".to_string(),
        sheet_id: "sheet:default".to_string(),
        kind,
        caller_context_dependent: false,
    })))
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
        assert_eq!(
            request.alt_text.as_ref().map(ExcelText::to_string_lossy),
            Some("Sphere".to_string())
        );
        assert_eq!(request.sizing, ImageSizingMode::FitCell);
        assert_eq!(request.height, None);
        assert_eq!(request.width, None);

        Ok(ImageProviderResult::Image(ResolvedWebImage {
            web_image_identifier: "img-1".to_string(),
            published_fallback: ExcelText::from_interop_assignment("-2146826273"),
        }))
    }
}

struct FailingHostInfoProvider;

impl HostInfoProvider for FailingHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        match query {
            CellInfoQuery::Filename => Err(HostInfoError::ProviderFailure {
                detail: "host offline".to_string(),
            }),
            _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedInfoQuery(query))
    }
}

struct BlockedImageHostInfoProvider;

impl HostInfoProvider for BlockedImageHostInfoProvider {
    fn query_image(&self, _request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        Ok(ImageProviderResult::CapabilityDenied)
    }
}

struct ValueRtdProvider;

impl RtdProvider for ValueRtdProvider {
    fn resolve_rtd(
        &self,
        _request: &oxfunc_core::functions::rtd_fn::RtdRequest,
    ) -> RtdProviderResult {
        RtdProviderResult::Value(EvalValue::Number(7.0))
    }
}

struct CapabilityDeniedRtdProvider;

impl RtdProvider for CapabilityDeniedRtdProvider {
    fn resolve_rtd(
        &self,
        _request: &oxfunc_core::functions::rtd_fn::RtdRequest,
    ) -> RtdProviderResult {
        RtdProviderResult::CapabilityDenied
    }
}
