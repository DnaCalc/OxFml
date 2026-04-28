use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
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

fn array_numbers(values: &[f64]) -> EvalValue {
    EvalValue::Array(
        EvalArray::from_rows(
            values
                .iter()
                .map(|value| vec![ArrayCellValue::Number(*value)])
                .collect::<Vec<_>>(),
        )
        .expect("column array"),
    )
}

fn array_shadow_expected() -> EvalValue {
    EvalValue::Array(
        EvalArray::from_rows(vec![
            vec![ArrayCellValue::Error(WorksheetErrorCode::Div0)],
            vec![ArrayCellValue::Number(1.0)],
            vec![ArrayCellValue::Number(0.5)],
            vec![ArrayCellValue::Number(1.0 / 3.0)],
        ])
        .expect("column array"),
    )
}

#[test]
fn evaluator_respects_case_insensitive_lambda_parameter_shadowing_ftc_1013() {
    let control_formula = "=LET(N,4,ks,SEQUENCE(N,,0),MAP(ks,LAMBDA(k,1/N)))";
    let shadow_formula = "=LET(N,4,ks,SEQUENCE(N,,0),MAP(ks,LAMBDA(n,1/N)))";

    let control = evaluate_formula_text("ftc-1013:control", control_formula);
    assert_eq!(
        control.oxfunc_value,
        array_numbers(&[0.25, 0.25, 0.25, 0.25])
    );

    let shadow = evaluate_formula_text("ftc-1013:shadow", shadow_formula);
    assert_eq!(shadow.oxfunc_value, array_shadow_expected());
}

#[test]
fn evaluator_respects_case_insensitive_lambda_parameter_shadowing_simple_invocation() {
    let hit = evaluate_formula_text("ftc-1013:simple-hit", "=LET(N,4,LAMBDA(n,1/N)(2))");
    assert_eq!(hit.oxfunc_value, EvalValue::Number(0.5));

    let zero = evaluate_formula_text("ftc-1013:simple-zero", "=LET(N,4,LAMBDA(n,1/N)(0))");
    assert_eq!(
        zero.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Div0)
    );
}

#[test]
fn runtime_respects_case_insensitive_lambda_parameter_shadowing_ftc_1013() {
    let cases = [
        (
            "control",
            "=LET(N,4,ks,SEQUENCE(N,,0),MAP(ks,LAMBDA(k,1/N)))",
            array_numbers(&[0.25, 0.25, 0.25, 0.25]),
        ),
        (
            "shadow",
            "=LET(N,4,ks,SEQUENCE(N,,0),MAP(ks,LAMBDA(n,1/N)))",
            array_shadow_expected(),
        ),
    ];

    for (case_id, formula, expected) in cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:ftc-1013:{case_id}"), 1, formula),
                TypedContextQueryBundle::default(),
            ))
            .expect("runtime execution should succeed");

        assert_eq!(
            result.bind_diagnostics,
            Vec::new(),
            "{case_id} bind diagnostics"
        );
        assert_eq!(
            result.published_worksheet_value, expected,
            "{case_id} runtime value"
        );
    }
}
