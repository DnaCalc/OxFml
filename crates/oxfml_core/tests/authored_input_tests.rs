use oxfml_core::consumer::runtime::{RuntimeAuthoredInputResult, RuntimeEnvironment};
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
