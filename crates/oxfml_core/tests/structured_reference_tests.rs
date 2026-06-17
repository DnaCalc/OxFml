use oxfunc_core::value::ExcelText;

use oxfml_core::EvaluationBackend;
use oxfml_core::binding::{
    BindContext, BindRequest, BoundExpr, NameKind, NormalizedReference, ReferenceExpr,
    StructuredReferenceSourceTokenKind, StructuredResolvedRef, StructuredSectionKind,
    StructuredSelectorKind, bind_formula,
};
use oxfml_core::interface::{
    TableCallerRegion, TableColumnDescriptor, TableDescriptor, TableRef, TableRegionKind,
    TypedContextQueryBundle,
};
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue};

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

    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_span_utf8, TextSpan::new(1, 14));
    assert_eq!(record.source_token_text, "Table1[Amount]");
    assert_eq!(
        record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(record.explicit_table_name.as_deref(), Some("Table1"));
    assert!(!record.omitted_table_name);
    assert_eq!(record.effective_table_id.as_deref(), Some("table:1"));
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
    assert_eq!(record.selected_sections, vec![StructuredSectionKind::Data]);
    assert_eq!(
        record.selected_regions[0].section_kind,
        StructuredSectionKind::Data
    );
    assert_eq!(record.selected_regions[0].column_range_refs, vec!["B2:B4"]);
    assert!(!record.uses_this_row);
    assert!(!record.caller_context_dependent);
    assert!(record.resolved_reference.is_some());
    assert!(record.diagnostics.is_empty());
}

#[test]
fn binds_sheet_qualified_structured_reference_preserving_full_source_token() {
    let mut table = sample_table();
    table.sheet_scope_ref = "Sheet1".to_string();
    let mut context = base_bind_context();
    context.sheet_id = "Sheet1".to_string();
    context.table_catalog = vec![table];

    let bound = bind_with_table_context("=Sheet1!Table1[Amount]", context);

    assert!(bound.diagnostics.is_empty());
    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_span_utf8, TextSpan::new(1, 21));
    assert_eq!(record.source_token_text, "Sheet1!Table1[Amount]");
    assert_eq!(record.explicit_table_name.as_deref(), Some("Table1"));
    assert!(!record.omitted_table_name);
    assert_eq!(record.effective_table_id.as_deref(), Some("table:1"));
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
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

    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_span_utf8, TextSpan::new(1, 9));
    assert_eq!(record.source_token_text, "[@Amount]");
    assert_eq!(
        record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(record.explicit_table_name, None);
    assert!(record.omitted_table_name);
    assert_eq!(record.effective_table_id.as_deref(), Some("table:1"));
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
    assert_eq!(
        record.selected_sections,
        vec![StructuredSectionKind::ThisRow]
    );
    assert!(record.uses_this_row);
    assert!(record.caller_context_dependent);
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

    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_token_text, "Table1[#Headers]");
    assert_eq!(
        record.selected_sections,
        vec![StructuredSectionKind::Headers]
    );
    assert_eq!(
        record.selected_regions[0].section_kind,
        StructuredSectionKind::Headers
    );
    assert_eq!(
        record.selected_regions[0].region_ref.as_deref(),
        Some("A1:C1")
    );
}

