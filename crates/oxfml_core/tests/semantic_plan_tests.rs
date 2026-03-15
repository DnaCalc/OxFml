use oxfunc_core::function::FecDependencyProfile;

use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::{
    CompileSemanticPlanRequest, EvaluationRequirement, FormulaDeterminismClass,
    FormulaThreadSafetyClass, FormulaVolatilityClass, compile_semantic_plan,
};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};

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

fn compile(formula: &str) -> oxfml_core::SemanticPlan {
    let source = FormulaSourceRecord::new("semantic-fixture", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("semantic-struct-v1".to_string()),
            ..BindContext::default()
        },
    });

    compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula,
        oxfunc_catalog_identity: "oxfunc:fixture".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
    })
    .semantic_plan
}
