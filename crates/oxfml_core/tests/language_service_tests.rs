use oxfml_core::binding::{BindContext, BoundExpr, NameKind};
use oxfml_core::consumer::editor::{
    CompletionProposalKind, EditorAnalysisStage, EditorEditService, EditorEnvironment,
    EditorPlanOptions, EditorSyntaxSnapshot, FormulaTextChangeRange, LiveDiagnosticStage,
};
use oxfml_core::interface::{
    InMemoryLibraryContextProvider, LibraryContextProvider, LibraryContextSnapshotRef,
    TableColumnDescriptor, TableDescriptor,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::green::{GreenChild, GreenNode};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::syntax::token::{TextSpan, TokenKind};
use oxfunc_core::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use oxfunc_core::registry::{
    CapabilityOverlay, FunctionEntry, FunctionRegistryMetadata, FunctionSource,
    ParameterDescriptor, RegistryFunctionMeta, SignatureForm, builtin_registry,
};

#[test]
fn editor_syntax_snapshot_tracks_leading_and_trailing_trivia() {
    let source = FormulaSourceRecord::new("editor-trivia", 1, "=SUM( A1 ) ");
    let service = EditorEditService::new(EditorEnvironment::new(BindContext::default()));

    let document = service.open_document(source, None);

    assert_eq!(document.editor_syntax_snapshot.tokens.len(), 5);
    assert_eq!(document.editor_syntax_snapshot.tokens[3].text, "A1");
    assert_eq!(
        document.editor_syntax_snapshot.tokens[3]
            .leading_trivia
            .len(),
        1
    );
    assert_eq!(
        document.editor_syntax_snapshot.tokens[3].leading_trivia[0].text,
        " "
    );
    assert_eq!(document.editor_syntax_snapshot.tokens[4].text, ")");
    assert_eq!(
        document.editor_syntax_snapshot.tokens[4]
            .leading_trivia
            .len(),
        1
    );
    assert_eq!(
        document.editor_syntax_snapshot.tokens[4].leading_trivia[0].text,
        " "
    );
    assert_eq!(
        document.editor_syntax_snapshot.tokens[4]
            .trailing_trivia
            .len(),
        1
    );
    assert_eq!(
        document.editor_syntax_snapshot.tokens[4].trailing_trivia[0].text,
        " "
    );
}

#[test]
fn editor_syntax_snapshot_preserves_tokens_after_leading_whitespace() {
    let cases = [
        ("leading-newline", "\n=aaa", "\n", vec!["=", "aaa"]),
        (
            "leading-multiple-newlines",
            "\n\n=aaa",
            "\n\n",
            vec!["=", "aaa"],
        ),
        ("leading-space", " =aaa", " ", vec!["=", "aaa"]),
        ("leading-tab", "\t=aaa", "\t", vec!["=", "aaa"]),
        (
            "leading-newline-call",
            "\n=SUM(1,2)",
            "\n",
            vec!["=", "SUM", "(", "1", ",", "2", ")"],
        ),
    ];

    let service = EditorEditService::new(EditorEnvironment::new(BindContext::default()));

    for (case_id, text, expected_leading_trivia, expected_texts) in cases {
        let source = FormulaSourceRecord::new(case_id, 1, text)
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);

        let result = service.apply_edit(source, None, EditorAnalysisStage::SyntaxOnly, None);
        let tokens = &result.document.editor_syntax_snapshot.tokens;
        let actual_texts = tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>();

        assert_eq!(actual_texts, expected_texts, "case {case_id}");
        assert_eq!(tokens[0].kind, TokenKind::Equals, "case {case_id}");
        assert_eq!(
            tokens[0].span.start,
            expected_leading_trivia.len(),
            "case {case_id}"
        );
        assert_eq!(
            tokens[0].span.end(),
            expected_leading_trivia.len() + 1,
            "case {case_id}"
        );
        assert_eq!(
            tokens[0]
                .leading_trivia
                .iter()
                .map(|trivia| trivia.text.as_str())
                .collect::<String>(),
            expected_leading_trivia,
            "case {case_id}"
        );
    }
}

