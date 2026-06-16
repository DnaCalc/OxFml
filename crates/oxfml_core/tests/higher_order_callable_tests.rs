use oxfunc_core::value::{CalcValue, CoreValue};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::binding::{BindContext, BindRequest, BoundExpr, NameKind, bind_formula};
use oxfml_core::eval::{
    CallableDefinedNameBinding, CallableValueCarrier, CallableValueProfile, DefinedNameBinding,
    EvaluationBackend,
};
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfunc_core::value::{ExcelText, ReferenceLike};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HigherOrderCallableFixture {
    case_id: String,
    formula: String,
    callable_name: Option<String>,
    binding_summary: Option<String>,
    callable_body_formula: Option<String>,
    callable_params: Option<Vec<String>>,
    #[serde(default)]
    closure_bindings: BTreeMap<String, String>,
    expected: HigherOrderCallableExpected,
}

#[derive(Debug, Deserialize)]
struct HigherOrderCallableExpected {
    payload_summary: Option<String>,
    evaluation_error_contains: Option<String>,
    #[serde(default)]
    array_numbers: Vec<f64>,
    #[serde(default)]
    array_logicals: Vec<bool>,
}

#[test]
fn higher_order_callable_fixtures_match_expected_snapshots() {
    let fixtures = load_fixtures();
    for fixture in fixtures {
        let mut host = SingleFormulaHost::new(
            format!("higher-order-callable-{}", fixture.case_id),
            fixture.formula.clone(),
        );
        if let Some(callable_name) = &fixture.callable_name {
            host.set_defined_name_callable(callable_name.clone(), into_callable_binding(&fixture));
        }

        let output = host.recalc_with_backend(
            EvaluationBackend::OxFuncBacked,
            None,
            Some(&oxfml_en_us_locale_context()),
        );

        if let Some(expected_message) = &fixture.expected.evaluation_error_contains {
            let error = output.expect_err("fixture should fail during evaluation");
            assert!(
                error.contains(expected_message),
                "evaluation error mismatch for {}: expected substring {expected_message:?}, got {:?}",
                fixture.case_id,
                error
            );
            continue;
        }

        let output = output.expect("higher-order callable fixture should evaluate");

        assert_eq!(
            output.evaluation.result.payload_summary,
            fixture
                .expected
                .payload_summary
                .as_deref()
                .expect("payload summary should exist for successful fixtures"),
            "payload summary mismatch for {}",
            fixture.case_id
        );

        if !fixture.expected.array_numbers.is_empty() {
            assert_eq!(
                array_numbers(&output.evaluation.oxfunc_value),
                fixture.expected.array_numbers,
                "numeric array mismatch for {}",
                fixture.case_id
            );
        }

        if !fixture.expected.array_logicals.is_empty() {
            assert_eq!(
                array_logicals(&output.evaluation.oxfunc_value),
                fixture.expected.array_logicals,
                "logical array mismatch for {}",
                fixture.case_id
            );
        }
    }
}

fn load_fixtures() -> Vec<HigherOrderCallableFixture> {
    let content = fs::read_to_string(fixture_path("higher_order_callable_cases.json"))
        .expect("higher-order callable fixture file should exist");
    serde_json::from_str(&content).expect("higher-order callable fixture file should deserialize")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn into_callable_binding(fixture: &HigherOrderCallableFixture) -> CallableDefinedNameBinding {
    let summary = fixture
        .binding_summary
        .as_ref()
        .expect("callable binding summary should exist");
    let callable_body_formula = fixture
        .callable_body_formula
        .as_ref()
        .expect("callable body formula should exist");
    let callable_params = fixture
        .callable_params
        .as_ref()
        .expect("callable params should exist");
    let profile = callable_profile_from_summary(summary);
    let mut body_names = callable_params
        .iter()
        .map(|name| (name.clone(), NameKind::HelperLocal))
        .collect::<BTreeMap<_, _>>();
    for name in fixture.closure_bindings.keys() {
        body_names
            .entry(name.clone())
            .or_insert(NameKind::ValueLike);
    }

    CallableDefinedNameBinding {
        summary: summary.clone(),
        carrier: CallableValueCarrier {
            origin_kind: oxfml_core::CallableOriginKind::HelperLambda,
            invocation_model: oxfml_core::CallableInvocationModel::TypedInvocationOnly,
            capture_mode: if profile.capture_names.is_empty() {
                oxfml_core::CallableCaptureMode::NoCapture
            } else {
                oxfml_core::CallableCaptureMode::LexicalCapture
            },
            arity: profile.arity,
        },
        profile,
        params: callable_params.clone(),
        optional_parameter_names: Vec::new(),
        body: bind_body_formula(&fixture.case_id, callable_body_formula, body_names),
        closure: fixture
            .closure_bindings
            .iter()
            .map(|(name, summary)| (name.clone(), parse_defined_name_summary(summary)))
            .collect(),
    }
}

fn bind_body_formula(case_id: &str, formula: &str, names: BTreeMap<String, NameKind>) -> BoundExpr {
    let source = FormulaSourceRecord::new(
        format!("higher-order-callable-body-{case_id}"),
        1,
        formula.to_string(),
    );
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
                "higher-order-callable-struct-v1".to_string(),
            ),
            names,
            formula_token: source.formula_token(),
            ..BindContext::default()
        },

        host_name_resolver: None,

        reference_bind_profile: None,
    });
    bind.bound_formula.root
}

