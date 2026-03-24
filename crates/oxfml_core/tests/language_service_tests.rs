use oxfml_core::binding::{BindContext, BindRequest, NameKind, bind_formula};
use oxfml_core::interface::{TableColumnDescriptor, TableDescriptor};
use oxfml_core::language_service::{
    CompletionProposalKind, CompletionRequest, CompletionValidationRequest, EditFollowOnStage,
    FormulaEditRequest, SemanticPlanEditOptions, apply_completion_proposal, apply_formula_edit,
    build_editor_syntax_snapshot, build_function_help_lookup_request,
    build_intelligent_completion_context, collect_completion_proposals,
    signature_help_context_at_cursor, validate_completion_candidate,
};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::green::{GreenChild, GreenNode};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};

#[test]
fn editor_syntax_snapshot_tracks_leading_and_trailing_trivia() {
    let source = FormulaSourceRecord::new("editor-trivia", 1, "=SUM( A1 ) ");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });

    let snapshot = build_editor_syntax_snapshot(&source, &parse.green_tree);

    assert_eq!(snapshot.tokens.len(), 5);
    assert_eq!(snapshot.tokens[3].text, "A1");
    assert_eq!(snapshot.tokens[3].leading_trivia.len(), 1);
    assert_eq!(snapshot.tokens[3].leading_trivia[0].text, " ");
    assert_eq!(snapshot.tokens[4].text, ")");
    assert_eq!(snapshot.tokens[4].leading_trivia.len(), 1);
    assert_eq!(snapshot.tokens[4].leading_trivia[0].text, " ");
    assert_eq!(snapshot.tokens[4].trailing_trivia.len(), 1);
    assert_eq!(snapshot.tokens[4].trailing_trivia[0].text, " ");
}

#[test]
fn green_tree_tokens_canonically_own_trivia() {
    let source = FormulaSourceRecord::new("editor-green-trivia", 1, "=SUM( A1 ) ");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });

    let tree_tokens = collect_green_tokens(&parse.green_tree.root);
    let a1 = tree_tokens
        .iter()
        .find(|token| token.text == "A1")
        .expect("A1 token should exist");
    let close = tree_tokens
        .iter()
        .find(|token| token.text == ")")
        .expect("closing paren token should exist");

    assert_eq!(a1.leading_trivia.len(), 1);
    assert_eq!(a1.leading_trivia[0].text, " ");
    assert_eq!(a1.trailing_trivia.len(), 1);
    assert_eq!(a1.trailing_trivia[0].text, " ");
    assert_eq!(close.leading_trivia.len(), 1);
    assert_eq!(close.leading_trivia[0].text, " ");
    assert_eq!(close.trailing_trivia.len(), 1);
    assert_eq!(close.trailing_trivia[0].text, " ");
}

#[test]
fn apply_formula_edit_reuses_green_red_and_bind_when_text_is_unchanged() {
    let source = FormulaSourceRecord::new("editor-reuse", 1, "=SUM(A1)");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree.clone(),
        red_projection: red.clone(),
        context: editor_bind_context(source.clone()),
    });

    let result = apply_formula_edit(FormulaEditRequest {
        source: source.clone(),
        bind_context: editor_bind_context(source.clone()),
        previous_green_tree: Some(&parse.green_tree),
        previous_red_projection: Some(&red),
        previous_bound_formula: Some(&bind.bound_formula),
        follow_on_stage: EditFollowOnStage::ParseAndBind,
        semantic_plan_options: None,
    });

    assert!(result.reuse_summary.reused_green_tree);
    assert!(result.reuse_summary.reused_red_projection);
    assert!(result.reuse_summary.reused_bound_formula);
    assert!(result.bound_formula.is_some());
    assert_eq!(result.text_change_range, None);
}

