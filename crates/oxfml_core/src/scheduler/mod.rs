use oxfunc_core::function::ArgPreparationProfile;

use crate::semantics::{
    FormulaDeterminismClass, FormulaThreadSafetyClass, FormulaVolatilityClass, SemanticPlan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ExecutionRestriction {
    HostSerialized,
    NotThreadSafe,
    ThreadAffine,
    AsyncCoupled,
    SerialOnly,
    SingleFlightAdvisable,
    Volatile,
    ContextualVolatility,
    HostQuery,
    CallerContext,
    ReferencePreserved,
    BranchLazy,
    FallbackLazy,
    LocaleSensitive,
    PseudoRandom,
    TimeDependent,
    ExternalEventDependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerLaneClass {
    ConcurrentSafe,
    Serialized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaySensitivityClass {
    Stable,
    ContextSensitive,
    NonDeterministic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContract {
    pub lane_class: SchedulerLaneClass,
    pub replay_sensitivity: ReplaySensitivityClass,
    pub restrictions: Vec<ExecutionRestriction>,
    pub single_flight_advisable: bool,
}

pub fn build_execution_contract(plan: &SemanticPlan) -> ExecutionContract {
    let mut restrictions = Vec::new();

    match plan.execution_profile.thread_safety {
        FormulaThreadSafetyClass::SafeConcurrent => {}
        FormulaThreadSafetyClass::HostSerialized => {
            restrictions.push(ExecutionRestriction::HostSerialized)
        }
        FormulaThreadSafetyClass::NotThreadSafe => {
            restrictions.push(ExecutionRestriction::NotThreadSafe)
        }
    }

    match plan.execution_profile.volatility {
        FormulaVolatilityClass::Stable => {}
        FormulaVolatilityClass::Contextual => {
            restrictions.push(ExecutionRestriction::ContextualVolatility)
        }
        FormulaVolatilityClass::Volatile => restrictions.push(ExecutionRestriction::Volatile),
    }

    if plan.execution_profile.requires_host_query {
        restrictions.push(ExecutionRestriction::HostQuery);
    }
    if plan.execution_profile.requires_thread_affinity {
        restrictions.push(ExecutionRestriction::ThreadAffine);
    }
    if plan.execution_profile.requires_async_coupling {
        restrictions.push(ExecutionRestriction::AsyncCoupled);
    }
    if plan.execution_profile.requires_caller_context {
        restrictions.push(ExecutionRestriction::CallerContext);
    }
    let reference_preservation_is_scheduler_relevant =
        plan.function_bindings.iter().any(|binding| {
            binding.arg_preparation_profile == ArgPreparationProfile::RefsVisibleInAdapter
        }) || plan.execution_profile.uses_implicit_intersection
            || plan.execution_profile.uses_spill_reference;
    if plan.execution_profile.requires_reference_preservation
        && reference_preservation_is_scheduler_relevant
    {
        restrictions.push(ExecutionRestriction::ReferencePreserved);
    }
    if plan.execution_profile.requires_branch_laziness {
        restrictions.push(ExecutionRestriction::BranchLazy);
    }
    if plan.execution_profile.requires_fallback_laziness {
        restrictions.push(ExecutionRestriction::FallbackLazy);
    }
    if plan.execution_profile.requires_locale {
        restrictions.push(ExecutionRestriction::LocaleSensitive);
    }
    if plan.execution_profile.contains_pseudo_random {
        restrictions.push(ExecutionRestriction::PseudoRandom);
    }
    if plan.execution_profile.contains_time_dependence {
        restrictions.push(ExecutionRestriction::TimeDependent);
    }
    if plan.execution_profile.contains_external_event_dependence {
        restrictions.push(ExecutionRestriction::ExternalEventDependent);
    }

    restrictions.sort();
    restrictions.dedup();

    let lane_class = if matches!(
        plan.execution_profile.thread_safety,
        FormulaThreadSafetyClass::SafeConcurrent
    ) && !plan.execution_profile.requires_serial_scheduler_lane
    {
        SchedulerLaneClass::ConcurrentSafe
    } else {
        SchedulerLaneClass::Serialized
    };

    if matches!(lane_class, SchedulerLaneClass::Serialized) {
        restrictions.push(ExecutionRestriction::SerialOnly);
    }

    if plan.execution_profile.single_flight_advisable {
        restrictions.push(ExecutionRestriction::SingleFlightAdvisable);
    }

    restrictions.sort();
    restrictions.dedup();

    let replay_sensitivity = match plan.execution_profile.determinism {
        FormulaDeterminismClass::Deterministic => ReplaySensitivityClass::Stable,
        FormulaDeterminismClass::Contextual => ReplaySensitivityClass::ContextSensitive,
        FormulaDeterminismClass::NonDeterministic => ReplaySensitivityClass::NonDeterministic,
    };

    let single_flight_advisable = plan.execution_profile.single_flight_advisable;

    ExecutionContract {
        lane_class,
        replay_sensitivity,
        restrictions,
        single_flight_advisable,
    }
}