#[test]
fn binds_headers_and_totals_from_exact_region_refs() {
    let mut context = base_bind_context();
    context.table_catalog = vec![sample_table_with_exact_section_refs()];

    let headers = bind_with_table_context("=Table1[#Headers]", context.clone());
    assert!(headers.diagnostics.is_empty());
    let NormalizedReference::Structured(headers_ref) = &headers.normalized_references[0] else {
        panic!("expected structured headers reference");
    };
    let StructuredResolvedRef::Area(headers_area) = &headers_ref.resolved_reference else {
        panic!("expected exact header region area");
    };
    assert_eq!(headers_area.top_left.row, 10);
    assert_eq!(headers_area.top_left.col, 8);
    assert_eq!(headers_area.height, 1);
    assert_eq!(headers_area.width, 3);

    let totals = bind_with_table_context("=Table1[#Totals]", context);
    assert!(totals.diagnostics.is_empty());
    let NormalizedReference::Structured(totals_ref) = &totals.normalized_references[0] else {
        panic!("expected structured totals reference");
    };
    let StructuredResolvedRef::Area(totals_area) = &totals_ref.resolved_reference else {
        panic!("expected exact totals region area");
    };
    assert_eq!(totals_area.top_left.row, 20);
    assert_eq!(totals_area.top_left.col, 8);
    assert_eq!(totals_area.height, 1);
    assert_eq!(totals_area.width, 3);

    let headers_record = &headers.structured_reference_bind_records[0];
    assert_eq!(
        headers_record.selected_regions[0].section_kind,
        StructuredSectionKind::Headers
    );
    assert_eq!(
        headers_record.selected_regions[0].region_ref.as_deref(),
        Some("H10:J10")
    );
    let totals_record = &totals.structured_reference_bind_records[0];
    assert_eq!(
        totals_record.selected_regions[0].section_kind,
        StructuredSectionKind::Totals
    );
    assert_eq!(
        totals_record.selected_regions[0].region_ref.as_deref(),
        Some("H20:J20")
    );
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

    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_token_text, "Table1[[#Data],[Amount]:[Tax]]");
    assert_eq!(
        record.selected_column_ids,
        vec!["column:amount", "column:tax"]
    );
    assert_eq!(record.selected_sections, vec![StructuredSectionKind::Data]);
    assert_eq!(
        record.selected_regions[0].column_range_refs,
        vec!["B2:B4", "C2:C4"]
    );
}

#[test]
fn binds_zero_row_data_column_reference_without_data_a1_area() {
    let mut context = base_bind_context();
    context.table_catalog = vec![sample_zero_row_table()];

    let bound = bind_with_table_context("=Table1[Amount]", context);

    assert!(bound.diagnostics.is_empty());
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!("expected structured reference");
    };
    assert_eq!(structured.table_id, "table:zero");
    assert_eq!(structured.selected_column_ids, vec!["column:amount"]);
    assert_eq!(
        structured.section_qualifiers,
        vec![StructuredSectionKind::Data]
    );
    let StructuredResolvedRef::EmptyArea(empty) = &structured.resolved_reference else {
        panic!(
            "expected empty data body reference, got {:?}",
            structured.resolved_reference
        );
    };
    assert_eq!(empty.section_kind, StructuredSectionKind::Data);
    assert_eq!(empty.selected_column_ids, vec!["column:amount"]);
    assert_eq!(empty.column_count, 1);
    assert_eq!(
        empty.row_membership_identity.as_deref(),
        Some("table:zero:rows:empty")
    );
    assert_eq!(
        empty.row_order_identity.as_deref(),
        Some("table:zero:row-order:empty")
    );

    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_span_utf8, TextSpan::new(1, 14));
    assert_eq!(record.source_token_text, "Table1[Amount]");
    assert_eq!(
        record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert_eq!(record.effective_table_id.as_deref(), Some("table:zero"));
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
    assert_eq!(record.selected_sections, vec![StructuredSectionKind::Data]);
    assert!(record.selected_regions[0].is_empty);
    assert!(record.selected_regions[0].column_range_refs.is_empty());
    assert!(record.diagnostics.is_empty());
}

