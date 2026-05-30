//! BF1 (fml-ds0.20): a formula whose top-level result is a callable surfaces a
//! portable callable payload (not only `#CALC!` text metadata), including the
//! captured-ref dependency facts a host needs for invalidation, and the payload
//! can be stored and re-supplied as a defined-name callable.

use oxfml_core::eval::{CallableCaptureMode, CallableOriginKind, DefinedNameBinding};
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfunc_core::value::{EvalValue, WorksheetErrorCode};

#[test]
fn top_level_lambda_with_capture_surfaces_portable_callable_payload() {
    let mut host = SingleFormulaHost::new("portable:capture", "=LAMBDA(x,MIN(x,Cap))");
    host.set_defined_name_value("Cap", EvalValue::Number(10.0));

    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    // Display stays oracle-consistent: a bare top-level LAMBDA still shows #CALC!.
    assert_eq!(
        output.published_worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Calc),
        "top-level callable display must remain #CALC! (Excel oracle)"
    );

    // But the portable payload is now surfaced instead of being discarded.
    let portable = output
        .portable_callable
        .as_ref()
        .expect("top-level callable should surface a portable payload");
    assert_eq!(
        output.evaluation.portable_callable,
        output.portable_callable
    );

    let binding = &portable.binding;
    assert_eq!(binding.params, vec!["x".to_string()]);
    assert!(binding.optional_parameter_names.is_empty());
    assert_eq!(binding.profile.arity, 1);
    assert_eq!(binding.profile.required_arity, 1);
    assert_eq!(
        binding.carrier.origin_kind,
        CallableOriginKind::HelperLambda
    );
    assert_eq!(binding.carrier.arity, 1);

    // Descriptive metadata mirrors the runtime lambda: a free top-level
    // defined-name (`Cap`) is resolved live at invocation time, so the engine
    // reports NoCapture and empty capture_names. The portable surface MUST agree
    // with `evaluation.result.callable_*` for the same value.
    assert_eq!(
        binding.carrier.capture_mode,
        CallableCaptureMode::NoCapture,
        "a free top-level defined name is not a lexical capture (engine resolves it live)"
    );
    assert!(
        binding.profile.capture_names.is_empty(),
        "capture_names must match the runtime lambda (no lexical captures)"
    );

    // Cross-check against the engine's own report for the same value.
    let runtime_carrier = output
        .evaluation
        .result
        .callable_carrier
        .as_ref()
        .expect("runtime result should carry a callable carrier");
    assert_eq!(
        binding.carrier.capture_mode, runtime_carrier.capture_mode,
        "portable capture_mode must match the runtime callable carrier"
    );
    let runtime_profile = output
        .evaluation
        .result
        .callable_profile_detail
        .as_ref()
        .expect("runtime result should carry a callable profile detail");
    assert_eq!(
        binding.profile.capture_names, runtime_profile.capture_names,
        "portable capture_names must match the runtime callable profile"
    );
    assert_eq!(
        binding.profile.body_kind, runtime_profile.body_kind,
        "portable body_kind must match the runtime callable profile"
    );

    // The captured-ref dependency fact for `Cap` is still surfaced (resolved in
    // the defining scope) so the host can wire an invalidation edge, even though
    // it is not a lexical capture.
    assert_eq!(portable.captured_refs.len(), 1);
    let captured = &portable.captured_refs[0];
    assert_eq!(captured.name, "Cap");
    assert_eq!(captured.identity, "name:Cap");
    assert_eq!(
        captured.binding,
        Some(DefinedNameBinding::Value(EvalValue::Number(10.0)))
    );

    // The closure is intentionally empty: top-level defined-name captures are
    // re-resolved live on re-supply, not snapshotted (which would shadow the
    // consumer's live value and defeat the invalidation-edge contract).
    assert!(
        binding.closure.is_empty(),
        "top-level defined-name captures must not be baked into the closure"
    );
}

