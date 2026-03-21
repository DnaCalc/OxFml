use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use oxfml_core::binding::{BindContext, BindRequest, BoundExpr, NameKind, bind_formula};
use oxfml_core::eval::{
    CallableDefinedNameBinding, CallableValueCarrier, CallableValueProfile, DefinedNameBinding,
    EvaluationBackend,
};
use oxfml_core::host::SingleFormulaHost;
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfunc_core::locale_format::en_us_context;
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceLike};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CallableTransportFixture {
    case_id: String,
    formula: String,
    callable_name: String,
    binding_summary: String,
    callable_body_formula: String,
    callable_params: Vec<String>,
    closure_bindings: BTreeMap<String, String>,
    expected: CallableTransportExpected,
}

#[derive(Debug, Deserialize)]
struct CallableTransportExpected {
    payload_summary: String,
    prepared_call_functions: Vec<String>,
    callable_carrier: Option<CallableCarrierExpected>,
    callable_profile: Option<String>,
    callable_profile_detail: Option<CallableProfileExpected>,
}

#[derive(Debug, Deserialize)]
struct CallableCarrierExpected {
    origin_kind: String,
    invocation_model: String,
    capture_mode: String,
    arity: usize,
}

#[derive(Debug, Deserialize)]
struct CallableProfileExpected {
    arity: usize,
    parameter_names: Vec<String>,
    capture_names: Vec<String>,
    body_kind: String,
}

#[test]
fn callable_transport_fixtures_match_expected_snapshots() {
    let fixtures = load_fixtures();
    for fixture in fixtures {
        let mut host = SingleFormulaHost::new(
            format!("callable-transport-{}", fixture.case_id),
            fixture.formula.clone(),
        );
        host.set_defined_name_callable(
            fixture.callable_name.clone(),
            into_callable_binding(&fixture),
        );

        let output = host
            .recalc_with_backend(
                EvaluationBackend::LocalBootstrap,
                None,
                Some(&en_us_context()),
            )
            .expect("callable transport fixture should evaluate");

        assert_eq!(
            output.evaluation.result.payload_summary, fixture.expected.payload_summary,
            "payload summary mismatch for {}",
            fixture.case_id
        );

        let actual_prepared_functions = output
            .evaluation
            .trace
            .prepared_calls
            .iter()
            .map(|call| call.function_id.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            actual_prepared_functions, fixture.expected.prepared_call_functions,
            "prepared call sequence mismatch for {}",
            fixture.case_id
        );

        assert_eq!(
            output
                .evaluation
                .result
                .callable_carrier
                .as_ref()
                .map(callable_origin_kind_name),
            fixture
                .expected
                .callable_carrier
                .as_ref()
                .map(|carrier| carrier.origin_kind.as_str()),
            "callable origin mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_carrier
                .as_ref()
                .map(callable_invocation_model_name),
            fixture
                .expected
                .callable_carrier
                .as_ref()
                .map(|carrier| carrier.invocation_model.as_str()),
            "callable invocation model mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_carrier
                .as_ref()
                .map(callable_capture_mode_name),
            fixture
                .expected
                .callable_carrier
                .as_ref()
                .map(|carrier| carrier.capture_mode.as_str()),
            "callable capture mode mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_carrier
                .as_ref()
                .map(|carrier| carrier.arity),
            fixture
                .expected
                .callable_carrier
                .as_ref()
                .map(|carrier| carrier.arity),
            "callable arity mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output.evaluation.result.callable_profile, fixture.expected.callable_profile,
            "callable profile mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.arity),
            fixture
                .expected
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.arity),
            "callable detail arity mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.parameter_names.clone()),
            fixture
                .expected
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.parameter_names.clone()),
            "callable detail parameter names mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.capture_names.clone()),
            fixture
                .expected
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.capture_names.clone()),
            "callable detail capture names mismatch for {}",
            fixture.case_id
        );
        assert_eq!(
            output
                .evaluation
                .result
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.body_kind.clone()),
            fixture
                .expected
                .callable_profile_detail
                .as_ref()
                .map(|detail| detail.body_kind.clone()),
            "callable detail body kind mismatch for {}",
            fixture.case_id
        );
    }
}