#[test]
fn binds_zero_row_headers_totals_and_all_without_data_column_a1_area() {
    let mut context = base_bind_context();
    context.table_catalog = vec![sample_zero_row_table()];

    let headers = bind_with_table_context("=Table1[#Headers]", context.clone());
    assert!(headers.diagnostics.is_empty());
    let NormalizedReference::Structured(headers_ref) = &headers.normalized_references[0] else {
        panic!("expected structured headers reference");
    };
    let StructuredResolvedRef::Area(headers_area) = &headers_ref.resolved_reference else {
        panic!("expected header area");
    };
    assert_eq!(headers_area.top_left.row, 1);
    assert_eq!(headers_area.top_left.col, 1);
    assert_eq!(headers_area.height, 1);
    assert_eq!(headers_area.width, 3);
    assert!(!headers.structured_reference_bind_records[0].selected_regions[0].is_empty);

    let totals = bind_with_table_context("=Table1[#Totals]", context.clone());
    assert!(totals.diagnostics.is_empty());
    let NormalizedReference::Structured(totals_ref) = &totals.normalized_references[0] else {
        panic!("expected structured totals reference");
    };
    let StructuredResolvedRef::Area(totals_area) = &totals_ref.resolved_reference else {
        panic!("expected totals area");
    };
    assert_eq!(totals_area.top_left.row, 2);
    assert_eq!(totals_area.top_left.col, 1);
    assert_eq!(totals_area.height, 1);
    assert_eq!(totals_area.width, 3);

    let all_amount = bind_with_table_context("=Table1[[#All],[Amount]]", context);
    assert!(all_amount.diagnostics.is_empty());
    let NormalizedReference::Structured(all_ref) = &all_amount.normalized_references[0] else {
        panic!("expected all-section structured reference");
    };
    let StructuredResolvedRef::Area(all_area) = &all_ref.resolved_reference else {
        panic!("expected all-section area");
    };
    assert_eq!(all_area.top_left.row, 1);
    assert_eq!(all_area.top_left.col, 2);
    assert_eq!(all_area.height, 2);
    assert_eq!(all_area.width, 1);
}

#[test]
fn zero_row_current_row_reference_reports_typed_packet_diagnostic() {
    let mut context = base_bind_context();
    context.table_catalog = vec![sample_zero_row_table()];
    context.enclosing_table_ref = Some(TableRef {
        table_id: "table:zero".to_string(),
    });
    context.caller_table_region = Some(TableCallerRegion {
        table_id: "table:zero".to_string(),
        region_kind: TableRegionKind::Data,
        data_row_offset: Some(0),
    });

    let bound = bind_with_table_context("=[@Amount]", context);

    assert_eq!(bound.diagnostics.len(), 1);
    assert!(bound.diagnostics[0].message.contains("no table data row"));
    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_token_text, "[@Amount]");
    assert_eq!(
        record.source_token_kind,
        StructuredReferenceSourceTokenKind::StructuredReference
    );
    assert!(record.omitted_table_name);
    assert_eq!(record.effective_table_id.as_deref(), Some("table:zero"));
    assert_eq!(record.selected_column_ids, vec!["column:amount"]);
    assert_eq!(
        record.selected_sections,
        vec![StructuredSectionKind::ThisRow]
    );
    assert!(record.uses_this_row);
    assert!(record.caller_context_dependent);
    assert_eq!(record.diagnostics.len(), 1);
    assert!(record.selected_regions[0].is_empty);
    assert!(record.selected_regions[0].column_range_refs.is_empty());
    assert_eq!(record.resolved_reference, None);
}

