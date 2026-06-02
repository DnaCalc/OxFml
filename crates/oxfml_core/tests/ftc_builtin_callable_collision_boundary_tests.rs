use oxfml_core::binding::{
    BindContext, BindRequest, BoundExpr, NameKind, NormalizedReference, ReferenceExpr, bind_formula,
};
use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{ExecutionOutcomeKind, ExecutionOutcomeStage, TypedContextQueryBundle};
use oxfunc_core::value::{ExcelText, FunctionValue, WorksheetErrorCode};

fn bind_formula_text(formula_stable_id: &str, formula: &str) -> oxfml_core::binding::BindResult {
    let source = FormulaSourceRecord::new(formula_stable_id, 1, formula);
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("fixture-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,
    })
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

#[test]
fn bind_surfaces_caller_context_locator_authoring_frontier_for_colliding_row_calls() {
    let bind = bind_formula_text(
        "fixture:row-collision-authoring-frontier",
        "=LET(row,LAMBDA(self,LAMBDA(n,n)),row(row)(7))",
    );

    assert!(bind.bound_formula.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.starts_with(
            "built-in function call 'ROW' rejects non-reference arguments at the authoring boundary"
        )
    }));

    let BoundExpr::FunctionCall {
        function_name,
        args,
        ..
    } = &bind.bound_formula.root
    else {
        panic!("expected LET root, got {:?}", bind.bound_formula.root);
    };
    assert_eq!(function_name, "LET");

    let BoundExpr::Invocation { callee, .. } = &args[2] else {
        panic!("expected nested invocation body, got {:?}", args[2]);
    };
    let BoundExpr::FunctionCall {
        function_name,
        args: row_args,
        ..
    } = callee.as_ref()
    else {
        panic!(
            "expected built-in ROW callable-position call, got {:?}",
            callee
        );
    };
    assert_eq!(function_name, "ROW");
    let BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name))) = &row_args[0]
    else {
        panic!("expected helper-local ROW arg, got {:?}", row_args[0]);
    };
    assert_eq!(name.kind, NameKind::HelperLocal);
    assert_eq!(name.name, "row");
}

#[test]
fn runtime_rejects_colliding_row_callable_shapes_at_bind_boundary() {
    let reject_cases = [
        ("plain-row-scalar", "=ROW(7)"),
        ("colliding-row-scalar", "=LET(row,LAMBDA(n,n),row(7))"),
        (
            "colliding-row-self",
            "=LET(row,LAMBDA(self,LAMBDA(n,n)),row(row)(7))",
        ),
        (
            "colliding-row-inner-only",
            "=LET(row,LAMBDA(self,LAMBDA(n,n)),g,row,row(g)(7))",
        ),
    ];

    for (case_id, formula) in reject_cases {
        let result = runtime_execute(&format!("runtime:{case_id}"), formula);
        assert_eq!(
            result.execution_outcome_surface.outcome_kind,
            ExecutionOutcomeKind::Rejected,
            "{case_id} outcome kind"
        );
        assert_eq!(
            result.execution_outcome_surface.outcome_stage,
            ExecutionOutcomeStage::BindBoundary,
            "{case_id} outcome stage"
        );
        assert_eq!(
            result.published_worksheet_value,
            FunctionValue::Error(WorksheetErrorCode::Value),
            "{case_id} worksheet value"
        );
        assert!(result.bind_diagnostics.iter().any(|diagnostic| {
            diagnostic.message.starts_with(
                "built-in function call 'ROW' rejects non-reference arguments at the authoring boundary"
            )
        }));
    }
}

#[test]
fn runtime_preserves_alias_escape_hatch_for_colliding_row_callables() {
    let cases = [
        (
            "colliding-row-reference",
            "=LET(row,LAMBDA(n,n),row(A1))",
            FunctionValue::Number(1.0),
            "1",
        ),
        (
            "colliding-row-aliased-scalar",
            "=LET(row,LAMBDA(n,n),g,row,g(7))",
            FunctionValue::Number(7.0),
            "7",
        ),
        (
            "colliding-row-self-aliased-outer-only",
            "=LET(row,LAMBDA(self,LAMBDA(n,n)),g,row,g(row)(7))",
            FunctionValue::Number(7.0),
            "7",
        ),
        (
            "colliding-t-direct",
            "=LET(t,LAMBDA(x,x+1),t(7))",
            FunctionValue::Text(ExcelText::from_interop_assignment("")),
            "",
        ),
        (
            "colliding-sum-direct",
            "=LET(sum,LAMBDA(x,x+1),sum(7))",
            FunctionValue::Number(7.0),
            "7",
        ),
        (
            "colliding-gcd-direct",
            "=LET(gcd,LAMBDA(a,b,a+b),gcd(8,4))",
            FunctionValue::Number(4.0),
            "4",
        ),
    ];

    for (case_id, formula, expected_value, expected_text) in cases {
        let result = runtime_execute(&format!("runtime:{case_id}"), formula);
        assert_eq!(
            result.execution_outcome_surface.outcome_kind,
            ExecutionOutcomeKind::ExecutedResult,
            "{case_id} outcome kind"
        );
        assert_eq!(
            result.published_worksheet_value, expected_value,
            "{case_id} value"
        );
        assert_eq!(
            result.verification_publication_surface.visible_value_text, expected_text,
            "{case_id} visible text"
        );
    }
}
