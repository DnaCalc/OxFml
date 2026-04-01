use oxfunc_core::value::{ArrayCellValue, EvalArray, EvalValue, ExcelText};

use oxfml_core::EvaluationBackend;
use oxfml_core::binding::{
    BindContext, BindRequest, BoundExpr, NameKind, NormalizedReference, ReferenceExpr,
    StructuredResolvedRef, StructuredSectionKind, StructuredSelectorKind, bind_formula,
};
use oxfml_core::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
    TypedContextQueryBundle,
};
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::test_support::host::SingleFormulaHost;

#[test]
fn binds_explicit_structured_column_reference() {
    let bound = bind_with_table_context("=Table1[Amount]", base_bind_context());

    assert!(bound.diagnostics.is_empty());
    assert_eq!(bound.normalized_references.len(), 1);
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!(
            "expected structured reference, got {:?}",
            bound.normalized_references[0]
        );
    };
    assert_eq!(structured.table_id, "table:1");
    assert_eq!(structured.selector_kind, StructuredSelectorKind::Column);
    assert_eq!(
        structured.section_qualifiers,
        vec![StructuredSectionKind::Data]
    );
    assert_eq!(structured.selected_column_ids, vec!["column:amount"]);
    assert!(!structured.caller_row_sensitive);
    let StructuredResolvedRef::Area(area) = &structured.resolved_reference else {
        panic!("expected data column area");
    };
    assert_eq!(area.top_left.row, 2);
    assert_eq!(area.top_left.col, 2);
    assert_eq!(area.height, 3);
    assert_eq!(area.width, 1);
}

#[test]
fn binds_omitted_table_name_with_current_row_context() {
    let mut context = base_bind_context();
    context.enclosing_table_ref = Some(TableRef {
        table_id: "table:1".to_string(),
    });
    context.caller_table_region = Some(TableCallerRegion {
        table_id: "table:1".to_string(),
        region_kind: TableRegionKind::Data,
        data_row_offset: Some(1),
    });

    let bound = bind_with_table_context("=[@Amount]", context);

    assert!(bound.diagnostics.is_empty());
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!("expected structured reference");
    };
    assert_eq!(
        structured.selector_kind,
        StructuredSelectorKind::ThisRowColumn
    );
    assert!(structured.caller_row_sensitive);
    assert_eq!(
        structured.section_qualifiers,
        vec![StructuredSectionKind::ThisRow]
    );
    let StructuredResolvedRef::Cell(cell) = &structured.resolved_reference else {
        panic!("expected current-row cell");
    };
    assert_eq!(cell.coord.row, 3);
    assert_eq!(cell.coord.col, 2);
}

#[test]
fn binds_section_only_headers_reference_across_all_columns() {
    let bound = bind_with_table_context("=Table1[#Headers]", base_bind_context());

    assert!(bound.diagnostics.is_empty());
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!("expected structured reference");
    };
    assert_eq!(structured.selector_kind, StructuredSelectorKind::Section);
    assert_eq!(
        structured.section_qualifiers,
        vec![StructuredSectionKind::Headers]
    );
    assert_eq!(
        structured.selected_column_ids,
        vec!["column:label", "column:amount", "column:tax"]
    );
    let StructuredResolvedRef::Area(area) = &structured.resolved_reference else {
        panic!("expected header-row area");
    };
    assert_eq!(area.top_left.row, 1);
    assert_eq!(area.top_left.col, 1);
    assert_eq!(area.height, 1);
    assert_eq!(area.width, 3);
}

#[test]
fn binds_all_qualified_multi_column_reference() {
    let bound = bind_with_table_context("=Table1[[#All],[Amount]:[Tax]]", base_bind_context());

    assert!(bound.diagnostics.is_empty());
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!("expected structured reference");
    };
    assert_eq!(
        structured.selector_kind,
        StructuredSelectorKind::SectionColumn
    );
    assert_eq!(
        structured.section_qualifiers,
        vec![StructuredSectionKind::All]
    );
    assert_eq!(
        structured.selected_column_ids,
        vec!["column:amount", "column:tax"]
    );
    let StructuredResolvedRef::Area(area) = &structured.resolved_reference else {
        panic!("expected all-section multi-column area");
    };
    assert_eq!(area.top_left.row, 1);
    assert_eq!(area.top_left.col, 2);
    assert_eq!(area.height, 5);
    assert_eq!(area.width, 2);
}

