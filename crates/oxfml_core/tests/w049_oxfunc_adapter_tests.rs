use oxfml_core::interface::{
    HostProviderOutcomeKind, InMemoryLibraryContextProvider, LibraryContextSnapshotRef,
    ReturnedValueSurfaceKind, TypedContextQueryBundle, TypedContextQueryFamily,
};
use oxfml_core::seam::Locus;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::test_support::oxfunc_adapter::{
    OxFuncAdapterRequest, OxFuncMismatchOwnerGuess, run_oxfunc_preparation_adapter,
};
use oxfml_core::{
    ExecutionOutcomeKind, ExecutionOutcomeStage, PreparedSourceClass, PreparedStructureClass,
};
use oxfunc_core::functions::call_register_id_family::{
    RegisterIdRequest, RegisteredExternalOriginKind, RegisteredExternalProvider,
    RegisteredExternalProviderError, RegisteredExternalTarget, RegisteredProcedureSpec,
};
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{
    HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, ResolvedWebImage,
};
use oxfunc_core::value::{CallArgValue, CellStyleHint, EvalValue, ExcelText, PresentationHint};

#[test]
fn adapter_projects_direct_scalar_and_array_like_preparation_artifacts() {
    let scalar_request = OxFuncAdapterRequest::new(
        "sum-direct-scalars",
        "formula:sum-direct-scalars",
        "=SUM(1,2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    );
    let scalar_run = run_oxfunc_preparation_adapter(scalar_request).expect("scalar adapter run");

    assert_eq!(
        scalar_run.preparation_artifact.prepared_calls[0].function_name,
        "SUM"
    );
    assert_eq!(
        scalar_run.preparation_artifact.prepared_calls[0]
            .prepared_arguments
            .iter()
            .map(|arg| arg.structure_class)
            .collect::<Vec<_>>(),
        vec![
            PreparedStructureClass::DirectScalar,
            PreparedStructureClass::DirectScalar,
        ]
    );
    assert_eq!(
        scalar_run
            .evaluation_artifact
            .evaluation_result
            .payload_summary,
        "Number(3)"
    );
    assert_eq!(
        scalar_run
            .evaluation_artifact
            .execution_outcome_surface
            .outcome_kind,
        ExecutionOutcomeKind::ExecutedResult
    );
    assert_eq!(
        scalar_run
            .evaluation_artifact
            .execution_outcome_surface
            .outcome_stage,
        ExecutionOutcomeStage::Executed
    );

    let mut array_like_request = OxFuncAdapterRequest::new(
        "sum-area-argument",
        "formula:sum-area-argument",
        "=SUM(A1:A2)",
        locus(1, 2),
        TypedContextQueryBundle::default(),
    );
    array_like_request
        .cell_fixture
        .insert("A1".to_string(), EvalValue::Number(10.0));
    array_like_request
        .cell_fixture
        .insert("A2".to_string(), EvalValue::Number(20.0));
    let array_like_run =
        run_oxfunc_preparation_adapter(array_like_request).expect("array-like adapter run");

    let prepared_argument =
        &array_like_run.preparation_artifact.prepared_calls[0].prepared_arguments[0];
    assert_eq!(
        prepared_argument.structure_class,
        PreparedStructureClass::ArrayLike
    );
    assert_eq!(
        prepared_argument.source_class,
        PreparedSourceClass::AreaReference
    );
    assert_eq!(
        array_like_run
            .evaluation_artifact
            .evaluation_result
            .payload_summary,
        "Number(30)"
    );
}

#[test]
fn adapter_executes_text_with_scientific_format_pattern_ftc_0655() {
    let locale = oxfml_core::format::en_us_context();
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0655-scientific-text",
        "formula:foundation:FTC-0655",
        "=TEXT(12345.6789,\"0.00E+00\")",
        locus(1, 1),
        TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
    ))
    .expect("FTC-0655 adapter run should succeed");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("1.23E+04"))
    );
    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Text(1.23E+04)"
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.payload_summary,
        "Text"
    );
    assert!(
        run.preparation_artifact
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::LocaleFormatContext)
    );
}