#[test]
fn binds_escaped_structured_column_names_without_section_confusion() {
    let mut context = base_bind_context();
    context.table_catalog = vec![sample_table_with_escaped_columns()];

    let single_column = bind_with_table_context("=Table1[['#Data]]", context.clone());

    assert!(single_column.diagnostics.is_empty());
    let NormalizedReference::Structured(single_column_ref) =
        &single_column.normalized_references[0]
    else {
        panic!("expected escaped single-column structured reference");
    };
    assert_eq!(
        single_column_ref.selector_kind,
        StructuredSelectorKind::Column
    );
    assert_eq!(
        single_column_ref.section_qualifiers,
        vec![StructuredSectionKind::Data]
    );
    assert_eq!(
        single_column_ref.selected_column_ids,
        vec!["column:hash-data"]
    );

    let section_column = bind_with_table_context("=Table1[[#Data],['#Data]]", context.clone());

    assert!(section_column.diagnostics.is_empty());
    let NormalizedReference::Structured(section_column_ref) =
        &section_column.normalized_references[0]
    else {
        panic!("expected escaped section-column structured reference");
    };
    assert_eq!(
        section_column_ref.selector_kind,
        StructuredSelectorKind::SectionColumn
    );
    assert_eq!(
        section_column_ref.section_qualifiers,
        vec![StructuredSectionKind::Data]
    );
    assert_eq!(
        section_column_ref.selected_column_ids,
        vec!["column:hash-data"]
    );
    let section_column_record = &section_column.structured_reference_bind_records[0];
    assert_eq!(
        section_column_record.source_token_text,
        "Table1[[#Data],['#Data]]"
    );

    let bound = bind_with_table_context("=Table1[[#Data],['#Data]:[Gross']Margin]]", context);

    assert!(bound.diagnostics.is_empty());
    let NormalizedReference::Structured(structured) = &bound.normalized_references[0] else {
        panic!("expected escaped structured reference");
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
        vec!["column:hash-data", "column:gross-margin"]
    );
    let StructuredResolvedRef::Area(area) = &structured.resolved_reference else {
        panic!("expected escaped multi-column data area");
    };
    assert_eq!(area.top_left.row, 2);
    assert_eq!(area.top_left.col, 2);
    assert_eq!(area.height, 3);
    assert_eq!(area.width, 2);

    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(
        record.source_token_text,
        "Table1[[#Data],['#Data]:[Gross']Margin]]"
    );
    assert_eq!(
        record.selected_column_ids,
        vec!["column:hash-data", "column:gross-margin"]
    );
    assert_eq!(record.selected_sections, vec![StructuredSectionKind::Data]);
    assert_eq!(
        record.selected_regions[0].column_range_refs,
        vec!["B2:B4", "C2:C4"]
    );
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
    let record = &bound.structured_reference_bind_records[0];
    assert_eq!(record.source_token_text, "[@Amount]");
    assert!(record.omitted_table_name);
    assert_eq!(record.effective_table_id, None);
    assert_eq!(record.resolved_reference, None);
    assert_eq!(record.diagnostics.len(), 1);
    assert_eq!(
        record.diagnostics[0].diagnostic_code,
        "structured_reference_bind_error"
    );
    assert_eq!(
        record.diagnostics[0].message,
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
    host.set_cell_value("B3", CalcValue::number(7.0));

    let output = host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::default(),
            None,
        )
        .expect("structured current-row host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Number(9)");
    assert_eq!(output.evaluation.oxfunc_value, CalcValue::number(9.0));
}

#[test]
fn host_evaluates_sum_over_explicit_structured_column_reference() {
    let mut host = SingleFormulaHost::new("host:structured-sum", "=SUM(Table1[Amount])");
    host.set_table_catalog(vec![sample_table()]);
    host.set_cell_value(
        "B2:B4",
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
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
    assert_eq!(output.evaluation.oxfunc_value, CalcValue::number(12.0));
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
        CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::number(3.0), CalcValue::number(1.0)],
                vec![CalcValue::number(4.0), CalcValue::number(2.0)],
                vec![CalcValue::number(5.0), CalcValue::number(3.0)],
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
    assert_eq!(output.evaluation.oxfunc_value, CalcValue::number(18.0));
}

