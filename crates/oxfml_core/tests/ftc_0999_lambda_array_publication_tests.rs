use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::en_us_context;
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, WorksheetErrorCode};

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

fn calc_error_row(width: usize) -> EvalValue {
    EvalValue::Array(
        EvalArray::from_rows(vec![vec![ArrayCellValue::Error(WorksheetErrorCode::Calc); width]])
            .expect("row array"),
    )
}

#[test]
fn evaluator_publishes_calc_for_lambda_valued_map_array_ftc_0999() {
    let output = evaluate_formula_text(
        "ftc-0999:evaluator",
        "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),fns)",
    );

    assert_eq!(output.oxfunc_value, calc_error_row(3));
}

#[test]
fn host_and_runtime_publish_calc_for_lambda_valued_map_array_ftc_0999() {
    let formula = "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),fns)";
    let expected = calc_error_row(3);
    let locale = en_us_context();

    let mut host = SingleFormulaHost::new("ftc-0999:host", formula);
    let host_output = host
        .recalc(None, Some(&locale))
        .expect("host recalc should succeed");
    assert_eq!(host_output.published_worksheet_value, expected);
    assert_eq!(
        host_output.verification_publication_surface.visible_value_text,
        "#CALC!"
    );

    let runtime = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("ftc-0999:runtime", 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed");
    assert_eq!(runtime.published_worksheet_value, expected);
    assert_eq!(runtime.verification_publication_surface.visible_value_text, "#CALC!");
}

#[test]
fn lambda_array_selector_control_remains_green_ftc_0999() {
    let formula = "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),INDEX(fns,1)(10))";
    let locale = en_us_context();

    let eval = evaluate_formula_text("ftc-0999:control:evaluator", formula);
    assert_eq!(eval.oxfunc_value, EvalValue::Number(10.0));

    let runtime = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("ftc-0999:control:runtime", 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed");
    assert_eq!(runtime.published_worksheet_value, EvalValue::Number(10.0));
    assert_eq!(runtime.verification_publication_surface.visible_value_text, "10");
}