#[test]
fn editor_syntax_snapshot_preserves_unexpected_trailing_tokens() {
    let cases = [
        ("extra-close-after-number", "=1)", vec!["=", "1", ")"]),
        (
            "extra-close-after-call",
            "=SUM(1,2))",
            vec!["=", "SUM", "(", "1", ",", "2", ")", ")"],
        ),
        (
            "trailing-operator-tail",
            "=1 + 2)",
            vec!["=", "1", "+", "2", ")"],
        ),
    ];

    let service = EditorEditService::new(EditorEnvironment::new(BindContext::default()));

    for (case_id, text, expected_texts) in cases {
        let source = FormulaSourceRecord::new(case_id, 1, text)
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);

        let result = service.apply_edit(source, None, EditorAnalysisStage::SyntaxOnly, None);
        let tokens = &result.document.editor_syntax_snapshot.tokens;
        let actual_texts = tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>();

        assert_eq!(actual_texts, expected_texts, "case {case_id}");
        assert!(
            result
                .document
                .live_diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("unexpected trailing token")),
            "case {case_id}"
        );
    }
}

#[test]
fn editor_syntax_snapshot_preserves_inter_reference_whitespace() {
    let cases = [
        ("inter-identifiers", "= a a", vec!["=", "a", "a"]),
        ("adjacent-identifiers", "=A A", vec!["=", "A", "A"]),
        (
            "identifier-chain",
            "= a b c d",
            vec!["=", "a", "b", "c", "d"],
        ),
        ("cell-intersection", "=A1 B1", vec!["=", "A1", "B1"]),
        (
            "multi-letter-identifiers",
            "= ABC DEF",
            vec!["=", "ABC", "DEF"],
        ),
        (
            "multi-space-intersection",
            "=A1   B1",
            vec!["=", "A1", "B1"],
        ),
        ("tab-intersection", "=a\tb", vec!["=", "a", "b"]),
    ];

    let service = EditorEditService::new(EditorEnvironment::new(BindContext::default()));

    for (case_id, text, expected_texts) in cases {
        let source = FormulaSourceRecord::new(case_id, 1, text)
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);

        let result = service.apply_edit(source, None, EditorAnalysisStage::SyntaxOnly, None);
        let snapshot = &result.document.editor_syntax_snapshot;
        let actual_texts = snapshot
            .tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>();

        assert_eq!(actual_texts, expected_texts, "case {case_id}");
        assert_eq!(snapshot_source_text(snapshot), text, "case {case_id}");
    }
}

#[test]
fn editor_syntax_snapshot_tiles_source_text_for_broad_editor_inputs() {
    let cases = [
        "",
        " ",
        "\t",
        "\n",
        "\n=aaa",
        "= a a",
        "=A A",
        "=SUM( A1 ) ",
        "=A1   B1",
        "=a\tb",
        "=1)",
        "=SUM(1,2))",
        "=",
        "=(",
        "=A1+",
        "=@A1#",
        "=\"a b\"",
        "=#VALUE!",
        "=A1&\" text \"",
        "=1 @ 2",
    ];

    let service = EditorEditService::new(EditorEnvironment::new(BindContext::default()));

    for text in cases {
        let source = FormulaSourceRecord::new(format!("tile-{text:?}"), 1, text)
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);

        let result = service.apply_edit(source, None, EditorAnalysisStage::SyntaxOnly, None);

        assert_eq!(
            snapshot_source_text(&result.document.editor_syntax_snapshot),
            text,
            "source tiling mismatch for {text:?}"
        );
    }
}