#[test]
fn adapter_preserves_randarray_width_for_columns_ftc_0505_without_explicit_random_bundle() {
    let locale = oxfml_core::format::en_us_context();
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "ftc-0505-randarray-width",
        "formula:foundation:FTC-0505",
        "=COLUMNS(RANDARRAY(5,3))",
        locus(1, 1),
        TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
    ))
    .expect("FTC-0505 adapter run should succeed");

    assert_eq!(run.evaluation_artifact.worksheet_value, EvalValue::Number(3.0));
    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Number(3)"
    );
    assert!(
        run.preparation_artifact
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::RandomValue)
    );
    assert_eq!(
        run.preparation_artifact.prepared_calls[1].prepared_arguments[0].structure_class,
        PreparedStructureClass::ArrayLike
    );
}

#[test]
fn adapter_handles_unary_negative_literals_in_ordinary_calls() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "sign-negative-literal",
        "formula:sign-negative-literal",
        "=SIGN(-5)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("sign adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(-1.0)
    );
}

#[test]
fn adapter_treats_absent_single_cell_reference_as_blank_stand_in() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "blank-single-cell-standin",
        "formula:blank-single-cell-standin",
        "=ISBLANK(A9)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("blank stand-in adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Logical(true)
    );
}

#[test]
fn adapter_respects_requested_snapshot_and_caller_anchor() {
    let current_snapshot = test_snapshot("snapshot:current", "v1", "SUM");
    let selected_snapshot = test_snapshot("snapshot:selected", "v2", "ROW");
    let provider = InMemoryLibraryContextProvider::with_snapshots(
        LibraryContextSnapshotRef::from(&current_snapshot),
        vec![current_snapshot, selected_snapshot.clone()],
    );

    let mut request = OxFuncAdapterRequest::new(
        "row-caller-anchor",
        "formula:row-caller-anchor",
        "=ROW()",
        locus(7, 3),
        TypedContextQueryBundle::default(),
    );
    request.library_context_provider = Some(&provider);
    request.library_context_snapshot_ref =
        Some(LibraryContextSnapshotRef::from(&selected_snapshot));
    request.host_query_capability_profile = Some("caller-row-only".to_string());

    let run = run_oxfunc_preparation_adapter(request).expect("row adapter run");

    assert_eq!(
        run.preparation_artifact.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::from(&selected_snapshot))
    );
    assert_eq!(run.preparation_artifact.caller_anchor.row, 7);
    assert_eq!(run.preparation_artifact.caller_anchor.col, 3);
    assert_eq!(
        run.preparation_artifact
            .host_query_capability_profile
            .as_deref(),
        Some("caller-row-only")
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(7.0)
    );
}

#[test]
fn adapter_surfaces_typed_rtd_outcome_and_bundle_spec() {
    let provider = InMemoryLibraryContextProvider::new(test_snapshot("snapshot:rtd", "v1", "RTD"));
    let mut request = OxFuncAdapterRequest::new(
        "rtd-capability-denied",
        "formula:rtd-capability-denied",
        "=RTD(\"prog\",\"server\",\"topic\")",
        locus(1, 1),
        TypedContextQueryBundle::new(None, Some(&CapabilityDeniedRtdProvider), None, None, None),
    );
    request.library_context_provider = Some(&provider);

    let run = run_oxfunc_preparation_adapter(request).expect("rtd adapter run");

    assert!(
        run.preparation_artifact
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::Rtd)
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .map(|surface| surface.outcome_kind),
        Some(HostProviderOutcomeKind::CapabilityDenied)
    );
    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Error(Blocked)"
    );
}

#[test]
fn adapter_preserves_hyperlink_publication_intent() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "hyperlink-publication-intent",
        "formula:hyperlink-publication-intent",
        "=HYPERLINK(\"https://example.com\",\"Go\")",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("hyperlink adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("Go"))
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::ValueWithPresentation
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .presentation_hint,
        Some(PresentationHint::style(CellStyleHint::Hyperlink))
    );
}

