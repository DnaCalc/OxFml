use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use serde::Deserialize;

mod common;

#[derive(Debug, Deserialize)]
struct LibraryContextSnapshotFixture {
    case_id: String,
    formula: String,
    snapshot: LibraryContextSnapshotWire,
    expected: LibraryContextExpected,
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
    registration_source_kind: String,
    parse_bind_state: String,
    semantic_plan_state: String,
    runtime_capability_state: Option<String>,
    post_dispatch_state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LibraryContextExpected {
    snapshot_ref: String,
    availability_summaries: Vec<AvailabilitySummaryExpected>,
}

#[derive(Debug, Deserialize)]
struct AvailabilitySummaryExpected {
    surface_name: String,
    canonical_id: Option<String>,
    registration_source_kind: Option<String>,
    parse_bind_state: String,
    semantic_plan_state: String,
    runtime_capability_state: Option<String>,
    post_dispatch_state: Option<String>,
}

#[test]
fn semantic_plan_library_context_snapshot_fixtures_round_trip() {
    let fixtures = load_fixtures();
    for fixture in fixtures {
        let compiled = common::compile_formula_with_library_context(
            "library-context-fixture",
            &fixture.formula,
            BTreeMap::new(),
            "library-context-struct-v1",
            "oxfunc:library-context-fixture",
            Some(into_snapshot(&fixture.snapshot)),
        );
        let plan = compiled.semantic_plan;

        assert_eq!(
            plan.library_context_snapshot_ref.as_deref(),
            Some(fixture.expected.snapshot_ref.as_str()),
            "snapshot ref mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            plan.availability_summaries.len(),
            fixture.expected.availability_summaries.len(),
            "availability summary count mismatch for {}",
            fixture.case_id
        );

        for expected in &fixture.expected.availability_summaries {
            let actual = plan
                .availability_summaries
                .iter()
                .find(|item| {
                    item.surface_name
                        .eq_ignore_ascii_case(&expected.surface_name)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "missing availability summary {} for {}",
                        expected.surface_name, fixture.case_id
                    )
                });

            assert_eq!(
                actual.canonical_id, expected.canonical_id,
                "canonical id mismatch for {}",
                fixture.case_id
            );
            assert_eq!(
                actual
                    .registration_source_kind
                    .map(registration_source_kind_name),
                expected.registration_source_kind.as_deref(),
                "registration source mismatch for {}",
                fixture.case_id
            );
            assert_eq!(
                availability_state_name(actual.parse_bind_state),
                expected.parse_bind_state,
                "parse/bind state mismatch for {}",
                fixture.case_id
            );
            assert_eq!(
                availability_state_name(actual.semantic_plan_state),
                expected.semantic_plan_state,
                "semantic-plan state mismatch for {}",
                fixture.case_id
            );
            assert_eq!(
                actual.runtime_capability_state.map(availability_state_name),
                expected.runtime_capability_state.as_deref(),
                "runtime-capability state mismatch for {}",
                fixture.case_id
            );
            assert_eq!(
                actual.post_dispatch_state.map(availability_state_name),
                expected.post_dispatch_state.as_deref(),
                "post-dispatch state mismatch for {}",
                fixture.case_id
            );
        }
    }
}

fn load_fixtures() -> Vec<LibraryContextSnapshotFixture> {
    let content = fs::read_to_string(fixture_path("library_context_snapshot_cases.json"))
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

fn registration_source_kind_name(value: RegistrationSourceKind) -> &'static str {
    match value {
        RegistrationSourceKind::BuiltIn => "BuiltIn",
        RegistrationSourceKind::AddIn => "AddIn",
        RegistrationSourceKind::ProviderBacked => "ProviderBacked",
        RegistrationSourceKind::UserDefined => "UserDefined",
        RegistrationSourceKind::Vba => "Vba",
        RegistrationSourceKind::CompatibilityAlias => "CompatibilityAlias",
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
