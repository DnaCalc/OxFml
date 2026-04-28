use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::{en_us_context, render_with_code};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, run_oxfunc_preparation_adapter,
};
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::locale_format::FormatFailure;
use oxfunc_core::value::WorksheetErrorCode;
use oxfunc_core::value::{EvalValue, ExcelText};

fn evaluate_formula_text(formula_stable_id: &str, formula: &str) -> oxfml_core::EvaluationOutput {
    let compiled = common::compile_formula(
        formula_stable_id,
        formula,
        BTreeMap::new(),
        "eval-struct-v1",
        "oxfunc:test",
    );
    let context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    evaluate_formula(context).expect("evaluation should succeed")
}

#[test]
fn format_engine_rejects_unsupported_fraction_placeholder_text_code_ftc_0654() {
    let locale = en_us_context();
    let rendered = render_with_code(&locale.profile, locale.date_system, 0.25, "# ?/?");
    assert_eq!(
        rendered,
        Err(FormatFailure::UnsupportedCode("# ?/?".to_string()))
    );
}

#[test]
fn evaluator_rejects_unsupported_fraction_placeholder_text_code_ftc_0654() {
    let output = evaluate_formula_text("ftc-0654:evaluator", "=TEXT(0.25,\"# ?/?\")");
    assert_eq!(
        output.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Value)
    );
}

#[test]
fn runtime_rejects_unsupported_fraction_placeholder_text_code_ftc_0654() {
    let locale = en_us_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("ftc-0654:runtime", 1, "=TEXT(0.25,\"# ?/?\")"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed");

    assert_eq!(
        result.published_worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Value)
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "#VALUE!"
    );
}

#[test]
fn adapter_rejects_unsupported_fraction_placeholder_text_code_ftc_0654() {
    let locale = en_us_context();
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0654-fraction-text",
        "formula:foundation:FTC-0654",
        "=TEXT(0.25,\"# ?/?\")",
        oxfml_core::seam::Locus {
            sheet_id: "sheet:default".to_string(),
            row: 1,
            col: 1,
        },
        TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
    ))
    .expect("FTC-0654 adapter run should succeed");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Value)
    );
    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Error(Value)"
    );
}

#[test]
fn scientific_text_control_remains_green_after_ftc_0654_fix() {
    let locale = en_us_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("ftc-0654:control", 1, "=TEXT(12345.6789,\"0.00E+00\")"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime control should succeed");

    assert_eq!(
        result.published_worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("1.23E+04"))
    );
}