#[test]
fn top_level_lambda_without_capture_reports_no_capture_mode_and_no_facts() {
    let mut host = SingleFormulaHost::new("portable:no-capture", "=LAMBDA(x,x+1)");

    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let portable = output
        .portable_callable
        .as_ref()
        .expect("top-level callable should surface a portable payload");
    assert!(
        portable.captured_refs.is_empty(),
        "a closed lambda has no captured-ref dependency facts"
    );
    assert_eq!(
        portable.binding.carrier.capture_mode,
        CallableCaptureMode::NoCapture
    );
    assert!(portable.binding.closure.is_empty());
}

#[test]
fn ordinary_top_level_result_has_no_portable_callable() {
    let mut host = SingleFormulaHost::new("portable:none", "=1+2");
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");
    assert_eq!(output.published_worksheet_value, EvalValue::Number(3.0));
    assert!(output.portable_callable.is_none());
}

#[test]
fn portable_callable_round_trips_through_defined_name_supply_and_invokes() {
    // First host: produce a portable callable from a top-level LAMBDA result.
    let mut producer = SingleFormulaHost::new("portable:rt:producer", "=LAMBDA(x,MIN(x,Cap))");
    producer.set_defined_name_value("Cap", EvalValue::Number(10.0));
    let produced = producer
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("producer recalc should succeed")
        .portable_callable
        .expect("producer should surface a portable callable");

    // The captured ref `Cap` is surfaced as a dependency fact (so the host can
    // wire an invalidation edge and re-supply its current value), NOT baked into
    // the closure.
    assert_eq!(produced.captured_refs.len(), 1);
    assert_eq!(produced.captured_refs[0].name, "Cap");
    assert!(produced.binding.closure.is_empty());

    // Second host: store the produced callable as a defined name AND re-supply
    // the captured `Cap` value into the consuming scope (the host's job, driven
    // by the captured-ref dependency facts). Invocation resolves `Cap` live.
    let mut consumer = SingleFormulaHost::new("portable:rt:consumer", "=CapFn(99)");
    consumer.set_defined_name_callable("CapFn", produced.binding.clone());
    consumer.set_defined_name_value("Cap", EvalValue::Number(10.0));
    let output = consumer
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("consumer recalc should succeed");

    // `Cap=10` resolves live in the consumer, so MIN(99,10)=10.
    assert_eq!(output.published_worksheet_value, EvalValue::Number(10.0));
}

#[test]
fn portable_callable_round_trip_resolves_capture_live_in_consuming_scope() {
    // Oracle-faithful capture model: a top-level defined name like `Cap` is
    // resolved at INVOCATION time, not at LAMBDA-definition time. Producing with
    // Cap=5 then invoking in a consumer that defines Cap=100 must see 100 (the
    // consumer's live value), NOT the producer's stale 5. This is the contract
    // that makes the captured-ref invalidation edges meaningful.
    let mut producer = SingleFormulaHost::new("portable:rt2:producer", "=LAMBDA(x,MIN(x,Cap))");
    producer.set_defined_name_value("Cap", EvalValue::Number(5.0));
    let produced = producer
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("producer recalc should succeed")
        .portable_callable
        .expect("producer should surface a portable callable");

    // No snapshot of `Cap` is baked into the closure; only the dependency fact
    // travels with the payload.
    assert!(produced.binding.closure.is_empty());
    assert_eq!(produced.captured_refs[0].name, "Cap");

    // Consumer defines a *different* live Cap; the callable must observe it.
    let mut consumer = SingleFormulaHost::new("portable:rt2:consumer", "=Capped(99)");
    consumer.set_defined_name_callable("Capped", produced.binding.clone());
    consumer.set_defined_name_value("Cap", EvalValue::Number(100.0));
    let output = consumer
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("consumer recalc should succeed");
    assert_eq!(
        output.published_worksheet_value,
        EvalValue::Number(99.0),
        "Cap resolves live to the consumer's 100, so MIN(99,100)=99 (not stale 5)"
    );
}