fn load_fixtures() -> Vec<CallableTransportFixture> {
    let content = fs::read_to_string(fixture_path("callable_transport_cases.json"))
        .expect("callable transport fixture file should exist");
    serde_json::from_str(&content).expect("callable transport fixture file should deserialize")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn into_callable_binding(fixture: &CallableTransportFixture) -> CallableDefinedNameBinding {
    let profile = callable_profile_from_summary(&fixture.binding_summary);
    let mut body_names = fixture
        .callable_params
        .iter()
        .map(|name| (name.clone(), NameKind::HelperLocal))
        .collect::<BTreeMap<_, _>>();
    for name in fixture.closure_bindings.keys() {
        body_names
            .entry(name.clone())
            .or_insert(NameKind::ValueLike);
    }

    CallableDefinedNameBinding {
        summary: fixture.binding_summary.clone(),
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
        params: fixture.callable_params.clone(),
        body: bind_body_formula(&fixture.case_id, &fixture.callable_body_formula, body_names),
        closure: fixture
            .closure_bindings
            .iter()
            .map(|(name, summary)| (name.clone(), parse_defined_name_summary(summary)))
            .collect(),
    }
}

fn bind_body_formula(case_id: &str, formula: &str, names: BTreeMap<String, NameKind>) -> BoundExpr {
    let source =
        FormulaSourceRecord::new(format!("callable-body-{case_id}"), 1, formula.to_string());
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
                "callable-body-struct-v1".to_string(),
            ),
            names,
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });
    bind.bound_formula.root
}

fn parse_defined_name_summary(summary: &str) -> DefinedNameBinding {
    if let Some(target) = summary
        .strip_prefix("Reference(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return DefinedNameBinding::Reference(ReferenceLike {
            kind: oxfunc_core::value::ReferenceKind::A1,
            target: target.to_string(),
        });
    }

    DefinedNameBinding::Value(parse_eval_value_summary(summary))
}

fn parse_eval_value_summary(summary: &str) -> EvalValue {
    if let Some(number) = summary
        .strip_prefix("Number(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Number(number.parse::<f64>().expect("numeric fixture binding"));
    }

    if let Some(text) = summary
        .strip_prefix("Text(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return EvalValue::Text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        ));
    }

    if let Some(logical) = summary
        .strip_prefix("Logical(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return match logical {
            "true" | "True" | "TRUE" => EvalValue::Logical(true),
            "false" | "False" | "FALSE" => EvalValue::Logical(false),
            _ => panic!("unsupported logical fixture binding {summary}"),
        };
    }

    panic!("unsupported eval-value summary {summary}");
}

fn callable_profile_from_summary(summary: &str) -> CallableValueProfile {
    let mut arity = None;
    let mut parameter_names = None;
    let mut capture_names = None;
    let mut body_kind = None;

    for part in summary.split(';') {
        let (key, value) = part
            .split_once('=')
            .expect("callable summary entries should be key=value");
        match key {
            "arity" => arity = Some(value.parse::<usize>().expect("callable arity should parse")),
            "params" => parameter_names = Some(split_profile_list(value)),
            "captures" => capture_names = Some(split_profile_list(value)),
            "body" => body_kind = Some(value.to_string()),
            _ => {}
        }
    }

    CallableValueProfile {
        arity: arity.expect("callable arity should exist"),
        parameter_names: parameter_names.unwrap_or_default(),
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

fn callable_origin_kind_name(carrier: &oxfml_core::CallableValueCarrier) -> &'static str {
    match carrier.origin_kind {
        oxfml_core::CallableOriginKind::HelperLambda => "HelperLambda",
        oxfml_core::CallableOriginKind::DefinedNameCallable => "DefinedNameCallable",
    }
}

fn callable_invocation_model_name(carrier: &oxfml_core::CallableValueCarrier) -> &'static str {
    match carrier.invocation_model {
        oxfml_core::CallableInvocationModel::TypedInvocationOnly => "TypedInvocationOnly",
    }
}

fn callable_capture_mode_name(carrier: &oxfml_core::CallableValueCarrier) -> &'static str {
    match carrier.capture_mode {
        oxfml_core::CallableCaptureMode::NoCapture => "NoCapture",
        oxfml_core::CallableCaptureMode::LexicalCapture => "LexicalCapture",
    }
}