#[test]
fn editor_treats_non_formula_cell_entries_as_literals() {
    let cases = [
        ("ABC", "text"),
        ("'=123", "text"),
        ("12.1.1", "text"),
        ("x y z = 12.3", "text"),
        ("\"ABC\"", "text"),
        ("123.4", "number"),
        ("TRUE", "logical"),
        ("FALSE", "logical"),
    ];

    let service = EditorEditService::new(EditorEnvironment::new(BindContext::default()));

    for (text, expected_kind) in cases {
        let source = FormulaSourceRecord::new(format!("entry-{text:?}"), 1, text)
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);

        let result = service.apply_edit(source, None, EditorAnalysisStage::SyntaxAndBind, None);

        assert_eq!(
            snapshot_source_text(&result.document.editor_syntax_snapshot),
            text,
            "source tiling mismatch for {text:?}"
        );
        assert!(
            result.document.live_diagnostics.diagnostics.is_empty(),
            "literal cell entry should not produce diagnostics for {text:?}: {:?}",
            result.document.live_diagnostics.diagnostics
        );

        let bound = result
            .document
            .bound_formula
            .as_ref()
            .expect("syntax-and-bind should produce a bound formula");
        match (expected_kind, &bound.root) {
            ("text", BoundExpr::StringLiteral(_)) => {}
            ("number", BoundExpr::NumberLiteral(_)) => {}
            ("logical", BoundExpr::LogicalLiteral(_)) => {}
            _ => panic!(
                "unexpected bound literal kind for {text:?}: {:?}",
                bound.root
            ),
        }
    }
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
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let first = service.apply_edit(
        source.clone(),
        None,
        EditorAnalysisStage::SyntaxAndBind,
        None,
    );
    let second = service.apply_edit(
        source,
        Some(&first.document),
        EditorAnalysisStage::SyntaxAndBind,
        None,
    );

    assert!(second.document.reuse_summary.reused_green_tree);
    assert!(second.document.reuse_summary.reused_red_projection);
    assert!(second.document.reuse_summary.reused_bound_formula);
    assert!(second.document.bound_formula.is_some());
    assert_eq!(second.document.text_change_range, None);
}

#[test]
fn apply_formula_edit_reports_smallest_text_change_range() {
    let previous_source = FormulaSourceRecord::new("editor-change-range", 1, "=SUM(A1)");
    let service = EditorEditService::new(EditorEnvironment::new(editor_bind_context(
        previous_source.clone(),
    )));
    let previous = service.apply_edit(previous_source, None, EditorAnalysisStage::SyntaxOnly, None);

    let updated_source = FormulaSourceRecord::new("editor-change-range", 2, "=SUM(B1)");
    let result = service.apply_edit(
        updated_source,
        Some(&previous.document),
        EditorAnalysisStage::SyntaxOnly,
        None,
    );

    assert_eq!(
        result.document.text_change_range,
        Some(FormulaTextChangeRange {
            start: 5,
            old_len: 1,
            new_len: 1,
        })
    );
}

#[test]
fn apply_formula_edit_surfaces_bind_diagnostics_in_live_snapshot() {
    let source = FormulaSourceRecord::new("editor-bind-diag", 1, "=[@Amount]");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let result = service.apply_edit(source, None, EditorAnalysisStage::SyntaxAndBind, None);

    assert!(
        result
            .document
            .live_diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_deref()
                == Some("structured_reference_unresolved")
                && diagnostic.suggested_fix_kind.as_deref()
                    == Some("supply_enclosing_table_context"))
    );
}

#[test]
fn live_diagnostics_use_exact_symbol_spans_for_unknown_function_and_name() {
    let source = FormulaSourceRecord::new("editor-symbol-diag", 1, "=YYYY(1,2)+ABS(-12)+QQQQ");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let result = service.apply_edit(source, None, EditorAnalysisStage::FullSemanticPlan, None);
    let diagnostics = &result.document.live_diagnostics.diagnostics;

    let unknown_function = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("unknown_function"))
        .expect("unknown function diagnostic");
    assert_eq!(unknown_function.stage, LiveDiagnosticStage::SemanticPlan);
    assert_eq!(unknown_function.primary_span, TextSpan::new(1, 4));
    assert_eq!(
        unknown_function.worksheet_error_class.as_deref(),
        Some("#NAME?")
    );

    let unknown_name = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("unknown_name"))
        .expect("unknown name diagnostic");
    assert_eq!(unknown_name.stage, LiveDiagnosticStage::Bind);
    assert_eq!(unknown_name.primary_span, TextSpan::new(20, 4));
    assert_eq!(
        unknown_name.worksheet_error_class.as_deref(),
        Some("#NAME?")
    );

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("ABS")),
        "ABS should not produce a diagnostic in the mixed formula: {diagnostics:?}"
    );
}