#[test]
fn adapter_preserves_image_rich_value_surface() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "image-rich-value",
        "formula:image-rich-value",
        "=IMAGE(\"https://example.com/sphere.png\",\"Sphere\")",
        locus(1, 1),
        TypedContextQueryBundle::new(Some(&ImageHostInfoProvider), None, None, None, None),
    ))
    .expect("image adapter run");

    assert!(
        run.preparation_artifact
            .typed_query_bundle_spec
            .families
            .contains(&TypedContextQueryFamily::Image)
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Text(ExcelText::from_interop_assignment("-2146826273"))
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::RichValue
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .rich_value_type_name
            .as_deref(),
        Some("_webimage")
    );
}

#[test]
fn adapter_projects_image_capability_denied_as_blocked_error() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "image-capability-denied",
        "formula:image-capability-denied",
        "=IMAGE(\"https://example.com/sphere.png\")",
        locus(1, 1),
        TypedContextQueryBundle::new(Some(&BlockedImageHostInfoProvider), None, None, None, None),
    ))
    .expect("blocked image adapter run");

    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Blocked)
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::OrdinaryValue
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .payload_summary,
        "Error(Blocked)"
    );
}

#[test]
fn adapter_projects_registered_external_requests_for_register_id_and_call() {
    let provider = RecordingRegisteredExternalProvider::default();

    let register_run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "register-id-request",
        "formula:register-id-request",
        "=REGISTER.ID(\"Kernel32\",\"GetTickCount\",\"J!\")",
        locus(1, 1),
        TypedContextQueryBundle::default().with_registered_external_provider(Some(&provider)),
    ))
    .expect("register.id adapter run");

    assert_eq!(
        register_run.preparation_artifact.prepared_calls[0]
            .register_id_request
            .as_ref(),
        Some(&RegisterIdRequest {
            library_name: ExcelText::from_interop_assignment("Kernel32"),
            procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment(
                "GetTickCount"
            )),
            declared_type_text: Some(ExcelText::from_interop_assignment("J!")),
        })
    );

    let call_run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "call-request",
        "formula:call-request",
        "=CALL(4242,6,7,3)",
        locus(1, 1),
        TypedContextQueryBundle::default().with_registered_external_provider(Some(&provider)),
    ))
    .expect("call adapter run");

    match call_run.preparation_artifact.prepared_calls[0]
        .registered_external_call_request
        .as_ref()
        .expect("normalized call request")
    {
        oxfml_core::RegisteredExternalCallRequest {
            target: RegisteredExternalTarget::RegisterId(register_id),
            invocation_args,
        } => {
            assert_eq!(*register_id, 4242.0);
            assert_eq!(
                invocation_args,
                &vec![
                    CallArgValue::Eval(EvalValue::Number(6.0)),
                    CallArgValue::Eval(EvalValue::Number(7.0)),
                    CallArgValue::Eval(EvalValue::Number(3.0)),
                ]
            );
        }
        other => panic!("unexpected normalized call request: {other:?}"),
    }
}

#[test]
fn adapter_builds_structured_mismatch_artifact() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "mismatch-seed",
        "formula:mismatch-seed",
        "=SUM(1,2)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("mismatch seed run");

    let mismatch = run.mismatch_artifact(
        "prepared_argument_family",
        "prepared_calls",
        OxFuncMismatchOwnerGuess::SharedFreezeNeeded,
        "expected DirectScalar/ArrayLike split",
        "observed narrower prepared-call field family",
        Some("narrow field name mismatch only".to_string()),
    );

    assert_eq!(mismatch.fixture_case_id, "mismatch-seed");
    assert_eq!(mismatch.expected_seam_family, "prepared_argument_family");
    assert_eq!(mismatch.failing_packet_family, "prepared_calls");
    assert_eq!(
        mismatch.owner_guess,
        OxFuncMismatchOwnerGuess::SharedFreezeNeeded
    );
    assert_eq!(
        mismatch.detail.as_deref(),
        Some("narrow field name mismatch only")
    );
}

