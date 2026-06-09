use oxfml_core::binding::{BoundExpr, HostNameBindRecord, NameKind};
use oxfml_core::consumer::editor::{
    EditorAnalysisStage, EditorEditService, EditorEnvironment, EditorHostReferenceInsertionRequest,
    EditorHostReferenceTarget, EditorPlanOptions,
};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::{
    BindContext, FormulaChannelKind, FormulaSourceRecord, HostReferenceCollectionSyntax,
    HostReferenceStructuralSelectorSyntax, HostReferenceSyntaxProfile,
    InMemoryLibraryContextProvider, LibraryContextProvider, LibraryContextSnapshotRef,
};

#[test]
fn editor_edit_service_applies_edit_and_returns_document() {
    let environment = EditorEnvironment::new(BindContext::default());
    let service = EditorEditService::new(environment);
    let source = FormulaSourceRecord::new("editor:sum", 1, "=SUM(1,2)")
        .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);

    let document = service.open_document(
        source.clone(),
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: None,
        }),
    );

    assert_eq!(document.source, source);
    assert!(document.bound_formula.is_some());
    assert!(document.semantic_plan.is_some());
}

#[test]
fn editor_edit_service_interaction_uses_registry_for_completion_and_snapshot_for_help() {
    let pinned_snapshot = editor_snapshot_v1();
    let pinned_snapshot_ref = LibraryContextSnapshotRef::from(&pinned_snapshot);
    let current_snapshot = editor_snapshot_v2();
    let current_snapshot_ref = LibraryContextSnapshotRef::from(&current_snapshot);
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        current_snapshot_ref,
        vec![pinned_snapshot, current_snapshot],
    );
    let environment = EditorEnvironment::new(BindContext::default())
        .with_pinned_library_context(&provider, pinned_snapshot_ref.clone());
    let service = EditorEditService::new(environment);
    let completion_interaction = service.open_and_interact(
        FormulaSourceRecord::new("editor:complete", 1, "="),
        1,
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: provider.snapshot_by_identity(&pinned_snapshot_ref),
        }),
    );
    let completion = completion_interaction
        .completion_result
        .expect("completion interaction should carry proposals");
    assert!(
        completion
            .proposals
            .iter()
            .any(|proposal| proposal.display_text.eq_ignore_ascii_case("SUM"))
    );
    assert!(
        completion
            .proposals
            .iter()
            .any(|proposal| proposal.display_text.eq_ignore_ascii_case("TAKE")),
        "completion proposals come from the OxFunc registry, not the pinned snapshot"
    );

    let help_interaction = service.open_and_interact(
        FormulaSourceRecord::new("editor:help", 1, "=SUM("),
        "=SUM(".len(),
        Some(EditorPlanOptions {
            oxfunc_catalog_identity: "oxfunc:editor".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: provider.snapshot_by_identity(&pinned_snapshot_ref),
        }),
    );
    let help_packet = help_interaction
        .function_help_packet
        .expect("help packet should be built");
    let completion_context = help_interaction
        .intelligent_completion_context
        .expect("intelligent completion context should be built");
    assert_eq!(
        help_packet.library_context_snapshot_ref,
        Some(pinned_snapshot_ref)
    );
    assert_eq!(help_packet.lookup_key, "SUM");
    assert_eq!(help_packet.display_name, "SUM");
    assert_eq!(completion_context.formula_text, "=SUM(");
    assert_eq!(
        completion_context.library_context_snapshot_ref,
        help_packet.library_context_snapshot_ref
    );
    assert!(
        help_packet
            .availability_summary
            .as_deref()
            .is_some_and(|summary| summary.contains("parse_bind=CatalogKnown"))
    );
}

