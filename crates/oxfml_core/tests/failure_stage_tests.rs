use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::RejectCode;
use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::{
    CompileSemanticPlanRequest, LibraryAvailabilityState, LibraryContextSnapshot,
    LibraryContextSnapshotEntry, RegistrationSourceKind, compile_semantic_plan,
};
use oxfml_core::session::{CapabilityViewSpec, PrepareRequest, SessionService};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use serde::Deserialize;

mod common;

#[derive(Debug, Deserialize)]
struct FailureStageFixture {
    case_id: String,
    lane: String,
    formula: String,
    snapshot: Option<LibraryContextSnapshotWire>,
    expected: FailureStageExpected,
}

#[derive(Debug, Deserialize)]
struct LibraryContextSnapshotWire {
    snapshot_id: String,
    snapshot_version: String,
    entries: Vec<LibraryContextSnapshotEntryWire>,
}

#[derive(Debug, Deserialize)]
struct LibraryContextSnapshotEntryWire {
    surface_name: String,
    canonical_id: Option<String>,
    surface_stable_id: Option<String>,
    name_resolution_table_ref: Option<String>,
    semantic_trait_profile_ref: Option<String>,
    gating_profile_ref: Option<String>,
    metadata_status: Option<String>,
    special_interface_kind: Option<String>,
    admission_interface_kind: Option<String>,
    preparation_owner: Option<String>,
    runtime_boundary_kind: Option<String>,
    arity_shape_note: Option<String>,
    interface_contract_ref: Option<String>,
    registration_source_kind: String,
    parse_bind_state: String,
    semantic_plan_state: String,
    runtime_capability_state: Option<String>,
    post_dispatch_state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FailureStageExpected {
    unresolved_reasons: Option<Vec<String>>,
    surface_name: Option<String>,
    parse_bind_state: Option<String>,
    semantic_plan_state: Option<String>,
    runtime_capability_state: Option<String>,
    post_dispatch_state: Option<String>,
    reject_code: Option<String>,
}

#[test]
fn failure_stage_fixtures_match_expected_classification() {
    let fixtures = load_fixtures();
    for fixture in fixtures {
        match fixture.lane.as_str() {
            "AcceptedUnresolvedName" => {
                let compiled = common::compile_formula_with_library_context(
                    "failure-stage-fixture",
                    &fixture.formula,
                    BTreeMap::new(),
                    "failure-stage-struct-v1",
                    "oxfunc:failure-stage-fixture",
                    fixture.snapshot.as_ref().map(into_snapshot),
                );

                let unresolved_reasons = compiled
                    .bound_formula
                    .unresolved_references
                    .iter()
                    .map(|item| item.reason.clone())
                    .collect::<Vec<_>>();
                assert_eq!(
                    Some(unresolved_reasons),
                    fixture.expected.unresolved_reasons,
                    "unresolved-name classification mismatch for {}",
                    fixture.case_id
                );
            }
            "SemanticPlanGated" | "PostDispatchProviderUnavailable" => {
                let compiled = common::compile_formula_with_library_context(
                    "failure-stage-fixture",
                    &fixture.formula,
                    BTreeMap::new(),
                    "failure-stage-struct-v1",
                    "oxfunc:failure-stage-fixture",
                    fixture.snapshot.as_ref().map(into_snapshot),
                );

                let summary = compiled
                    .semantic_plan
                    .availability_summaries
                    .iter()
                    .find(|summary| {
                        summary.surface_name.eq_ignore_ascii_case(
                            fixture
                                .expected
                                .surface_name
                                .as_deref()
                                .expect("surface_name should exist"),
                        )
                    })
                    .expect("availability summary should exist");

                assert_eq!(
                    fixture.expected.parse_bind_state.as_deref(),
                    Some(availability_state_name(summary.parse_bind_state)),
                    "parse/bind state mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    fixture.expected.semantic_plan_state.as_deref(),
                    Some(availability_state_name(summary.semantic_plan_state)),
                    "semantic-plan state mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    fixture.expected.runtime_capability_state.as_deref(),
                    summary
                        .runtime_capability_state
                        .map(availability_state_name),
                    "runtime capability state mismatch for {}",
                    fixture.case_id
                );
                assert_eq!(
                    fixture.expected.post_dispatch_state.as_deref(),
                    summary.post_dispatch_state.map(availability_state_name),
                    "post-dispatch state mismatch for {}",
                    fixture.case_id
                );
            }
            "RuntimeCapabilityDenied" => {
                let prepared = compile_prepared(&fixture.formula);
                let mut service = SessionService::new();
                let prepared = service.prepare(prepared).expect("prepare should succeed");
                let open = service.open_session(prepared);
                let reject = service
                    .establish_capability_view(
                        &open.session_id,
                        CapabilityViewSpec {
                            host_query_enabled: false,
                            locale_format_enabled: true,
                            caller_context_enabled: true,
                            external_provider_enabled: false,
                        },
                    )
                    .expect_err("capability view should reject");

                assert_eq!(
                    Some(reject_code_name(reject.reject_code)),
                    fixture.expected.reject_code.as_deref(),
                    "runtime-capability reject mismatch for {}",
                    fixture.case_id
                );
            }
            other => panic!("unsupported lane {other}"),
        }
    }
}

fn compile_prepared(formula: &str) -> PrepareRequest {
    let source = FormulaSourceRecord::new("failure-stage-session", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion(
                "failure-stage-session-v1".to_string(),
            ),
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });
    let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:failure-stage-session".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan;