#[test]
fn adapter_preserves_internal_lambda_but_publishes_calc_for_bare_lambda() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "lambda-publication",
        "formula:lambda-publication",
        "=LAMBDA(x,x+1)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("lambda publication adapter run");

    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=x;optional_params=-;captures=-;body=Binary)"
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Calc)
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::OrdinaryValue
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .payload_summary,
        "Error(Calc)"
    );
}

#[test]
fn adapter_preserves_internal_lambda_but_publishes_calc_for_helper_bound_returned_lambda() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "helper-returned-lambda-publication",
        "formula:helper-returned-lambda-publication",
        "=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),adder(5))",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("helper returned lambda adapter run");

    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Lambda(arity=1;required_arity=1;params=x;optional_params=-;captures=n;body=Binary)"
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Calc)
    );
    assert_eq!(
        run.evaluation_artifact.returned_value_surface.kind,
        ReturnedValueSurfaceKind::OrdinaryValue
    );
    assert_eq!(
        run.evaluation_artifact
            .returned_value_surface
            .payload_summary,
        "Error(Calc)"
    );
    assert_eq!(
        run.preparation_artifact
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec![
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "SPECIAL.LAMBDA",
            "SPECIAL.LET",
        ]
    );
}

#[test]
fn adapter_executes_helper_bound_returned_lambda_invocation() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "helper-returned-lambda-invocation",
        "formula:helper-returned-lambda-invocation",
        "=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("helper returned lambda invocation adapter run");

    assert_eq!(
        run.evaluation_artifact.evaluation_result.payload_summary,
        "Number(15)"
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Number(15.0)
    );
    assert_eq!(
        run.preparation_artifact
            .prepared_calls
            .iter()
            .map(|call| call.function_id)
            .collect::<Vec<_>>(),
        vec![
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "SPECIAL.LAMBDA",
            "SPECIAL.LAMBDA_INVOKE",
            "FUNC.OP_ADD",
            "SPECIAL.LET",
        ]
    );
}

#[test]
fn adapter_rejects_duplicate_let_binding_names_as_bind_mismatch() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "duplicate-let-binding",
        "formula:duplicate-let-binding",
        "=LET(x,1,x,2,x)",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("duplicate let adapter run");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .outcome_kind,
        ExecutionOutcomeKind::Rejected
    );
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .outcome_stage,
        ExecutionOutcomeStage::BindBoundary
    );
    assert_eq!(
        run.evaluation_artifact.execution_outcome_surface.class_id,
        "bind_boundary_reject"
    );
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .lane_reason_code
            .as_deref(),
        Some("BindMismatch")
    );
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(oxfml_core::RejectCode::BindMismatch)
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
    assert!(
        run.preparation_artifact
            .bind_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "duplicate LET binding name 'x'")
    );
}

#[test]
fn adapter_rejects_builtin_collision_arity_helper_local_call_as_bind_mismatch_ftc_0444() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "builtin-collision-arity-helper-call",
        "formula:foundation:FTC-0444",
        "=LET(THUNK,LAMBDA(x,LAMBDA(x)),t,THUNK(42),t())",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("FTC-0444 adapter run should classify bind mismatch");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .outcome_kind,
        ExecutionOutcomeKind::Rejected
    );
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .outcome_stage,
        ExecutionOutcomeStage::BindBoundary
    );
    assert_eq!(
        run.evaluation_artifact.execution_outcome_surface.class_id,
        "bind_boundary_reject"
    );
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .lane_reason_code
            .as_deref(),
        Some("BindMismatch")
    );
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(oxfml_core::RejectCode::BindMismatch)
    );
    assert_eq!(
        run.evaluation_artifact.worksheet_value,
        EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value)
    );
    assert!(
        run.preparation_artifact
            .bind_diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic.message.starts_with(
                    "built-in function call 'T' rejects 0 arguments at the authoring boundary",
                )
            })
    );
}

