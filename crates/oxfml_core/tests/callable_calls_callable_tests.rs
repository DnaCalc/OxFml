//! A callable defined-name can capture another callable defined-name in its
//! closure and invoke it (composition / mutual reference). Regression coverage
//! for the previous behavior that dropped nested `DefinedNameBinding::Callable`
//! closure entries when lowering a `CallableDefinedNameBinding` to a runtime
//! lambda, which made a callable unable to call another callable.

use oxfunc_core::value::CalcValue;

use std::collections::BTreeMap;

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

#[test]
fn callable_can_invoke_another_callable_captured_in_its_closure() {
    // `Inner(y) = y * 10`, captured (by name) inside the closure of
    // `Compose(x) = Inner(x) + x`. Invoking `Compose(5)` must resolve and call
    // the captured `Inner`, yielding `Inner(5) + 5 = 50 + 5 = 55`.
    let inner = callable_binding(
        "callable-calls-callable-inner",
        "arity=1;required_arity=1;params=y;captures=-;body=binary",
        "=y*10",
        &["y"],
        BTreeMap::new(),
    );

    let mut compose_closure = BTreeMap::new();
    compose_closure.insert("Inner".to_string(), DefinedNameBinding::Callable(inner));

    let compose = callable_binding(
        "callable-calls-callable-compose",
        "arity=1;required_arity=1;params=x;captures=Inner;body=binary",
        "=Inner(x)+x",
        &["x"],
        compose_closure,
    );

    let mut host = SingleFormulaHost::new("callable-calls-callable", "=Compose(5)");
    host.set_defined_name_callable("Compose", compose);

    let output = host
        .recalc_with_backend(
            EvaluationBackend::OxFuncBacked,
            None,
            Some(&oxfml_en_us_locale_context()),
        )
        .expect("callable-calls-callable formula should evaluate");

    assert_eq!(
        output.evaluation.oxfunc_value,
        CalcValue::number(55.0),
        "captured callable should be invoked: Inner(5) + 5 = 55"
    );
}

fn callable_binding(
    case_id: &str,
    binding_summary: &str,
    body_formula: &str,
    params: &[&str],
    closure: BTreeMap<String, DefinedNameBinding>,
) -> CallableDefinedNameBinding {
    let profile = callable_profile_from_summary(binding_summary);
    let mut body_names = params
        .iter()
        .map(|name| (name.to_string(), NameKind::HelperLocal))
        .collect::<BTreeMap<_, _>>();
    for name in closure.keys() {
        body_names
            .entry(name.clone())
            .or_insert(NameKind::ValueLike);
    }

    CallableDefinedNameBinding {
        summary: binding_summary.to_string(),
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
        params: params.iter().map(|name| name.to_string()).collect(),
        optional_parameter_names: Vec::new(),
        body: bind_body_formula(case_id, body_formula, body_names),
        closure,
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
                "callable-calls-callable-struct-v1".to_string(),
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