#[test]
fn live_diagnostics_classify_arity_and_noncallable_symbols() {
    let source = FormulaSourceRecord::new("editor-arity-diag", 1, "=ABS(1,2)+RefName(1)");
    let mut bind_context = editor_bind_context(source.clone());
    bind_context
        .names
        .insert("RefName".to_string(), NameKind::ReferenceLike);
    let service = EditorEditService::new(EditorEnvironment::new(bind_context));

    let result = service.apply_edit(source, None, EditorAnalysisStage::FullSemanticPlan, None);
    let diagnostics = &result.document.live_diagnostics.diagnostics;

    let arity = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("function_arity_mismatch"))
        .expect("arity diagnostic");
    assert_eq!(arity.stage, LiveDiagnosticStage::Bind);
    assert_eq!(arity.primary_span, TextSpan::new(1, 3));

    let noncallable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("known_symbol_not_callable"))
        .expect("noncallable diagnostic");
    assert_eq!(noncallable.stage, LiveDiagnosticStage::Bind);
    assert_eq!(noncallable.primary_span, TextSpan::new(10, 7));
    assert_eq!(noncallable.worksheet_error_class.as_deref(), Some("#NAME?"));
}

#[test]
fn live_diagnostics_classify_gated_function_surfaces() {
    let source = FormulaSourceRecord::new("editor-gated-diag", 1, "=YYYY(1)");
    let mut snapshot = sample_library_context_snapshot();
    snapshot.entries.push(LibraryContextSnapshotEntry {
        surface_name: "YYYY".to_string(),
        canonical_id: Some("FUNC.YYYY".to_string()),
        surface_stable_id: Some("surface:yyyy".to_string()),
        name_resolution_table_ref: None,
        semantic_trait_profile_ref: None,
        gating_profile_ref: Some("gate:fixture".to_string()),
        metadata_status: Some("fixture-gated".to_string()),
        special_interface_kind: None,
        admission_interface_kind: None,
        preparation_owner: None,
        runtime_boundary_kind: None,
        interface_contract_ref: Some("contract:yyyy".to_string()),
        registration_source_kind: RegistrationSourceKind::BuiltIn,
        parse_bind_state: LibraryAvailabilityState::FeatureGated,
        semantic_plan_state: LibraryAvailabilityState::FeatureGated,
        runtime_capability_state: Some(LibraryAvailabilityState::FeatureGated),
        post_dispatch_state: None,
    });
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let result = service.apply_edit(
        source,
        None,
        EditorAnalysisStage::FullSemanticPlan,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "editor-catalog".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: Some(snapshot),
        }),
    );

    let gated = result
        .document
        .live_diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("function_gated_or_unavailable"))
        .expect("gated function diagnostic");
    assert_eq!(gated.primary_span, TextSpan::new(1, 4));
}

#[test]
fn completion_proposals_include_functions_defined_names_and_table_names() {
    let source = FormulaSourceRecord::new("editor-complete", 1, "=SU");
    let mut bind_context = editor_bind_context(source.clone());
    bind_context
        .names
        .insert("SummaryName".to_string(), NameKind::ValueLike);
    let service = EditorEditService::new(EditorEnvironment::new(bind_context));

    let document = service.open_document(source.clone(), None);
    let result =
        service.completion_at_cursor(&document, source.entered_formula_text.chars().count());

    for expected_function in ["SUM", "SUMIF", "SUMIFS", "SUMPRODUCT", "SUBSTITUTE"] {
        assert!(
            result.proposals.iter().any(|proposal| {
                proposal.proposal_kind == CompletionProposalKind::Function
                    && proposal.display_text == expected_function
            }),
            "default registry completion proposals should include {expected_function}"
        );
    }
    assert!(result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::DefinedName
            && proposal.display_text == "SummaryName"
    }));
}

#[test]
fn completion_proposals_use_registry_not_library_context_snapshot() {
    let source = FormulaSourceRecord::new("editor-registry-complete", 1, "=SUMI");
    let bind_context = editor_bind_context(source.clone());
    let pinned_snapshot = sample_library_context_snapshot();
    let pinned_snapshot_ref = LibraryContextSnapshotRef::from(&pinned_snapshot);
    let provider = InMemoryLibraryContextProvider::new(pinned_snapshot);
    let service = EditorEditService::new(
        EditorEnvironment::new(bind_context)
            .with_pinned_library_context(&provider, pinned_snapshot_ref),
    );

    let document = service.open_document(source.clone(), None);
    let result =
        service.completion_at_cursor(&document, source.entered_formula_text.chars().count());

    assert!(
        result.proposals.iter().any(|proposal| {
            proposal.proposal_kind == CompletionProposalKind::Function
                && proposal.display_text == "SUMIF"
        }),
        "function proposals must come from the OxFunc registry even when the pinned snapshot omits the function"
    );
}

