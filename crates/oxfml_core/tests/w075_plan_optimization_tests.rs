mod common;

use std::collections::BTreeMap;

use oxfml_core::eval::{EvaluationContext, EvaluationTraceMode, evaluate_formula};
use oxfunc_core::value::FunctionValue;

#[test]
fn w075_context_free_precompute_preserves_prepared_call_trace() {
    let compiled = common::compile_formula(
        "w075-context-free-trace",
        "=SUM(HSTACK(0,0,0))",
        BTreeMap::new(),
        "w075-plan-optimization-v1",
        "oxfunc:w075",
    );

    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context.set_trace_mode(EvaluationTraceMode::PreparedCalls);
    let output = evaluate_formula(context).expect("evaluation should succeed");

    assert_eq!(output.oxfunc_value, FunctionValue::Number(0.0));
    let function_names = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_name.as_str())
        .collect::<Vec<_>>();
    assert!(function_names.contains(&"HSTACK"));
    assert!(function_names.contains(&"SUM"));
}

#[test]
fn w075_context_free_precompute_keeps_runtime_context_sensitive_calls_dynamic() {
    let compiled = common::compile_formula(
        "w075-runtime-sensitive-no-hoist",
        "=NOW()",
        BTreeMap::new(),
        "w075-plan-optimization-v1",
        "oxfunc:w075",
    );

    let mut first = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    first.now_serial = Some(123.0);
    let first = evaluate_formula(first).expect("first evaluation should succeed");

    let mut second = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    second.now_serial = Some(456.0);
    let second = evaluate_formula(second).expect("second evaluation should succeed");

    assert_eq!(first.oxfunc_value, FunctionValue::Number(123.0));
    assert_eq!(second.oxfunc_value, FunctionValue::Number(456.0));
}

#[test]
fn w075_value_only_hot_helpers_emit_no_prepared_call_records() {
    let compiled = common::compile_formula(
        "w075-value-only-hot-helper-trace",
        "=MAKEARRAY(10,10,LAMBDA(r,c,1))",
        BTreeMap::new(),
        "w075-plan-optimization-v1",
        "oxfunc:w075",
    );

    let context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    let output = evaluate_formula(context).expect("evaluation should succeed");

    assert!(output.trace.prepared_calls.is_empty());
    match output.oxfunc_value {
        FunctionValue::Array(array) => {
            let shape = array.shape();
            assert_eq!(shape.rows, 10);
            assert_eq!(shape.cols, 10);
        }
        other => panic!("expected array result, got {other:?}"),
    }
}

#[test]
fn w075_prepared_call_trace_covers_slot_only_let_path() {
    let compiled = common::compile_formula(
        "w075-slot-only-let-trace",
        "=LET(x,1,y,2,x+y)",
        BTreeMap::new(),
        "w075-plan-optimization-v1",
        "oxfunc:w075",
    );

    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context.set_trace_mode(EvaluationTraceMode::PreparedCalls);
    let output = evaluate_formula(context).expect("evaluation should succeed");

    assert_eq!(output.oxfunc_value, FunctionValue::Number(3.0));
    assert!(
        output
            .trace
            .prepared_calls
            .iter()
            .any(|call| call.function_name == "LET"
                && call.returned_value == Some(FunctionValue::Number(3.0)))
    );
}

#[test]
fn w075_slot_only_let_preserves_lexical_shadowing() {
    let compiled = common::compile_formula(
        "w075-slot-only-let-shadowing",
        "=LET(x,1,y,LET(x,2,x),x+y)",
        BTreeMap::new(),
        "w075-plan-optimization-v1",
        "oxfunc:w075",
    );

    let context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    let output = evaluate_formula(context).expect("evaluation should succeed");

    assert_eq!(output.oxfunc_value, FunctionValue::Number(3.0));
}

#[test]
fn w075_narrowed_lambda_closure_preserves_named_capture() {
    let compiled = common::compile_formula(
        "w075-narrowed-lambda-capture",
        "=LET(a,10,b,20,f,LAMBDA(x,a+x),f(5))",
        BTreeMap::new(),
        "w075-plan-optimization-v1",
        "oxfunc:w075",
    );

    let context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    let output = evaluate_formula(context).expect("evaluation should succeed");

    assert_eq!(output.oxfunc_value, FunctionValue::Number(15.0));
}