#[test]
fn apply_formula_edit_reports_smallest_text_change_range() {
    let previous_source = FormulaSourceRecord::new("editor-change-range", 1, "=SUM(A1)");
    let previous_parse = parse_formula(ParseRequest {
        source: previous_source.clone(),
    });

    let updated_source = FormulaSourceRecord::new("editor-change-range", 2, "=SUM(B1)");
    let result = apply_formula_edit(FormulaEditRequest {
        source: updated_source,
        bind_context: editor_bind_context(previous_source.clone()),
        previous_green_tree: Some(&previous_parse.green_tree),
        previous_red_projection: None,
        previous_bound_formula: None,
        follow_on_stage: EditFollowOnStage::ParseOnly,
        semantic_plan_options: None,
    });

    assert_eq!(
        result.text_change_range,
        Some(oxfml_core::FormulaTextChangeRange {
            start: 5,
            old_len: 1,
            new_len: 1,
        })
    );
}

#[test]
fn apply_formula_edit_surfaces_bind_diagnostics_in_live_snapshot() {
    let source = FormulaSourceRecord::new("editor-bind-diag", 1, "=[@Amount]");

    let result = apply_formula_edit(FormulaEditRequest {
        source: source.clone(),
        bind_context: editor_bind_context(source),
        previous_green_tree: None,
        previous_red_projection: None,
        previous_bound_formula: None,
        follow_on_stage: EditFollowOnStage::ParseAndBind,
        semantic_plan_options: None,
    });

    assert!(
        result
            .live_diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_deref() == Some("bind")
                && diagnostic.suggested_fix_kind.as_deref()
                    == Some("supply_enclosing_table_context"))
    );
}

#[test]
fn completion_proposals_include_functions_defined_names_and_table_names() {
    let source = FormulaSourceRecord::new("editor-complete", 1, "=SU");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let mut bind_context = editor_bind_context(source.clone());
    bind_context
        .names
        .insert("SummaryName".to_string(), NameKind::ValueLike);

    let result = collect_completion_proposals(CompletionRequest {
        source: &source,
        green_tree: &parse.green_tree,
        red_projection: &red,
        bind_context: &bind_context,
        library_context_snapshot: Some(&sample_library_context_snapshot()),
        cursor_offset: source.entered_formula_text.chars().count(),
    });

    assert!(
        result
            .proposals
            .iter()
            .any(
                |proposal| proposal.proposal_kind == CompletionProposalKind::Function
                    && proposal.display_text == "SUM"
            )
    );
    assert!(
        result
            .proposals
            .iter()
            .any(
                |proposal| proposal.proposal_kind == CompletionProposalKind::DefinedName
                    && proposal.display_text == "SummaryName"
            )
    );
}

#[test]
fn completion_proposals_include_structured_selectors_and_columns() {
    let source = FormulaSourceRecord::new("editor-structured", 1, "=Table1[#H");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind_context = editor_bind_context(source.clone());

    let selector_result = collect_completion_proposals(CompletionRequest {
        source: &source,
        green_tree: &parse.green_tree,
        red_projection: &red,
        bind_context: &bind_context,
        library_context_snapshot: None,
        cursor_offset: source.entered_formula_text.chars().count(),
    });

    assert!(
        selector_result
            .proposals
            .iter()
            .any(
                |proposal| proposal.proposal_kind == CompletionProposalKind::StructuredSelector
                    && proposal.display_text == "#Headers"
            )
    );

    let column_source = FormulaSourceRecord::new("editor-column", 1, "=Table1[A");
    let column_parse = parse_formula(ParseRequest {
        source: column_source.clone(),
    });
    let column_red = project_red_view(
        column_source.formula_stable_id.clone(),
        &column_parse.green_tree,
    );
    let column_result = collect_completion_proposals(CompletionRequest {
        source: &column_source,
        green_tree: &column_parse.green_tree,
        red_projection: &column_red,
        bind_context: &bind_context,
        library_context_snapshot: None,
        cursor_offset: column_source.entered_formula_text.chars().count(),
    });

    assert!(
        column_result
            .proposals
            .iter()
            .any(
                |proposal| proposal.proposal_kind == CompletionProposalKind::TableColumn
                    && proposal.display_text == "Amount"
            )
    );
}

