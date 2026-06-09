use oxfml_core::consumer::runtime::{
    RuntimeAuthoredInputResult, RuntimeDryBindInputKind, RuntimeDryBindProfileViolationKind,
    RuntimeEnvironment, RuntimeValueLiteralizationKind, RuntimeValueLiteralizationResult,
    RuntimeValueLiteralizationUnsupportedKind,
};
use oxfml_core::{BoundExpr, FormulaSourceRecord};
use oxfunc_core::registry::{CapabilityOverlay, builtin_registry};
use oxfunc_core::value::{
    CalcArray, CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike,
};

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

    match environment.interpret_authored_input(source("#DIV/0!")) {
        RuntimeAuthoredInputResult::Literal(value) => {
            assert_eq!(
                value.core,
                CoreValue::Error(oxfunc_core::value::WorksheetErrorCode::Div0)
            );
        }
        other => panic!("expected worksheet error literal, got {other:?}"),
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
fn dry_bind_reports_capability_profile_violations_without_evaluation() {
    let sum_id = builtin_registry()
        .lookup_by_surface_name("SUM")
        .map(|entry| entry.meta.function_id.clone())
        .expect("SUM has a canonical function id");
    let mut overlay = CapabilityOverlay::new();
    overlay.deny_function_id(&sum_id, "disabled for authoring preview test");
    let environment = RuntimeEnvironment::new().with_capability_overlay(&overlay);

    let verdict = environment.dry_bind_authored_input(source("=SUM(1,2)"));

    assert_eq!(verdict.input_kind, RuntimeDryBindInputKind::Formula);
    assert!(!verdict.legal);
    assert!(verdict.syntax_diagnostics.is_empty());
    assert!(verdict.bind_diagnostics.is_empty());
    assert_eq!(verdict.profile_violations.len(), 1);
    let violation = &verdict.profile_violations[0];
    assert_eq!(
        violation.kind,
        RuntimeDryBindProfileViolationKind::FunctionUnavailable {
            function_id: sum_id.clone(),
            function_name: "SUM".to_string(),
            reason: "disabled for authoring preview test".to_string()
        }
    );
    assert_eq!(violation.feature, format!("function:{sum_id}"));
    assert!(
        violation
            .message
            .contains("disabled for authoring preview test")
    );
    assert_eq!(violation.span.start, 1);
    assert_eq!(violation.span.len, 3);
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

#[test]
fn literalize_calc_value_for_authoring_round_trips_scalar_cell_values() {
    let environment = RuntimeEnvironment::new();
    let cases = [
        (
            CalcValue::number(12.5),
            "12.5",
            RuntimeValueLiteralizationKind::Number,
        ),
        (
            CalcValue::number(-0.0),
            "0",
            RuntimeValueLiteralizationKind::Number,
        ),
        (
            CalcValue::text(ExcelText::from_interop_assignment("=A1")),
            "'=A1",
            RuntimeValueLiteralizationKind::Text,
        ),
        (
            CalcValue::logical(true),
            "TRUE",
            RuntimeValueLiteralizationKind::Logical,
        ),
        (
            CalcValue::error(oxfunc_core::value::WorksheetErrorCode::NA),
            "#N/A",
            RuntimeValueLiteralizationKind::WorksheetError,
        ),
        (
            CalcValue::empty(),
            "",
            RuntimeValueLiteralizationKind::Blank,
        ),
    ];

    for (value, expected_text, expected_kind) in cases {
        let RuntimeValueLiteralizationResult::AuthoredInput(literalized) =
            environment.literalize_calc_value_for_authoring(&value)
        else {
            panic!("expected authored literal for {value:?}");
        };
        assert_eq!(literalized.authored_input_text, expected_text);
        assert_eq!(literalized.kind, expected_kind);

        let RuntimeAuthoredInputResult::Literal(round_trip) =
            environment.interpret_authored_input(source(&literalized.authored_input_text))
        else {
            panic!("literalized text should interpret as a literal");
        };
        assert_eq!(round_trip.core, value.core);
    }
}

#[test]
fn literalize_calc_value_for_authoring_rejects_non_scalar_or_non_cell_values() {
    let environment = RuntimeEnvironment::new();
    let array = CalcValue::array(CalcArray::from_scalar(CalcValue::number(1.0)).unwrap());
    let reference = CalcValue::reference(ReferenceLike::new(ReferenceKind::A1, "A1"));
    let cases = [
        (
            CalcValue::number(f64::INFINITY),
            RuntimeValueLiteralizationUnsupportedKind::NonFiniteNumber,
        ),
        (
            CalcValue::missing(),
            RuntimeValueLiteralizationUnsupportedKind::Missing,
        ),
        (array, RuntimeValueLiteralizationUnsupportedKind::Array),
        (
            reference,
            RuntimeValueLiteralizationUnsupportedKind::Reference,
        ),
        (
            CalcValue::callable(oxfunc_core::value::CallableValue {
                arity: oxfunc_core::value::CallableArityShape::exact(0),
                summary: "test".to_string(),
                handle: std::rc::Rc::new(TestCallable),
            }),
            RuntimeValueLiteralizationUnsupportedKind::RichValue,
        ),
    ];

    for (value, expected_kind) in cases {
        let RuntimeValueLiteralizationResult::Unsupported(unsupported) =
            environment.literalize_calc_value_for_authoring(&value)
        else {
            panic!("expected unsupported literalization for {value:?}");
        };
        assert_eq!(unsupported.kind, expected_kind);
    }
}

#[derive(Debug)]
struct TestCallable;

impl oxfunc_core::value::OpaqueCallable for TestCallable {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
