use std::collections::BTreeMap;

mod common;

use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::locale_format::{LocaleFormatContext, en_us_context};
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike};

use oxfml_core::binding::{
    BinaryOp, BoundExpr, NameKind, NameRef, NormalizedReference, ReferenceExpr,
};
use oxfml_core::eval::{
    CallableDefinedNameBinding, CallableValueCarrier, CallableValueProfile, DefinedNameBinding,
    EvaluationContext, evaluate_formula,
};

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
fn evaluator_runs_row_and_column_with_caller_context() {
    let row_output = evaluate("=ROW()", None, None, Some(&en_us_context()));
    assert_eq!(row_output.result.payload_summary, "Number(1)");
    assert_eq!(row_output.trace.prepared_calls[0].function_id, "FUNC.ROW");
    assert_eq!(
        row_output.result.capability_dependencies,
        vec!["caller_context".to_string()]
    );

    let column_output = evaluate("=COLUMN(A1:B2)", None, None, Some(&en_us_context()));
    assert_eq!(column_output.result.payload_summary, "Array(1x2)");
    assert_eq!(
        column_output.trace.prepared_calls[0].function_id,
        "FUNC.COLUMN"
    );
}

#[test]
fn evaluator_runs_indirect_offset_and_iferror() {
    let indirect_output = evaluate("=INDIRECT(\"A1\")", None, None, Some(&en_us_context()));
    assert_eq!(
        indirect_output.trace.prepared_calls[0].function_id,
        "FUNC.INDIRECT"
    );

    let offset_output = evaluate("=OFFSET(A1,0,0)", None, None, Some(&en_us_context()));
    assert_eq!(
        offset_output.trace.prepared_calls[0].function_id,
        "FUNC.OFFSET"
    );

    let iferror_output = evaluate(
        "=IFERROR(UnknownName,2)",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(iferror_output.result.payload_summary, "Number(2)");
    assert_eq!(
        iferror_output.trace.prepared_calls[0].function_id,
        "FUNC.IFERROR"
    );
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
    assert_eq!(
        output.result.callable_profile.as_deref(),
        Some("arity=1;params=x;captures=-;body=Binary")
    );
    let carrier = output
        .result
        .callable_carrier
        .as_ref()
        .expect("callable carrier should exist");
    assert_eq!(
        carrier.origin_kind,
        oxfml_core::CallableOriginKind::HelperLambda
    );
    assert_eq!(
        carrier.invocation_model,
        oxfml_core::CallableInvocationModel::TypedInvocationOnly
    );
    assert_eq!(
        carrier.capture_mode,
        oxfml_core::CallableCaptureMode::NoCapture
    );
    assert_eq!(carrier.arity, 1);
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert_eq!(detail.arity, 1);
    assert_eq!(detail.parameter_names, vec!["x".to_string()]);
    assert!(detail.capture_names.is_empty());
    assert_eq!(detail.body_kind, "Binary");
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
    assert_eq!(
        output.result.callable_profile.as_deref(),
        Some("arity=1;params=y;captures=x;body=Binary")
    );
    let carrier = output
        .result
        .callable_carrier
        .as_ref()
        .expect("callable carrier should exist");
    assert_eq!(
        carrier.origin_kind,
        oxfml_core::CallableOriginKind::HelperLambda
    );
    assert_eq!(
        carrier.invocation_model,
        oxfml_core::CallableInvocationModel::TypedInvocationOnly
    );
    assert_eq!(
        carrier.capture_mode,
        oxfml_core::CallableCaptureMode::LexicalCapture
    );
    assert_eq!(carrier.arity, 1);
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert_eq!(detail.arity, 1);
    assert_eq!(detail.parameter_names, vec!["y".to_string()]);
    assert_eq!(detail.capture_names, vec!["x".to_string()]);
    assert_eq!(detail.body_kind, "Binary");
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

#[test]
fn evaluator_invokes_defined_name_callable_binding() {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        "NamedLambda".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=x;captures=-;body=Binary",
            vec!["x"],
            BoundExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(name_ref_expr("x", NameKind::HelperLocal)),
                right: Box::new(BoundExpr::NumberLiteral("1".to_string())),
            },
            BTreeMap::new(),
        )),
    );

    let output = evaluate(
        "=NamedLambda(2)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(output.result.payload_summary, "Number(3)");
    assert_eq!(output.trace.prepared_calls.len(), 1);
    assert_eq!(
        output.trace.prepared_calls[0].function_id,
        "SPECIAL.LAMBDA_INVOKE"
    );
}

