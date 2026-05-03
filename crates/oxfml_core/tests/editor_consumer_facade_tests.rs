use oxfml_core::consumer::editor::{EditorEditService, EditorEnvironment, EditorPlanOptions};
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::{
    BindContext, FormulaChannelKind, FormulaSourceRecord, InMemoryLibraryContextProvider,
    LibraryContextProvider, LibraryContextSnapshotRef,
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
fn editor_edit_service_interaction_uses_pinned_snapshot_for_help_and_completion() {
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
        !completion
            .proposals
            .iter()
            .any(|proposal| proposal.display_text.eq_ignore_ascii_case("TAKE"))
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