#[test]
fn binds_data_qualified_multi_column_reference() {
    let bound = bind_with_table_context("=Table1[[#Data],[Amount]:[Tax]]", base_bind_context());

    assert!(bound.diagnostics.is_empty());
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!("expected structured reference");
    };
    assert_eq!(
        structured.selector_kind,
        StructuredSelectorKind::SectionColumn
    );
    assert_eq!(
        structured.section_qualifiers,
        vec![StructuredSectionKind::Data]
    );
    assert_eq!(
        structured.selected_column_ids,
        vec!["column:amount", "column:tax"]
    );
    let StructuredResolvedRef::Area(area) = &structured.resolved_reference else {
        panic!("expected data-section multi-column area");
    };
    assert_eq!(area.top_left.row, 2);
    assert_eq!(area.top_left.col, 2);
    assert_eq!(area.height, 3);
    assert_eq!(area.width, 2);
}

#[test]
fn omitted_table_name_without_context_fails_bind_honestly() {
    let bound = bind_with_table_context("=[@Amount]", base_bind_context());

    assert_eq!(bound.normalized_references.len(), 1);
    assert_eq!(bound.diagnostics.len(), 1);
    assert_eq!(bound.unresolved_references.len(), 1);
    assert!(
        bound.diagnostics[0]
            .message
            .contains("structured reference requires enclosing table context")
    );
    assert_eq!(
        bound.unresolved_references[0].reason,
        "structured reference requires enclosing table context"
    );
}

#[test]
fn this_row_illegal_combination_fails_bind() {
    let mut context = base_bind_context();
    context.enclosing_table_ref = Some(TableRef {
        table_id: "table:1".to_string(),
    });
    context.caller_table_region = Some(TableCallerRegion {
        table_id: "table:1".to_string(),
        region_kind: TableRegionKind::Data,
        data_row_offset: Some(0),
    });

    let bound = bind_with_table_context("=Table1[[#This Row],[#Data],[Amount]]", context);

    assert_eq!(bound.diagnostics.len(), 1);
    assert_eq!(bound.unresolved_references.len(), 1);
    assert!(
        bound.diagnostics[0]
            .message
            .contains("#This Row must not be combined")
    );
}

#[test]
fn structured_reference_disambiguates_against_defined_name_collision() {
    let mut context = base_bind_context();
    context
        .names
        .insert("Table1".to_string(), NameKind::ValueLike);

    let bound = bind_with_table_context("=Table1[Amount]", context);

    let BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Structured(_))) = &bound.root
    else {
        panic!("structured syntax should bind as structured reference, not a defined name");
    };
}

#[test]
fn host_evaluates_current_row_structured_reference_lane() {
    let mut host = SingleFormulaHost::new("host:structured-row", "=[@Amount]+2");
    host.set_table_catalog(vec![sample_table()]);
    host.set_enclosing_table_ref(Some(TableRef {
        table_id: "table:1".to_string(),
    }));
    host.set_caller_table_region(Some(TableCallerRegion {
        table_id: "table:1".to_string(),
        region_kind: TableRegionKind::Data,
        data_row_offset: Some(1),
    }));
    host.set_cell_value("B3", EvalValue::Number(7.0));

    let output = host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::default(),
            None,
        )
        .expect("structured current-row host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Number(9)");
    assert_eq!(output.evaluation.oxfunc_value, EvalValue::Number(9.0));
}

#[test]
fn host_evaluates_sum_over_explicit_structured_column_reference() {
    let mut host = SingleFormulaHost::new("host:structured-sum", "=SUM(Table1[Amount])");
    host.set_table_catalog(vec![sample_table()]);
    host.set_cell_value(
        "B2:B4",
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Number(3.0),
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Number(5.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    );

    let output = host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::default(),
            None,
        )
        .expect("structured explicit-column host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Number(12)");
    assert_eq!(output.evaluation.oxfunc_value, EvalValue::Number(12.0));
}