    PrepareRequest {
        source,
        bound_formula: bind.bound_formula,
        semantic_plan,
        primary_locus: oxfml_core::Locus {
            sheet_id: "sheet:default".to_string(),
            row: 1,
            col: 1,
        },
    }
}

fn load_fixtures() -> Vec<FailureStageFixture> {
    let content = fs::read_to_string(fixture_path("failure_stage_cases.json"))
        .expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn into_snapshot(wire: &LibraryContextSnapshotWire) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: wire.snapshot_id.clone(),
        snapshot_version: wire.snapshot_version.clone(),
        entries: wire.entries.iter().map(into_snapshot_entry).collect(),
    }
}

fn into_snapshot_entry(wire: &LibraryContextSnapshotEntryWire) -> LibraryContextSnapshotEntry {
    LibraryContextSnapshotEntry {
        surface_name: wire.surface_name.clone(),
        canonical_id: wire.canonical_id.clone(),
        surface_stable_id: wire.surface_stable_id.clone(),
        name_resolution_table_ref: wire.name_resolution_table_ref.clone(),
        semantic_trait_profile_ref: wire.semantic_trait_profile_ref.clone(),
        gating_profile_ref: wire.gating_profile_ref.clone(),
        metadata_status: wire.metadata_status.clone(),
        special_interface_kind: wire.special_interface_kind.clone(),
        admission_interface_kind: wire.admission_interface_kind.clone(),
        preparation_owner: wire.preparation_owner.clone(),
        runtime_boundary_kind: wire.runtime_boundary_kind.clone(),
        arity_shape_note: wire.arity_shape_note.clone(),
        interface_contract_ref: wire.interface_contract_ref.clone(),
        registration_source_kind: parse_registration_source_kind(&wire.registration_source_kind),
        parse_bind_state: parse_availability_state(&wire.parse_bind_state),
        semantic_plan_state: parse_availability_state(&wire.semantic_plan_state),
        runtime_capability_state: wire
            .runtime_capability_state
            .as_deref()
            .map(parse_availability_state),
        post_dispatch_state: wire
            .post_dispatch_state
            .as_deref()
            .map(parse_availability_state),
    }
}

fn parse_registration_source_kind(value: &str) -> RegistrationSourceKind {
    match value {
        "BuiltIn" => RegistrationSourceKind::BuiltIn,
        "AddIn" => RegistrationSourceKind::AddIn,
        "ProviderBacked" => RegistrationSourceKind::ProviderBacked,
        "UserDefined" => RegistrationSourceKind::UserDefined,
        "Vba" => RegistrationSourceKind::Vba,
        "CompatibilityAlias" => RegistrationSourceKind::CompatibilityAlias,
        _ => panic!("unsupported registration source kind {value}"),
    }
}

fn parse_availability_state(value: &str) -> LibraryAvailabilityState {
    match value {
        "CatalogKnown" => LibraryAvailabilityState::CatalogKnown,
        "FeatureGated" => LibraryAvailabilityState::FeatureGated,
        "CompatibilityGated" => LibraryAvailabilityState::CompatibilityGated,
        "HostProfileUnavailable" => LibraryAvailabilityState::HostProfileUnavailable,
        "AddInAbsent" => LibraryAvailabilityState::AddInAbsent,
        "ProviderUnavailable" => LibraryAvailabilityState::ProviderUnavailable,
        "UnknownSurface" => LibraryAvailabilityState::UnknownSurface,
        _ => panic!("unsupported availability state {value}"),
    }
}

fn availability_state_name(value: LibraryAvailabilityState) -> &'static str {
    match value {
        LibraryAvailabilityState::CatalogKnown => "CatalogKnown",
        LibraryAvailabilityState::FeatureGated => "FeatureGated",
        LibraryAvailabilityState::CompatibilityGated => "CompatibilityGated",
        LibraryAvailabilityState::HostProfileUnavailable => "HostProfileUnavailable",
        LibraryAvailabilityState::AddInAbsent => "AddInAbsent",
        LibraryAvailabilityState::ProviderUnavailable => "ProviderUnavailable",
        LibraryAvailabilityState::UnknownSurface => "UnknownSurface",
    }
}

fn reject_code_name(code: RejectCode) -> &'static str {
    match code {
        RejectCode::FenceMismatch => "FenceMismatch",
        RejectCode::CapabilityDenied => "CapabilityDenied",
        RejectCode::SessionTerminated => "SessionTerminated",
        RejectCode::BindMismatch => "BindMismatch",
        RejectCode::StructuralConflict => "StructuralConflict",
        RejectCode::DynamicReferenceFailure => "DynamicReferenceFailure",
        RejectCode::ResourceInvariantFailure => "ResourceInvariantFailure",
    }
}
