use std::collections::BTreeMap;
use std::path::PathBuf;

use csv::ReaderBuilder;
use oxfml_core::semantics::{
    FunctionAvailabilitySummary, LibraryAvailabilityState, LibraryContextSnapshot,
    LibraryContextSnapshotEntry, RegistrationSourceKind,
};
use serde::Deserialize;

mod common;

#[derive(Debug, Deserialize, Clone)]
struct OxfuncExportRow {
    snapshot_id: String,
    snapshot_generation: String,
    source_commit_short: String,
    source_commit_full: String,
    source_tree_state: String,
    lane_id: String,
    entry_kind: String,
    registration_source_kind: String,
    surface_stable_id: String,
    canonical_surface_name: String,
    name_resolution_table_ref: String,
    semantic_trait_profile_ref: String,
    gating_profile_ref: String,
    metadata_status: String,
    special_interface_kind: String,
    admission_interface_kind: String,
    preparation_owner: String,
    runtime_boundary_kind: String,
    arity_shape_note: String,
    interface_contract_ref: String,
}

#[test]
fn w044_export_ordinary_rows_round_trip_into_semantic_plan() {
    let snapshot = load_snapshot_for_surfaces(&["SUM", "CHOOSECOLS", "FILTER", "UNIQUE", "VSTACK"]);
    let expected_snapshot_ref = format!("{}@{}", snapshot.snapshot_id, snapshot.snapshot_version);

    let compiled = common::compile_formula_with_library_context(
        "w044-ordinary-consumption",
        "=SUM(1)+CHOOSECOLS(A1:C3,1)+FILTER(A1:A3,B1:B3)+UNIQUE(A1:A3)+VSTACK(A1:A2,B1:B2)",
        BTreeMap::new(),
        "w044-export-struct-v1",
        "oxfunc:w044-export",
        Some(snapshot),
    );
    let plan = compiled.semantic_plan;

    assert_eq!(
        plan.library_context_snapshot_ref.as_deref(),
        Some(expected_snapshot_ref.as_str())
    );

    assert_ordinary_row(
        find_summary(&plan.availability_summaries, "SUM"),
        "FUNC.SUM",
        None,
    );
    assert_ordinary_row(
        find_summary(&plan.availability_summaries, "CHOOSECOLS"),
        "FUNC.CHOOSECOLS",
        Some(
            "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md",
        ),
    );
    assert_ordinary_row(
        find_summary(&plan.availability_summaries, "FILTER"),
        "FUNC.FILTER",
        Some(
            "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md",
        ),
    );
    assert_ordinary_row(
        find_summary(&plan.availability_summaries, "UNIQUE"),
        "FUNC.UNIQUE",
        Some(
            "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md",
        ),
    );
    assert_ordinary_row(
        find_summary(&plan.availability_summaries, "VSTACK"),
        "FUNC.VSTACK",
        Some(
            "docs/function-lane/FUNCTION_SLICE_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY_CONTRACT_PRELIM.md",
        ),
    );
}

