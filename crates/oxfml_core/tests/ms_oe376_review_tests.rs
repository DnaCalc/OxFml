use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MsOe376ReviewCase {
    family_id: String,
    coverage_classification: String,
    follow_on_owner: String,
    requires_enclosing_table_context: bool,
    same_external_book_only: bool,
    formula_channel_kind: String,
    restriction_profile_id: String,
    source_anchor_ids: Vec<String>,
}

#[test]
fn ms_oe376_review_fixture_covers_all_w031_families() {
    let cases: Vec<MsOe376ReviewCase> = load_fixture("ms_oe376_review_cases.json");

    let family_ids = cases
        .iter()
        .map(|case| case.family_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        family_ids,
        BTreeSet::from([
            "conditional_formatting_formulas",
            "data_validation_formulas",
            "external_name_formulas",
            "formula_significant_table_and_formatting_surfaces",
            "name_formulas",
            "r1c1_formulas",
            "structured_references",
        ])
    );

    for case in &cases {
        assert!(!case.follow_on_owner.is_empty());
        assert!(!case.formula_channel_kind.is_empty());
        assert!(!case.restriction_profile_id.is_empty());
        assert!(!case.source_anchor_ids.is_empty());
        assert!(case.source_anchor_ids.iter().all(|id| !id.is_empty()));
    }

    let structured = find_case(&cases, "structured_references");
    assert_eq!(structured.coverage_classification, "partial");
    assert_eq!(structured.follow_on_owner, "W036");
    assert!(structured.requires_enclosing_table_context);
    assert_eq!(structured.formula_channel_kind, "WorksheetCarrier");

    let cf = find_case(&cases, "conditional_formatting_formulas");
    assert_eq!(cf.coverage_classification, "partial_to_missing");
    assert_eq!(cf.follow_on_owner, "W039");
    assert_eq!(cf.restriction_profile_id, "cf_restricted_not_equal_to_dv");

    let dv = find_case(&cases, "data_validation_formulas");
    assert_eq!(dv.coverage_classification, "missing");
    assert_eq!(dv.follow_on_owner, "W039");
    assert_eq!(dv.restriction_profile_id, "dv_restricted_not_equal_to_cf");
    assert_ne!(cf.restriction_profile_id, dv.restriction_profile_id);

    let external = find_case(&cases, "external_name_formulas");
    assert_eq!(external.coverage_classification, "missing_to_partial");
    assert_eq!(external.follow_on_owner, "W038");
    assert!(external.same_external_book_only);
    assert_eq!(external.formula_channel_kind, "ExternalNameCarrier");

    let r1c1 = find_case(&cases, "r1c1_formulas");
    assert_eq!(r1c1.coverage_classification, "missing");
    assert_eq!(r1c1.follow_on_owner, "W037");
    assert_eq!(r1c1.formula_channel_kind, "R1C1Channel");
}

fn find_case<'a>(cases: &'a [MsOe376ReviewCase], family_id: &str) -> &'a MsOe376ReviewCase {
    cases
        .iter()
        .find(|case| case.family_id == family_id)
        .unwrap_or_else(|| panic!("missing family fixture for {family_id}"))
}

fn load_fixture<T: for<'de> Deserialize<'de>>(file_name: &str) -> T {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(file_name);
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}
