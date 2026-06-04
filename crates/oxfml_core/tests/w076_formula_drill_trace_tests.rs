use oxfml_core::consumer::runtime::{
    FormulaArgumentNameSource, FormulaDrillBranchDisposition, FormulaDrillEvaluationState,
    FormulaDrillNodeId, FormulaDrillNodeKind, FormulaDrillTrace, FormulaDrillTraceNode,
    RuntimeEnvironment, RuntimeFormulaRequest,
};
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::{EvaluationTraceMode, FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::value::CalcValue;
use oxfunc_core::value::WorksheetErrorCode;

fn execute(formula: &str) -> FormulaDrillTrace {
    RuntimeEnvironment::new()
        .execute(
            RuntimeFormulaRequest::new(
                FormulaSourceRecord::new(format!("w076:{formula}"), 1, formula),
                TypedContextQueryBundle::default(),
            )
            .with_trace_mode(EvaluationTraceMode::PreparedCalls),
        )
        .expect("formula should execute")
        .formula_drill_trace
        .expect("runtime should emit formula drill trace")
}

fn node<'a>(trace: &'a FormulaDrillTrace, id: &FormulaDrillNodeId) -> &'a FormulaDrillTraceNode {
    trace
        .nodes
        .iter()
        .find(|node| &node.node_id == id)
        .expect("node id should exist")
}

fn function_node<'a>(trace: &'a FormulaDrillTrace, name: &str) -> &'a FormulaDrillTraceNode {
    trace
        .nodes
        .iter()
        .find(|node| {
            node.kind == FormulaDrillNodeKind::FunctionCall
                && node
                    .function_surface_name
                    .as_deref()
                    .is_some_and(|surface| surface.eq_ignore_ascii_case(name))
        })
        .unwrap_or_else(|| panic!("{name} node should exist"))
}

fn argument_node<'a>(
    trace: &'a FormulaDrillTrace,
    parent: &FormulaDrillNodeId,
    name: &str,
) -> &'a FormulaDrillTraceNode {
    node(trace, parent)
        .child_node_ids
        .iter()
        .map(|id| node(trace, id))
        .find(|node| {
            node.kind == FormulaDrillNodeKind::Argument
                && node.argument_name.as_deref() == Some(name)
        })
        .unwrap_or_else(|| panic!("{name} argument should exist"))
}