#[test]
fn completion_proposals_reflect_udf_registry_mutation() {
    let source = FormulaSourceRecord::new("editor-complete-udf", 1, "=MY");
    let mut registry = builtin_registry().clone();
    registry
        .register_udf(test_udf_entry())
        .expect("UDF registration should be accepted by OxFunc registry");
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_function_registry(&registry),
    );

    let document = service.open_document(source.clone(), None);
    let result =
        service.completion_at_cursor(&document, source.entered_formula_text.chars().count());

    assert!(result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::Function
            && proposal.display_text == "MYFUNC"
    }));
}

#[test]
fn completion_proposals_filter_capability_denied_registry_entries() {
    let source = FormulaSourceRecord::new("editor-complete-capability", 1, "=R");
    let mut overlay = CapabilityOverlay::new();
    overlay.deny_function_id("FUNC.RTD", "provider unavailable");
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_capability_overlay(&overlay),
    );

    let document = service.open_document(source.clone(), None);
    let result =
        service.completion_at_cursor(&document, source.entered_formula_text.chars().count());

    assert!(!result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::Function && proposal.display_text == "RTD"
    }));
}

#[test]
fn completion_proposals_include_structured_selectors_and_columns() {
    let bind_context = editor_bind_context(FormulaSourceRecord::new("editor-structured", 1, "=1"));
    let service = EditorEditService::new(EditorEnvironment::new(bind_context));

    let selector_source = FormulaSourceRecord::new("editor-structured", 1, "=Table1[#H");
    let selector_document = service.open_document(selector_source.clone(), None);
    let selector_result = service.completion_at_cursor(
        &selector_document,
        selector_source.entered_formula_text.chars().count(),
    );
    assert!(selector_result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::StructuredSelector
            && proposal.display_text == "#Headers"
    }));

    let column_source = FormulaSourceRecord::new("editor-column", 1, "=Table1[A");
    let column_document = service.open_document(column_source.clone(), None);
    let column_result = service.completion_at_cursor(
        &column_document,
        column_source.entered_formula_text.chars().count(),
    );
    assert!(column_result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::TableColumn
            && proposal.display_text == "Amount"
    }));
}

#[test]
fn completion_proposals_include_r1c1_syntax_assists_in_r1c1_channel() {
    let source = FormulaSourceRecord::new("editor-r1c1-complete", 1, "=R")
        .with_formula_channel_kind(FormulaChannelKind::WorksheetR1C1);
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let document = service.open_document(source.clone(), None);
    let result =
        service.completion_at_cursor(&document, source.entered_formula_text.chars().count());

    assert!(result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::SyntaxAssist
            && proposal.display_text == "R"
    }));
    assert!(result.proposals.iter().any(|proposal| {
        proposal.proposal_kind == CompletionProposalKind::SyntaxAssist
            && proposal.display_text == "RC"
    }));
}

#[test]
fn signature_help_context_tracks_active_argument_index() {
    let source = FormulaSourceRecord::new("editor-signature", 1, "=SUM(1,2,3)");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let document = service.open_document(source, None);
    let signature = service
        .signature_help_at_cursor(&document, 9)
        .expect("cursor should be inside SUM argument list");

    assert_eq!(signature.callee_text, "SUM");
    assert_eq!(signature.active_argument_index, 2);
}

#[test]
fn signature_help_context_is_absent_after_closed_call_close_paren() {
    let source = FormulaSourceRecord::new("editor-signature-closed-end", 1, "=SUM(1,2,3)");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let document = service.open_document(source, None);

    assert!(service.signature_help_at_cursor(&document, 11).is_none());
    assert!(service.signature_help_at_cursor(&document, 12).is_none());
}

#[test]
fn signature_help_context_still_shows_before_closed_call_close_paren() {
    let source = FormulaSourceRecord::new("editor-signature-before-close", 1, "=SUM(1,2,3)");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let document = service.open_document(source, None);
    let signature = service
        .signature_help_at_cursor(&document, 10)
        .expect("cursor immediately before close paren should still be inside the call");

    assert_eq!(signature.callee_text, "SUM");
    assert_eq!(signature.active_argument_index, 2);
}

