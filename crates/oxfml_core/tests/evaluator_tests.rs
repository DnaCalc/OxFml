use std::collections::BTreeMap;

use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::{LocaleFormatContext, en_us_context};
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike};

use oxfml_core::binding::{BindContext, BindRequest, NameKind, bind_formula};
use oxfml_core::eval::{DefinedNameBinding, EvaluationContext, evaluate_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};

#[test]
fn evaluator_runs_text_with_locale_format_context() {
    let output = evaluate(
        "=TEXT(1234.567,\"0.00\")",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Text(1234.57)");
    assert_eq!(
        output.result.format_hint.as_deref(),
        Some("locale_format_semantics")
    );
    assert_eq!(output.trace.prepared_calls.len(), 1);
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.TEXT");
    assert_eq!(
        output.result.capability_dependencies,
        vec!["locale_format_context".to_string()]
    );
}

#[test]
fn evaluator_runs_value_with_locale_parser() {
    let output = evaluate("=VALUE(\"12%\")", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(0.12)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.VALUE");
}

#[test]
fn evaluator_runs_cell_with_host_info_provider() {
    let output = evaluate(
        "=CELL(\"filename\",A1)",
        None,
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Text([Book1]Sheet1)");
    assert_eq!(
        output.result.publication_hint.as_deref(),
        Some("host_query_surface")
    );
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.CELL");
    assert!(output.trace.prepared_calls[0].host_query_enabled);
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[1].structure_class,
        oxfml_core::PreparedStructureClass::ReferenceVisible
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[1].evaluation_mode,
        oxfml_core::PreparedEvaluationMode::ReferencePreserved
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[1].blankness_class,
        oxfml_core::PreparedBlanknessClass::NonBlank
    );
    assert_eq!(
        output.result.capability_dependencies,
        vec!["caller_context".to_string(), "host_query".to_string()]
    );
}

#[test]
fn evaluator_runs_info_with_host_info_provider() {
    let output = evaluate(
        "=INFO(\"directory\")",
        None,
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Text(C:\\Work)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.INFO");
}

#[test]
fn evaluator_runs_now_and_today_with_supplied_serial() {
    let now_output = evaluate("=NOW()", None, None, Some(&en_us_context()));
    assert_eq!(now_output.result.payload_summary, "Number(46000)");

    let today_output = evaluate("=TODAY()", None, None, Some(&en_us_context()));
    assert_eq!(today_output.result.payload_summary, "Number(46000)");
}

#[test]
fn evaluator_uses_defined_name_bindings_for_sum() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "InputValue".to_string(),
        DefinedNameBinding::Value(EvalValue::Number(5.0)),
    );

    let output = evaluate(
        "=SUM(InputValue,2)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(7)");
    assert_eq!(output.trace.prepared_calls[0].function_id, "FUNC.SUM");
}

#[test]
fn evaluator_uses_defined_name_reference_for_cell_contents() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "InputRef".to_string(),
        DefinedNameBinding::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: "A1".to_string(),
        }),
    );

    let output = evaluate(
        "=CELL(\"contents\",InputRef)",
        Some(bindings),
        Some(&MockHostInfoProvider),
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(7)");
}

#[test]
fn evaluator_runs_let_with_helper_bindings() {
    let output = evaluate("=LET(x,1,x+2)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(3)");
}

#[test]
fn evaluator_runs_let_with_reference_preserved_binding() {
    let output = evaluate("=LET(r,A1,SUM(r,2))", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(9)");
}

#[test]
fn evaluator_runs_legacy_single_compat() {
    let output = evaluate("=_xlfn.SINGLE(A1)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(7)");
}

#[test]
fn evaluator_returns_lambda_value_summary() {
    let output = evaluate("=LAMBDA(x,x+1)", None, None, Some(&en_us_context()));
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;params=x;captures=-;body=Binary)"
    );
}

#[test]
fn evaluator_returns_lambda_value_summary_with_lexical_capture_metadata() {
    let output = evaluate(
        "=LET(x,10,LAMBDA(y,x+y))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;params=y;captures=x;body=Binary)"
    );
}

#[test]
fn evaluator_runs_immediate_lambda_invocation() {
    let output = evaluate("=LAMBDA(x,x+1)(2)", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Number(3)");
    assert_eq!(output.trace.prepared_calls.len(), 1);
    assert_eq!(
        output.trace.prepared_calls[0].function_id,
        "SPECIAL.LAMBDA_INVOKE"
    );
}

#[test]
fn evaluator_runs_helper_bound_lambda_invocation() {
    let output = evaluate(
        "=LET(f,LAMBDA(x,x+1),f(2))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(3)");
    let function_ids = output
        .trace
        .prepared_calls
        .iter()
        .map(|call| call.function_id)
        .collect::<Vec<_>>();
    assert_eq!(
        function_ids,
        vec!["SPECIAL.LAMBDA", "SPECIAL.LAMBDA_INVOKE", "SPECIAL.LET"]
    );
}

#[test]
fn evaluator_uses_lexical_not_dynamic_scope_for_helper_bound_lambda() {
    let output = evaluate(
        "=LET(x,10,f,LAMBDA(y,x+y),LET(x,20,f(2)))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(12)");
}

fn evaluate(
    formula: &str,
    defined_names: Option<BTreeMap<String, DefinedNameBinding>>,
    host_info: Option<&dyn HostInfoProvider>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> oxfml_core::EvaluationOutput {
    let source = FormulaSourceRecord::new("eval-fixture", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);

    let mut names = BTreeMap::new();
    if let Some(bindings) = &defined_names {
        for (name, binding) in bindings {
            names.insert(
                name.clone(),
                match binding {
                    DefinedNameBinding::Value(_) => NameKind::ValueLike,
                    DefinedNameBinding::Reference(_) => NameKind::ReferenceLike,
                },
            );
        }
    }

    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("eval-struct-v1".to_string()),
            names,
            ..BindContext::default()
        },
    });

    let plan = oxfml_core::compile_semantic_plan(oxfml_core::CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:test".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
    })
    .semantic_plan;

    let mut context = EvaluationContext::new(&bind.bound_formula, &plan);
    context
        .cell_values
        .insert("A1".to_string(), EvalValue::Number(7.0));
    context.defined_names = defined_names.unwrap_or_default();
    context.host_info = host_info;
    context.locale_ctx = locale_ctx;
    context.now_serial = Some(46000.0);
    context.random_value = Some(0.25);

    evaluate_formula(context).expect("evaluation should succeed")
}

struct MockHostInfoProvider;

impl HostInfoProvider for MockHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        match query {
            CellInfoQuery::Filename => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "[Book1]Sheet1".encode_utf16().collect(),
            ))),
            _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        match query {
            InfoQuery::Directory => Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
                "C:\\Work".encode_utf16().collect(),
            ))),
            _ => Err(HostInfoError::UnsupportedInfoQuery(query)),
        }
    }
}
