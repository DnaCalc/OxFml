use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use oxfunc_core::function::{
    ArgPreparationProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, ThreadSafetyClass, VolatilityClass,
};
use oxfunc_core::functions::{
    abs::ABS_META, and_fn::AND_META, average::AVERAGE_META, cell::CELL_META,
    column_fn::COLUMN_META, count::COUNT_META, counta::COUNTA_META, dollar_fn::DOLLAR_META,
    fixed_fn::FIXED_META, if_fn::IF_META, iferror::IFERROR_META, index::INDEX_META,
    indirect::INDIRECT_META, info_fn::INFO_META, isnumber::ISNUMBER_META, match_fn::MATCH_META,
    n_fn::N_META, now_fn::NOW_META, offset::OFFSET_META, rand_fn::RAND_META, row_fn::ROW_META,
    sequence::SEQUENCE_META, sum::SUM_META, t_fn::T_META, text_fn::TEXT_META, today_fn::TODAY_META,
    type_fn::TYPE_META, value_fn::VALUE_META, xlookup::XLOOKUP_META, xmatch::XMATCH_META,
};

use crate::binding::{BoundExpr, BoundFormula, ReferenceExpr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileSemanticPlanRequest {
    pub bound_formula: BoundFormula,
    pub oxfunc_catalog_identity: String,
    pub locale_profile: Option<String>,
    pub date_system: Option<String>,
    pub format_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileSemanticPlanResult {
    pub semantic_plan: SemanticPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticPlan {
    pub semantic_plan_key: String,
    pub formula_stable_id: String,
    pub bind_hash: String,
    pub oxfunc_catalog_identity: String,
    pub locale_profile: Option<String>,
    pub date_system: Option<String>,
    pub format_profile: Option<String>,
    pub function_bindings: Vec<FunctionPlanBinding>,
    pub evaluation_requirements: Vec<EvaluationRequirement>,
    pub execution_profile: ExecutionProfileSummary,
    pub helper_profile: HelperEnvironmentProfile,
    pub capability_requirements: Vec<String>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionPlanBinding {
    pub function_name: String,
    pub function_id: &'static str,
    pub arg_count: usize,
    pub arity_min: usize,
    pub arity_max: usize,
    pub arg_preparation_profile: ArgPreparationProfile,
    pub fec_dependency_profile: FecDependencyProfile,
    pub surface_fec_dependency_profile: FecDependencyProfile,
    pub host_interaction: HostInteractionClass,
    pub thread_safety: ThreadSafetyClass,
    pub volatility: VolatilityClass,
    pub determinism: DeterminismClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationRequirement {
    BranchLazy,
    FallbackLazy,
    ReferencePreservedCalls,
    CallerContextSensitive,
    HostQueryCapability,
    LocaleAware,
    ImplicitIntersection,
    SpillReference,
    ReferenceExpression,
    LegacySingleCompat,
    HelperEnvironment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaThreadSafetyClass {
    SafeConcurrent,
    HostSerialized,
    NotThreadSafe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaVolatilityClass {
    Stable,
    Contextual,
    Volatile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaDeterminismClass {
    Deterministic,
    Contextual,
    NonDeterministic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionProfileSummary {
    pub thread_safety: FormulaThreadSafetyClass,
    pub volatility: FormulaVolatilityClass,
    pub determinism: FormulaDeterminismClass,
    pub fec_dependencies: Vec<FecDependencyProfile>,
    pub requires_host_interaction: bool,
    pub requires_host_query: bool,
    pub requires_thread_affinity: bool,
    pub requires_async_coupling: bool,
    pub requires_caller_context: bool,
    pub requires_reference_preservation: bool,
    pub uses_implicit_intersection: bool,
    pub uses_spill_reference: bool,
    pub requires_branch_laziness: bool,
    pub requires_fallback_laziness: bool,
    pub requires_locale: bool,
    pub contains_pseudo_random: bool,
    pub contains_time_dependence: bool,
    pub contains_external_event_dependence: bool,
    pub requires_serial_scheduler_lane: bool,
    pub single_flight_advisable: bool,
}

impl Default for ExecutionProfileSummary {
    fn default() -> Self {
        Self {
            thread_safety: FormulaThreadSafetyClass::SafeConcurrent,
            volatility: FormulaVolatilityClass::Stable,
            determinism: FormulaDeterminismClass::Deterministic,
            fec_dependencies: Vec::new(),
            requires_host_interaction: false,
            requires_host_query: false,
            requires_thread_affinity: false,
            requires_async_coupling: false,
            requires_caller_context: false,
            requires_reference_preservation: false,
            uses_implicit_intersection: false,
            uses_spill_reference: false,
            requires_branch_laziness: false,
            requires_fallback_laziness: false,
            requires_locale: false,
            contains_pseudo_random: false,
            contains_time_dependence: false,
            contains_external_event_dependence: false,
            requires_serial_scheduler_lane: false,
            single_flight_advisable: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelperEnvironmentProfile {
    pub contains_let: bool,
    pub contains_lambda: bool,
    pub contains_lambda_invocation: bool,
    pub lambda_literal_count: usize,
    pub lambda_invocation_count: usize,
    pub max_lambda_arity: usize,
    pub lexical_capture_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub message: String,
    pub function_name: Option<String>,
}

pub fn compile_semantic_plan(request: CompileSemanticPlanRequest) -> CompileSemanticPlanResult {
    let CompileSemanticPlanRequest {
        bound_formula,
        oxfunc_catalog_identity,
        locale_profile,
        date_system,
        format_profile,
    } = request;

    let mut compiler = SemanticCompiler {
        function_bindings: Vec::new(),
        evaluation_requirements: Vec::new(),
        execution_profile: ExecutionProfileSummary::default(),
        helper_profile: HelperEnvironmentProfile::default(),
        capability_requirements: bound_formula.capability_requirements.clone(),
        diagnostics: Vec::new(),
    };

    compiler.visit_expr(&bound_formula.root);

    let semantic_plan_key = hash_debug(&(
        bound_formula.formula_stable_id.as_str(),
        bound_formula.bind_hash.as_str(),
        &compiler.function_bindings,
        &compiler.evaluation_requirements,
        &compiler.execution_profile,
        &compiler.helper_profile,
        &compiler.capability_requirements,
        &compiler.diagnostics,
        oxfunc_catalog_identity.as_str(),
        locale_profile.as_deref(),
        date_system.as_deref(),
        format_profile.as_deref(),
    ));

    CompileSemanticPlanResult {
        semantic_plan: SemanticPlan {
            semantic_plan_key,
            formula_stable_id: bound_formula.formula_stable_id,
            bind_hash: bound_formula.bind_hash,
            oxfunc_catalog_identity,
            locale_profile,
            date_system,
            format_profile,
            function_bindings: compiler.function_bindings,
            evaluation_requirements: compiler.evaluation_requirements,
            execution_profile: compiler.execution_profile,
            helper_profile: compiler.helper_profile,
            capability_requirements: compiler.capability_requirements,
            diagnostics: compiler.diagnostics,
        },
    }
}

struct SemanticCompiler {
    function_bindings: Vec<FunctionPlanBinding>,
    evaluation_requirements: Vec<EvaluationRequirement>,
    execution_profile: ExecutionProfileSummary,
    helper_profile: HelperEnvironmentProfile,
    capability_requirements: Vec<String>,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl SemanticCompiler {
    fn visit_expr(&mut self, expr: &BoundExpr) {
        match expr {
            BoundExpr::NumberLiteral(_)
            | BoundExpr::StringLiteral(_)
            | BoundExpr::HelperParameterName(_) => {}
            BoundExpr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            BoundExpr::FunctionCall {
                function_name,
                args,
            } => {
                self.record_helper_surface(function_name, args);
                for arg in args {
                    self.visit_expr(arg);
                }
                self.record_function_call(function_name, args.len());
            }
            BoundExpr::Invocation { callee, args } => {
                self.record_lambda_invocation(callee);
                self.visit_expr(callee);
                for arg in args {
                    self.visit_expr(arg);
                }
                self.execution_profile.requires_reference_preservation = true;
                self.push_evaluation_requirement(EvaluationRequirement::HelperEnvironment);
                self.push_capability_requirement("helper_environment");
            }
            BoundExpr::Reference(reference) => {
                self.record_reference_expression();
                self.visit_reference_expr(reference);
            }
            BoundExpr::ImplicitIntersection(inner) => {
                self.record_reference_expression();
                self.execution_profile.uses_implicit_intersection = true;
                self.execution_profile.requires_caller_context = true;
                self.execution_profile.requires_reference_preservation = true;
                self.push_evaluation_requirement(EvaluationRequirement::ImplicitIntersection);
                self.push_evaluation_requirement(EvaluationRequirement::CallerContextSensitive);
                self.push_capability_requirement("caller_context");
                self.visit_expr(inner);
            }
        }
    }

    fn visit_reference_expr(&mut self, reference: &ReferenceExpr) {
        match reference {
            ReferenceExpr::Atom(_) => {}
            ReferenceExpr::Range { start, end }
            | ReferenceExpr::Union {
                left: start,
                right: end,
            }
            | ReferenceExpr::Intersection {
                left: start,
                right: end,
            } => {
                self.visit_reference_expr(start);
                self.visit_reference_expr(end);
            }
            ReferenceExpr::Spill { anchor } => {
                self.execution_profile.uses_spill_reference = true;
                self.execution_profile.requires_reference_preservation = true;
                self.push_evaluation_requirement(EvaluationRequirement::SpillReference);
                self.push_capability_requirement("spill_reference");
                self.visit_reference_expr(anchor);
            }
        }
    }

    fn record_reference_expression(&mut self) {
        self.execution_profile.requires_reference_preservation = true;
        self.push_evaluation_requirement(EvaluationRequirement::ReferenceExpression);
    }

    fn record_helper_surface(&mut self, function_name: &str, args: &[BoundExpr]) {
        match function_name {
            "LET" => {
                self.helper_profile.contains_let = true;
            }
            "LAMBDA" => {
                self.helper_profile.contains_lambda = true;
                self.helper_profile.lambda_literal_count += 1;
                if !args.is_empty() {
                    let arity = args.len() - 1;
                    self.helper_profile.max_lambda_arity =
                        self.helper_profile.max_lambda_arity.max(arity);
                    if lambda_has_lexical_capture(args) {
                        self.helper_profile.lexical_capture_required = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn record_lambda_invocation(&mut self, callee: &BoundExpr) {
        self.helper_profile.contains_lambda_invocation = true;
        self.helper_profile.lambda_invocation_count += 1;
        if let BoundExpr::FunctionCall {
            function_name,
            args,
        } = callee
        {
            if function_name == "LAMBDA" && lambda_has_lexical_capture(args) {
                self.helper_profile.lexical_capture_required = true;
            }
        }
    }

    fn record_function_call(&mut self, function_name: &str, arg_count: usize) {
        self.record_special_function_lane(function_name);
        let Some(meta) = lookup_function_meta(function_name) else {
            self.diagnostics.push(SemanticDiagnostic {
                message: format!("no OxFunc metadata registered for function {function_name}"),
                function_name: Some(function_name.to_string()),
            });
            return;
        };

        self.function_bindings.push(FunctionPlanBinding {
            function_name: function_name.to_string(),
            function_id: meta.function_id,
            arg_count,
            arity_min: meta.arity.min,
            arity_max: meta.arity.max,
            arg_preparation_profile: meta.arg_preparation_profile,
            fec_dependency_profile: meta.fec_dependency_profile,
            surface_fec_dependency_profile: meta.surface_fec_dependency_profile,
            host_interaction: meta.host_interaction,
            thread_safety: meta.thread_safety,
            volatility: meta.volatility,
            determinism: meta.determinism,
        });

        self.promote_thread_safety(meta.thread_safety);
        self.promote_volatility(meta.volatility);
        self.promote_determinism(meta.determinism);
        self.push_fec_dependency(meta.fec_dependency_profile);
        self.push_fec_dependency(meta.surface_fec_dependency_profile);

        if meta.host_interaction != HostInteractionClass::None {
            self.execution_profile.requires_host_interaction = true;
            self.execution_profile.requires_serial_scheduler_lane = true;
            self.execution_profile.requires_thread_affinity = true;
            self.execution_profile.single_flight_advisable = true;

            match meta.host_interaction {
                HostInteractionClass::WorkbookState
                | HostInteractionClass::ApplicationState
                | HostInteractionClass::EnvironmentState => {}
                HostInteractionClass::ExternalProvider => {
                    self.execution_profile.requires_async_coupling = true;
                }
                HostInteractionClass::None => {}
            }
        }

        if matches!(
            meta.arg_preparation_profile,
            ArgPreparationProfile::RefsVisibleInAdapter
        ) {
            self.execution_profile.requires_reference_preservation = true;
            self.push_evaluation_requirement(EvaluationRequirement::ReferencePreservedCalls);
        }

        match meta.fec_dependency_profile {
            FecDependencyProfile::CallerContext => {
                self.execution_profile.requires_caller_context = true;
                self.push_evaluation_requirement(EvaluationRequirement::CallerContextSensitive);
                self.push_capability_requirement("caller_context");
            }
            FecDependencyProfile::TimeProvider => {
                self.execution_profile.contains_time_dependence = true;
                self.push_capability_requirement("time_provider");
            }
            FecDependencyProfile::RandomProvider => {
                self.execution_profile.contains_pseudo_random = true;
                self.push_capability_requirement("random_provider");
            }
            FecDependencyProfile::ExternalProvider => {
                self.execution_profile.contains_external_event_dependence = true;
                self.execution_profile.requires_async_coupling = true;
                self.execution_profile.single_flight_advisable = true;
                self.push_capability_requirement("external_provider");
            }
            FecDependencyProfile::LocaleProfile => {
                self.execution_profile.requires_locale = true;
                self.push_evaluation_requirement(EvaluationRequirement::LocaleAware);
                self.push_capability_requirement("locale_format_context");
            }
            FecDependencyProfile::Composite => {
                self.push_capability_requirement("composite_fec_dependency");
            }
            FecDependencyProfile::None | FecDependencyProfile::RefOnly => {}
        }

        match meta.surface_fec_dependency_profile {
            FecDependencyProfile::CallerContext => {
                self.execution_profile.requires_caller_context = true;
                self.push_evaluation_requirement(EvaluationRequirement::CallerContextSensitive);
                self.push_capability_requirement("caller_context");
            }
            FecDependencyProfile::TimeProvider => {
                self.execution_profile.contains_time_dependence = true;
                self.push_capability_requirement("time_provider");
            }
            FecDependencyProfile::RandomProvider => {
                self.execution_profile.contains_pseudo_random = true;
                self.push_capability_requirement("random_provider");
            }
            FecDependencyProfile::ExternalProvider => {
                self.execution_profile.contains_external_event_dependence = true;
                self.execution_profile.requires_async_coupling = true;
                self.execution_profile.single_flight_advisable = true;
                self.push_capability_requirement("external_provider");
            }
            FecDependencyProfile::LocaleProfile => {
                self.execution_profile.requires_locale = true;
                self.push_evaluation_requirement(EvaluationRequirement::LocaleAware);
                self.push_capability_requirement("locale_format_context");
            }
            FecDependencyProfile::Composite => {
                self.push_capability_requirement("composite_fec_dependency");
            }
            FecDependencyProfile::None | FecDependencyProfile::RefOnly => {}
        }

        match function_name {
            "IF" => {
                self.execution_profile.requires_branch_laziness = true;
                self.push_evaluation_requirement(EvaluationRequirement::BranchLazy);
            }
            "IFERROR" => {
                self.execution_profile.requires_fallback_laziness = true;
                self.push_evaluation_requirement(EvaluationRequirement::FallbackLazy);
            }
            "CELL" | "INFO" => {
                self.execution_profile.requires_host_query = true;
                self.execution_profile.requires_serial_scheduler_lane = true;
                self.execution_profile.requires_thread_affinity = true;
                self.execution_profile.single_flight_advisable = true;
                self.push_evaluation_requirement(EvaluationRequirement::HostQueryCapability);
                self.push_capability_requirement("host_query");
            }
            "TEXT" | "VALUE" | "DOLLAR" | "FIXED" => {
                self.execution_profile.requires_locale = true;
                self.push_evaluation_requirement(EvaluationRequirement::LocaleAware);
                self.push_capability_requirement("locale_format_context");
            }
            "NOW" | "TODAY" => {
                self.execution_profile.contains_time_dependence = true;
                self.execution_profile.single_flight_advisable = true;
                self.push_capability_requirement("time_provider");
            }
            "RAND" => {
                self.execution_profile.contains_pseudo_random = true;
                self.execution_profile.single_flight_advisable = true;
                self.push_capability_requirement("random_provider");
            }
            _ => {}
        }
    }

    fn record_special_function_lane(&mut self, function_name: &str) {
        match function_name {
            "_XLFN.SINGLE" | "SINGLE" => {
                self.execution_profile.uses_implicit_intersection = true;
                self.execution_profile.requires_caller_context = true;
                self.execution_profile.requires_reference_preservation = true;
                self.push_evaluation_requirement(EvaluationRequirement::LegacySingleCompat);
                self.push_evaluation_requirement(EvaluationRequirement::ImplicitIntersection);
                self.push_evaluation_requirement(EvaluationRequirement::CallerContextSensitive);
                self.push_capability_requirement("caller_context");
                self.push_capability_requirement("legacy_single_compat");
                self.diagnostics.push(SemanticDiagnostic {
                    message: format!(
                        "legacy SINGLE compatibility lane preserved without OxFunc metadata for function {function_name}"
                    ),
                    function_name: Some(function_name.to_string()),
                });
            }
            "LET" | "LAMBDA" => {
                self.push_evaluation_requirement(EvaluationRequirement::HelperEnvironment);
                self.push_capability_requirement("helper_environment");
                self.diagnostics.push(SemanticDiagnostic {
                    message: format!(
                        "helper-form environment preserved without OxFunc metadata for function {function_name}"
                    ),
                    function_name: Some(function_name.to_string()),
                });
            }
            _ => {}
        }
    }

    fn promote_thread_safety(&mut self, thread_safety: ThreadSafetyClass) {
        self.execution_profile.thread_safety =
            match (self.execution_profile.thread_safety, thread_safety) {
                (_, ThreadSafetyClass::NotThreadSafe) => FormulaThreadSafetyClass::NotThreadSafe,
                (FormulaThreadSafetyClass::NotThreadSafe, _) => {
                    FormulaThreadSafetyClass::NotThreadSafe
                }
                (_, ThreadSafetyClass::HostSerialized) => FormulaThreadSafetyClass::HostSerialized,
                (FormulaThreadSafetyClass::HostSerialized, _) => {
                    FormulaThreadSafetyClass::HostSerialized
                }
                _ => FormulaThreadSafetyClass::SafeConcurrent,
            };

        if thread_safety != ThreadSafetyClass::SafePure {
            self.execution_profile.requires_serial_scheduler_lane = true;
            self.execution_profile.requires_thread_affinity = true;
            self.execution_profile.single_flight_advisable = true;
        }
    }

    fn promote_volatility(&mut self, volatility: VolatilityClass) {
        self.execution_profile.volatility = match (self.execution_profile.volatility, volatility) {
            (_, VolatilityClass::VolatileFull) => FormulaVolatilityClass::Volatile,
            (FormulaVolatilityClass::Volatile, _) => FormulaVolatilityClass::Volatile,
            (_, VolatilityClass::VolatileContextual) => FormulaVolatilityClass::Contextual,
            (FormulaVolatilityClass::Contextual, _) => FormulaVolatilityClass::Contextual,
            _ => FormulaVolatilityClass::Stable,
        };
    }

    fn promote_determinism(&mut self, determinism: DeterminismClass) {
        self.execution_profile.determinism = match determinism {
            DeterminismClass::Deterministic => self.execution_profile.determinism,
            DeterminismClass::TimeDependent | DeterminismClass::ExternalEventDependent => {
                if self.execution_profile.determinism != FormulaDeterminismClass::NonDeterministic {
                    FormulaDeterminismClass::Contextual
                } else {
                    self.execution_profile.determinism
                }
            }
            DeterminismClass::PseudoRandom => FormulaDeterminismClass::NonDeterministic,
        };
    }

    fn push_fec_dependency(&mut self, profile: FecDependencyProfile) {
        if !self.execution_profile.fec_dependencies.contains(&profile) {
            self.execution_profile.fec_dependencies.push(profile);
        }
    }

    fn push_evaluation_requirement(&mut self, requirement: EvaluationRequirement) {
        if !self.evaluation_requirements.contains(&requirement) {
            self.evaluation_requirements.push(requirement);
        }
    }

    fn push_capability_requirement(&mut self, requirement: &str) {
        if !self
            .capability_requirements
            .iter()
            .any(|existing| existing == requirement)
        {
            self.capability_requirements.push(requirement.to_string());
        }
    }
}

pub fn lookup_function_meta(function_name: &str) -> Option<FunctionMeta> {
    match function_name {
        "ABS" => Some(ABS_META),
        "AND" => Some(AND_META),
        "AVERAGE" => Some(AVERAGE_META),
        "CELL" => Some(CELL_META),
        "COLUMN" => Some(COLUMN_META),
        "COUNT" => Some(COUNT_META),
        "COUNTA" => Some(COUNTA_META),
        "DOLLAR" => Some(DOLLAR_META),
        "FIXED" => Some(FIXED_META),
        "IF" => Some(IF_META),
        "IFERROR" => Some(IFERROR_META),
        "INDEX" => Some(INDEX_META),
        "INDIRECT" => Some(INDIRECT_META),
        "INFO" => Some(INFO_META),
        "ISNUMBER" => Some(ISNUMBER_META),
        "MATCH" => Some(MATCH_META),
        "N" => Some(N_META),
        "NOW" => Some(NOW_META),
        "OFFSET" => Some(OFFSET_META),
        "RAND" => Some(RAND_META),
        "ROW" => Some(ROW_META),
        "SEQUENCE" => Some(SEQUENCE_META),
        "SUM" => Some(SUM_META),
        "T" => Some(T_META),
        "TEXT" => Some(TEXT_META),
        "TODAY" => Some(TODAY_META),
        "TYPE" => Some(TYPE_META),
        "VALUE" => Some(VALUE_META),
        "XLOOKUP" => Some(XLOOKUP_META),
        "XMATCH" => Some(XMATCH_META),
        _ => None,
    }
}

fn hash_debug<T: std::fmt::Debug>(value: &T) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{value:?}").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn lambda_has_lexical_capture(args: &[BoundExpr]) -> bool {
    if args.is_empty() {
        return false;
    }
    let body_index = args.len() - 1;
    let params = args[..body_index]
        .iter()
        .filter_map(|arg| match arg {
            BoundExpr::HelperParameterName(name) => Some(name.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut captures = BTreeSet::new();
    collect_helper_capture_names(&args[body_index], &params, &mut captures);
    !captures.is_empty()
}

fn collect_helper_capture_names<'a>(
    expr: &'a BoundExpr,
    params: &BTreeSet<&'a str>,
    captures: &mut BTreeSet<String>,
) {
    match expr {
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::HelperParameterName(_) => {}
        BoundExpr::Binary { left, right, .. } => {
            collect_helper_capture_names(left, params, captures);
            collect_helper_capture_names(right, params, captures);
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => {
            if function_name == "LAMBDA" {
                return;
            }
            for arg in args {
                collect_helper_capture_names(arg, params, captures);
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_helper_capture_names(callee, params, captures);
            for arg in args {
                collect_helper_capture_names(arg, params, captures);
            }
        }
        BoundExpr::Reference(ReferenceExpr::Atom(crate::binding::NormalizedReference::Name(
            name,
        ))) if matches!(name.kind, crate::binding::NameKind::HelperLocal)
            && !params.contains(name.name.as_str()) =>
        {
            captures.insert(name.name.clone());
        }
        BoundExpr::Reference(reference) => match reference {
            ReferenceExpr::Range { start, end }
            | ReferenceExpr::Union {
                left: start,
                right: end,
            }
            | ReferenceExpr::Intersection {
                left: start,
                right: end,
            } => {
                collect_helper_capture_names_in_ref(start, params, captures);
                collect_helper_capture_names_in_ref(end, params, captures);
            }
            ReferenceExpr::Spill { anchor } => {
                collect_helper_capture_names_in_ref(anchor, params, captures);
            }
            ReferenceExpr::Atom(_) => {}
        },
        BoundExpr::ImplicitIntersection(inner) => {
            collect_helper_capture_names(inner, params, captures);
        }
    }
}

fn collect_helper_capture_names_in_ref<'a>(
    expr: &'a ReferenceExpr,
    params: &BTreeSet<&'a str>,
    captures: &mut BTreeSet<String>,
) {
    match expr {
        ReferenceExpr::Atom(crate::binding::NormalizedReference::Name(name))
            if matches!(name.kind, crate::binding::NameKind::HelperLocal)
                && !params.contains(name.name.as_str()) =>
        {
            captures.insert(name.name.clone());
        }
        ReferenceExpr::Range { start, end }
        | ReferenceExpr::Union {
            left: start,
            right: end,
        }
        | ReferenceExpr::Intersection {
            left: start,
            right: end,
        } => {
            collect_helper_capture_names_in_ref(start, params, captures);
            collect_helper_capture_names_in_ref(end, params, captures);
        }
        ReferenceExpr::Spill { anchor } => {
            collect_helper_capture_names_in_ref(anchor, params, captures);
        }
        ReferenceExpr::Atom(_) => {}
    }
}
