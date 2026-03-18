use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
use oxfml_core::red::project_red_view;
use oxfml_core::scheduler::{
    ExecutionRestriction, ReplaySensitivityClass, SchedulerLaneClass, build_execution_contract,
};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};

#[test]
fn scheduler_contract_for_sum_is_concurrent_safe() {
    let contract = contract_for("=SUM(A1,2)");
    assert_eq!(contract.lane_class, SchedulerLaneClass::ConcurrentSafe);
    assert_eq!(contract.replay_sensitivity, ReplaySensitivityClass::Stable);
    assert!(contract.restrictions.is_empty());
    assert!(!contract.single_flight_advisable);
}

#[test]
fn scheduler_contract_for_cell_is_serialized_and_host_query_bound() {
    let contract = contract_for("=CELL(\"filename\",A1)");
    assert_eq!(contract.lane_class, SchedulerLaneClass::Serialized);
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::HostSerialized)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::ThreadAffine)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::HostQuery)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::CallerContext)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::SerialOnly)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::SingleFlightAdvisable)
    );
    assert!(contract.single_flight_advisable);
}

#[test]
fn scheduler_contract_for_rand_is_nondeterministic_and_serialized() {
    let contract = contract_for("=RAND()");
    assert_eq!(contract.lane_class, SchedulerLaneClass::Serialized);
    assert_eq!(
        contract.replay_sensitivity,
        ReplaySensitivityClass::NonDeterministic
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::PseudoRandom)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::Volatile)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::ThreadAffine)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::SerialOnly)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::SingleFlightAdvisable)
    );
}

#[test]
fn scheduler_contract_can_express_async_coupled_external_lanes() {
    let plan = oxfml_core::SemanticPlan {
        semantic_plan_key: "plan:async".to_string(),
        formula_stable_id: "formula:async".to_string(),
        bind_hash: "bind:async".to_string(),
        oxfunc_catalog_identity: "oxfunc:test".to_string(),
        library_context_snapshot_ref: None,
        locale_profile: None,
        date_system: None,
        format_profile: None,
        function_bindings: Vec::new(),
        availability_summaries: Vec::new(),
        evaluation_requirements: Vec::new(),
        execution_profile: oxfml_core::ExecutionProfileSummary {
            requires_async_coupling: true,
            contains_external_event_dependence: true,
            requires_serial_scheduler_lane: true,
            single_flight_advisable: true,
            ..oxfml_core::ExecutionProfileSummary::default()
        },
        helper_profile: oxfml_core::HelperEnvironmentProfile::default(),
        capability_requirements: vec!["external_provider".to_string()],
        diagnostics: Vec::new(),
    };

    let contract = build_execution_contract(&plan);
    assert_eq!(contract.lane_class, SchedulerLaneClass::Serialized);
    assert_eq!(contract.replay_sensitivity, ReplaySensitivityClass::Stable);
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::AsyncCoupled)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::ExternalEventDependent)
    );
    assert!(
        contract
            .restrictions
            .contains(&ExecutionRestriction::SingleFlightAdvisable)
    );
}

fn contract_for(formula: &str) -> oxfml_core::ExecutionContract {
    let source = FormulaSourceRecord::new("scheduler-fixture", 1, formula.to_string());
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            structure_context_version: StructureContextVersion("scheduler-struct-v1".to_string()),
            ..BindContext::default()
        },
    });

    let plan = oxfml_core::compile_semantic_plan(oxfml_core::CompileSemanticPlanRequest {
        bound_formula: bind.bound_formula,
        oxfunc_catalog_identity: "oxfunc:scheduler".to_string(),
        locale_profile: Some("en-US".to_string()),
        date_system: Some("1900".to_string()),
        format_profile: Some("excel-default".to_string()),
        library_context_snapshot: None,
    })
    .semantic_plan;

    build_execution_contract(&plan)
}