fn parse_defined_name_summary(summary: &str) -> DefinedNameBinding {
    if let Some(target) = summary
        .strip_prefix("Reference(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return DefinedNameBinding::Reference(ReferenceLike::new(
            oxfunc_core::value::ReferenceKind::A1,
            target,
        ));
    }

    DefinedNameBinding::Value(parse_eval_value_summary(summary))
}

fn parse_eval_value_summary(summary: &str) -> CalcValue {
    if let Some(number) = summary
        .strip_prefix("Number(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return CalcValue::number(number.parse::<f64>().expect("numeric fixture binding"));
    }

    if let Some(text) = summary
        .strip_prefix("Text(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return CalcValue::text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ));
    }

    if let Some(logical) = summary
        .strip_prefix("Logical(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return match logical {
            "true" | "True" | "TRUE" => CalcValue::logical(true),
            "false" | "False" | "FALSE" => CalcValue::logical(false),
            _ => panic!("unsupported logical fixture binding {summary}"),
        };
    }

    panic!("unsupported eval-value summary {summary}");
}

fn callable_profile_from_summary(summary: &str) -> CallableValueProfile {
    let mut arity = None;
    let mut required_arity = None;
    let mut parameter_names = None;
    let mut optional_parameter_names = None;
    let mut capture_names = None;
    let mut body_kind = None;

    for part in summary.split(';') {
        let (key, value) = part
            .split_once('=')
            .expect("callable summary entries should be key=value");
        match key {
            "arity" => arity = Some(value.parse::<usize>().expect("callable arity should parse")),
            "required_arity" => {
                required_arity = Some(
                    value
                        .parse::<usize>()
                        .expect("callable required arity should parse"),
                )
            }
            "params" => parameter_names = Some(split_profile_list(value)),
            "optional_params" => optional_parameter_names = Some(split_profile_list(value)),
            "captures" => capture_names = Some(split_profile_list(value)),
            "body" => body_kind = Some(value.to_string()),
            _ => {}
        }
    }

    let arity = arity.expect("callable arity should exist");
    CallableValueProfile {
        arity,
        required_arity: required_arity.unwrap_or(arity),
        parameter_names: parameter_names.unwrap_or_default(),
        optional_parameter_names: optional_parameter_names.unwrap_or_default(),
        capture_names: capture_names.unwrap_or_default(),
        body_kind: body_kind.expect("callable body kind should exist"),
    }
}

fn split_profile_list(value: &str) -> Vec<String> {
    if value == "-" || value.is_empty() {
        Vec::new()
    } else if value.contains('|') {
        value.split('|').map(|item| item.to_string()).collect()
    } else {
        value.split(',').map(|item| item.to_string()).collect()
    }
}

fn array_numbers(value: &CalcValue) -> Vec<f64> {
    let CoreValue::Array(array) = value.core() else {
        panic!("expected array result, got {value:?}");
    };
    array
        .iter_row_major()
        .map(|cell| match cell.core() {
            CoreValue::Number(number) => *number,
            other => panic!("expected numeric array cell, got {other:?}"),
        })
        .collect()
}

fn array_logicals(value: &CalcValue) -> Vec<bool> {
    let CoreValue::Array(array) = value.core() else {
        panic!("expected array result, got {value:?}");
    };
    array
        .iter_row_major()
        .map(|cell| match cell.core() {
            CoreValue::Logical(value) => *value,
            other => panic!("expected logical array cell, got {other:?}"),
        })
        .collect()
}