#[test]
fn adapter_rejects_plain_t_zero_arity_call_as_bind_mismatch() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "plain-t-zero-arity",
        "formula:plain-t-zero-arity",
        "=T()",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("plain T adapter run should classify bind mismatch");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact
            .execution_outcome_surface
            .outcome_stage,
        ExecutionOutcomeStage::BindBoundary
    );
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(oxfml_core::RejectCode::BindMismatch)
    );
    assert!(
        run.preparation_artifact
            .bind_diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic.message.starts_with(
                    "built-in function call 'T' rejects 0 arguments at the authoring boundary",
                )
            })
    );
}

#[test]
fn adapter_rejects_plain_gcd_zero_arity_call_as_bind_mismatch() {
    let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
        "plain-gcd-zero-arity",
        "formula:plain-gcd-zero-arity",
        "=GCD()",
        locus(1, 1),
        TypedContextQueryBundle::default(),
    ))
    .expect("plain GCD adapter run should classify bind mismatch");

    assert_eq!(run.evaluation_artifact.commit_decision_kind, "rejected");
    assert_eq!(
        run.evaluation_artifact.execution_outcome_surface.outcome_stage,
        ExecutionOutcomeStage::BindBoundary
    );
    assert_eq!(
        run.evaluation_artifact.reject_code,
        Some(oxfml_core::RejectCode::BindMismatch)
    );
    assert!(run.preparation_artifact.bind_diagnostics.iter().any(|diagnostic| {
        diagnostic.message.starts_with(
            "built-in function call 'GCD' rejects 0 arguments at the authoring boundary",
        )
    }));
}

#[test]
fn adapter_executes_colliding_let_calls_when_builtin_frontier_accepts_shape() {
    let cases = [
        (
            "collision-t-accepted-shape",
            "=LET(t,LAMBDA(42),t(\"x\"))",
            EvalValue::Text(ExcelText::from_interop_assignment("x")),
        ),
        (
            "collision-gcd-accepted-shape",
            "=LET(gcd,LAMBDA(42),gcd(48,36))",
            EvalValue::Number(12.0),
        ),
    ];

    for (case_id, formula, expected_value) in cases {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            format!("adapter:{case_id}"),
            format!("formula:{case_id}"),
            formula,
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .unwrap_or_else(|error| panic!("{case_id} adapter run should execute: {error}"));

        assert_eq!(run.evaluation_artifact.commit_decision_kind, "accepted");
        assert_eq!(
            run.evaluation_artifact.execution_outcome_surface.outcome_kind,
            ExecutionOutcomeKind::ExecutedResult,
            "{case_id} outcome kind"
        );
        assert_eq!(
            run.evaluation_artifact.worksheet_value,
            expected_value,
            "{case_id} worksheet value"
        );
    }
}

