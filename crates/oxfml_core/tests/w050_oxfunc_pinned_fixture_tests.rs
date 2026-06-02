use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::interface::{InMemoryLibraryContextProvider, TypedContextQueryBundle};
use oxfml_core::seam::Locus;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, run_oxfunc_preparation_adapter,
};
use oxfunc_core::value::{ExcelText, FunctionValue, WorksheetErrorCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AdmittedFixtureCase {
    scenario_id: String,
    seam_family: String,
    formula: String,
    caller_row: u32,
    caller_col: u32,
    snapshot_surface_name: String,
    cell_fixture: BTreeMap<String, String>,
    expected_value_summary: String,
    expected_returned_value_surface_kind: String,
    expected_commit_decision_kind: Option<String>,
    expected_reject_code: Option<String>,
    expected_prepared_argument_structures: Option<Vec<String>>,
    expected_prepared_argument_sources: Option<Vec<String>>,
    now_serial: Option<f64>,
    random_provider: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeferredFixtureCase {
    scenario_id: String,
    seam_family: String,
    defer_reason: String,
}

#[derive(Debug, Deserialize)]
struct ConsolidatedFixtureCorpus {
    artifact_id: String,
    authoritative_first_wave_count: usize,
    admitted_count: usize,
    deferred_count: usize,
    admitted_cases: Vec<AdmittedFixtureCase>,
    deferred_cases: Vec<DeferredFixtureCase>,
}

#[test]
fn w050_admitted_fixture_cases_match_current_adapter_floor() {
    let fixtures = load_admitted_fixtures();
    let mut failures = Vec::new();
    for fixture in fixtures {
        let provider = InMemoryLibraryContextProvider::new(test_snapshot(
            "oxfunc-libctx-v1",
            "2026-03-22",
            &fixture.snapshot_surface_name,
        ));
        let mut request = OxFuncAdapterRequest::new(
            fixture.scenario_id.clone(),
            format!("formula:{}", fixture.scenario_id),
            fixture.formula.clone(),
            locus(fixture.caller_row, fixture.caller_col),
            TypedContextQueryBundle::new(
                None,
                None,
                None,
                fixture.now_serial,
                random_provider_for_fixture(fixture.random_provider.as_deref()),
            ),
        );
        request.library_context_provider = Some(&provider);
        for (target, summary) in &fixture.cell_fixture {
            request
                .cell_fixture
                .insert(target.clone(), parse_eval_value_summary(summary));
        }

        let run = match run_oxfunc_preparation_adapter(request) {
            Ok(run) => run,
            Err(err) => {
                failures.push(format!(
                    "{} [{}] adapter failed: {err}",
                    fixture.scenario_id, fixture.seam_family
                ));
                continue;
            }
        };

        let actual_value_summary = eval_value_summary(&run.evaluation_artifact.worksheet_value);
        if actual_value_summary != fixture.expected_value_summary {
            failures.push(format!(
                "{} [{}] worksheet-value mismatch: expected {}, got {}",
                fixture.scenario_id,
                fixture.seam_family,
                fixture.expected_value_summary,
                actual_value_summary
            ));
        }
        let actual_surface_kind =
            format!("{:?}", run.evaluation_artifact.returned_value_surface.kind);
        if actual_surface_kind != fixture.expected_returned_value_surface_kind {
            failures.push(format!(
                "{} [{}] return surface mismatch: expected {}, got {}",
                fixture.scenario_id,
                fixture.seam_family,
                fixture.expected_returned_value_surface_kind,
                actual_surface_kind
            ));
        }

        if let Some(expected_structures) = &fixture.expected_prepared_argument_structures {
            let Some(prepared_call) = run.preparation_artifact.prepared_calls.first() else {
                failures.push(format!(
                    "{} [{}] expected prepared structures {:?}, but adapter emitted no prepared calls",
                    fixture.scenario_id, fixture.seam_family, expected_structures
                ));
                continue;
            };
            let actual = prepared_call
                .prepared_arguments
                .iter()
                .map(|arg| format!("{:?}", arg.structure_class))
                .collect::<Vec<_>>();
            if &actual != expected_structures {
                failures.push(format!(
                    "{} [{}] prepared structure mismatch: expected {:?}, got {:?}",
                    fixture.scenario_id, fixture.seam_family, expected_structures, actual
                ));
            }
        }

        if let Some(expected_sources) = &fixture.expected_prepared_argument_sources {
            let Some(prepared_call) = run.preparation_artifact.prepared_calls.first() else {
                failures.push(format!(
                    "{} [{}] expected prepared sources {:?}, but adapter emitted no prepared calls",
                    fixture.scenario_id, fixture.seam_family, expected_sources
                ));
                continue;
            };
            let actual = prepared_call
                .prepared_arguments
                .iter()
                .map(|arg| format!("{:?}", arg.source_class))
                .collect::<Vec<_>>();
            if &actual != expected_sources {
                failures.push(format!(
                    "{} [{}] prepared source mismatch: expected {:?}, got {:?}",
                    fixture.scenario_id, fixture.seam_family, expected_sources, actual
                ));
            }
        }

        let expected_commit_decision_kind = fixture
            .expected_commit_decision_kind
            .as_deref()
            .unwrap_or("accepted");
        if run.evaluation_artifact.commit_decision_kind != expected_commit_decision_kind {
            failures.push(format!(
                "{} [{}] commit decision mismatch: expected {}, got {}",
                fixture.scenario_id,
                fixture.seam_family,
                expected_commit_decision_kind,
                run.evaluation_artifact.commit_decision_kind
            ));
        }

        let actual_reject_code = run
            .evaluation_artifact
            .reject_code
            .map(|code| format!("{code:?}"));
        if actual_reject_code.as_deref() != fixture.expected_reject_code.as_deref() {
            failures.push(format!(
                "{} [{}] reject code mismatch: expected {:?}, got {:?}",
                fixture.scenario_id,
                fixture.seam_family,
                fixture.expected_reject_code,
                actual_reject_code
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "admitted fixtures diverged:\n{}",
        failures.join("\n")
    );
}

fn random_provider_for_fixture(
    provider: Option<&str>,
) -> Option<&'static dyn oxfunc_core::functions::rand_fn::RandomProvider> {
    match provider {
        Some("fixed_0_5") => Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_05),
        None => None,
        Some(other) => panic!("unsupported fixture random provider {other}"),
    }
}

#[test]
fn w050_deferred_fixture_register_is_explicit_for_unmapped_first_wave_cases() {
    let admitted = load_admitted_fixtures();
    let deferred = load_deferred_fixtures();
    let all_scenario_ids = [
        "A01", "A02", "A03", "A04", "A05", "A06", "A07", "A08", "A09", "A10", "B01", "B02", "B03",
        "B04", "B05", "B06", "B07", "C01", "C02", "C03", "C04", "C05", "C06", "C07", "C08", "C09",
        "C10", "C11", "C12", "C13", "C14", "D01", "D02", "D03", "D04", "D05", "D06", "E01", "E02",
        "E03", "F01", "F02", "F03", "F04", "F05",
    ];

    let mut seen = std::collections::BTreeSet::new();
    for scenario_id in admitted
        .iter()
        .map(|case| case.scenario_id.as_str())
        .chain(deferred.iter().map(|case| case.scenario_id.as_str()))
    {
        assert!(
            seen.insert(scenario_id.to_string()),
            "duplicate scenario coverage entry for {scenario_id}"
        );
    }
    for scenario_id in all_scenario_ids {
        assert!(
            seen.contains(scenario_id),
            "missing pinned first-wave scenario coverage for {scenario_id}"
        );
    }
    assert!(
        deferred
            .iter()
            .all(|case| !case.defer_reason.trim().is_empty() && !case.seam_family.trim().is_empty()),
        "every deferred entry should explain why it is outside the current admitted floor"
    );
}

#[test]
fn w050_consolidated_fixture_corpus_matches_split_registers() {
    let admitted = load_admitted_fixtures();
    let deferred = load_deferred_fixtures();
    let corpus = load_consolidated_corpus();

    assert_eq!(
        corpus.artifact_id, "w050.oxfunc.pinned_fixture_corpus.v1",
        "unexpected consolidated corpus id"
    );
    assert_eq!(
        corpus.authoritative_first_wave_count, 45,
        "consolidated corpus should reflect the authoritative 45-scenario first wave"
    );
    assert_eq!(
        corpus.admitted_count,
        admitted.len(),
        "consolidated corpus admitted-count drifted from split admitted register"
    );
    assert_eq!(
        corpus.deferred_count,
        deferred.len(),
        "consolidated corpus deferred-count drifted from split deferred register"
    );

    let admitted_ids = admitted
        .iter()
        .map(|case| case.scenario_id.as_str())
        .collect::<Vec<_>>();
    let corpus_admitted_ids = corpus
        .admitted_cases
        .iter()
        .map(|case| case.scenario_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        corpus_admitted_ids, admitted_ids,
        "consolidated corpus admitted cases drifted from split admitted register"
    );

    let deferred_ids = deferred
        .iter()
        .map(|case| case.scenario_id.as_str())
        .collect::<Vec<_>>();
    let corpus_deferred_ids = corpus
        .deferred_cases
        .iter()
        .map(|case| case.scenario_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        corpus_deferred_ids, deferred_ids,
        "consolidated corpus deferred cases drifted from split deferred register"
    );
}

fn load_admitted_fixtures() -> Vec<AdmittedFixtureCase> {
    let mut path = fixture_dir();
    path.push("w050_oxfunc_admitted_fixture_cases.json");
    let content = fs::read_to_string(&path).expect("admitted fixture file should exist");
    serde_json::from_str(&content).expect("admitted fixture file should deserialize")
}

fn load_deferred_fixtures() -> Vec<DeferredFixtureCase> {
    let mut path = fixture_dir();
    path.push("w050_oxfunc_deferred_fixture_register.json");
    let content = fs::read_to_string(&path).expect("deferred fixture register should exist");
    serde_json::from_str(&content).expect("deferred fixture register should deserialize")
}

fn load_consolidated_corpus() -> ConsolidatedFixtureCorpus {
    let mut path = fixture_dir();
    path.push("w050_oxfunc_pinned_fixture_corpus.json");
    let content = fs::read_to_string(&path).expect("consolidated fixture corpus should exist");
    serde_json::from_str(&content).expect("consolidated fixture corpus should deserialize")
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn parse_eval_value_summary(summary: &str) -> FunctionValue {
    if let Some(number) = summary
        .strip_prefix("Number(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return FunctionValue::Number(number.parse().expect("numeric summary should parse"));
    }
    if let Some(text) = summary
        .strip_prefix("Text(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return FunctionValue::Text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ));
    }
    if let Some(logical) = summary
        .strip_prefix("Logical(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return FunctionValue::Logical(matches!(logical, "TRUE" | "True" | "true"));
    }
    if let Some(code) = summary
        .strip_prefix("Error(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let code = match code {
            "#DIV/0!" => WorksheetErrorCode::Div0,
            "#VALUE!" => WorksheetErrorCode::Value,
            "#NAME?" => WorksheetErrorCode::Name,
            "#N/A" => WorksheetErrorCode::NA,
            "#REF!" => WorksheetErrorCode::Ref,
            other => panic!("unsupported error summary {other}"),
        };
        return FunctionValue::Error(code);
    }

    panic!("unsupported cell summary {summary}");
}

fn eval_value_summary(value: &FunctionValue) -> String {
    match value {
        FunctionValue::Number(number) => format!("Number({number})"),
        FunctionValue::Text(text) => format!("Text({})", text.to_string_lossy()),
        FunctionValue::Logical(value) => format!("Logical({value})"),
        FunctionValue::Error(code) => format!("Error({code:?})"),
        FunctionValue::Array(array) => {
            let shape = array.shape();
            format!("Array({}x{})", shape.rows, shape.cols)
        }
        FunctionValue::Reference(reference) => format!("Reference({})", reference.target()),
        other => format!("Unsupported({other:?})"),
    }
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

fn test_snapshot(
    snapshot_id: &str,
    snapshot_version: &str,
    surface_name: &str,
) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: snapshot_id.to_string(),
        snapshot_version: snapshot_version.to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: surface_name.to_string(),
            canonical_id: Some(format!("FUNC.{surface_name}")),
            surface_stable_id: Some(format!("surface:{surface_name}")),
            name_resolution_table_ref: Some("name-table:v1".to_string()),
            semantic_trait_profile_ref: Some("traits:v1".to_string()),
            gating_profile_ref: Some("gating:v1".to_string()),
            metadata_status: Some("runtime".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("ordinary".to_string()),
            preparation_owner: Some("oxfunc".to_string()),
            runtime_boundary_kind: Some("ordinary_eval".to_string()),
            interface_contract_ref: Some("iface:v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}
