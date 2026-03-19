use oxfml_core::semantics::{
    EvaluationRequirement, FormulaDeterminismClass, FormulaThreadSafetyClass,
    FormulaVolatilityClass, LibraryAvailabilityState, LibraryContextSnapshot,
    LibraryContextSnapshotEntry, RegistrationSourceKind,
};
use oxfunc_core::function::FecDependencyProfile;

mod common;

#[test]
fn semantic_plan_for_sum_binds_known_oxfunc_metadata() {
    let plan = compile("=SUM(A1,2)");

    assert_eq!(plan.function_bindings.len(), 1);
    let binding = &plan.function_bindings[0];
    assert_eq!(binding.function_name, "SUM");
    assert_eq!(binding.function_id, "FUNC.SUM");
    assert_eq!(binding.arg_count, 2);
    assert_eq!(
        plan.execution_profile.thread_safety,
        FormulaThreadSafetyClass::SafeConcurrent
    );
    assert_eq!(
        plan.execution_profile.volatility,
        FormulaVolatilityClass::Stable
    );
    assert_eq!(
        plan.execution_profile.determinism,
        FormulaDeterminismClass::Deterministic
    );
    assert!(
        plan.execution_profile
            .fec_dependencies
            .contains(&FecDependencyProfile::RefOnly)
    );
    assert!(!plan.execution_profile.requires_host_query);
    assert!(!plan.execution_profile.requires_branch_laziness);
}

#[test]
fn semantic_plan_for_if_marks_branch_laziness_and_reference_preservation() {
    let plan = compile("=IF(A1,2,3)");

    assert_eq!(plan.function_bindings.len(), 1);
    assert_eq!(plan.function_bindings[0].function_id, "FUNC.IF");
    assert!(
        plan.evaluation_requirements
            .contains(&EvaluationRequirement::BranchLazy)
    );
    assert!(
        plan.evaluation_requirements
            .contains(&EvaluationRequirement::ReferencePreservedCalls)
    );
    assert!(plan.execution_profile.requires_branch_laziness);
    assert!(plan.execution_profile.requires_reference_preservation);
}

#[test]
fn semantic_plan_for_cell_marks_host_query_and_serial_lane() {
    let plan = compile("=CELL(\"filename\",A1)");

    assert_eq!(plan.function_bindings.len(), 1);
    assert_eq!(plan.function_bindings[0].function_id, "FUNC.CELL");
    assert!(plan.execution_profile.requires_host_query);
    assert!(plan.execution_profile.requires_host_interaction);
    assert!(plan.execution_profile.requires_serial_scheduler_lane);
    assert!(plan.execution_profile.requires_caller_context);
    assert!(
        plan.capability_requirements
            .iter()
            .any(|item| item == "host_query")
    );
}

#[test]
fn semantic_plan_for_rand_marks_nondeterministic_volatile_profile() {
    let plan = compile("=RAND()");

    assert_eq!(plan.function_bindings.len(), 1);
    assert_eq!(plan.function_bindings[0].function_id, "FUNC.RAND");
    assert_eq!(
        plan.execution_profile.determinism,
        FormulaDeterminismClass::NonDeterministic
    );
    assert_eq!(
        plan.execution_profile.volatility,
        FormulaVolatilityClass::Volatile
    );
    assert_eq!(
        plan.execution_profile.thread_safety,
        FormulaThreadSafetyClass::HostSerialized
    );
    assert!(plan.execution_profile.contains_pseudo_random);
    assert!(
        plan.capability_requirements
            .iter()
            .any(|item| item == "random_provider")
    );
}

#[test]
fn semantic_plan_for_row_and_column_mark_caller_context_and_serial_lane() {
    let row_plan = compile("=ROW()");
    let column_plan = compile("=COLUMN(A1:B2)");

    assert_eq!(row_plan.function_bindings[0].function_id, "FUNC.ROW");
    assert_eq!(column_plan.function_bindings[0].function_id, "FUNC.COLUMN");
    assert!(row_plan.execution_profile.requires_caller_context);
    assert!(column_plan.execution_profile.requires_caller_context);
    assert!(row_plan.execution_profile.requires_serial_scheduler_lane);
    assert!(column_plan.execution_profile.requires_serial_scheduler_lane);
    assert!(
        row_plan
            .evaluation_requirements
            .contains(&EvaluationRequirement::CallerContextSensitive)
    );
    assert!(
        column_plan
            .evaluation_requirements
            .contains(&EvaluationRequirement::ReferencePreservedCalls)
    );
}

