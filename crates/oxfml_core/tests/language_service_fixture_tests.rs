use std::fs;
use std::path::PathBuf;

use oxfml_core::binding::BindContext;
use oxfml_core::interface::{
    InMemoryLibraryContextProvider, LibraryContextSnapshotRef, TableColumnDescriptor,
    TableDescriptor,
};
use oxfml_core::language_service::{
    CompletionProposalKind, CompletionRequest, CompletionValidationRequest, EditFollowOnStage,
    apply_completion_proposal, build_editor_syntax_snapshot, collect_completion_proposals,
};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "case_kind", rename_all = "snake_case")]
enum LanguageServiceCase {
    TriviaSnapshot {
        case_id: String,
        formula: String,
        expected_token_text: String,
        expected_leading_trivia: String,
        expected_trailing_trivia: String,
    },
    CompletionProposal {
        case_id: String,
        formula: String,
        formula_channel_kind: String,
        cursor_offset: usize,
        expected_kind: String,
        expected_display_text: String,
    },
    CompletionApplication {
        case_id: String,
        formula: String,
        cursor_offset: usize,
        proposal_id: String,
        expected_updated_formula: String,
        expected_change_start: usize,
        expected_change_old_len: usize,
        expected_change_new_len: usize,
    },
}

#[test]
fn language_service_fixtures_match_expected_snapshots() {
    let cases = load_fixture();

    for case in cases {
        match case {
            LanguageServiceCase::TriviaSnapshot {
                case_id,
                formula,
                expected_token_text,
                expected_leading_trivia,
                expected_trailing_trivia,
            } => {
                let source = FormulaSourceRecord::new(case_id, 1, formula);
                let parse = parse_formula(ParseRequest {
                    source: source.clone(),
                });
                let snapshot = build_editor_syntax_snapshot(&source, &parse.green_tree);
                let token = snapshot
                    .tokens
                    .iter()
                    .find(|token| token.text == expected_token_text)
                    .expect("expected token should exist in snapshot");

                assert_eq!(
                    token
                        .leading_trivia
                        .iter()
                        .map(|trivia| trivia.text.as_str())
                        .collect::<String>(),
                    expected_leading_trivia
                );
                assert_eq!(
                    token
                        .trailing_trivia
                        .iter()
                        .map(|trivia| trivia.text.as_str())
                        .collect::<String>(),
                    expected_trailing_trivia
                );
            }
            LanguageServiceCase::CompletionProposal {
                case_id,
                formula,
                formula_channel_kind,
                cursor_offset,
                expected_kind,
                expected_display_text,
            } => {
                let source = FormulaSourceRecord::new(case_id, 1, formula)
                    .with_formula_channel_kind(parse_channel_kind(&formula_channel_kind));
                let parse = parse_formula(ParseRequest {
                    source: source.clone(),
                });
                let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
                let bind_context = editor_bind_context(source.clone());
                let snapshot = sample_library_context_snapshot();
                let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
                let provider = InMemoryLibraryContextProvider::new(snapshot);

                let result = collect_completion_proposals(CompletionRequest {
                    source: &source,
                    green_tree: &parse.green_tree,
                    red_projection: &red,
                    bind_context: &bind_context,
                    library_context_provider: Some(&provider),
                    library_context_snapshot_ref: Some(&snapshot_ref),
                    library_context_snapshot: None,
                    cursor_offset,
                });

                assert!(result.proposals.iter().any(|proposal| {
                    proposal.display_text == expected_display_text
                        && completion_kind_name(proposal.proposal_kind) == expected_kind
                }));
            }
            LanguageServiceCase::CompletionApplication {
                case_id,
                formula,
                cursor_offset,
                proposal_id,
                expected_updated_formula,
                expected_change_start,
                expected_change_old_len,
                expected_change_new_len,
            } => {
                let source = FormulaSourceRecord::new(case_id, 1, formula);
                let parse = parse_formula(ParseRequest {
                    source: source.clone(),
                });
                let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
                let bind_context = editor_bind_context(source.clone());
                let snapshot = sample_library_context_snapshot();
                let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
                let provider = InMemoryLibraryContextProvider::new(snapshot);

                let completion = collect_completion_proposals(CompletionRequest {
                    source: &source,
                    green_tree: &parse.green_tree,
                    red_projection: &red,
                    bind_context: &bind_context,
                    library_context_provider: Some(&provider),
                    library_context_snapshot_ref: Some(&snapshot_ref),
                    library_context_snapshot: None,
                    cursor_offset,
                });
                let proposal = completion
                    .proposals
                    .iter()
                    .find(|proposal| proposal.proposal_id == proposal_id)
                    .expect("expected proposal should exist");

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
                    proposal,
                );

                assert_eq!(
                    result.updated_source.entered_formula_text,
                    expected_updated_formula
                );
                let change = result
                    .edit_result
                    .text_change_range
                    .expect("completion application should report a change range");
                assert_eq!(change.start, expected_change_start);
                assert_eq!(change.old_len, expected_change_old_len);
                assert_eq!(change.new_len, expected_change_new_len);
            }
        }
    }
}

fn fixture_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("language_service_cases.json");
    path
}

fn load_fixture() -> Vec<LanguageServiceCase> {
    let content = fs::read_to_string(fixture_path()).expect("fixture file should be readable");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn parse_channel_kind(raw: &str) -> FormulaChannelKind {
    match raw {
        "WorksheetA1" => FormulaChannelKind::WorksheetA1,
        "WorksheetR1C1" => FormulaChannelKind::WorksheetR1C1,
        "ConditionalFormatting" => FormulaChannelKind::ConditionalFormatting,
        "DataValidation" => FormulaChannelKind::DataValidation,
        other => panic!("unexpected formula channel kind fixture value: {other}"),
    }
}

fn completion_kind_name(kind: CompletionProposalKind) -> &'static str {
    match kind {
        CompletionProposalKind::Function => "Function",
        CompletionProposalKind::DefinedName => "DefinedName",
        CompletionProposalKind::TableName => "TableName",
        CompletionProposalKind::TableColumn => "TableColumn",
        CompletionProposalKind::StructuredSelector => "StructuredSelector",
        CompletionProposalKind::SyntaxAssist => "SyntaxAssist",
    }
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
