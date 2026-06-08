use oxfml_core::consumer::runtime::{
    RuntimeAuthoredInputResult, RuntimeDryBindInputKind, RuntimeEnvironment,
};
use oxfml_core::{BoundExpr, FormulaSourceRecord};
use oxfunc_core::value::CoreValue;

fn source(text: &str) -> FormulaSourceRecord {
    FormulaSourceRecord::new("authored-input:test", 1, text)
}

#[test]
fn authored_input_returns_calc_value_for_cell_entry_literals() {
    let environment = RuntimeEnvironment::new();

    match environment.interpret_authored_input(source("123.4")) {
        RuntimeAuthoredInputResult::Literal(value) => {
            assert_eq!(value.core, CoreValue::Number(123.4));
        }
        other => panic!("expected numeric literal, got {other:?}"),
    }

    match environment.interpret_authored_input(source("'123.4")) {
        RuntimeAuthoredInputResult::Literal(value) => match value.core {
            CoreValue::Text(text) => assert_eq!(text.to_string_lossy(), "123.4"),
            other => panic!("expected forced text, got {other:?}"),
        },
        other => panic!("expected forced text literal, got {other:?}"),
    }
}

#[test]
fn authored_input_returns_bound_formula_for_formula_text() {
    let environment = RuntimeEnvironment::new();

    match environment.interpret_authored_input(source("=SUM(1,2)")) {
        RuntimeAuthoredInputResult::Formula(bound) => match bound.root {
            BoundExpr::FunctionCall {
                function_name,
                args,
            } => {
                assert_eq!(function_name, "SUM");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected function call, got {other:?}"),
        },
        other => panic!("expected bound formula, got {other:?}"),
    }
}

#[test]
fn authored_input_returns_diagnostics_for_formula_acceptance_error() {
    let environment = RuntimeEnvironment::new();

    match environment.interpret_authored_input(source("=1+")) {
        RuntimeAuthoredInputResult::Diagnostics(diagnostics) => {
            assert!(!diagnostics.syntax_diagnostics.is_empty());
            assert!(diagnostics.bind_diagnostics.is_empty());
        }
        other => panic!("expected diagnostics, got {other:?}"),
    }
}

#[test]
fn dry_bind_reports_formula_verdict_without_evaluation() {
    let environment = RuntimeEnvironment::new();

    let verdict = environment.dry_bind_authored_input(source("=SUM(1,2)"));

    assert_eq!(verdict.input_kind, RuntimeDryBindInputKind::Formula);
    assert!(verdict.legal);
    assert!(verdict.syntax_diagnostics.is_empty());
    assert!(verdict.bind_diagnostics.is_empty());
    assert!(verdict.profile_violations.is_empty());
}

#[test]
fn dry_bind_reports_syntax_diagnostics_without_binding() {
    let environment = RuntimeEnvironment::new();

    let verdict = environment.dry_bind_authored_input(source("=1+"));

    assert_eq!(verdict.input_kind, RuntimeDryBindInputKind::Formula);
    assert!(!verdict.legal);
    assert!(!verdict.syntax_diagnostics.is_empty());
    assert!(verdict.bind_diagnostics.is_empty());
}

#[test]
fn dry_bind_reports_bind_diagnostics_without_evaluation() {
    let environment = RuntimeEnvironment::new();

    let verdict = environment.dry_bind_authored_input(source("=LAMBDA(x,x,x)"));

    assert_eq!(verdict.input_kind, RuntimeDryBindInputKind::Formula);
    assert!(!verdict.legal);
    assert!(verdict.syntax_diagnostics.is_empty());
    assert!(
        verdict
            .bind_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "duplicate LAMBDA parameter name 'x'")
    );
}

#[test]
fn dry_bind_classifies_literals_without_parsing_as_formula() {
    let environment = RuntimeEnvironment::new();

    let verdict = environment.dry_bind_authored_input(source("123"));

    assert_eq!(verdict.input_kind, RuntimeDryBindInputKind::Literal);
    assert!(verdict.legal);
    assert!(verdict.syntax_diagnostics.is_empty());
    assert!(verdict.bind_diagnostics.is_empty());
}