#[test]
fn w076_sum_trace_has_root_call_named_arguments_and_value() {
    let trace = execute("=SUM(1,2,3)");
    assert_eq!(trace.schema_id, "oxfml.formula_drill_trace.v1");
    assert_eq!(trace.final_value, CalcValue::number(6.0));

    let root = node(&trace, &trace.root_node_id);
    let sum_id = root.child_node_ids.first().expect("root child").clone();
    let sum = node(&trace, &sum_id);
    assert_eq!(sum.kind, FormulaDrillNodeKind::FunctionCall);
    assert_eq!(sum.function_surface_name.as_deref(), Some("SUM"));
    assert_eq!(sum.returned_value, Some(CalcValue::number(6.0)));
    assert_eq!(sum.source_span, Some(TextSpan::new(1, 10)));

    let arg_names = sum
        .child_node_ids
        .iter()
        .map(|id| node(&trace, id).argument_name.clone().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(arg_names, vec!["number1", "number2", "number3"]);
    assert!(sum.child_node_ids.iter().all(|id| {
        node(&trace, id).argument_name_source == FormulaArgumentNameSource::OxFuncMetadata
    }));
}

#[test]
fn w076_nested_if_stays_under_sum_argument_and_false_branch_is_skipped() {
    let trace = execute("=SUM(IF(TRUE,2,3),4)");
    let sum = function_node(&trace, "SUM");
    let first_arg = argument_node(&trace, &sum.node_id, "number1");
    let nested_if_id = first_arg
        .child_node_ids
        .first()
        .expect("IF should be nested under SUM argument");
    let nested_if = node(&trace, nested_if_id);
    assert_eq!(nested_if.function_surface_name.as_deref(), Some("IF"));
    assert_eq!(nested_if.returned_value, Some(CalcValue::number(2.0)));

    let false_arg = argument_node(&trace, &nested_if.node_id, "value_if_false");
    assert_eq!(
        false_arg.branch_disposition,
        Some(FormulaDrillBranchDisposition::Skipped)
    );
    assert_eq!(
        false_arg.evaluation_state,
        FormulaDrillEvaluationState::Skipped
    );
    assert!(
        trace
            .evaluation_order
            .iter()
            .position(|id| id == nested_if_id)
            .unwrap()
            < trace
                .evaluation_order
                .iter()
                .position(|id| id == &sum.node_id)
                .unwrap()
    );
}

#[test]
fn w076_same_named_nested_calls_keep_post_order_prepared_values() {
    let trace = execute("=SUM(SUM(1,2),SUM(3,4))");
    let root = node(&trace, &trace.root_node_id);
    let outer_sum = node(
        &trace,
        root.child_node_ids.first().expect("outer SUM child"),
    );
    assert_eq!(outer_sum.function_surface_name.as_deref(), Some("SUM"));
    assert_eq!(outer_sum.returned_value, Some(CalcValue::number(10.0)));

    let first_inner = node(
        &trace,
        argument_node(&trace, &outer_sum.node_id, "number1")
            .child_node_ids
            .first()
            .expect("first inner SUM"),
    );
    let second_inner = node(
        &trace,
        argument_node(&trace, &outer_sum.node_id, "number2")
            .child_node_ids
            .first()
            .expect("second inner SUM"),
    );
    assert_eq!(first_inner.returned_value, Some(CalcValue::number(3.0)));
    assert_eq!(second_inner.returned_value, Some(CalcValue::number(7.0)));
}

#[test]
fn w076_if_false_skips_true_branch_and_evaluates_false_sum() {
    let trace = execute("=IF(FALSE,SUM(1,2),SUM(3,4))");
    let if_node = function_node(&trace, "IF");
    let true_arg = argument_node(&trace, &if_node.node_id, "value_if_true");
    let false_arg = argument_node(&trace, &if_node.node_id, "value_if_false");
    assert_eq!(
        true_arg.branch_disposition,
        Some(FormulaDrillBranchDisposition::Skipped)
    );
    assert_eq!(
        false_arg.branch_disposition,
        Some(FormulaDrillBranchDisposition::Taken)
    );
    let false_sum = node(
        &trace,
        false_arg
            .child_node_ids
            .first()
            .expect("false branch SUM child"),
    );
    assert_eq!(false_sum.function_surface_name.as_deref(), Some("SUM"));
    assert_eq!(false_sum.returned_value, Some(CalcValue::number(7.0)));
}

#[test]
fn w076_let_trace_exposes_bindings_body_and_name_reference_values() {
    let trace = execute("=LET(x,1,y,2,SUM(x,y))");
    let let_node = function_node(&trace, "LET");
    let binding_names = let_node
        .child_node_ids
        .iter()
        .map(|id| node(&trace, id))
        .filter(|node| node.kind == FormulaDrillNodeKind::LetBinding)
        .map(|node| node.argument_name.clone().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(binding_names, vec!["x", "y"]);

    let body = argument_node(&trace, &let_node.node_id, "body");
    let sum = node(
        &trace,
        body.child_node_ids
            .first()
            .expect("LET body should contain SUM"),
    );
    assert_eq!(sum.function_surface_name.as_deref(), Some("SUM"));
    assert_eq!(sum.returned_value, Some(CalcValue::number(3.0)));
    assert_eq!(
        argument_node(&trace, &sum.node_id, "number1").expression_text,
        Some("x".to_string())
    );
    assert_eq!(
        argument_node(&trace, &sum.node_id, "number1").value_after_coercion,
        Some(CalcValue::number(1.0))
    );
    assert_eq!(
        argument_node(&trace, &sum.node_id, "number2").expression_text,
        Some("y".to_string())
    );
    assert_eq!(
        argument_node(&trace, &sum.node_id, "number2").value_after_coercion,
        Some(CalcValue::number(2.0))
    );
}

#[test]
fn w076_divide_by_zero_trace_links_error_to_divide_node() {
    let trace = execute("=1/0");
    assert_eq!(
        trace.final_value,
        CalcValue::error(WorksheetErrorCode::Div0)
    );
    let divide = trace
        .nodes
        .iter()
        .find(|node| node.kind == FormulaDrillNodeKind::OperatorCall)
        .expect("divide node should exist");
    let error = divide.error.as_ref().expect("divide should carry error");
    assert_eq!(error.code.as_deref(), Some("#DIV/0!"));
    assert_eq!(error.causal_node_id.as_ref(), Some(&divide.node_id));
    assert_eq!(
        argument_node(&trace, &divide.node_id, "right").expression_text,
        Some("0".to_string())
    );
}

#[test]
fn w076_sequence_trace_exposes_typed_array_preview() {
    let trace = execute("=SEQUENCE(2,2)");
    let sequence = function_node(&trace, "SEQUENCE");
    let preview = sequence
        .value_preview
        .as_ref()
        .expect("SEQUENCE should carry array preview");
    assert_eq!(preview.value_kind, "array");
    assert_eq!(
        preview
            .array_shape
            .as_ref()
            .map(|shape| (shape.rows, shape.cols)),
        Some((2, 2))
    );
    assert_eq!(preview.preview, vec!["1", "2", "3", "4"]);
    assert!(!preview.truncated);
}

#[test]
fn w076_incomplete_sum_trace_links_diagnostics_to_partial_call() {
    let trace = RuntimeEnvironment::new().formula_drill_trace_for_source(FormulaSourceRecord::new(
        "w076:incomplete-sum",
        1,
        "=SUM(",
    ));
    let sum = function_node(&trace, "SUM");
    assert_eq!(sum.source_span, Some(TextSpan::new(1, 4)));
    assert!(
        node(&trace, &sum.node_id)
            .child_node_ids
            .iter()
            .map(|id| node(&trace, id))
            .any(|node| node.kind == FormulaDrillNodeKind::DiagnosticPlaceholder)
    );
    assert!(!trace.diagnostics.is_empty());
    assert!(
        trace
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.node_id.is_some())
    );
}