#[test]
fn w044_export_seam_heavy_rows_round_trip_into_semantic_plan() {
    let snapshot = load_snapshot_for_surfaces(&["LET", "LAMBDA", "RTD"]);
    let expected_snapshot_ref = format!("{}@{}", snapshot.snapshot_id, snapshot.snapshot_version);

    let compiled = common::compile_formula_with_library_context(
        "w044-seam-heavy-consumption",
        "=LET(x,1,LAMBDA(y,y+x))+RTD(\"prog\",\"server\",\"topic\")",
        BTreeMap::new(),
        "w044-export-struct-v1",
        "oxfunc:w044-export",
        Some(snapshot),
    );
    let plan = compiled.semantic_plan;

    assert_eq!(
        plan.library_context_snapshot_ref.as_deref(),
        Some(expected_snapshot_ref.as_str())
    );
    assert!(plan.helper_profile.contains_let);
    assert!(plan.helper_profile.contains_lambda);

    let let_summary = find_summary(&plan.availability_summaries, "LET");
    assert_eq!(
        let_summary.special_interface_kind.as_deref(),
        Some("callable_helper_formation")
    );
    assert_eq!(
        let_summary.admission_interface_kind.as_deref(),
        Some("helper_formation")
    );
    assert_eq!(
        let_summary.preparation_owner.as_deref(),
        Some("oxfml_then_oxfunc")
    );
    assert_eq!(
        let_summary.runtime_boundary_kind.as_deref(),
        Some("callable_helper_runtime_after_formation")
    );
    assert_eq!(
        let_summary.arity_shape_note.as_deref(),
        Some("odd-style helper shape: final arg is body; preceding args form name/value pairs")
    );
    assert_eq!(
        let_summary.interface_contract_ref.as_deref(),
        Some("docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md")
    );

    let lambda_summary = find_summary(&plan.availability_summaries, "LAMBDA");
    assert_eq!(
        lambda_summary.special_interface_kind.as_deref(),
        Some("callable_helper_formation")
    );
    assert_eq!(
        lambda_summary.admission_interface_kind.as_deref(),
        Some("helper_formation")
    );
    assert_eq!(
        lambda_summary.runtime_boundary_kind.as_deref(),
        Some("callable_helper_runtime_after_formation")
    );
    assert_eq!(
        lambda_summary.interface_contract_ref.as_deref(),
        Some("docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md")
    );

    let rtd_summary = find_summary(&plan.availability_summaries, "RTD");
    assert_eq!(
        rtd_summary.special_interface_kind.as_deref(),
        Some("host_subscription_provider")
    );
    assert_eq!(
        rtd_summary.admission_interface_kind.as_deref(),
        Some("host_subscription_call")
    );
    assert_eq!(
        rtd_summary.preparation_owner.as_deref(),
        Some("host_above_oxfunc_then_oxfunc_projection")
    );
    assert_eq!(
        rtd_summary.runtime_boundary_kind.as_deref(),
        Some("host_provider_projection")
    );
    assert_eq!(
        rtd_summary.arity_shape_note.as_deref(),
        Some("prog_id, server_name, then ordered topic strings")
    );
    assert_eq!(
        rtd_summary.interface_contract_ref.as_deref(),
        Some("docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md")
    );
}

#[test]
fn w044_export_higher_order_rows_round_trip_into_semantic_plan() {
    let snapshot =
        load_snapshot_for_surfaces(&["MAP", "REDUCE", "SCAN", "BYROW", "BYCOL", "MAKEARRAY"]);
    let expected_snapshot_ref = format!("{}@{}", snapshot.snapshot_id, snapshot.snapshot_version);

    let compiled = common::compile_formula_with_library_context(
        "w044-higher-order-consumption",
        "=MAP(SEQUENCE(3),LAMBDA(x,x+1))+REDUCE(0,SEQUENCE(3),LAMBDA(a,b,a+b))+SCAN(0,SEQUENCE(3),LAMBDA(a,b,a+b))+SUM(BYROW(SEQUENCE(2,2),LAMBDA(r,SUM(r))))+SUM(BYCOL(SEQUENCE(2,2),LAMBDA(c,SUM(c))))+SUM(MAKEARRAY(2,2,LAMBDA(r,c,r+c)))",
        BTreeMap::new(),
        "w044-export-struct-v1",
        "oxfunc:w044-export",
        Some(snapshot),
    );
    let plan = compiled.semantic_plan;

    assert_eq!(
        plan.library_context_snapshot_ref.as_deref(),
        Some(expected_snapshot_ref.as_str())
    );
    assert!(plan.helper_profile.contains_lambda);
    assert_eq!(plan.helper_profile.lambda_invocation_count, 0);

    assert_higher_order_row(
        find_summary(&plan.availability_summaries, "MAP"),
        "FUNC.MAP",
        "trailing arg callable; preceding args are mapped arrays",
    );
    assert_higher_order_row(
        find_summary(&plan.availability_summaries, "REDUCE"),
        "FUNC.REDUCE",
        "initial accumulator, array, callable",
    );
    assert_higher_order_row(
        find_summary(&plan.availability_summaries, "SCAN"),
        "FUNC.SCAN",
        "initial accumulator, array, callable",
    );
    assert_higher_order_row(
        find_summary(&plan.availability_summaries, "BYROW"),
        "FUNC.BYROW",
        "array plus callable applied per row",
    );
    assert_higher_order_row(
        find_summary(&plan.availability_summaries, "BYCOL"),
        "FUNC.BYCOL",
        "array plus callable applied per column",
    );
    assert_higher_order_row(
        find_summary(&plan.availability_summaries, "MAKEARRAY"),
        "FUNC.MAKEARRAY",
        "rows, cols, callable producing each coordinate cell",
    );
}

