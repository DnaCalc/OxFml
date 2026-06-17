use oxfml_core::binding::{
    BoundExpr, NormalizedReference, ProfilePayload, ProfileReferenceRecord, ProfileVersion,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceExpr, ReferenceNameBindRequest,
    ReferenceNormalFormKey, ReferenceOperatorCapabilities, ReferencePolicy,
    ReferenceProfileFingerprint, ReferenceProfileFingerprintContext, ReferenceSelectorBindRequest,
    ReferenceSelectorSyntax, ReferenceSourceInfo, ReferenceValidity,
};
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
    let profile = TestEditorReferenceProfile;
    let environment =
        EditorEnvironment::new(BindContext::default()).with_reference_bind_profile(&profile);
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
    let record = profile_record_from_bound_expr(&args[0]).expect("profile selector");
    assert_eq!(record.source_info.source_text, "Base.@PARENT");
    assert!(record.profile_payload.data.contains("family=parent"));
    assert!(record.profile_payload.data.contains("base=name:Base"));
}

#[test]
fn editor_insert_host_reference_escapes_bracketed_host_name_and_rebinds() {
    let profile = TestEditorReferenceProfile;
    let environment =
        EditorEnvironment::new(BindContext::default()).with_reference_bind_profile(&profile);
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
    let record = profile_record_from_bound_expr(&args[0]).expect("profile name");
    assert_eq!(record.profile_payload.payload_kind, "name");
    assert_eq!(record.source_info.source_text, "Net Revenue");
}

#[test]
fn editor_insert_host_reference_composes_collection_token_from_profile() {
    let profile = TestEditorReferenceProfile;
    let environment =
        EditorEnvironment::new(BindContext::default()).with_reference_bind_profile(&profile);
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
    let record = profile_record_from_bound_expr(&args[0]).expect("profile collection");
    assert_eq!(record.source_info.source_text, "Base.@CHILDREN");
    assert!(record.profile_payload.data.contains("family=children"));
    assert!(record.profile_payload.data.contains("base=name:Base"));
}

#[test]
fn editor_insert_host_reference_composes_star_collection_without_double_dot() {
    let profile = TestEditorStarReferenceProfile;
    let service = EditorEditService::new(
        EditorEnvironment::new(BindContext::default()).with_reference_bind_profile(&profile),
    );
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

struct TestEditorReferenceProfile;

impl ReferenceBindProfile for TestEditorReferenceProfile {
    fn profile_id(&self) -> &str {
        "editor.test.profile"
    }

    fn profile_version(&self) -> ProfileVersion {
        ProfileVersion::v1()
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn fingerprint(
        &self,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint("editor-test-profile".to_string())
    }

    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities {
        ReferenceOperatorCapabilities::worksheet_legacy()
    }

    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax> {
        vec![
            ReferenceSelectorSyntax::collection("CHILDREN", "children"),
            ReferenceSelectorSyntax::collection("PARENT", "parent"),
            ReferenceSelectorSyntax::collection("NEXT", "next"),
        ]
    }

    fn bind_name(&self, request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::Bound(test_editor_profile_record(
            "name",
            &request.source_text,
            &format!("name:{}", request.source_text),
            request.source_channel,
            request.source_span,
        ))
    }

    fn bind_selector(&self, request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult {
        bind_editor_selector(request)
    }
}

struct TestEditorStarReferenceProfile;

impl ReferenceBindProfile for TestEditorStarReferenceProfile {
    fn profile_id(&self) -> &str {
        "editor.test.star-profile"
    }

    fn profile_version(&self) -> ProfileVersion {
        ProfileVersion::v1()
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn fingerprint(
        &self,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint("editor-test-star-profile".to_string())
    }

    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities {
        ReferenceOperatorCapabilities::worksheet_legacy()
    }

    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax> {
        vec![ReferenceSelectorSyntax::collection("*", "children")]
    }

    fn bind_name(&self, request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        ReferenceAtomBindResult::Bound(test_editor_profile_record(
            "name",
            &request.source_text,
            &format!("name:{}", request.source_text),
            request.source_channel,
            request.source_span,
        ))
    }

    fn bind_selector(&self, request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult {
        bind_editor_selector(request)
    }
}

fn bind_editor_selector(request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult {
    let base = request
        .base
        .as_ref()
        .map(|record| record.normal_form_key.0.clone())
        .unwrap_or_else(|| "owner".to_string());
    let mut record = test_editor_profile_record(
        "selector",
        &request.source_text,
        &format!("selector:{}:{base}", request.selector_family),
        request.source_channel,
        request.source_span,
    );
    record.profile_payload.data = format!("family={};base={base}", request.selector_family);
    ReferenceAtomBindResult::Bound(record)
}

fn test_editor_profile_record(
    payload_kind: &str,
    source_text: &str,
    normal_form_key: &str,
    source_channel: FormulaChannelKind,
    source_span: TextSpan,
) -> ProfileReferenceRecord {
    ProfileReferenceRecord {
        profile_id: "editor.test.profile".to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel,
            source_span,
            source_text: source_text.to_string(),
            parsed_qualifier: None,
            address_fidelity: None,
        },
        profile_payload: ProfilePayload::textual(payload_kind, source_text),
        normal_form_key: ReferenceNormalFormKey(normal_form_key.to_string()),
        render_hint: Some(source_text.to_string()),
        validity: ReferenceValidity::ValidNow,
    }
}

fn profile_record_from_bound_expr(expr: &BoundExpr) -> Option<&ProfileReferenceRecord> {
    match expr {
        BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record))) => {
            Some(record)
        }
        _ => None,
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
