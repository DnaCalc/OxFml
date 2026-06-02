use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::{FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::value::FunctionValue;

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
fn evaluator_characterizes_ftc_0630_subnormal_boundary_family() {
    let cases = [
        (
            "FTC-0630",
            "=2.2250738585072014E-308/10",
            FunctionValue::Number(0.0),
        ),
        (
            "nearby-div2",
            "=2.2250738585072014E-308/2",
            FunctionValue::Number(0.0),
        ),
        (
            "nearby-div100",
            "=2.2250738585072014E-308/100",
            FunctionValue::Number(0.0),
        ),
        (
            "nearby-double-div10",
            "=4.450147717014403E-308/10",
            FunctionValue::Number(0.0),
        ),
        (
            "literal-boundary-preserved",
            "=2.2250738585072100E-308",
            FunctionValue::Number(2.22507385850721e-308),
        ),
        (
            "preserved-boundary-div2",
            "=2.2250738585072100E-308/2",
            FunctionValue::Number(0.0),
        ),
        (
            "nonzero-control",
            "=5E-308/2",
            FunctionValue::Number(2.5e-308),
        ),
        (
            "power-underflow",
            "=POWER(2,-1023)",
            FunctionValue::Number(0.0),
        ),
        (
            "power-min-normal",
            "=POWER(2,-1022)",
            FunctionValue::Number(f64::MIN_POSITIVE),
        ),
    ];

    for (case_id, formula, expected) in cases {
        let output = evaluate_formula_text(&format!("ftc-0630:{case_id}"), formula);
        assert_eq!(output.oxfunc_value, expected, "{case_id} evaluator value");
        assert_eq!(
            output.result.payload_summary,
            match expected {
                FunctionValue::Number(number) => format!("Number({number})"),
                _ => unreachable!(),
            },
            "{case_id} payload_summary"
        );
    }
}

#[test]
fn runtime_characterizes_ftc_0630_subnormal_boundary_family() {
    let cases = [
        (
            "FTC-0630",
            "=2.2250738585072014E-308/10",
            FunctionValue::Number(0.0),
        ),
        (
            "nearby-div2",
            "=2.2250738585072014E-308/2",
            FunctionValue::Number(0.0),
        ),
        (
            "nearby-div100",
            "=2.2250738585072014E-308/100",
            FunctionValue::Number(0.0),
        ),
        (
            "nearby-double-div10",
            "=4.450147717014403E-308/10",
            FunctionValue::Number(0.0),
        ),
        (
            "literal-boundary-preserved",
            "=2.2250738585072100E-308",
            FunctionValue::Number(2.22507385850721e-308),
        ),
        (
            "preserved-boundary-div2",
            "=2.2250738585072100E-308/2",
            FunctionValue::Number(0.0),
        ),
        (
            "nonzero-control",
            "=5E-308/2",
            FunctionValue::Number(2.5e-308),
        ),
        (
            "power-underflow",
            "=POWER(2,-1023)",
            FunctionValue::Number(0.0),
        ),
        (
            "power-min-normal",
            "=POWER(2,-1022)",
            FunctionValue::Number(f64::MIN_POSITIVE),
        ),
    ];

    for (case_id, formula, expected) in cases {
        let result = RuntimeEnvironment::new()
            .execute(RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("runtime:ftc-0630:{case_id}"), 1, formula),
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