#[test]
fn completion_proposals_include_r1c1_syntax_assists_in_r1c1_channel() {
    let source = FormulaSourceRecord::new("editor-r1c1-complete", 1, "=R")
        .with_formula_channel_kind(FormulaChannelKind::WorksheetR1C1);
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind_context = editor_bind_context(source.clone());

    let result = collect_completion_proposals(CompletionRequest {
        source: &source,
        green_tree: &parse.green_tree,
        red_projection: &red,
        bind_context: &bind_context,
        library_context_snapshot: None,
        cursor_offset: source.entered_formula_text.chars().count(),
    });

    assert!(
        result
            .proposals
            .iter()
            .any(
                |proposal| proposal.proposal_kind == CompletionProposalKind::SyntaxAssist
                    && proposal.display_text == "R"
            )
    );
    assert!(
        result
            .proposals
            .iter()
            .any(
                |proposal| proposal.proposal_kind == CompletionProposalKind::SyntaxAssist
                    && proposal.display_text == "RC"
            )
    );
}

#[test]
fn signature_help_context_tracks_active_argument_index() {
    let source = FormulaSourceRecord::new("editor-signature", 1, "=SUM(1,2,3)");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });

    let signature = signature_help_context_at_cursor(&source, &parse.green_tree, 9)
        .expect("cursor should be inside SUM argument list");

    assert_eq!(signature.callee_text, "SUM");
    assert_eq!(signature.active_argument_index, 2);
}

#[test]
fn function_help_lookup_request_tracks_active_callee_and_snapshot() {
    let source = FormulaSourceRecord::new("editor-help", 1, "=SUM(1,2,3)");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });

    let request = build_function_help_lookup_request(
        &source,
        &parse.green_tree,
        9,
        Some(&sample_library_context_snapshot()),
    )
    .expect("cursor should resolve to a call-site help request");

    assert_eq!(request.lookup_key, "SUM");
    assert_eq!(
        request.library_context_snapshot_ref.as_deref(),
        Some("editor-snapshot@v1")
    );
}

#[test]
fn intelligent_completion_context_carries_scope_and_active_diagnostics() {
    let source = FormulaSourceRecord::new("editor-intel", 1, "=[@Amount]");
    let bind_context = editor_bind_context(source.clone());
    let result = apply_formula_edit(FormulaEditRequest {
        source: source.clone(),
        bind_context: bind_context.clone(),
        previous_green_tree: None,
        previous_red_projection: None,
        previous_bound_formula: None,
        follow_on_stage: EditFollowOnStage::ParseBindAndPlan,
        semantic_plan_options: Some(SemanticPlanEditOptions {
            oxfunc_catalog_identity: "editor-catalog".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: Some(sample_library_context_snapshot()),
        }),
    });

    let context = build_intelligent_completion_context(
        CompletionRequest {
            source: &source,
            green_tree: &result.green_tree,
            red_projection: &result.red_projection,
            bind_context: &bind_context,
            library_context_snapshot: Some(&sample_library_context_snapshot()),
            cursor_offset: 5,
        },
        &result.live_diagnostics,
    );

    assert!(context.red_context_summary.starts_with("kind="));
    assert!(context.visible_tables.iter().any(|table| table == "Table1"));
    assert!(!context.active_diagnostics.is_empty());
    assert_eq!(
        context.library_context_snapshot_ref.as_deref(),
        Some("editor-snapshot@v1")
    );
}

#[test]
fn validate_completion_candidate_reenters_normal_edit_pipeline() {
    let source = FormulaSourceRecord::new("editor-validate", 3, "=SU");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);

    let result = validate_completion_candidate(CompletionValidationRequest {
        source: source.clone(),
        bind_context: editor_bind_context(source),
        previous_green_tree: Some(&parse.green_tree),
        previous_red_projection: Some(&red),
        previous_bound_formula: None,
        replacement_span: Some(oxfml_core::syntax::token::TextSpan::new(1, 2)),
        insert_text: "SUM".to_string(),
        follow_on_stage: EditFollowOnStage::ParseAndBind,
        semantic_plan_options: None,
    });

    assert_eq!(result.updated_source.entered_formula_text, "=SUM");
    assert_eq!(result.updated_source.formula_text_version.0, 4);
    assert_eq!(
        result.edit_result.text_change_range,
        Some(oxfml_core::FormulaTextChangeRange {
            start: 3,
            old_len: 0,
            new_len: 1,
        })
    );
}