#[test]
fn semantic_plan_for_indirect_offset_and_iferror_marks_runtime_requirements() {
    let indirect_plan = compile("=INDIRECT(\"A1\")");
    let offset_plan = compile("=OFFSET(A1,0,0)");
    let iferror_plan = compile("=IFERROR(UnknownName,2)");

    assert_eq!(
        indirect_plan.function_bindings[0].function_id,
        "FUNC.INDIRECT"
    );
    assert_eq!(offset_plan.function_bindings[0].function_id, "FUNC.OFFSET");
    assert_eq!(
        iferror_plan.function_bindings[0].function_id,
        "FUNC.IFERROR"
    );
    assert!(indirect_plan.execution_profile.requires_caller_context);
    assert!(
        indirect_plan
            .execution_profile
            .requires_serial_scheduler_lane
    );
    assert!(
        offset_plan
            .execution_profile
            .requires_reference_preservation
    );
    assert!(offset_plan.execution_profile.requires_caller_context);
    assert!(iferror_plan.execution_profile.requires_fallback_laziness);
    assert!(
        iferror_plan
            .evaluation_requirements
            .contains(&EvaluationRequirement::FallbackLazy)
    );
}

#[test]
fn semantic_plan_for_external_reference_marks_deferred_capability_lane() {
    let plan = compile("=[Book.xlsx]Sheet2!A1");

    assert!(plan.function_bindings.is_empty());
    assert!(
        plan.evaluation_requirements
            .contains(&EvaluationRequirement::ExternalReferenceDeferred)
    );
    assert!(plan.execution_profile.requires_host_interaction);
    assert!(plan.execution_profile.requires_async_coupling);
    assert!(plan.execution_profile.contains_external_event_dependence);
    assert!(plan.execution_profile.requires_serial_scheduler_lane);
    assert!(
        plan.capability_requirements
            .iter()
            .any(|item| item == "external_reference")
    );
}

#[test]
fn semantic_plan_for_index_and_xmatch_binds_registered_catalog_entries() {
    let index_plan = compile("=INDEX(SEQUENCE(3),2)");
    let xmatch_plan = compile("=XMATCH(3,SEQUENCE(5))");

    let index_function_ids = index_plan
        .function_bindings
        .iter()
        .map(|binding| binding.function_id)
        .collect::<Vec<_>>();
    let xmatch_function_ids = xmatch_plan
        .function_bindings
        .iter()
        .map(|binding| binding.function_id)
        .collect::<Vec<_>>();

    assert_eq!(index_function_ids, vec!["FUNC.SEQUENCE", "FUNC.INDEX"]);
    assert_eq!(xmatch_function_ids, vec!["FUNC.SEQUENCE", "FUNC.XMATCH"]);
    assert!(index_plan.execution_profile.requires_reference_preservation);
    assert!(
        !xmatch_plan
            .execution_profile
            .requires_reference_preservation
    );
}

#[test]
fn semantic_plan_for_implicit_intersection_and_spill_tracks_context_requirements() {
    let plan = compile("=@A1#");

    assert!(plan.function_bindings.is_empty());
    assert!(plan.execution_profile.uses_implicit_intersection);
    assert!(plan.execution_profile.uses_spill_reference);
    assert!(plan.execution_profile.requires_caller_context);
    assert!(plan.execution_profile.requires_reference_preservation);
    assert!(
        plan.evaluation_requirements
            .contains(&EvaluationRequirement::ImplicitIntersection)
    );
    assert!(
        plan.evaluation_requirements
            .contains(&EvaluationRequirement::SpillReference)
    );
}

#[test]
fn semantic_plan_reports_unknown_functions_as_typed_diagnostics() {
    let plan = compile("=UNKNOWNFUNC(1)");

    assert!(plan.function_bindings.is_empty());
    assert_eq!(plan.diagnostics.len(), 1);
    assert!(plan.diagnostics[0].message.contains("UNKNOWNFUNC"));
}

#[test]
fn semantic_plan_preserves_legacy_single_lane_without_oxfunc_binding() {
    let plan = compile("=_xlfn.SINGLE(A1)");

    assert!(plan.function_bindings.is_empty());
    assert!(plan.execution_profile.uses_implicit_intersection);
    assert!(plan.execution_profile.requires_caller_context);
    assert!(plan.execution_profile.requires_reference_preservation);
    assert!(
        plan.evaluation_requirements
            .contains(&EvaluationRequirement::LegacySingleCompat)
    );
    assert!(
        plan.capability_requirements
            .iter()
            .any(|item| item == "legacy_single_compat")
    );
}

