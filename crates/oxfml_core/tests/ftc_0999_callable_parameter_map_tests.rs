use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::en_us_context;
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

fn number_row(values: &[f64]) -> EvalValue {
    EvalValue::Array(
        EvalArray::from_rows(vec![
            values
                .iter()
                .map(|value| ArrayCellValue::Number(*value))
                .collect(),
        ])
        .expect("row array"),
    )
}

#[test]
fn evaluator_rehydrates_callable_map_parameters_ftc_0999() {
    let cases = [
        (
            "direct",
            "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),MAP(fns,LAMBDA(f,f(10))))",
        ),
        (
            "via-let",
            "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),MAP(fns,LAMBDA(f,LET(g,f,g(10)))))",
        ),
    ];

    for (case_id, formula) in cases {
        let output = evaluate_formula_text(&format!("ftc-0999:{case_id}"), formula);
        assert_eq!(
            output.oxfunc_value,
            number_row(&[10.0, 20.0, 30.0]),
            "{case_id} value"
        );
    }
}

#[test]
fn runtime_rehydrates_callable_map_parameters_ftc_0999() {
    let locale = en_us_context();
    let cases = [
        (
            "direct",
            "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),MAP(fns,LAMBDA(f,f(10))))",
        ),
        (
            "via-let",
            "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),MAP(fns,LAMBDA(f,LET(g,f,g(10)))))",
        ),
    ];

    for (case_id, formula) in cases {
        let runtime = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:ftc-0999:{case_id}"), 1, formula),
                TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
            ))
            .expect("runtime execution should succeed");

        assert_eq!(
            runtime.published_worksheet_value,
            number_row(&[10.0, 20.0, 30.0]),
            "{case_id} runtime value"
        );
        assert_eq!(
            runtime.verification_publication_surface.visible_value_text, "10",
            "{case_id} visible text"
        );
    }
}

#[test]
fn nested_callable_map_reuse_projects_calc_ftc_0999() {
    let formula = "=LET(fns,MAP({1,2,3},LAMBDA(n,LAMBDA(x,x*n))),MAP(fns,LAMBDA(f,MAP({10,20,30},LAMBDA(x,f(x))))))";

    let eval = evaluate_formula_text("ftc-0999:nested", formula);
    assert_eq!(
        eval.oxfunc_value,
        EvalValue::Error(WorksheetErrorCode::Calc)
    );

    let locale = en_us_context();
    let runtime = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:ftc-0999:nested", 1, formula),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed");
    assert_eq!(
        runtime.published_worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Calc)
    );
    assert_eq!(
        runtime.verification_publication_surface.visible_value_text,
        "#CALC!"
    );
}
