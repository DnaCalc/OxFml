use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::seam::{Locus, RejectCode};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, run_oxfunc_preparation_adapter,
};
use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText};

fn text_cell(value: &str) -> ArrayCellValue {
    ArrayCellValue::Text(ExcelText::from_interop_assignment(value))
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

#[test]
fn adapter_executes_groupby_default_callable_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-default-callable",
        "formula:groupby-default-callable",
        "=GROUPBY({\"2024\";\"2024\";\"2025\";\"2025\"},{10;20;30;40},LAMBDA(x,SUM(x)))",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![text_cell("2024"), ArrayCellValue::Number(30.0)],
        vec![text_cell("2025"), ArrayCellValue::Number(70.0)],
        vec![text_cell("Total"), ArrayCellValue::Number(100.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_groupby_builtin_sum_callable_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-builtin-sum-callable",
        "formula:groupby-builtin-sum-callable",
        "=GROUPBY({\"2024\";\"2024\";\"2025\";\"2025\"},{10;20;30;40},SUM)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby builtin callable adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![text_cell("2024"), ArrayCellValue::Number(30.0)],
        vec![text_cell("2025"), ArrayCellValue::Number(70.0)],
        vec![text_cell("Total"), ArrayCellValue::Number(100.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_groupby_sort_sensitive_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-sort-sensitive",
        "formula:groupby-sort-sensitive",
        "=GROUPBY({\"2024\";\"2024\";\"2025\";\"2025\"},{10;20;30;40},LAMBDA(x,SUM(x)),,1,-2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby sort adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![text_cell("2025"), ArrayCellValue::Number(70.0)],
        vec![text_cell("2024"), ArrayCellValue::Number(30.0)],
        vec![text_cell("Total"), ArrayCellValue::Number(100.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_groupby_builtin_sum_sort_sensitive_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-builtin-sum-sort-sensitive",
        "formula:groupby-builtin-sum-sort-sensitive",
        "=GROUPBY({\"2024\";\"2024\";\"2025\";\"2025\"},{10;20;30;40},SUM,,1,-2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby builtin sort adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![text_cell("2025"), ArrayCellValue::Number(70.0)],
        vec![text_cell("2024"), ArrayCellValue::Number(30.0)],
        vec![text_cell("Total"), ArrayCellValue::Number(100.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_groupby_builtin_sum_hierarchical_subtotal_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-builtin-sum-hierarchical-subtotal",
        "formula:groupby-builtin-sum-hierarchical-subtotal",
        "=GROUPBY({\"East\",\"A\";\"East\",\"B\";\"East\",\"A\";\"West\",\"A\";\"West\",\"B\"},{10;20;30;40;50},SUM,,2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby builtin hierarchical subtotal adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            text_cell("East"),
            text_cell("A"),
            ArrayCellValue::Number(40.0),
        ],
        vec![
            text_cell("East"),
            text_cell("B"),
            ArrayCellValue::Number(20.0),
        ],
        vec![
            text_cell("East"),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::Number(60.0),
        ],
        vec![
            text_cell("West"),
            text_cell("A"),
            ArrayCellValue::Number(40.0),
        ],
        vec![
            text_cell("West"),
            text_cell("B"),
            ArrayCellValue::Number(50.0),
        ],
        vec![
            text_cell("West"),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::Number(90.0),
        ],
        vec![
            text_cell("Grand Total"),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::Number(150.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_groupby_builtin_sum_visible_headers_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-builtin-sum-visible-headers",
        "formula:groupby-builtin-sum-visible-headers",
        "=GROUPBY({\"Region\",\"Product\";\"East\",\"A\";\"East\",\"B\"},{\"Sales\";10;20},SUM,3)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby builtin visible headers adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            text_cell("Region"),
            text_cell("Product"),
            text_cell("Sales"),
        ],
        vec![
            text_cell("East"),
            text_cell("A"),
            ArrayCellValue::Number(10.0),
        ],
        vec![
            text_cell("East"),
            text_cell("B"),
            ArrayCellValue::Number(20.0),
        ],
        vec![
            text_cell("Total"),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::Number(30.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_groupby_filter_and_descending_value_sort_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-filter-and-descending-value-sort",
        "formula:groupby-filter-and-descending-value-sort",
        "=GROUPBY({\"A\";\"B\";\"A\";\"B\"},{10;20;40;50},SUM,,,-2,{TRUE;FALSE;TRUE;FALSE})",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby filter/sort adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![text_cell("A"), ArrayCellValue::Number(50.0)],
        vec![text_cell("Total"), ArrayCellValue::Number(50.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_rejects_groupby_tabular_subtotals_as_value_error() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "groupby-tabular-subtotals-rejected",
        "formula:groupby-tabular-subtotals-rejected",
        "=GROUPBY({\"East\",\"A\";\"East\",\"B\"},{10;20},SUM,,2,,,1)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("groupby tabular subtotals should now project a worksheet-visible value error");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
    assert_eq!(run.evaluation_artifact.commit_decision_kind, "accepted");
}

#[test]
fn adapter_executes_pivotby_default_callable_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-default-callable",
        "formula:pivotby-default-callable",
        "=PIVOTBY({\"East\";\"East\";\"West\";\"West\"},{\"A\";\"B\";\"A\";\"B\"},{10;20;40;50},LAMBDA(x,SUM(x)))",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("A"),
            text_cell("B"),
            text_cell("Total"),
        ],
        vec![
            text_cell("East"),
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
        ],
        vec![
            text_cell("West"),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(90.0),
        ],
        vec![
            text_cell("Total"),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(70.0),
            ArrayCellValue::Number(120.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_pivotby_builtin_sum_callable_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-builtin-sum-callable",
        "formula:pivotby-builtin-sum-callable",
        "=PIVOTBY({\"East\";\"East\";\"West\";\"West\"},{\"A\";\"B\";\"A\";\"B\"},{10;20;40;50},SUM)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby builtin adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("A"),
            text_cell("B"),
            text_cell("Total"),
        ],
        vec![
            text_cell("East"),
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
        ],
        vec![
            text_cell("West"),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(90.0),
        ],
        vec![
            text_cell("Total"),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(70.0),
            ArrayCellValue::Number(120.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_pivotby_filter_and_totals_sensitive_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-filter-sensitive",
        "formula:pivotby-filter-sensitive",
        "=PIVOTBY({\"East\";\"East\";\"West\";\"West\"},{\"A\";\"B\";\"A\";\"B\"},{10;20;40;50},LAMBDA(x,SUM(x)),,0,,0,,{TRUE;FALSE;TRUE;FALSE})",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby filter adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![ArrayCellValue::EmptyCell, text_cell("A")],
        vec![text_cell("East"), ArrayCellValue::Number(10.0)],
        vec![text_cell("West"), ArrayCellValue::Number(40.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_pivotby_builtin_sum_filter_and_totals_sensitive_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-builtin-sum-filter-sensitive",
        "formula:pivotby-builtin-sum-filter-sensitive",
        "=PIVOTBY({\"East\";\"East\";\"West\";\"West\"},{\"A\";\"B\";\"A\";\"B\"},{10;20;40;50},SUM,,0,,0,,{TRUE;FALSE;TRUE;FALSE})",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby builtin filter adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![ArrayCellValue::EmptyCell, text_cell("A")],
        vec![text_cell("East"), ArrayCellValue::Number(10.0)],
        vec![text_cell("West"), ArrayCellValue::Number(40.0)],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_pivotby_builtin_sum_row_and_column_total_sort_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-builtin-sum-row-col-total-sort",
        "formula:pivotby-builtin-sum-row-col-total-sort",
        "=PIVOTBY({\"East\";\"East\";\"West\";\"West\"},{\"A\";\"B\";\"A\";\"B\"},{10;20;40;50},SUM,,,-2,,-2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby builtin row/column total sort adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("B"),
            text_cell("A"),
            text_cell("Total"),
        ],
        vec![
            text_cell("West"),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(90.0),
        ],
        vec![
            text_cell("East"),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(30.0),
        ],
        vec![
            text_cell("Total"),
            ArrayCellValue::Number(70.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(120.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_executes_pivotby_builtin_sum_visible_headers_lane() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "pivotby-builtin-sum-visible-headers",
        "formula:pivotby-builtin-sum-visible-headers",
        "=PIVOTBY({\"Region\";\"East\";\"West\"},{\"Product\";\"A\";\"B\"},{\"Sales\";40;50},SUM,3)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("pivotby builtin visible headers adapter run");

    let expected = EvalArray::from_rows(vec![
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("Product"),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::EmptyCell,
        ],
        vec![
            ArrayCellValue::EmptyCell,
            text_cell("A"),
            text_cell("B"),
            text_cell("Total"),
        ],
        vec![
            text_cell("Region"),
            text_cell("Sales"),
            text_cell("Sales"),
            text_cell("Sales"),
        ],
        vec![
            text_cell("East"),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(0.0),
            ArrayCellValue::Number(40.0),
        ],
        vec![
            text_cell("West"),
            ArrayCellValue::Number(0.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(50.0),
        ],
        vec![
            text_cell("Total"),
            ArrayCellValue::Number(40.0),
            ArrayCellValue::Number(50.0),
            ArrayCellValue::Number(90.0),
        ],
    ])
    .expect("expected array");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Array(expected)
    );
}

#[test]
fn adapter_rejects_duplicate_lambda_parameter_names_as_bind_mismatch() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "duplicate-lambda-parameters",
        "formula:duplicate-lambda-parameters",
        "=LAMBDA(x,x,x)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("duplicate lambda adapter run");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(RejectCode::BindMismatch)
    );
    assert!(
        run.preparation_artifact
            .bind_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "duplicate LAMBDA parameter name 'x'")
    );
}

#[test]
fn adapter_rejects_malformed_lambda_parameter_declaration_as_bind_mismatch() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "malformed-lambda-parameter",
        "formula:malformed-lambda-parameter",
        "=LAMBDA(1,1)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("malformed lambda adapter run");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(RejectCode::BindMismatch)
    );
    assert!(run.preparation_artifact.bind_diagnostics.iter().any(
        |diagnostic| diagnostic.message == "LAMBDA parameter did not bind as helper parameter"
    ));
}