#[test]
fn host_evaluates_sum_over_escaped_structured_column_reference() {
    let mut host = SingleFormulaHost::new("host:structured-escaped", "=SUM(Table1['#Data])");
    host.set_table_catalog(vec![sample_table_with_escaped_columns()]);
    host.set_cell_value(
        "B2:B4",
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::number(3.0),
                CalcValue::number(4.0),
                CalcValue::number(5.0),
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
        .expect("escaped structured column host recalculation should succeed");

    assert_eq!(output.evaluation.result.payload_summary, "Number(12)");
    assert_eq!(output.evaluation.oxfunc_value, CalcValue::number(12.0));
}

#[test]
fn host_evaluates_headers_section_only_structured_reference() {
    let mut host = SingleFormulaHost::new("host:structured-headers", "=Table1[#Headers]");
    host.set_table_catalog(vec![sample_table()]);
    host.set_cell_value(
        "A1:C1",
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::text(ExcelText::from_interop_assignment("Label")),
                CalcValue::text(ExcelText::from_interop_assignment("Amount")),
                CalcValue::text(ExcelText::from_interop_assignment("Tax")),
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
    let CoreValue::Array(result) = output.evaluation.oxfunc_value.core() else {
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
        CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::text(ExcelText::from_interop_assignment("Total")),
                CalcValue::number(12.0),
                CalcValue::number(6.0),
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
    let CoreValue::Array(result) = output.evaluation.oxfunc_value.core() else {
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

        reference_bind_profile: None,
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
        row_membership_identity: Some("table:1:rows:v1".to_string()),
        row_order_identity: Some("table:1:row-order:v1".to_string()),
        header_region_ref: Some("A1:C1".to_string()),
        totals_region_ref: Some("A5:C5".to_string()),
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

fn sample_zero_row_table() -> TableDescriptor {
    TableDescriptor {
        table_id: "table:zero".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "A1:C2".to_string(),
        row_membership_identity: Some("table:zero:rows:empty".to_string()),
        row_order_identity: Some("table:zero:row-order:empty".to_string()),
        header_region_ref: Some("A1:C1".to_string()),
        totals_region_ref: Some("A2:C2".to_string()),
        header_row_present: true,
        totals_row_present: true,
        columns: vec![
            TableColumnDescriptor {
                column_id: "column:label".to_string(),
                column_name: "Label".to_string(),
                ordinal: 1,
                column_range_ref: String::new(),
            },
            TableColumnDescriptor {
                column_id: "column:amount".to_string(),
                column_name: "Amount".to_string(),
                ordinal: 2,
                column_range_ref: String::new(),
            },
            TableColumnDescriptor {
                column_id: "column:tax".to_string(),
                column_name: "Tax".to_string(),
                ordinal: 3,
                column_range_ref: String::new(),
            },
        ],
    }
}

fn sample_table_with_exact_section_refs() -> TableDescriptor {
    TableDescriptor {
        table_id: "table:1".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "H10:J20".to_string(),
        row_membership_identity: Some("table:1:rows:exact-section".to_string()),
        row_order_identity: Some("table:1:row-order:exact-section".to_string()),
        header_region_ref: Some("H10:J10".to_string()),
        totals_region_ref: Some("H20:J20".to_string()),
        header_row_present: true,
        totals_row_present: true,
        columns: vec![
            TableColumnDescriptor {
                column_id: "column:label".to_string(),
                column_name: "Label".to_string(),
                ordinal: 1,
                column_range_ref: "H11:H19".to_string(),
            },
            TableColumnDescriptor {
                column_id: "column:amount".to_string(),
                column_name: "Amount".to_string(),
                ordinal: 2,
                column_range_ref: "I11:I19".to_string(),
            },
            TableColumnDescriptor {
                column_id: "column:tax".to_string(),
                column_name: "Tax".to_string(),
                ordinal: 3,
                column_range_ref: "J11:J19".to_string(),
            },
        ],
    }
}

fn sample_table_with_escaped_columns() -> TableDescriptor {
    TableDescriptor {
        table_id: "table:escaped".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:default".to_string(),
        sheet_scope_ref: "sheet:default".to_string(),
        table_range_ref: "A1:C5".to_string(),
        row_membership_identity: Some("table:escaped:rows:v1".to_string()),
        row_order_identity: Some("table:escaped:row-order:v1".to_string()),
        header_region_ref: Some("A1:C1".to_string()),
        totals_region_ref: Some("A5:C5".to_string()),
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
                column_id: "column:hash-data".to_string(),
                column_name: "#Data".to_string(),
                ordinal: 2,
                column_range_ref: "B2:B4".to_string(),
            },
            TableColumnDescriptor {
                column_id: "column:gross-margin".to_string(),
                column_name: "Gross]Margin".to_string(),
                ordinal: 3,
                column_range_ref: "C2:C4".to_string(),
            },
        ],
    }
}
