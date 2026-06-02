use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::value::{FunctionArray, FunctionArrayCell, FunctionValue, WorksheetErrorCode};

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

fn runtime_execute(
    formula_stable_id: &str,
    formula: &str,
) -> oxfml_core::consumer::runtime::RuntimeFormulaResult {
    let locale = oxfml_en_us_locale_context();
    RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(formula_stable_id, 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed")
}

fn row_numbers(values: &[f64]) -> FunctionValue {
    FunctionValue::Array(
        FunctionArray::from_rows(vec![
            values
                .iter()
                .map(|value| FunctionArrayCell::Number(*value))
                .collect(),
        ])
        .expect("row array"),
    )
}

#[test]
fn evaluator_collapses_hstack_empty_carrier_formula_ftc_1043() {
    let formula = "=LET(b,{5,3,8,1,9,2},j,1,n,COLUMNS(b),curr,INDEX(b,1,j),next,INDEX(b,1,j+1),left,IF(j>1,TAKE(b,,j-1),TAKE(b,,0)),right,IF(j+1<n,DROP(b,,j+1),TAKE(b,,0)),HSTACK(left,next,curr,right))";
    let output = evaluate_formula_text("ftc-1043:evaluator", formula);
    assert_eq!(
        output.oxfunc_value,
        FunctionValue::Error(WorksheetErrorCode::Calc)
    );
}

#[test]
fn runtime_collapses_hstack_empty_carrier_formula_ftc_1043() {
    let formula = "=LET(b,{5,3,8,1,9,2},j,1,n,COLUMNS(b),curr,INDEX(b,1,j),next,INDEX(b,1,j+1),left,IF(j>1,TAKE(b,,j-1),TAKE(b,,0)),right,IF(j+1<n,DROP(b,,j+1),TAKE(b,,0)),HSTACK(left,next,curr,right))";
    let result = runtime_execute("ftc-1043:runtime", formula);
    assert_eq!(
        result.published_worksheet_value,
        FunctionValue::Error(WorksheetErrorCode::Calc)
    );
    assert_eq!(
        result.verification_publication_surface.visible_value_text,
        "#CALC!"
    );
}

#[test]
fn hstack_collapse_applies_to_direct_take_zero_carriers() {
    let collapse_cases = [
        "=HSTACK(TAKE({5,3,8,1,9,2},,0),3)",
        "=HSTACK(3,TAKE({5,3,8,1,9,2},,0))",
    ];

    for (index, formula) in collapse_cases.iter().enumerate() {
        let eval = evaluate_formula_text(&format!("ftc-1043:collapse:{index}"), formula);
        assert_eq!(
            eval.oxfunc_value,
            FunctionValue::Error(WorksheetErrorCode::Calc)
        );

        let runtime = runtime_execute(&format!("ftc-1043:runtime:collapse:{index}"), formula);
        assert_eq!(
            runtime.published_worksheet_value,
            FunctionValue::Error(WorksheetErrorCode::Calc)
        );
        assert_eq!(
            runtime.verification_publication_surface.visible_value_text,
            "#CALC!"
        );
    }
}

#[test]
fn hstack_without_empty_carrier_remains_array_control() {
    let formula = "=LET(b,{5,3,8,1,9,2},j,1,n,COLUMNS(b),right,IF(j+1<n,DROP(b,,j+1),TAKE(b,,0)),HSTACK(3,right))";
    let eval = evaluate_formula_text("ftc-1043:control:evaluator", formula);
    assert_eq!(eval.oxfunc_value, row_numbers(&[3.0, 8.0, 1.0, 9.0, 2.0]));

    let runtime = runtime_execute("ftc-1043:control:runtime", formula);
    assert_eq!(
        runtime.published_worksheet_value,
        row_numbers(&[3.0, 8.0, 1.0, 9.0, 2.0])
    );
    assert_eq!(
        runtime.verification_publication_surface.visible_value_text,
        "3"
    );
}