fn find_summary<'a>(
    summaries: &'a [FunctionAvailabilitySummary],
    surface_name: &str,
) -> &'a FunctionAvailabilitySummary {
    summaries
        .iter()
        .find(|summary| summary.surface_name.eq_ignore_ascii_case(surface_name))
        .unwrap_or_else(|| panic!("missing availability summary for {surface_name}"))
}

fn assert_ordinary_row(
    summary: &FunctionAvailabilitySummary,
    expected_surface_stable_id: &str,
    expected_interface_contract_ref: Option<&str>,
) {
    assert_eq!(
        summary.surface_stable_id.as_deref(),
        Some(expected_surface_stable_id)
    );
    assert_eq!(
        summary.name_resolution_table_ref.as_deref(),
        Some("docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv")
    );
    assert_eq!(
        summary.semantic_trait_profile_ref.as_deref(),
        Some("oxfunc.local.profile.function_surface.current_baseline.v1")
    );
    assert_eq!(
        summary.gating_profile_ref.as_deref(),
        Some("oxfunc.local.gating.current_baseline.default.v1")
    );
    assert_eq!(
        summary.registration_source_kind,
        Some(RegistrationSourceKind::BuiltIn)
    );
    assert_eq!(
        summary.parse_bind_state,
        LibraryAvailabilityState::CatalogKnown
    );
    assert_eq!(
        summary.semantic_plan_state,
        LibraryAvailabilityState::CatalogKnown
    );
    assert_eq!(
        summary.metadata_status.as_deref(),
        Some("function_meta_extracted")
    );
    assert_eq!(summary.special_interface_kind.as_deref(), Some("ordinary"));
    assert_eq!(
        summary.admission_interface_kind.as_deref(),
        Some("ordinary_call")
    );
    assert_eq!(
        summary.preparation_owner.as_deref(),
        Some("oxfml_then_oxfunc")
    );
    assert_eq!(
        summary.runtime_boundary_kind.as_deref(),
        Some("ordinary_eval")
    );
    assert_eq!(
        summary.interface_contract_ref.as_deref(),
        expected_interface_contract_ref
    );
}

fn assert_higher_order_row(
    summary: &FunctionAvailabilitySummary,
    expected_surface_stable_id: &str,
    expected_arity_shape_note: &str,
) {
    assert_eq!(
        summary.surface_stable_id.as_deref(),
        Some(expected_surface_stable_id)
    );
    assert_eq!(
        summary.name_resolution_table_ref.as_deref(),
        Some("docs/function-lane/W28_FUNCTION_NAME_LOCALIZATION_LIBRARY_SEED.csv")
    );
    assert_eq!(
        summary.semantic_trait_profile_ref.as_deref(),
        Some("oxfunc.local.profile.function_surface.current_baseline.v1")
    );
    assert_eq!(
        summary.gating_profile_ref.as_deref(),
        Some("oxfunc.local.gating.current_baseline.default.v1")
    );
    assert_eq!(
        summary.registration_source_kind,
        Some(RegistrationSourceKind::BuiltIn)
    );
    assert_eq!(
        summary.parse_bind_state,
        LibraryAvailabilityState::CatalogKnown
    );
    assert_eq!(
        summary.semantic_plan_state,
        LibraryAvailabilityState::CatalogKnown
    );
    assert_eq!(summary.metadata_status.as_deref(), Some("catalog_only"));
    assert_eq!(
        summary.special_interface_kind.as_deref(),
        Some("callable_helper_runtime")
    );
    assert_eq!(
        summary.admission_interface_kind.as_deref(),
        Some("higher_order_call")
    );
    assert_eq!(
        summary.preparation_owner.as_deref(),
        Some("oxfml_then_oxfunc")
    );
    assert_eq!(
        summary.runtime_boundary_kind.as_deref(),
        Some("callable_helper_runtime")
    );
    assert_eq!(
        summary.arity_shape_note.as_deref(),
        Some(expected_arity_shape_note)
    );
    assert_eq!(
        summary.interface_contract_ref.as_deref(),
        Some("docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md")
    );
}