#[test]
fn apply_completion_proposal_preserves_proposal_identity() {
    let source = FormulaSourceRecord::new("editor-proposal", 5, "=SU");
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind_context = editor_bind_context(source.clone());

    let completion = collect_completion_proposals(CompletionRequest {
        source: &source,
        green_tree: &parse.green_tree,
        red_projection: &red,
        bind_context: &bind_context,
        library_context_snapshot: Some(&sample_library_context_snapshot()),
        cursor_offset: source.entered_formula_text.chars().count(),
    });
    let sum = completion
        .proposals
        .iter()
        .find(|proposal| proposal.display_text == "SUM")
        .expect("SUM proposal should exist");

    let result = apply_completion_proposal(
        CompletionValidationRequest {
            source: source.clone(),
            bind_context,
            previous_green_tree: Some(&parse.green_tree),
            previous_red_projection: Some(&red),
            previous_bound_formula: None,
            replacement_span: None,
            insert_text: String::new(),
            follow_on_stage: EditFollowOnStage::ParseAndBind,
            semantic_plan_options: None,
        },
        sum,
    );

    assert_eq!(result.proposal_id.as_deref(), Some("function:SUM"));
    assert_eq!(result.updated_source.entered_formula_text, "=SUM");
}

fn editor_bind_context(source: FormulaSourceRecord) -> BindContext {
    BindContext {
        workbook_id: "book:editor".to_string(),
        sheet_id: "sheet:editor".to_string(),
        caller_row: 1,
        caller_col: 1,
        formula_token: source.formula_token(),
        structure_context_version: StructureContextVersion("editor-struct-v1".to_string()),
        table_catalog: vec![sample_table()],
        ..BindContext::default()
    }
}

fn sample_table() -> TableDescriptor {
    TableDescriptor {
        table_id: "table:1".to_string(),
        table_name: "Table1".to_string(),
        workbook_scope_ref: "book:editor".to_string(),
        sheet_scope_ref: "sheet:editor".to_string(),
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

fn sample_library_context_snapshot() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "editor-snapshot".to_string(),
        snapshot_version: "v1".to_string(),
        entries: vec![
            LibraryContextSnapshotEntry {
                surface_name: "SUM".to_string(),
                canonical_id: Some("FUNC.SUM".to_string()),
                surface_stable_id: Some("surface:sum".to_string()),
                name_resolution_table_ref: None,
                semantic_trait_profile_ref: None,
                gating_profile_ref: None,
                metadata_status: None,
                special_interface_kind: None,
                admission_interface_kind: None,
                preparation_owner: None,
                runtime_boundary_kind: None,
                arity_shape_note: None,
                interface_contract_ref: Some("contract:sum".to_string()),
                registration_source_kind: RegistrationSourceKind::BuiltIn,
                parse_bind_state: LibraryAvailabilityState::CatalogKnown,
                semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
                runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
                post_dispatch_state: None,
            },
            LibraryContextSnapshotEntry {
                surface_name: "SUBSTITUTE".to_string(),
                canonical_id: Some("FUNC.SUBSTITUTE".to_string()),
                surface_stable_id: Some("surface:substitute".to_string()),
                name_resolution_table_ref: None,
                semantic_trait_profile_ref: None,
                gating_profile_ref: None,
                metadata_status: None,
                special_interface_kind: None,
                admission_interface_kind: None,
                preparation_owner: None,
                runtime_boundary_kind: None,
                arity_shape_note: None,
                interface_contract_ref: Some("contract:substitute".to_string()),
                registration_source_kind: RegistrationSourceKind::BuiltIn,
                parse_bind_state: LibraryAvailabilityState::CatalogKnown,
                semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
                runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
                post_dispatch_state: None,
            },
        ],
    }
}

fn collect_green_tokens(node: &GreenNode) -> Vec<oxfml_core::syntax::token::Token> {
    let mut tokens = Vec::new();
    collect_green_tokens_recursive(node, &mut tokens);
    tokens
}

fn collect_green_tokens_recursive(
    node: &GreenNode,
    tokens: &mut Vec<oxfml_core::syntax::token::Token>,
) {
    for child in &node.children {
        match child {
            GreenChild::Node(child_node) => collect_green_tokens_recursive(child_node, tokens),
            GreenChild::Token(token) => tokens.push(token.clone()),
        }
    }
}