#[test]
fn editor_insert_host_reference_composes_profile_selector_and_rebinds() {
    let environment = EditorEnvironment::new(treecalc_bind_context(["Base"]));
    let service = EditorEditService::new(environment);
    let document = service.open_document(
        FormulaSourceRecord::new("editor:host-ref-insert", 1, "=SUM()"),
        None,
    );

    let result = service
        .insert_host_reference(
            &document,
            EditorHostReferenceInsertionRequest {
                target: EditorHostReferenceTarget::HostStructuralSelector {
                    base_canonical_name: "Base".to_string(),
                    selector_family: "parent".to_string(),
                },
                replacement_span: Some(TextSpan::new("=SUM(".chars().count(), 0)),
            },
            EditorAnalysisStage::SyntaxAndBind,
            None,
        )
        .expect("known selector family should compose");

    assert_eq!(result.inserted_text, "Base.@PARENT");
    assert_eq!(
        result
            .interaction_result
            .document
            .source
            .entered_formula_text,
        "=SUM(Base.@PARENT)"
    );
    let bound = result
        .interaction_result
        .document
        .bound_formula
        .expect("inserted formula should bind");
    let BoundExpr::FunctionCall { args, .. } = bound.root else {
        panic!("expected SUM call");
    };
    assert!(matches!(
        args.as_slice(),
        [BoundExpr::HostStructuralSelector(selector)]
            if selector.selector_family == "parent"
                && selector.source_token_text == "Base.@PARENT"
                && matches!(&*selector.base, BoundExpr::HostReference(record)
                    if record.canonical_name == "Base")
    ));
}

#[test]
fn editor_insert_host_reference_escapes_bracketed_host_name_and_rebinds() {
    let environment = EditorEnvironment::new(treecalc_bind_context(["Net Revenue"]));
    let service = EditorEditService::new(environment);
    let document = service.open_document(
        FormulaSourceRecord::new("editor:escaped-host-ref-insert", 1, "=SUM()"),
        None,
    );

    let result = service
        .insert_host_reference(
            &document,
            EditorHostReferenceInsertionRequest {
                target: EditorHostReferenceTarget::HostName {
                    canonical_name: "Net Revenue".to_string(),
                },
                replacement_span: Some(TextSpan::new("=SUM(".chars().count(), 0)),
            },
            EditorAnalysisStage::SyntaxAndBind,
            None,
        )
        .expect("host name should compose");

    assert_eq!(result.inserted_text, "[Net Revenue]");
    assert_eq!(
        result
            .interaction_result
            .document
            .source
            .entered_formula_text,
        "=SUM([Net Revenue])"
    );
    let bound = result
        .interaction_result
        .document
        .bound_formula
        .expect("inserted formula should bind");
    let BoundExpr::FunctionCall { args, .. } = bound.root else {
        panic!("expected SUM call");
    };
    assert!(matches!(
        args.as_slice(),
        [BoundExpr::HostReference(record)]
            if record.canonical_name == "Net Revenue"
                && record.source_token_text == "Net Revenue"
    ));
}

#[test]
fn editor_insert_host_reference_composes_collection_token_from_profile() {
    let environment = EditorEnvironment::new(treecalc_bind_context(["Base"]));
    let service = EditorEditService::new(environment);
    let document = service.open_document(
        FormulaSourceRecord::new("editor:host-ref-collection-insert", 1, "=SUM()"),
        None,
    );

    let result = service
        .insert_host_reference(
            &document,
            EditorHostReferenceInsertionRequest {
                target: EditorHostReferenceTarget::HostReferenceCollection {
                    base_canonical_name: Some("Base".to_string()),
                    collection_family: "children".to_string(),
                },
                replacement_span: Some(TextSpan::new("=SUM(".chars().count(), 0)),
            },
            EditorAnalysisStage::SyntaxAndBind,
            None,
        )
        .expect("known collection family should compose");

    assert_eq!(result.inserted_text, "Base.@CHILDREN");
    let bound = result
        .interaction_result
        .document
        .bound_formula
        .expect("inserted formula should bind");
    let BoundExpr::FunctionCall { args, .. } = bound.root else {
        panic!("expected SUM call");
    };
    assert!(matches!(
        args.as_slice(),
        [BoundExpr::HostStructuralSelector(selector)]
            if selector.selector_family == "children"
                && selector.source_token_text == "Base.@CHILDREN"
    ));
}