fn load_snapshot_for_surfaces(surface_names: &[&str]) -> LibraryContextSnapshot {
    let rows = load_export_rows();
    let selected_rows: Vec<_> = surface_names
        .iter()
        .map(|surface_name| {
            rows.iter()
                .find(|row| {
                    row.canonical_surface_name
                        .eq_ignore_ascii_case(surface_name)
                })
                .unwrap_or_else(|| panic!("missing export row for {surface_name}"))
                .clone()
        })
        .collect();

    let first = selected_rows
        .first()
        .expect("selected export rows should not be empty");
    assert!(
        selected_rows
            .iter()
            .all(|row| row.snapshot_id == first.snapshot_id)
    );
    assert!(
        selected_rows
            .iter()
            .all(|row| row.snapshot_generation == first.snapshot_generation)
    );
    assert!(
        selected_rows
            .iter()
            .all(|row| row.source_commit_short == first.source_commit_short)
    );
    assert!(
        selected_rows
            .iter()
            .all(|row| row.source_tree_state == first.source_tree_state)
    );
    assert!(
        selected_rows
            .iter()
            .all(|row| row.entry_kind == "built_in_function")
    );
    assert!(selected_rows.iter().all(|row| row.lane_id == "oxfunc"));
    assert!(
        selected_rows
            .iter()
            .all(|row| !row.source_commit_full.is_empty())
    );

    LibraryContextSnapshot {
        snapshot_id: first.snapshot_id.clone(),
        snapshot_version: format!(
            "{}+{}+{}",
            first.snapshot_generation, first.source_commit_short, first.source_tree_state
        ),
        entries: selected_rows
            .iter()
            .map(snapshot_entry_from_export_row)
            .collect(),
    }
}

fn snapshot_entry_from_export_row(row: &OxfuncExportRow) -> LibraryContextSnapshotEntry {
    LibraryContextSnapshotEntry {
        surface_name: row.canonical_surface_name.clone(),
        canonical_id: some_if_non_empty(&row.surface_stable_id),
        surface_stable_id: some_if_non_empty(&row.surface_stable_id),
        name_resolution_table_ref: some_if_non_empty(&row.name_resolution_table_ref),
        semantic_trait_profile_ref: some_if_non_empty(&row.semantic_trait_profile_ref),
        gating_profile_ref: some_if_non_empty(&row.gating_profile_ref),
        metadata_status: some_if_non_empty(&row.metadata_status),
        special_interface_kind: some_if_non_empty(&row.special_interface_kind),
        admission_interface_kind: some_if_non_empty(&row.admission_interface_kind),
        preparation_owner: some_if_non_empty(&row.preparation_owner),
        runtime_boundary_kind: some_if_non_empty(&row.runtime_boundary_kind),
        arity_shape_note: some_if_non_empty(&row.arity_shape_note),
        interface_contract_ref: some_if_non_empty(&row.interface_contract_ref),
        registration_source_kind: parse_registration_source_kind(&row.registration_source_kind),
        parse_bind_state: LibraryAvailabilityState::CatalogKnown,
        semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
        runtime_capability_state: None,
        post_dispatch_state: None,
    }
}

fn load_export_rows() -> Vec<OxfuncExportRow> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(export_csv_path())
        .expect("OxFunc W044 export CSV should exist");
    reader
        .deserialize()
        .map(|row| row.expect("W044 export row should deserialize"))
        .collect()
}

fn export_csv_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("OxFunc")
        .join("docs")
        .join("function-lane")
        .join("OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv")
}

fn some_if_non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_registration_source_kind(value: &str) -> RegistrationSourceKind {
    match value {
        "built_in_catalog_function" | "built_in_operator_export" | "doc_modeled_operator" => {
            RegistrationSourceKind::BuiltIn
        }
        other => panic!("unsupported registration source kind {other}"),
    }
}
