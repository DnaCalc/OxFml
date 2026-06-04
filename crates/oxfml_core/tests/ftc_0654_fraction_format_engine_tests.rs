use oxfunc_core::value::CalcValue;
use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::{oxfml_en_us_locale_context, render_with_code};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, run_oxfunc_preparation_adapter,
};
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::value::ExcelText;

fn evaluate_formula_text_with_locale(
    formula_stable_id: &str,
    formula: &str,
) -> oxfml_core::EvaluationOutput {
    let locale = oxfml_en_us_locale_context();
    let compiled = common::compile_formula(
        formula_stable_id,
        formula,
        BTreeMap::new(),
        "eval-struct-v1",
        "oxfunc:test",
    );
    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context.apply_typed_context_query_bundle(TypedContextQueryBundle::new(
        None,
        None,
        Some(&locale),
        None,
        None,
    ));
    evaluate_formula(context).expect("evaluation should succeed")
}

#[test]
fn format_engine_renders_fraction_placeholder_text_code_ftc_0654() {
    let locale = oxfml_en_us_locale_context();
    let rendered = render_with_code(&locale.profile, locale.date_system, 0.25, "# ?/?");
    assert_eq!(rendered, Ok(" 1/4".to_string()));
}

#[test]
fn evaluator_renders_fraction_placeholder_text_code_ftc_0654() {
    let output = evaluate_formula_text_with_locale("ftc-0654:evaluator", "=TEXT(0.25,\"# ?/?\")");
    assert_eq!(
        output.oxfunc_value,
        CalcValue::text(ExcelText::from_interop_assignment(" 1/4"))
    );
}

#[test]
fn runtime_renders_fraction_placeholder_text_code_ftc_0654() {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("ftc-0654:runtime", 1, "=TEXT(0.25,\"# ?/?\")"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed");

    assert_eq!(
        result.published_worksheet_value,
        CalcValue::text(ExcelText::from_interop_assignment(" 1/4"))
    );
}

#[test]
fn adapter_renders_fraction_placeholder_text_code_ftc_0654() {
    let locale = oxfml_en_us_locale_context();
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
        CalcValue::text(ExcelText::from_interop_assignment(" 1/4"))
    );
    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Text( 1/4)"
    );
}

#[test]
fn scientific_text_control_remains_green_after_ftc_0654_fix() {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("ftc-0654:control", 1, "=TEXT(12345.6789,\"0.00E+00\")"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime control should succeed");

    assert_eq!(
        result.published_worksheet_value,
        CalcValue::text(ExcelText::from_interop_assignment("1.23E+04"))
    );
}