/// Helper: the portable carrier/profile/summary metadata must always equal the
/// engine's own report (`evaluation.result.callable_*`) for the same value.
fn assert_portable_matches_runtime(output: &oxfml_core::test_support::host::HostRecalcOutput) {
    let portable = output
        .portable_callable
        .as_ref()
        .expect("top-level callable should surface a portable payload");
    let runtime_carrier = output
        .evaluation
        .result
        .callable_carrier
        .as_ref()
        .expect("runtime result should carry a callable carrier");
    let runtime_profile = output
        .evaluation
        .result
        .callable_profile_detail
        .as_ref()
        .expect("runtime result should carry a callable profile detail");
    assert_eq!(
        &portable.binding.carrier, runtime_carrier,
        "portable carrier must match the runtime callable carrier"
    );
    assert_eq!(
        &portable.binding.profile, runtime_profile,
        "portable profile must match the runtime callable profile"
    );
    assert_eq!(
        portable.binding.summary,
        output
            .evaluation
            .result
            .callable_profile
            .clone()
            .expect("runtime result should carry a callable profile summary"),
        "portable summary must match the runtime callable summary"
    );
}

#[test]
fn unresolved_top_level_capture_surfaces_pending_name_dependency() {
    // Finding #3: when `Cap` is NOT defined, the binder lowers it to a `#NAME?`
    // error reference. The captured-ref dependency must still be surfaced keyed by
    // the original name `Cap` (not the opaque `error:#NAME?`), so a host can wire a
    // pending invalidation edge that fires when `Cap` later becomes defined.
    let mut host = SingleFormulaHost::new("portable:unresolved", "=LAMBDA(x,MIN(x,Cap))");
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let portable = output.portable_callable.as_ref().expect(
        "top-level callable should surface a portable payload even with an unresolved capture",
    );
    assert_eq!(portable.captured_refs.len(), 1);
    let captured = &portable.captured_refs[0];
    assert_eq!(captured.name, "Cap", "the original free name is preserved");
    assert_eq!(
        captured.identity, "name:Cap",
        "identity is name-based so the host can track the pending dependency"
    );
    assert_eq!(
        captured.binding, None,
        "unresolved name has no defining-scope binding"
    );
    // Still no lexical capture and no baked closure.
    assert!(portable.binding.closure.is_empty());
    assert_portable_matches_runtime(&output);
}

#[test]
fn let_returned_callable_has_no_portable_payload_narrowed_ac() {
    // Finding #4 (narrowed AC, follow-up fml-ds0.20.1): a callable that is the
    // top-level result but whose bound root is NOT a bare LAMBDA literal (here,
    // LET-returned) does not yet yield a portable payload and still publishes
    // #CALC!. Its portable `body: BoundExpr` is not statically derivable without
    // fabrication, so this is intentionally out of scope for fml-ds0.20.
    let mut host = SingleFormulaHost::new("portable:let-returned", "=LET(f,LAMBDA(x,x+1),f)");
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");
    assert!(
        output.portable_callable.is_none(),
        "LET-returned callables are intentionally not portable yet (see fml-ds0.22)"
    );
    assert_eq!(
        output.published_worksheet_value,
        EvalValue::Error(WorksheetErrorCode::Calc)
    );
}

#[test]
fn let_bodied_top_level_lambda_captures_outer_defined_name() {
    // Finding #5: a LET-bodied LAMBDA capturing an outer defined name. The LET-bound
    // local `y` is NOT a captured ref; the outer `Cap` is the only dependency fact.
    let mut host = SingleFormulaHost::new("portable:let-body", "=LAMBDA(x,LET(y,x,MIN(y,Cap)))");
    host.set_defined_name_value("Cap", EvalValue::Number(7.0));
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let portable = output
        .portable_callable
        .as_ref()
        .expect("LET-bodied LAMBDA literal should surface a portable payload");
    let names: Vec<&str> = portable
        .captured_refs
        .iter()
        .map(|captured| captured.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Cap"],
        "LET-local `y` is bound, not captured; only outer `Cap` is a dependency"
    );
    assert_portable_matches_runtime(&output);
}