#[test]
fn evaluator_preserves_defined_name_callable_as_first_class_value() {
    let mut bindings = BTreeMap::new();
    let mut closure = BTreeMap::new();
    closure.insert(
        "x".to_string(),
        DefinedNameBinding::Value(EvalValue::Number(10.0)),
    );
    bindings.insert(
        "NamedLambda".to_string(),
        DefinedNameBinding::Callable(local_callable_binding(
            "arity=1;params=y;captures=x;body=Binary",
            vec!["y"],
            BoundExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(name_ref_expr("x", NameKind::ValueLike)),
                right: Box::new(name_ref_expr("y", NameKind::HelperLocal)),
            },
            closure,
        )),
    );

    let value_output = evaluate(
        "=NamedLambda",
        Some(bindings.clone()),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        value_output.result.payload_summary,
        "Lambda(arity=1;params=y;captures=x;body=Binary)"
    );
    assert_eq!(
        value_output
            .result
            .callable_carrier
            .as_ref()
            .expect("callable carrier should exist")
            .capture_mode,
        oxfml_core::CallableCaptureMode::LexicalCapture
    );

    let invoke_output = evaluate(
        "=NamedLambda(2)",
        Some(bindings),
        None,
        Some(&en_us_context()),
    );
    assert_eq!(invoke_output.result.payload_summary, "Number(12)");
}

#[test]
fn evaluator_lambda_summary_ignores_unused_helper_bindings() {
    let output = evaluate(
        "=LET(x,10,unused,99,LAMBDA(y,x+y))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;params=y;captures=x;body=Binary)"
    );
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert_eq!(detail.capture_names, vec!["x".to_string()]);
}

#[test]
fn evaluator_lambda_summary_respects_parameter_shadowing() {
    let output = evaluate(
        "=LET(x,10,LAMBDA(x,x+1))",
        None,
        None,
        Some(&en_us_context()),
    );
    assert_eq!(
        output.result.payload_summary,
        "Lambda(arity=1;params=x;captures=-;body=Binary)"
    );
    let carrier = output
        .result
        .callable_carrier
        .as_ref()
        .expect("callable carrier should exist");
    assert_eq!(
        carrier.capture_mode,
        oxfml_core::CallableCaptureMode::NoCapture
    );
    let detail = output
        .result
        .callable_profile_detail
        .as_ref()
        .expect("callable detail should exist");
    assert!(detail.capture_names.is_empty());
}

#[test]
fn evaluator_surfaces_typed_external_reference_deferment() {
    let output = evaluate("=[Book.xlsx]Sheet2!A1", None, None, Some(&en_us_context()));
    assert_eq!(output.result.payload_summary, "Error(Ref)");
    assert_eq!(
        output.result.deferred_reason.as_deref(),
        Some("external_reference_deferred")
    );
    assert_eq!(
        output.result.capability_dependencies,
        vec!["external_reference".to_string()]
    );
    assert_eq!(output.trace.prepared_calls.len(), 1);
    assert_eq!(
        output.trace.prepared_calls[0].function_id,
        "SPECIAL.EXTERNAL_REFERENCE_DEFERRED"
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[0].source_class,
        oxfml_core::PreparedSourceClass::ExternalReference
    );
    assert_eq!(
        output.trace.prepared_calls[0].prepared_arguments[0]
            .opaque_reason
            .as_deref(),
        Some("external_reference_deferred")
    );
}

#[test]
fn evaluator_runs_index_and_xmatch_catalog_lanes() {
    let index_output = evaluate("=INDEX(SEQUENCE(3),2)", None, None, Some(&en_us_context()));
    assert_eq!(index_output.result.payload_summary, "Number(2)");
    assert_eq!(
        index_output.trace.prepared_calls[0].function_id,
        "FUNC.SEQUENCE"
    );
    assert_eq!(
        index_output.trace.prepared_calls[1].function_id,
        "FUNC.INDEX"
    );

    let xmatch_output = evaluate("=XMATCH(3,SEQUENCE(5))", None, None, Some(&en_us_context()));
    assert_eq!(xmatch_output.result.payload_summary, "Number(3)");
    assert_eq!(
        xmatch_output.trace.prepared_calls[0].function_id,
        "FUNC.SEQUENCE"
    );
    assert_eq!(
        xmatch_output.trace.prepared_calls[1].function_id,
        "FUNC.XMATCH"
    );
}