#[test]
fn semantic_plan_preserves_helper_environment_for_let_and_lambda() {
    let let_plan = compile("=LET(x,1,x+2)");
    let lambda_plan = compile("=LAMBDA(x,x+1)");
    let invocation_plan = compile("=LAMBDA(x,x+1)(2)");
    let lexical_capture_plan = compile("=LET(x,10,LAMBDA(y,x+y))");

    assert!(
        let_plan
            .evaluation_requirements
            .contains(&EvaluationRequirement::HelperEnvironment)
    );
    assert!(
        lambda_plan
            .evaluation_requirements
            .contains(&EvaluationRequirement::HelperEnvironment)
    );
    assert!(
        let_plan
            .capability_requirements
            .iter()
            .any(|item| item == "helper_environment")
    );
    assert!(
        lambda_plan
            .capability_requirements
            .iter()
            .any(|item| item == "helper_environment")
    );
    assert!(
        invocation_plan
            .evaluation_requirements
            .contains(&EvaluationRequirement::HelperEnvironment)
    );
    assert!(!invocation_plan.execution_profile.requires_caller_context);
    assert!(
        invocation_plan
            .execution_profile
            .requires_reference_preservation
    );
    assert!(
        invocation_plan
            .capability_requirements
            .iter()
            .any(|item| item == "helper_environment")
    );
    assert!(let_plan.helper_profile.contains_let);
    assert!(lambda_plan.helper_profile.contains_lambda);
    assert!(invocation_plan.helper_profile.contains_lambda_invocation);
    assert_eq!(lambda_plan.helper_profile.lambda_literal_count, 1);
    assert_eq!(invocation_plan.helper_profile.lambda_invocation_count, 1);
    assert_eq!(lambda_plan.helper_profile.max_lambda_arity, 1);
    assert!(lexical_capture_plan.helper_profile.lexical_capture_required);
}

#[test]
fn semantic_plan_uses_snapshot_minimum_fields_and_builtin_fallback_refs() {
    let compiled = common::compile_formula_with_library_context(
        "semantic-library-snapshot",
        "=SUM(1)+TRANSLATE(2)",
        std::collections::BTreeMap::new(),
        "semantic-struct-v1",
        "oxfunc:fixture",
        Some(LibraryContextSnapshot {
            snapshot_id: "libctx.semantic".to_string(),
            snapshot_version: "v1".to_string(),
            entries: vec![LibraryContextSnapshotEntry {
                surface_name: "TRANSLATE".to_string(),
                canonical_id: Some("FUNC.TRANSLATE".to_string()),
                surface_stable_id: Some("surface.translate.provider".to_string()),
                name_resolution_table_ref: Some("libctx.names.provider.en-US@v1".to_string()),
                semantic_trait_profile_ref: Some("oxfunc.profile.translate@v2".to_string()),
                gating_profile_ref: Some("gate.translate-provider@v1".to_string()),
                registration_source_kind: RegistrationSourceKind::ProviderBacked,
                parse_bind_state: LibraryAvailabilityState::CatalogKnown,
                semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
                runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
                post_dispatch_state: Some(LibraryAvailabilityState::ProviderUnavailable),
            }],
        }),
    );
    let plan = compiled.semantic_plan;

    assert_eq!(
        plan.library_context_snapshot_ref.as_deref(),
        Some("libctx.semantic@v1")
    );

    let sum = plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "SUM")
        .expect("SUM summary should exist");
    assert_eq!(sum.surface_stable_id.as_deref(), Some("FUNC.SUM"));
    assert_eq!(sum.semantic_trait_profile_ref.as_deref(), Some("FUNC.SUM"));
    assert_eq!(sum.name_resolution_table_ref, None);

    let translate = plan
        .availability_summaries
        .iter()
        .find(|summary| summary.surface_name == "TRANSLATE")
        .expect("TRANSLATE summary should exist");
    assert_eq!(
        translate.surface_stable_id.as_deref(),
        Some("surface.translate.provider")
    );
    assert_eq!(
        translate.name_resolution_table_ref.as_deref(),
        Some("libctx.names.provider.en-US@v1")
    );
    assert_eq!(
        translate.semantic_trait_profile_ref.as_deref(),
        Some("oxfunc.profile.translate@v2")
    );
    assert_eq!(
        translate.gating_profile_ref.as_deref(),
        Some("gate.translate-provider@v1")
    );
    assert_eq!(
        translate.runtime_capability_state,
        Some(LibraryAvailabilityState::CatalogKnown)
    );
    assert_eq!(
        translate.post_dispatch_state,
        Some(LibraryAvailabilityState::ProviderUnavailable)
    );
}

fn compile(formula: &str) -> oxfml_core::SemanticPlan {
    common::compile_formula(
        "semantic-fixture",
        formula,
        std::collections::BTreeMap::new(),
        "semantic-struct-v1",
        "oxfunc:fixture",
    )
    .semantic_plan
}