#[test]
fn nested_lambda_shadows_outer_param_and_captures_outer_defined_name() {
    // Finding #5: a nested LAMBDA whose inner parameter `x` shadows the outer
    // param `x`, while the body still captures the outer defined name `Cap`. The
    // shadowed `x` must NOT appear as a captured ref; `Cap` must.
    let mut host =
        SingleFormulaHost::new("portable:nested-shadow", "=LAMBDA(x,LAMBDA(x,MIN(x,Cap)))");
    host.set_defined_name_value("Cap", EvalValue::Number(3.0));
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let portable = output
        .portable_callable
        .as_ref()
        .expect("nested-lambda body should surface a portable payload");
    let names: Vec<&str> = portable
        .captured_refs
        .iter()
        .map(|captured| captured.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Cap"],
        "shadowed inner `x` is bound; only `Cap` is captured"
    );
    assert_portable_matches_runtime(&output);
}

#[test]
fn optional_parameter_lambda_reports_arity_and_optional_names() {
    // Finding #5: optional-parameter LAMBDA. Required arity excludes the optional
    // tail; optional_parameter_names lists the optional params.
    let mut host = SingleFormulaHost::new("portable:optional", "=LAMBDA(x,[y],MIN(x,y))");
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let binding = &output
        .portable_callable
        .as_ref()
        .expect("optional-parameter LAMBDA should surface a portable payload")
        .binding;
    assert_eq!(binding.params, vec!["x".to_string(), "y".to_string()]);
    assert_eq!(binding.optional_parameter_names, vec!["y".to_string()]);
    assert_eq!(binding.profile.arity, 2);
    assert_eq!(
        binding.profile.required_arity, 1,
        "optional `y` is excluded from required arity"
    );
    assert_eq!(
        binding.profile.optional_parameter_names,
        vec!["y".to_string()]
    );
    assert_portable_matches_runtime(&output);
}

#[test]
fn array_literal_bodied_lambda_with_capture_surfaces_payload() {
    // Finding #5: array-literal body capturing an outer defined name.
    let mut host = SingleFormulaHost::new("portable:array-body", "=LAMBDA(x,{1,2,Cap})");
    host.set_defined_name_value("Cap", EvalValue::Number(9.0));
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let portable = output
        .portable_callable
        .as_ref()
        .expect("array-literal-bodied LAMBDA should surface a portable payload");
    let names: Vec<&str> = portable
        .captured_refs
        .iter()
        .map(|captured| captured.name.as_str())
        .collect();
    assert_eq!(names, vec!["Cap"]);
    assert_portable_matches_runtime(&output);
}

#[test]
fn cell_captured_reference_uses_display_identity_and_no_defined_name_binding() {
    // Finding #5: a body capturing a cell reference. A cell is not a defined name,
    // so `name`/`identity` use the reference Display form and `binding` is None
    // (the host tracks it as a cell-keyed dependency, not a defined-name lookup).
    let mut host = SingleFormulaHost::new("portable:cell-capture", "=LAMBDA(x,MIN(x,A1))");
    let output = host
        .recalc(None, Some(&oxfml_en_us_locale_context()))
        .expect("recalc should succeed");

    let portable = output
        .portable_callable
        .as_ref()
        .expect("cell-capturing LAMBDA should surface a portable payload");
    assert_eq!(portable.captured_refs.len(), 1);
    let captured = &portable.captured_refs[0];
    assert_eq!(
        captured.name, captured.identity,
        "a non-name reference uses its Display form for both name and identity"
    );
    assert!(
        !captured.identity.starts_with("name:"),
        "a cell capture is not a defined-name identity"
    );
    assert!(
        captured.identity.contains('!'),
        "a cell identity is sheet-qualified (got {})",
        captured.identity
    );
    assert_eq!(captured.binding, None);
    // No baked closure regardless of reference kind.
    assert!(portable.binding.closure.is_empty());
}