fn evaluate(
    formula: &str,
    defined_names: Option<BTreeMap<String, DefinedNameBinding>>,
    host_info: Option<&dyn HostInfoProvider>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> oxfml_core::EvaluationOutput {
    let mut names = BTreeMap::new();
    if let Some(bindings) = &defined_names {
        for (name, binding) in bindings {
            names.insert(
                name.clone(),
                match binding {
                    DefinedNameBinding::Value(_) => NameKind::ValueLike,
                    DefinedNameBinding::Reference(_) => NameKind::ReferenceLike,
                    DefinedNameBinding::Callable(_) => NameKind::ValueLike,
                },
            );
        }
    }

    let compiled = common::compile_formula(
        "eval-fixture",
        formula,
        names,
        "eval-struct-v1",
        "oxfunc:test",
    );

    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context
        .cell_values
        .insert("A1".to_string(), EvalValue::Number(7.0));
    context
        .cell_values
        .insert("A2".to_string(), EvalValue::Number(11.0));
    context
        .cell_values
        .insert("B2".to_string(), EvalValue::Number(13.0));
    context.defined_names = defined_names.unwrap_or_default();
    context.host_info = host_info;
    context.locale_ctx = locale_ctx;
    context.now_serial = Some(46000.0);
    context.random_value = Some(0.25);

    evaluate_formula(context).expect("evaluation should succeed")
}

fn local_callable_binding(
    summary: &str,
    params: Vec<&str>,
    body: BoundExpr,
    closure: BTreeMap<String, DefinedNameBinding>,
) -> CallableDefinedNameBinding {
    let profile = callable_profile_from_summary(summary);
    CallableDefinedNameBinding {
        summary: summary.to_string(),
        carrier: CallableValueCarrier {
            origin_kind: oxfml_core::CallableOriginKind::HelperLambda,
            invocation_model: oxfml_core::CallableInvocationModel::TypedInvocationOnly,
            capture_mode: if profile.capture_names.is_empty() {
                oxfml_core::CallableCaptureMode::NoCapture
            } else {
                oxfml_core::CallableCaptureMode::LexicalCapture
            },
            arity: profile.arity,
        },
        profile,
        params: params.into_iter().map(|value| value.to_string()).collect(),
        body,
        closure,
    }
}

fn callable_profile_from_summary(summary: &str) -> CallableValueProfile {
    let mut arity = None;
    let mut parameter_names = None;
    let mut capture_names = None;
    let mut body_kind = None;

    for part in summary.split(';') {
        let (key, value) = part
            .split_once('=')
            .expect("callable summary entries should be key=value");
        match key {
            "arity" => arity = Some(value.parse::<usize>().expect("callable arity should parse")),
            "params" => parameter_names = Some(split_profile_list(value)),
            "captures" => capture_names = Some(split_profile_list(value)),
            "body" => body_kind = Some(value.to_string()),
            _ => {}
        }
    }

    CallableValueProfile {
        arity: arity.expect("callable arity should exist"),
        parameter_names: parameter_names.unwrap_or_default(),
        capture_names: capture_names.unwrap_or_default(),
        body_kind: body_kind.expect("callable body kind should exist"),
    }
}

fn split_profile_list(value: &str) -> Vec<String> {
    if value == "-" || value.is_empty() {
        Vec::new()
    } else if value.contains('|') {
        value.split('|').map(|item| item.to_string()).collect()
    } else {
        value.split(',').map(|item| item.to_string()).collect()
    }
}

fn name_ref_expr(name: &str, kind: NameKind) -> BoundExpr {
    BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(NameRef {
        name: name.to_string(),
        workbook_id: "book:default".to_string(),
        sheet_id: "sheet:default".to_string(),
        kind,
        caller_context_dependent: false,
    })))
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