#[test]
fn signature_help_context_still_shows_for_unclosed_call() {
    let source = FormulaSourceRecord::new("editor-signature-unclosed", 1, "=SUM(1,2,3");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let document = service.open_document(source, None);
    let signature = service
        .signature_help_at_cursor(&document, 10)
        .expect("unclosed calls should keep signature help active at the caret");

    assert_eq!(signature.callee_text, "SUM");
    assert_eq!(signature.active_argument_index, 2);
}

#[test]
fn function_help_packet_tracks_active_callee_and_snapshot() {
    let source = FormulaSourceRecord::new("editor-help", 1, "=SUM(1,2,3)");
    let snapshot = sample_library_context_snapshot();
    let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
    let provider = InMemoryLibraryContextProvider::new(snapshot);
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_pinned_library_context(&provider, snapshot_ref),
    );

    let interaction = service.open_and_interact(source, 9, None);
    let packet = interaction
        .function_help_packet
        .expect("cursor should resolve to a call-site help packet");

    assert_eq!(packet.lookup_key, "SUM");
    assert_eq!(
        packet.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::new("editor-snapshot", "v1"))
    );
}

#[test]
fn function_help_packet_uses_provider_current_snapshot_when_unpinned() {
    let source = FormulaSourceRecord::new("editor-help-current", 1, "=SUM(1,2,3)");
    let snapshot = sample_library_context_snapshot_v2();
    let provider = InMemoryLibraryContextProvider::new(snapshot);
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_library_context_provider(&provider),
    );

    let interaction = service.open_and_interact(source, 9, None);
    let packet = interaction
        .function_help_packet
        .expect("cursor should resolve to a call-site help packet");

    assert_eq!(
        packet.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::new("editor-snapshot", "v2"))
    );
}

#[test]
fn function_help_packet_uses_oxfunc_registry_signatures() {
    let service = EditorEditService::new(EditorEnvironment::new(editor_bind_context(
        FormulaSourceRecord::new("editor-registry-now", 1, "=NOW()"),
    )));

    let now_source = FormulaSourceRecord::new("editor-registry-now", 1, "=NOW()");
    let now_packet = service
        .open_and_interact(now_source, 5, None)
        .function_help_packet
        .expect("NOW should resolve through OxFunc registry");

    assert_eq!(now_packet.display_name, "NOW");
    assert_eq!(now_packet.signature_forms[0].display_signature, "NOW()");
    assert_eq!(now_packet.signature_forms[0].min_arity, 0);
    assert_eq!(now_packet.signature_forms[0].max_arity, Some(0));
    assert!(now_packet.argument_help.is_empty());

    let if_source = FormulaSourceRecord::new("editor-registry-if", 1, "=IF(TRUE,1,2)");
    let if_packet = service
        .open_and_interact(if_source, 10, None)
        .function_help_packet
        .expect("IF should resolve through OxFunc registry");

    assert_eq!(
        if_packet.signature_forms[0].display_signature,
        "IF(logical_test, value_if_true, [value_if_false])"
    );
    assert_eq!(
        if_packet.argument_help,
        vec!["logical_test", "*value_if_true", "value_if_false"]
    );

    let sum_source = FormulaSourceRecord::new("editor-registry-sum", 1, "=SUM(1,2)");
    let sum_packet = service
        .open_and_interact(sum_source, 7, None)
        .function_help_packet
        .expect("SUM should resolve through OxFunc registry");

    assert_eq!(
        sum_packet.signature_forms[0].display_signature,
        "SUM(number1, [number2], ...)"
    );
    assert_eq!(sum_packet.argument_help, vec!["number1", "*number2..."]);
}

#[test]
fn function_help_packet_is_absent_for_unknown_callee() {
    let source = FormulaSourceRecord::new("editor-help-unknown", 1, "=ZZZNOTAFUNCTION(");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));

    let interaction = service.open_and_interact(source, 16, None);

    assert!(interaction.function_help_packet.is_none());
}