#[test]
fn host_evaluates_sum_over_data_qualified_multi_column_structured_reference() {
    let mut host = SingleFormulaHost::new(
        "host:structured-data-multicol",
        "=SUM(Table1[[#Data],[Amount]:[Tax]])",
    );
    host.set_table_catalog(vec![sample_table()]);
    host.set_cell_value(
        "B2:C4",
        EvalValue::Array(
            EvalArray::from_rows(vec![
                vec![ArrayCellValue::Number(3.0), ArrayCellValue::Number(1.0)],
                vec![ArrayCellValue::Number(4.0), ArrayCellValue::Number(2.0)],
                vec![ArrayCellValue::Number(5.0), ArrayCellValue::Number(3.0)],
            ])
            .expect("array fixture should be valid"),
        ),
    );

    let output = host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::default(),
            None,
        )
        .expect("structured data multi-column host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Number(18)");
    assert_eq!(output.evaluation.oxfunc_value, EvalValue::Number(18.0));
}

#[test]
fn host_evaluates_headers_section_only_structured_reference() {
    let mut host = SingleFormulaHost::new("host:structured-headers", "=Table1[#Headers]");
    host.set_table_catalog(vec![sample_table()]);
    host.set_cell_value(
        "A1:C1",
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Text(ExcelText::from_interop_assignment("Label")),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("Amount")),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("Tax")),
            ]])
            .expect("array fixture should be valid"),
        ),
    );

    let output = host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::default(),
            None,
        )
        .expect("structured headers host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Array(1x3)");
    let EvalValue::Array(result) = &output.evaluation.oxfunc_value else {
        panic!("expected header row array");
    };
    assert_eq!(result.shape().rows, 1);
    assert_eq!(result.shape().cols, 3);
}

#[test]
fn host_evaluates_totals_section_only_structured_reference() {
    let mut host = SingleFormulaHost::new("host:structured-totals", "=Table1[#Totals]");
    host.set_table_catalog(vec![sample_table()]);
    host.set_cell_value(
        "A5:C5",
        EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Text(ExcelText::from_interop_assignment("Total")),
                ArrayCellValue::Number(12.0),
                ArrayCellValue::Number(6.0),
            ]])
            .expect("array fixture should be valid"),
        ),
    );

    let output = host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::default(),
            None,
        )
        .expect("structured totals host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Array(1x3)");
    let EvalValue::Array(result) = &output.evaluation.oxfunc_value else {
        panic!("expected totals row array");
    };
    assert_eq!(result.shape().rows, 1);
    assert_eq!(result.shape().cols, 3);
}

fn bind_with_table_context(formula: &str, context: BindContext) -> oxfml_core::BoundFormula {
    let source = FormulaSourceRecord::new("structured-fixture", 1, formula);
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("structured-struct-v1".to_string()),
            formula_token: source.formula_token(),
            ..context
        },
    })
    .bound_formula
}

fn base_bind_context() -> BindContext {
    BindContext {
        workbook_id: "book:default".to_string(),
        sheet_id: "sheet:default".to_string(),
        caller_row: 2,
        caller_col: 2,
        table_catalog: vec![sample_table()],
        ..BindContext::default()
    }
}

fn sample_table() -> TableDescriptor {
    TableDescriptor {
        table_id: "table:1".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "A1:C5".to_string(),
        header_row_present: true,
        totals_row_present: true,
        columns: vec![
            TableColumnDescriptor {
                column_id: "column:label".to_string(),
                column_name: "Label".to_string(),
                ordinal: 1,
                column_range_ref: "A2:A4".to_string(),
            },
            TableColumnDescriptor {
                column_id: "column:amount".to_string(),
                column_name: "Amount".to_string(),
                ordinal: 2,
                column_range_ref: "B2:B4".to_string(),
            },
            TableColumnDescriptor {
                column_id: "column:tax".to_string(),
                column_name: "Tax".to_string(),
                ordinal: 3,
                column_range_ref: "C2:C4".to_string(),
            },
        ],
    }
}