#[test]
fn editor_insert_host_reference_composes_star_collection_without_double_dot() {
    let mut context = treecalc_bind_context(["Base"]);
    context.host_reference_syntax =
        HostReferenceSyntaxProfile::with_collection_members([HostReferenceCollectionSyntax::new(
            "*", "children",
        )]);
    let service = EditorEditService::new(EditorEnvironment::new(context));
    let document = service.open_document(
        FormulaSourceRecord::new("editor:host-ref-star-collection-insert", 1, "=SUM(0)"),
        None,
    );

    let result = service
        .insert_host_reference(
            &document,
            EditorHostReferenceInsertionRequest {
                target: EditorHostReferenceTarget::HostReferenceCollection {
                    base_canonical_name: Some("Base".to_string()),
                    collection_family: "children".to_string(),
                },
                replacement_span: Some(TextSpan::new("=SUM(".chars().count(), 1)),
            },
            EditorAnalysisStage::SyntaxAndBind,
            None,
        )
        .expect("known collection family should compose");

    assert_eq!(result.inserted_text, "Base.*");
    assert_eq!(
        result
            .interaction_result
            .document
            .source
            .entered_formula_text,
        "=SUM(Base.*)"
    );
}

fn editor_snapshot_v1() -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "editor-runtime".to_string(),
        snapshot_version: "v1".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: "SUM".to_string(),
            canonical_id: Some("FUNC.SUM".to_string()),
            surface_stable_id: Some("surface:sum".to_string()),
            name_resolution_table_ref: None,
            semantic_trait_profile_ref: None,
            gating_profile_ref: None,
            metadata_status: Some("runtime_snapshot".to_string()),
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
        }],
    }
}

fn treecalc_bind_context<const N: usize>(host_names: [&str; N]) -> BindContext {
    let mut context = BindContext {
        host_reference_syntax: HostReferenceSyntaxProfile::with_members_and_structural_selectors(
            [
                HostReferenceCollectionSyntax::new("CHILDREN", "children"),
                HostReferenceCollectionSyntax::new("*", "children"),
            ],
            [
                HostReferenceStructuralSelectorSyntax::new("PARENT", "parent"),
                HostReferenceStructuralSelectorSyntax::new("SELF", "self"),
                HostReferenceStructuralSelectorSyntax::new("PREV", "previous"),
                HostReferenceStructuralSelectorSyntax::new("NEXT", "next"),
            ],
        ),
        ..BindContext::default()
    };
    for name in host_names {
        context
            .names
            .insert(name.to_string(), NameKind::ReferenceLike);
        context
            .host_name_bind_records
            .insert(name.to_string(), host_name_bind_record(name));
    }
    context
}

fn host_name_bind_record(name: &str) -> HostNameBindRecord {
    HostNameBindRecord {
        host_name_handle: format!("host-name:{name}"),
        canonical_name: name.to_string(),
        host_dependency_key: Some(format!("tree-node:{name}")),
        source_span: TextSpan::new(0, 0),
        source_token_text: name.to_string(),
        resolution_layer: "treecalc_host_name".to_string(),
        binding_kind: "tree_node_reference".to_string(),
        shape_hint: Some("scalar_node_value".to_string()),
        caller_context_dependent: false,
        diagnostics: Vec::new(),
        replay_identity_contribution: format!("host-name:{name}:replay"),
    }
}

fn editor_snapshot_v2() -> LibraryContextSnapshot {
    let mut snapshot = editor_snapshot_v1();
    snapshot.snapshot_version = "v2".to_string();
    snapshot.entries.push(LibraryContextSnapshotEntry {
        surface_name: "TAKE".to_string(),
        canonical_id: Some("FUNC.TAKE".to_string()),
        surface_stable_id: Some("surface:take".to_string()),
        name_resolution_table_ref: None,
        semantic_trait_profile_ref: None,
        gating_profile_ref: None,
        metadata_status: Some("runtime_snapshot".to_string()),
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