#[test]
fn function_help_packet_preserves_registry_signature_under_capability_overlay() {
    let source =
        FormulaSourceRecord::new("editor-help-capability", 1, "=RTD(\"p\",\"s\",\"topic\")");
    let mut overlay = CapabilityOverlay::new();
    overlay.deny_function_id("FUNC.RTD", "provider unavailable");
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_capability_overlay(&overlay),
    );

    let packet = service
        .open_and_interact(source, 6, None)
        .function_help_packet
        .expect("capability overlay must not remove registry signature metadata");

    assert_eq!(
        packet.signature_forms[0].display_signature,
        "RTD(prog_id, server, topic1, [topic2], ...)"
    );
    assert!(packet.deferred_or_profile_limited);
    assert_eq!(
        packet.availability_summary.as_deref(),
        Some("registry_capability=Unavailable(provider unavailable)")
    );
}

#[test]
fn function_help_packet_reflects_udf_registry_mutation() {
    let source = FormulaSourceRecord::new("editor-help-udf", 1, "=MYFUNC(10,\"x\")");
    let mut registry = builtin_registry().clone();
    registry
        .register_udf(test_udf_entry())
        .expect("UDF registration should be accepted by OxFunc registry");

    {
        let service = EditorEditService::new(
            EditorEnvironment::new(editor_bind_context(source.clone()))
                .with_function_registry(&registry),
        );
        let packet = service
            .open_and_interact(source.clone(), 11, None)
            .function_help_packet
            .expect("registered UDF should resolve through provided registry");

        assert_eq!(packet.display_name, "MYFUNC");
        assert_eq!(
            packet.signature_forms[0].display_signature,
            "MYFUNC(value, label)"
        );
        assert_eq!(packet.argument_help, vec!["value", "*label"]);
    }

    registry
        .unregister_udf("FUNC.UDF.MYFUNC")
        .expect("UDF unregister should be accepted by OxFunc registry");
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_function_registry(&registry),
    );

    assert!(
        service
            .open_and_interact(source, 11, None)
            .function_help_packet
            .is_none()
    );
}

#[test]
fn w068_editor_source_has_no_legacy_signature_synthesis() {
    let editor_source = include_str!("../src/consumer/editor/mod.rs");
    for needle in [
        "parse_arity_shape_note",
        "signature_suffix",
        "build_argument_help",
        "additional_args",
        "arg1",
        "arity_shape_note",
    ] {
        assert!(
            !editor_source.contains(needle),
            "editor function-help source must not contain legacy synthesis token {needle}"
        );
    }

    let semantics_source = include_str!("../src/semantics/mod.rs");
    assert!(
        !semantics_source.contains("arity_shape_note"),
        "snapshot/schema source must not retain arity_shape_note"
    );
}

#[test]
fn intelligent_completion_context_carries_scope_and_active_diagnostics() {
    let source = FormulaSourceRecord::new("editor-intel", 1, "=[@Amount]");
    let bind_context = editor_bind_context(source.clone());
    let snapshot = sample_library_context_snapshot();
    let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
    let provider = InMemoryLibraryContextProvider::new(snapshot.clone());
    let service = EditorEditService::new(
        EditorEnvironment::new(bind_context)
            .with_library_context_provider(&provider)
            .with_inline_library_context_snapshot(snapshot),
    );

    let document = service.open_document(
        source,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "editor-catalog".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: provider.snapshot_by_identity(&snapshot_ref),
        }),
    );
    let context = service.intelligent_completion_context_at_cursor(&document, 5);

    assert!(context.red_context_summary.starts_with("kind="));
    assert!(context.visible_tables.iter().any(|table| table == "Table1"));
    assert!(!context.active_diagnostics.is_empty());
    assert_eq!(
        context.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::new("editor-snapshot", "v1"))
    );
}

#[test]
fn validate_completion_candidate_reenters_normal_edit_pipeline() {
    let source = FormulaSourceRecord::new("editor-validate", 3, "=SU");
    let service =
        EditorEditService::new(EditorEnvironment::new(editor_bind_context(source.clone())));
    let initial = service.apply_edit(source, None, EditorAnalysisStage::SyntaxAndBind, None);

    let result = service.validate_completion(
        &initial.document,
        Some(TextSpan::new(1, 2)),
        "SUM",
        EditorAnalysisStage::SyntaxAndBind,
        None,
    );

    assert_eq!(
        result
            .interaction_result
            .document
            .source
            .entered_formula_text,
        "=SUM"
    );
    assert_eq!(
        result
            .interaction_result
            .document
            .source
            .formula_text_version
            .0,
        4
    );
    assert_eq!(
        result.interaction_result.document.text_change_range,
        Some(FormulaTextChangeRange {
            start: 3,
            old_len: 0,
            new_len: 1,
        })
    );
}