#[test]
fn adapter_rejects_lambda_array_constant_authoring_frontier_cases_as_bind_mismatch() {
    let cases = [
        ("S3", "={\"x\",LAMBDA(100)}"),
        (
            "S9",
            "=LET(dict,{\"x\",LAMBDA(100);\"y\",LAMBDA(200)},GETlambda,LAMBDA(d,LAMBDA(key,LET(keys,TAKE(d,,1),objects,DROP(d,,1),obj,XLOOKUP(key,keys,objects,\"not found\"),obj()))),getter,GETlambda(dict),getter(\"y\"))",
        ),
    ];

    for (case_id, formula) in cases {
        let run = run_oxfunc_preparation_adapter(OxFuncAdapterRequest::new(
            format!("lambda-array-frontier:{case_id}"),
            format!("formula:lambda-array-frontier:{case_id}"),
            formula,
            locus(1, 1),
            TypedContextQueryBundle::default(),
        ))
        .unwrap_or_else(|error| {
            panic!("{case_id} adapter run should classify bind mismatch: {error}")
        });

        assert_eq!(
            run.evaluation_artifact.commit_decision_kind, "rejected",
            "{case_id} commit decision"
        );
        assert_eq!(
            run.evaluation_artifact
                .execution_outcome_surface
                .outcome_stage,
            ExecutionOutcomeStage::BindBoundary,
            "{case_id} outcome stage"
        );
        assert_eq!(
            run.evaluation_artifact.reject_code,
            Some(oxfml_core::RejectCode::BindMismatch),
            "{case_id} reject code"
        );
        assert_eq!(
            run.evaluation_artifact.worksheet_value,
            EvalValue::Error(oxfunc_core::value::WorksheetErrorCode::Value),
            "{case_id} worksheet value"
        );
        assert!(
            run.preparation_artifact
                .bind_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message
                    == "LAMBDA cannot appear inside array constants"),
            "{case_id} bind diagnostic"
        );
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
            arity_shape_note: None,
            interface_contract_ref: Some("iface:v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}

fn locus(row: u32, col: u32) -> Locus {
    Locus {
        sheet_id: "sheet:default".to_string(),
        row,
        col,
    }
}

struct CapabilityDeniedRtdProvider;

impl RtdProvider for CapabilityDeniedRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        RtdProviderResult::CapabilityDenied
    }
}

struct ImageHostInfoProvider;

impl HostInfoProvider for ImageHostInfoProvider {
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

struct BlockedImageHostInfoProvider;

impl HostInfoProvider for BlockedImageHostInfoProvider {
    fn query_image(&self, _request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        Ok(ImageProviderResult::CapabilityDenied)
    }
}

#[derive(Default)]
struct RecordingRegisteredExternalProvider;

impl RegisteredExternalProvider for RecordingRegisteredExternalProvider {
    fn resolve_register_id(
        &self,
        request: &RegisterIdRequest,
    ) -> Result<oxfml_core::RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        let display_name = match &request.procedure {
            RegisteredProcedureSpec::Name(name) => name.clone(),
            RegisteredProcedureSpec::Ordinal(ordinal) => {
                ExcelText::from_interop_assignment(&ordinal.to_string())
            }
        };
        Ok(oxfml_core::RegisteredExternalDescriptor {
            stable_registration_id: format!(
                "REG.{}",
                request.library_name.to_string_lossy().to_ascii_lowercase()
            ),
            register_id: 4242.0,
            origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
            display_name: Some(display_name),
            library_name: request.library_name.clone(),
            procedure: request.procedure.clone(),
            declared_type_text: request.declared_type_text.clone(),
        })
    }

    fn lookup_registered_external(
        &self,
        register_id: f64,
    ) -> Result<oxfml_core::RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        Ok(oxfml_core::RegisteredExternalDescriptor {
            stable_registration_id: "REG.by-id".to_string(),
            register_id,
            origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
            display_name: Some(ExcelText::from_interop_assignment("LookupById")),
            library_name: ExcelText::from_interop_assignment("Kernel32"),
            procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment("MulDiv")),
            declared_type_text: Some(ExcelText::from_interop_assignment("JJJJ")),
        })
    }

    fn invoke_registered_external(
        &self,
        descriptor: &oxfml_core::RegisteredExternalDescriptor,
        args: &[CallArgValue],
    ) -> Result<EvalValue, RegisteredExternalProviderError> {
        match (&descriptor.procedure, args) {
            (
                RegisteredProcedureSpec::Name(name),
                [
                    CallArgValue::Eval(EvalValue::Number(a)),
                    CallArgValue::Eval(EvalValue::Number(b)),
                    CallArgValue::Eval(EvalValue::Number(c)),
                ],
            ) if name.to_string_lossy() == "MulDiv" => Ok(EvalValue::Number((a * b) / c)),
            _ => Ok(EvalValue::Number(descriptor.register_id)),
        }
    }
}
