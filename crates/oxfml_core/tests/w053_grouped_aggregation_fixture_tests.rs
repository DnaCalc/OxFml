use std::fs;
use std::path::PathBuf;

use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::seam::{Locus, RejectCode};
use oxfml_core::substrate::oxfunc_adapter::{OxFuncAdapterRequest, run_oxfunc_preparation_adapter};
use oxfunc_core::host_info::{
    HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ResolvedWebImage,
};
use oxfunc_core::value::ExcelText;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct W053FixtureCase {
    case_id: String,
    formula: String,
    caller_row: u32,
    caller_col: u32,
    host_query_profile: Option<String>,
    expected_mode: String,
    expected_result_payload_summary: Option<String>,
    expected_surface_kind: Option<String>,
    expected_reject_code: Option<String>,
    expected_bind_message: Option<String>,
    expected_error_contains: Option<String>,
    expected_rich_value_type_name: Option<String>,
}

#[test]
fn w053_grouped_aggregation_fixture_cases_match_current_adapter_floor() {
    for case in load_fixture_cases() {
        let host_info = match case.host_query_profile.as_deref() {
            Some("image_success") => Some(&ImageFixtureHostInfoProvider as &dyn HostInfoProvider),
            None => None,
            other => panic!("unexpected host query profile: {other:?}"),
        };

        let request = OxFuncAdapterRequest::new(
            case.case_id.clone(),
            format!("formula:{}", case.case_id),
            case.formula.clone(),
            locus(case.caller_row, case.caller_col),
            TypedContextQueryBundle::new(host_info, None, None, None, None),
        );

        match case.expected_mode.as_str() {
            "success" => {
                let run = run_oxfunc_preparation_adapter(request)
                    .expect("fixture success case should run");
                assert_eq!(
                    Some(
                        run.evaluation_artifact
                            .evaluation_result
                            .payload_summary
                            .as_str()
                    ),
                    case.expected_result_payload_summary.as_deref(),
                    "unexpected payload summary for {}",
                    case.case_id
                );
                assert_eq!(
                    Some(format!(
                        "{:?}",
                        run.evaluation_artifact.returned_value_surface.kind
                    )),
                    case.expected_surface_kind.clone(),
                    "unexpected returned surface kind for {}",
                    case.case_id
                );
                assert_eq!(
                    run.evaluation_artifact
                        .returned_value_surface
                        .rich_value_type_name
                        .as_deref(),
                    case.expected_rich_value_type_name.as_deref(),
                    "unexpected rich value type name for {}",
                    case.case_id
                );
            }
            "reject" => {
                let run = run_oxfunc_preparation_adapter(request)
                    .expect("fixture reject case should still produce artifact");
                assert_eq!(
                    run.evaluation_artifact.commit_decision_kind, "rejected",
                    "unexpected commit decision kind for {}",
                    case.case_id
                );
                assert_eq!(
                    run.evaluation_artifact.reject_code,
                    case.expected_reject_code.as_deref().map(parse_reject_code),
                    "unexpected reject code for {}",
                    case.case_id
                );
                assert!(
                    run.preparation_artifact
                        .bind_diagnostics
                        .iter()
                        .any(|diagnostic| Some(diagnostic.message.as_str())
                            == case.expected_bind_message.as_deref()),
                    "missing expected bind diagnostic for {}",
                    case.case_id
                );
            }
            "adapter_error" => {
                let error =
                    run_oxfunc_preparation_adapter(request).expect_err("expected adapter error");
                assert!(
                    case.expected_error_contains
                        .as_deref()
                        .is_some_and(|needle| error.contains(needle)),
                    "unexpected adapter error for {}: {}",
                    case.case_id,
                    error
                );
            }
            other => panic!("unexpected expected_mode: {other}"),
        }
    }
}

fn load_fixture_cases() -> Vec<W053FixtureCase> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("w053_grouped_aggregation_cases.json");
    let content = fs::read_to_string(path).expect("fixture file should exist");
    serde_json::from_str(&content).expect("fixture file should deserialize")
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

fn parse_reject_code(value: &str) -> RejectCode {
    match value {
        "BindMismatch" => RejectCode::BindMismatch,
        other => panic!("unexpected reject code string: {other}"),
    }
}

struct ImageFixtureHostInfoProvider;

impl HostInfoProvider for ImageFixtureHostInfoProvider {
    fn query_image(&self, request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        assert_eq!(
            request.source.to_string_lossy(),
            "https://example.com/sphere.png"
        );
        Ok(ImageProviderResult::Image(ResolvedWebImage {
            web_image_identifier: "img-1".to_string(),
            published_fallback: ExcelText::from_interop_assignment("-2146826273"),
        }))
    }
}