#[test]
fn apply_completion_proposal_preserves_proposal_identity() {
    let source = FormulaSourceRecord::new("editor-proposal", 5, "=SU");
    let snapshot = sample_library_context_snapshot();
    let snapshot_ref = LibraryContextSnapshotRef::from(&snapshot);
    let provider = InMemoryLibraryContextProvider::new(snapshot);
    let service = EditorEditService::new(
        EditorEnvironment::new(editor_bind_context(source.clone()))
            .with_pinned_library_context(&provider, snapshot_ref),
    );

    let document = service.open_document(source.clone(), None);
    let completion =
        service.completion_at_cursor(&document, source.entered_formula_text.chars().count());
    let sum = completion
        .proposals
        .iter()
        .find(|proposal| proposal.display_text == "SUM")
        .expect("SUM proposal should exist");

    let result =
        service.apply_completion_proposal(&document, sum, EditorAnalysisStage::SyntaxAndBind, None);

    assert_eq!(result.proposal_id.as_deref(), Some("function:SUM"));
    assert_eq!(
        result
            .interaction_result
            .document
            .source
            .entered_formula_text,
        "=SUM"
    );
}

fn snapshot_source_text(snapshot: &EditorSyntaxSnapshot) -> String {
    let mut text = String::new();
    let last_index = snapshot.tokens.len().saturating_sub(1);

    for (index, token) in snapshot.tokens.iter().enumerate() {
        for trivia in &token.leading_trivia {
            text.push_str(&trivia.text);
        }
        text.push_str(&token.text);
        if index == last_index {
            for trivia in &token.trailing_trivia {
                text.push_str(&trivia.text);
            }
        }
    }

    text
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

fn sample_library_context_snapshot_v2() -> LibraryContextSnapshot {
    let mut snapshot = sample_library_context_snapshot();
    snapshot.snapshot_id = "editor-snapshot".to_string();
    snapshot.snapshot_version = "v2".to_string();
    snapshot.entries.push(LibraryContextSnapshotEntry {
        surface_name: "TAKE".to_string(),
        canonical_id: Some("FUNC.TAKE".to_string()),
        surface_stable_id: Some("surface:take".to_string()),
        name_resolution_table_ref: None,
        semantic_trait_profile_ref: None,
        gating_profile_ref: None,
        metadata_status: None,
        special_interface_kind: None,
        admission_interface_kind: None,
        preparation_owner: None,
        runtime_boundary_kind: None,
        interface_contract_ref: Some("contract:take".to_string()),
        registration_source_kind: RegistrationSourceKind::BuiltIn,
        parse_bind_state: LibraryAvailabilityState::CatalogKnown,
        semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
        runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
        post_dispatch_state: None,
    });
    snapshot
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

fn test_udf_entry() -> FunctionEntry {
    FunctionEntry {
        meta: RegistryFunctionMeta {
            function_id: "FUNC.UDF.MYFUNC".to_string(),
            arity: Arity::exact(2),
            determinism: DeterminismClass::Deterministic,
            volatility: VolatilityClass::NonVolatile,
            host_interaction: HostInteractionClass::None,
            thread_safety: ThreadSafetyClass::SafePure,
            arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
            coercion_lift_profile: CoercionLiftProfile::Custom,
            kernel_signature_class: KernelSignatureClass::Custom,
            fec_dependency_profile: FecDependencyProfile::None,
            surface_fec_dependency_profile: FecDependencyProfile::None,
        },
        surface_name: "MYFUNC".to_string(),
        display_signature: SignatureForm {
            signature_display: "MYFUNC(value, label)".to_string(),
            parameters: vec![
                ParameterDescriptor {
                    name: "value".to_string(),
                    optional: false,
                    repeats: false,
                    short_description: None,
                },
                ParameterDescriptor {
                    name: "label".to_string(),
                    optional: false,
                    repeats: false,
                    short_description: None,
                },
            ],
            trailing_repeats: false,
        },
        registry_metadata: FunctionRegistryMetadata::default(),
        short_description: None,
        long_description: None,
        source: FunctionSource::Udf {
            provenance: Some("language_service_tests".to_string()),
            replaces_builtin: false,
        },
    }
}
