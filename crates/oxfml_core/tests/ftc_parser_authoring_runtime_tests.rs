use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::value::{ExcelText, FunctionValue, WorksheetErrorCode};

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
fn runtime_executes_ftc_0837_after_error_literal_authoring_fix() {
    let formula = "=SUM(TOCOL({1,2,#N/A;4,5,6},1))";
    let expected = FunctionValue::Error(WorksheetErrorCode::NA);

    let evaluation = evaluate_formula_text("ftc-0837", formula);
    assert_eq!(evaluation.oxfunc_value, expected);

    let runtime = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:ftc-0837", 1, formula),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    assert!(runtime.bind_diagnostics.is_empty());
    assert_eq!(runtime.published_worksheet_value, expected);
}

#[test]
fn runtime_executes_ftc_1041_after_embedded_xml_quote_authoring_fix() {
    let formula = "=FILTERXML(\"<items><item id=\"\"1\"\">apple</item><item id=\"\"2\"\">banana</item></items>\",\"//item[@id=2]\")";
    let expected = FunctionValue::Text(ExcelText::from_interop_assignment("banana"));

    let evaluation = evaluate_formula_text("ftc-1041", formula);
    assert_eq!(evaluation.oxfunc_value, expected);

    let runtime = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:ftc-1041", 1, formula),
            TypedContextQueryBundle::default(),
        ))
        .expect("runtime execution should succeed");

    assert!(runtime.bind_diagnostics.is_empty());
    assert_eq!(runtime.published_worksheet_value, expected);
}
