use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{
    EvaluationContext, FunctionArray, FunctionArrayCell, FunctionValue, evaluate_formula,
};
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};

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

fn array_numbers(values: &[f64]) -> FunctionValue {
    FunctionValue::Array(
        FunctionArray::from_rows(vec![
            values
                .iter()
                .copied()
                .map(FunctionArrayCell::Number)
                .collect::<Vec<_>>(),
        ])
        .expect("row array"),
    )
}

fn array_number_false_number(lhs: f64, rhs: f64) -> FunctionValue {
    FunctionValue::Array(
        FunctionArray::from_rows(vec![vec![
            FunctionArrayCell::Number(lhs),
            FunctionArrayCell::Logical(false),
            FunctionArrayCell::Number(rhs),
        ]])
        .expect("row array"),
    )
}

#[test]
fn evaluator_characterizes_ftc_0176_if_array_condition_family() {
    let cases = [
        (
            "FTC-0176",
            "=SUM(IF({TRUE,FALSE,TRUE},{10,20,30},0))",
            FunctionValue::Number(40.0),
        ),
        (
            "if-array-cond-scalars",
            "=IF({TRUE,FALSE,TRUE},1,0)",
            array_numbers(&[1.0, 0.0, 1.0]),
        ),
        (
            "if-array-cond-array-values",
            "=IF({TRUE,FALSE,TRUE},{10,20,30},0)",
            array_numbers(&[10.0, 0.0, 30.0]),
        ),
        (
            "if-scalar-cond-array-values",
            "=IF(TRUE,{10,20,30},0)",
            array_numbers(&[10.0, 20.0, 30.0]),
        ),
        (
            "if-array-cond-array-values-omitted-false",
            "=IF({TRUE,FALSE,TRUE},{10,20,30})",
            array_number_false_number(10.0, 30.0),
        ),
        (
            "sum-if-array-cond-scalars",
            "=SUM(IF({TRUE,FALSE,TRUE},1,0))",
            FunctionValue::Number(2.0),
        ),
        (
            "FTC-0878",
            "=SUM(IF({TRUE,FALSE,TRUE},{10,20,30}))",
            FunctionValue::Number(40.0),
        ),
        (
            "sum-if-scalar-cond-array-values",
            "=SUM(IF(TRUE,{10,20,30},0))",
            FunctionValue::Number(60.0),
        ),
    ];

    for (case_id, formula, expected) in cases {
        let output = evaluate_formula_text(&format!("ftc-0176:{case_id}"), formula);
        assert_eq!(output.oxfunc_value, expected, "{case_id} evaluator value");
    }
}

#[test]
fn runtime_characterizes_ftc_0176_if_array_condition_family() {
    let cases = [
        (
            "FTC-0176",
            "=SUM(IF({TRUE,FALSE,TRUE},{10,20,30},0))",
            FunctionValue::Number(40.0),
        ),
        (
            "if-array-cond-scalars",
            "=IF({TRUE,FALSE,TRUE},1,0)",
            array_numbers(&[1.0, 0.0, 1.0]),
        ),
        (
            "if-array-cond-array-values",
            "=IF({TRUE,FALSE,TRUE},{10,20,30},0)",
            array_numbers(&[10.0, 0.0, 30.0]),
        ),
        (
            "if-scalar-cond-array-values",
            "=IF(TRUE,{10,20,30},0)",
            array_numbers(&[10.0, 20.0, 30.0]),
        ),
        (
            "if-array-cond-array-values-omitted-false",
            "=IF({TRUE,FALSE,TRUE},{10,20,30})",
            array_number_false_number(10.0, 30.0),
        ),
        (
            "sum-if-array-cond-scalars",
            "=SUM(IF({TRUE,FALSE,TRUE},1,0))",
            FunctionValue::Number(2.0),
        ),
        (
            "FTC-0878",
            "=SUM(IF({TRUE,FALSE,TRUE},{10,20,30}))",
            FunctionValue::Number(40.0),
        ),
        (
            "sum-if-scalar-cond-array-values",
            "=SUM(IF(TRUE,{10,20,30},0))",
            FunctionValue::Number(60.0),
        ),
    ];

    for (case_id, formula, expected) in cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:ftc-0176:{case_id}"), 1, formula),
                TypedContextQueryBundle::default(),
            ))
            .expect("runtime execution should succeed");

        assert_eq!(
            result.published_worksheet_value, expected,
            "{case_id} runtime value"
        );
        assert_eq!(
            result.verification_publication_surface.published_value, expected,
            "{case_id} publication value"
        );
    }
}
