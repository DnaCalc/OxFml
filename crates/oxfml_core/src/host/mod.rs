use std::collections::BTreeMap;

use oxfunc_core::host_info::HostInfoProvider;
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceLike};

use crate::binding::{BindContext, BindRequest, BoundFormula, NameKind, bind_formula_incremental};
use crate::eval::{
    CallableDefinedNameBinding, DefinedNameBinding, EvaluationBackend, EvaluationContext,
    EvaluationOutput, evaluate_formula,
};
use crate::red::{RedProjection, project_red_view_incremental};
use crate::scheduler::{ExecutionContract, build_execution_contract};
use crate::seam::{
    AcceptDecision, AcceptedCandidateResult, CapabilityEffectFact, CommitRequest,
    DependencyConsequenceFact, DisplayDelta, Extent, FenceSnapshot, FormatDelta,
    FormatDependencyFact, Locus, ShapeDelta, ShapeOutcomeClass, SpillEvent, SpillEventKind,
    TopologyDelta, TraceEvent, TraceEventKind, TracePayload, ValueDelta, ValuePayload,
    WorksheetValueClass, commit_candidate,
};
use crate::semantics::{CompileSemanticPlanRequest, SemanticPlan, compile_semantic_plan};
use crate::source::{FormulaSourceRecord, StructureContextVersion};
use crate::syntax::green::GreenTreeRoot;
use crate::syntax::parser::{ParseRequest, parse_formula_incremental};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactReuseReport {
    pub green_tree_reused: bool,
    pub red_projection_reused: bool,
    pub bound_formula_reused: bool,
    pub semantic_plan_reused: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct CachedHostArtifacts {
    green_tree: GreenTreeRoot,
    red_projection: RedProjection,
    bound_formula: BoundFormula,
    semantic_plan: SemanticPlan,
    semantic_plan_catalog_identity: String,
    locale_profile: Option<String>,
    date_system: Option<String>,
    format_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SingleFormulaHost {
    pub formula_stable_id: String,
    pub formula_text: String,
    pub formula_text_version: u64,
    pub structure_context_version: String,
    pub caller_row: u32,
    pub caller_col: u32,
    pub primary_locus: Locus,
    pub defined_names: BTreeMap<String, DefinedNameBinding>,
    pub cell_values: BTreeMap<String, EvalValue>,
    pub now_serial: Option<f64>,
    pub random_value: Option<f64>,
    next_session_id: u64,
    next_commit_attempt_id: u64,
    cached_artifacts: Option<CachedHostArtifacts>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostRecalcOutput {
    pub source: FormulaSourceRecord,
    pub semantic_plan: SemanticPlan,
    pub execution_contract: ExecutionContract,
    pub evaluation: EvaluationOutput,
    pub candidate_result: AcceptedCandidateResult,
    pub commit_decision: AcceptDecision,
    pub trace_events: Vec<TraceEvent>,
    pub artifact_reuse: ArtifactReuseReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmpiricalOracleScenario {
    pub scenario_id: String,
    pub formula: String,
    pub entered_formula_text: String,
    pub stored_formula_text: Option<String>,
    pub input_bindings: BTreeMap<String, String>,
    pub cell_bindings: BTreeMap<String, String>,
    pub expected_result_summary: String,
    pub locale_profile: Option<String>,
    pub date_system: Option<String>,
    pub host_query_profile: Option<String>,
}

impl SingleFormulaHost {
    pub fn new(formula_stable_id: impl Into<String>, formula_text: impl Into<String>) -> Self {
        Self {
            formula_stable_id: formula_stable_id.into(),
            formula_text: formula_text.into(),
            formula_text_version: 1,
            structure_context_version: "host-struct-v1".to_string(),
            caller_row: 1,
            caller_col: 1,
            primary_locus: Locus {
                sheet_id: "sheet:default".to_string(),
                row: 1,
                col: 1,
            },
            defined_names: BTreeMap::new(),
            cell_values: BTreeMap::new(),
            now_serial: Some(46000.0),
            random_value: Some(0.25),
            next_session_id: 1,
            next_commit_attempt_id: 1,
            cached_artifacts: None,
        }
    }

    pub fn set_formula_text(&mut self, formula_text: impl Into<String>) {
        self.formula_text = formula_text.into();
        self.formula_text_version += 1;
        self.cached_artifacts = None;
    }

    pub fn set_defined_name_value(&mut self, name: impl Into<String>, value: EvalValue) {
        self.defined_names
            .insert(name.into(), DefinedNameBinding::Value(value));
    }

    pub fn set_defined_name_reference(
        &mut self,
        name: impl Into<String>,
        reference: ReferenceLike,
    ) {
        self.defined_names
            .insert(name.into(), DefinedNameBinding::Reference(reference));
    }

    pub fn set_defined_name_callable(
        &mut self,
        name: impl Into<String>,
        callable: CallableDefinedNameBinding,
    ) {
        self.defined_names
            .insert(name.into(), DefinedNameBinding::Callable(callable));
    }

    pub fn set_cell_value(&mut self, target: impl Into<String>, value: EvalValue) {
        self.cell_values.insert(target.into(), value);
    }

    pub fn recalc(
        &mut self,
        host_info: Option<&dyn HostInfoProvider>,
        locale_ctx: Option<&LocaleFormatContext<'_>>,
    ) -> Result<HostRecalcOutput, String> {
        self.recalc_with_backend(EvaluationBackend::OxFuncBacked, host_info, locale_ctx)
    }

    pub fn recalc_with_backend(
        &mut self,
        backend: EvaluationBackend,
        host_info: Option<&dyn HostInfoProvider>,
        locale_ctx: Option<&LocaleFormatContext<'_>>,
    ) -> Result<HostRecalcOutput, String> {
        let source = FormulaSourceRecord::new(
            self.formula_stable_id.clone(),
            self.formula_text_version,
            self.formula_text.clone(),
        );
        let cached_artifacts = self.cached_artifacts.as_ref();
        let parse = parse_formula_incremental(
            ParseRequest {
                source: source.clone(),
            },
            cached_artifacts.map(|artifacts| &artifacts.green_tree),
        );
        let red = project_red_view_incremental(
            source.formula_stable_id.clone(),
            &parse.green_tree,
            cached_artifacts.map(|artifacts| &artifacts.red_projection),
        );
        let bind = bind_formula_incremental(
            BindRequest {
                source: source.clone(),
                green_tree: parse.green_tree.clone(),
                red_projection: red.red_projection.clone(),
                context: BindContext {
                    structure_context_version: StructureContextVersion(
                        self.structure_context_version.clone(),
                    ),
                    caller_row: self.caller_row,
                    caller_col: self.caller_col,
                    formula_token: source.formula_token(),
                    names: self
                        .defined_names
                        .iter()
                        .map(|(name, binding)| {
                            (
                                name.clone(),
                                match binding {
                                    DefinedNameBinding::Value(_) => NameKind::ValueLike,
                                    DefinedNameBinding::Reference(_) => NameKind::ReferenceLike,
                                    DefinedNameBinding::Callable(_) => NameKind::ValueLike,
                                },
                            )
                        })
                        .collect(),
                    ..BindContext::default()
                },
            },
            cached_artifacts.map(|artifacts| &artifacts.bound_formula),
        );

        let semantic_plan_catalog_identity = "oxfunc:host".to_string();
        let locale_profile = locale_ctx.map(|ctx| format!("{:?}", ctx.profile.id));
        let date_system = locale_ctx.map(|ctx| format!("{:?}", ctx.date_system));
        let format_profile = locale_ctx.map(|_| "locale-format-context".to_string());
        let (semantic_plan, semantic_plan_reused) = if let Some(previous) = cached_artifacts {
            if previous.bound_formula.bind_hash == bind.bound_formula.bind_hash
                && previous.semantic_plan_catalog_identity == semantic_plan_catalog_identity
                && previous.locale_profile == locale_profile
                && previous.date_system == date_system
                && previous.format_profile == format_profile
            {
                (previous.semantic_plan.clone(), true)
            } else {
                (
                    compile_semantic_plan(CompileSemanticPlanRequest {
                        bound_formula: bind.bound_formula.clone(),
                        oxfunc_catalog_identity: semantic_plan_catalog_identity.clone(),
                        locale_profile: locale_profile.clone(),
                        date_system: date_system.clone(),
                        format_profile: format_profile.clone(),
                        library_context_snapshot: None,
                    })
                    .semantic_plan,
                    false,
                )
            }
        } else {
            (
                compile_semantic_plan(CompileSemanticPlanRequest {
                    bound_formula: bind.bound_formula.clone(),
                    oxfunc_catalog_identity: semantic_plan_catalog_identity.clone(),
                    locale_profile: locale_profile.clone(),
                    date_system: date_system.clone(),
                    format_profile: format_profile.clone(),
                    library_context_snapshot: None,
                })
                .semantic_plan,
                false,
            )
        };
        let artifact_reuse = ArtifactReuseReport {
            green_tree_reused: parse.reused_green_tree,
            red_projection_reused: red.reused_red_projection,
            bound_formula_reused: bind.reused_bound_formula,
            semantic_plan_reused,
        };
        self.cached_artifacts = Some(CachedHostArtifacts {
            green_tree: parse.green_tree,
            red_projection: red.red_projection,
            bound_formula: bind.bound_formula.clone(),
            semantic_plan: semantic_plan.clone(),
            semantic_plan_catalog_identity,
            locale_profile,
            date_system,
            format_profile,
        });

        let execution_contract = build_execution_contract(&semantic_plan);

        let mut evaluation_context = EvaluationContext::new(&bind.bound_formula, &semantic_plan);
        evaluation_context.backend = backend;
        evaluation_context.caller_row = self.caller_row as usize;
        evaluation_context.caller_col = self.caller_col as usize;
        evaluation_context.cell_values = self.cell_values.clone();
        evaluation_context.defined_names = self.defined_names.clone();
        evaluation_context.host_info = host_info;
        evaluation_context.locale_ctx = locale_ctx;
        evaluation_context.now_serial = self.now_serial;
        evaluation_context.random_value = self.random_value;

        let evaluation = evaluate_formula(evaluation_context).map_err(|err| err.message)?;

        let session_id = format!("session:{:04}", self.next_session_id);
        self.next_session_id += 1;
        let commit_attempt_id = format!("commit:{:04}", self.next_commit_attempt_id);
        self.next_commit_attempt_id += 1;

        let candidate_result = build_candidate_result(
            &source,
            &semantic_plan,
            &evaluation,
            &self.primary_locus,
            &session_id,
        );
        let commit_decision = commit_candidate(CommitRequest {
            candidate_result: candidate_result.clone(),
            commit_attempt_id: commit_attempt_id.clone(),
            observed_fence: candidate_result.fence_snapshot.clone(),
        });
        let trace_events =
            build_trace_events(&candidate_result, &commit_decision, &commit_attempt_id);

        Ok(HostRecalcOutput {
            source,
            semantic_plan,
            execution_contract,
            evaluation,
            candidate_result,
            commit_decision,
            trace_events,
            artifact_reuse,
        })
    }

    pub fn recalc_with_observed_fence_override(
        &mut self,
        host_info: Option<&dyn HostInfoProvider>,
        locale_ctx: Option<&LocaleFormatContext<'_>>,
        observed_fence: FenceSnapshot,
    ) -> Result<HostRecalcOutput, String> {
        let mut output = self.recalc(host_info, locale_ctx)?;
        let commit_attempt_id = format!("commit:{:04}:override", self.next_commit_attempt_id);
        self.next_commit_attempt_id += 1;
        output.commit_decision = commit_candidate(CommitRequest {
            candidate_result: output.candidate_result.clone(),
            commit_attempt_id: commit_attempt_id.clone(),
            observed_fence,
        });
        output.trace_events = build_trace_events(
            &output.candidate_result,
            &output.commit_decision,
            &commit_attempt_id,
        );
        Ok(output)
    }

    pub fn run_empirical_oracle_scenario(
        scenario: &EmpiricalOracleScenario,
        host_info: Option<&dyn HostInfoProvider>,
        locale_ctx: Option<&LocaleFormatContext<'_>>,
    ) -> Result<HostRecalcOutput, String> {
        let mut host = SingleFormulaHost::new(&scenario.scenario_id, &scenario.formula);
        for (name, summary) in &scenario.input_bindings {
            apply_empirical_input_binding(&mut host, name, summary)?;
        }
        for (target, summary) in &scenario.cell_bindings {
            apply_empirical_cell_binding(&mut host, target, summary)?;
        }
        host.recalc(host_info, locale_ctx)
    }
}

fn build_candidate_result(
    source: &FormulaSourceRecord,
    semantic_plan: &SemanticPlan,
    evaluation: &EvaluationOutput,
    primary_locus: &Locus,
    session_id: &str,
) -> AcceptedCandidateResult {
    let candidate_result_id = format!("candidate:{}", source.formula_text_version.0);
    let (worksheet_value_class, value_payload, extent) =
        value_payload_for_eval_value(&evaluation.oxfunc_value);
    let spill_events = if let Some(extent) = &extent {
        if extent.rows > 1 || extent.cols > 1 {
            vec![SpillEvent {
                spill_event_kind: SpillEventKind::SpillTakeover,
                formula_stable_id: source.formula_stable_id.0.clone(),
                anchor_locus: primary_locus.clone(),
                intended_extent: extent.clone(),
                affected_extent: Some(extent.clone()),
                blocking_loci: Vec::new(),
                blocking_reason_class: None,
                correlation_id: candidate_result_id.clone(),
            }]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let capability_effect_facts = if semantic_plan.execution_profile.requires_host_query {
        vec![CapabilityEffectFact {
            formula_stable_id: source.formula_stable_id.0.clone(),
            capability_kind: "host_query".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "admitted".to_string(),
            fallback_class: None,
        }]
    } else {
        Vec::new()
    };

    let format_dependency_facts = if semantic_plan.execution_profile.requires_locale {
        vec![FormatDependencyFact {
            formula_stable_id: source.formula_stable_id.0.clone(),
            dependency_token: "locale_format_context".to_string(),
            dependency_class: "semantic_formatting".to_string(),
            scope: semantic_plan.locale_profile.clone(),
        }]
    } else {
        Vec::new()
    };
    let format_delta = format_delta_from_evaluation(source, evaluation, primary_locus);
    let display_delta = display_delta_from_evaluation(source, evaluation, primary_locus);

    AcceptedCandidateResult {
        formula_stable_id: source.formula_stable_id.0.clone(),
        session_id: Some(session_id.to_string()),
        candidate_result_id: candidate_result_id.clone(),
        fence_snapshot: FenceSnapshot {
            formula_token: source.formula_token().0,
            snapshot_epoch: format!("epoch:{}", source.formula_text_version.0),
            bind_hash: semantic_plan.bind_hash.clone(),
            profile_version: semantic_plan
                .locale_profile
                .clone()
                .unwrap_or_else(|| "profile:default".to_string()),
            capability_view_key: Some(format!(
                "cap:{}",
                if semantic_plan.execution_profile.requires_host_query {
                    "host-query"
                } else {
                    "default"
                }
            )),
        },
        value_delta: ValueDelta {
            formula_stable_id: source.formula_stable_id.0.clone(),
            primary_locus: primary_locus.clone(),
            affected_value_loci: vec![primary_locus.clone()],
            published_value_class: worksheet_value_class,
            published_payload: value_payload,
            result_extent: extent.clone(),
            candidate_result_id: Some(candidate_result_id.clone()),
        },
        shape_delta: ShapeDelta {
            formula_stable_id: source.formula_stable_id.0.clone(),
            anchor_locus: primary_locus.clone(),
            intended_extent: extent.clone().unwrap_or(Extent { rows: 1, cols: 1 }),
            published_extent: extent.clone(),
            blocked_loci: Vec::new(),
            shape_outcome_class: ShapeOutcomeClass::Established,
            candidate_result_id: Some(candidate_result_id.clone()),
        },
        topology_delta: TopologyDelta {
            formula_stable_id: source.formula_stable_id.0.clone(),
            dependency_additions: semantic_plan
                .diagnostics
                .iter()
                .map(|diag| diag.message.clone())
                .collect(),
            dependency_removals: Vec::new(),
            dependency_reclassifications: Vec::new(),
            dependency_consequence_facts: dependency_consequence_facts_for_plan(
                source,
                semantic_plan,
            ),
            dynamic_reference_facts: Vec::new(),
            spill_facts: Vec::new(),
            format_dependency_facts,
            capability_effect_facts,
            candidate_result_id: Some(candidate_result_id.clone()),
        },
        format_delta,
        display_delta,
        spill_events,
        execution_profile: Some(semantic_plan.execution_profile.clone()),
        trace_correlation_id: format!("trace:{candidate_result_id}"),
    }
}

fn dependency_consequence_facts_for_plan(
    source: &FormulaSourceRecord,
    semantic_plan: &SemanticPlan,
) -> Vec<DependencyConsequenceFact> {
    let mut facts = semantic_plan
        .diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| DependencyConsequenceFact {
            formula_stable_id: source.formula_stable_id.0.clone(),
            dependency_identity: format!("diagnostic:{index}"),
            consequence_kind: "addition".to_string(),
            evidence_class: "semantic_diagnostic".to_string(),
            projection_state: diagnostic.message.clone(),
        })
        .collect::<Vec<_>>();

    if semantic_plan
        .capability_requirements
        .iter()
        .any(|item| item == "external_reference")
    {
        facts.push(DependencyConsequenceFact {
            formula_stable_id: source.formula_stable_id.0.clone(),
            dependency_identity: "reference_lane:external_reference".to_string(),
            consequence_kind: "reclassification".to_string(),
            evidence_class: "dynamic_reference_deferred".to_string(),
            projection_state: "retained_without_target_resolution".to_string(),
        });
    }

    facts
}

fn format_delta_from_evaluation(
    source: &FormulaSourceRecord,
    evaluation: &EvaluationOutput,
    primary_locus: &Locus,
) -> Option<FormatDelta> {
    evaluation
        .result
        .format_hint
        .as_ref()
        .map(|hint| FormatDelta {
            formula_stable_id: source.formula_stable_id.0.clone(),
            target_loci: vec![primary_locus.clone()],
            format_effect_class: hint.clone(),
            format_effect_payload: evaluation.result.payload_summary.clone(),
        })
}

fn display_delta_from_evaluation(
    source: &FormulaSourceRecord,
    evaluation: &EvaluationOutput,
    primary_locus: &Locus,
) -> Option<DisplayDelta> {
    evaluation
        .result
        .publication_hint
        .as_ref()
        .map(|hint| DisplayDelta {
            formula_stable_id: source.formula_stable_id.0.clone(),
            target_loci: vec![primary_locus.clone()],
            display_effect_class: hint.clone(),
            display_effect_payload: evaluation.result.payload_summary.clone(),
        })
}

fn apply_empirical_input_binding(
    host: &mut SingleFormulaHost,
    name: &str,
    summary: &str,
) -> Result<(), String> {
    match parse_empirical_defined_name_binding(summary)? {
        DefinedNameBinding::Value(value) => host.set_defined_name_value(name, value),
        DefinedNameBinding::Reference(reference) => {
            host.set_defined_name_reference(name, reference)
        }
        DefinedNameBinding::Callable(callable) => host.set_defined_name_callable(name, callable),
    }
    Ok(())
}

fn apply_empirical_cell_binding(
    host: &mut SingleFormulaHost,
    target: &str,
    summary: &str,
) -> Result<(), String> {
    match parse_empirical_defined_name_binding(summary)? {
        DefinedNameBinding::Value(value) => {
            host.set_cell_value(target, value);
            Ok(())
        }
        DefinedNameBinding::Reference(_) => Err(format!(
            "cell empirical binding cannot be a reference for {target}: {summary}"
        )),
        DefinedNameBinding::Callable(_) => Err(format!(
            "cell empirical binding cannot be callable for {target}: {summary}"
        )),
    }
}

fn parse_empirical_defined_name_binding(summary: &str) -> Result<DefinedNameBinding, String> {
    if let Some(target) = summary
        .strip_prefix("Reference(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return Ok(DefinedNameBinding::Reference(ReferenceLike {
            kind: oxfunc_core::value::ReferenceKind::A1,
            target: target.to_string(),
        }));
    }

    Ok(DefinedNameBinding::Value(parse_empirical_eval_value(
        summary,
    )?))
}

fn parse_empirical_eval_value(summary: &str) -> Result<EvalValue, String> {
    if let Some(number) = summary
        .strip_prefix("Number(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parsed = number
            .parse::<f64>()
            .map_err(|_| format!("invalid numeric empirical binding {summary}"))?;
        return Ok(EvalValue::Number(parsed));
    }

    if let Some(text) = summary
        .strip_prefix("Text(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return Ok(EvalValue::Text(ExcelText::from_utf16_code_units(
            text.encode_utf16().collect(),
        )));
    }

    if let Some(logical) = summary
        .strip_prefix("Logical(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return match logical {
            "true" | "True" | "TRUE" => Ok(EvalValue::Logical(true)),
            "false" | "False" | "FALSE" => Ok(EvalValue::Logical(false)),
            _ => Err(format!("invalid logical empirical binding {summary}")),
        };
    }

    if let Some(code) = summary
        .strip_prefix("Error(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let error = match code {
            "#REF!" => oxfunc_core::value::WorksheetErrorCode::Ref,
            "#VALUE!" => oxfunc_core::value::WorksheetErrorCode::Value,
            "#NAME?" => oxfunc_core::value::WorksheetErrorCode::Name,
            "#N/A" => oxfunc_core::value::WorksheetErrorCode::NA,
            "#DIV/0!" => oxfunc_core::value::WorksheetErrorCode::Div0,
            "#NUM!" => oxfunc_core::value::WorksheetErrorCode::Num,
            "#NULL!" => oxfunc_core::value::WorksheetErrorCode::Null,
            _ => {
                return Err(format!(
                    "unsupported error empirical binding summary {summary}"
                ));
            }
        };
        return Ok(EvalValue::Error(error));
    }

    Err(format!("unsupported empirical binding summary: {summary}"))
}

fn build_trace_events(
    candidate_result: &AcceptedCandidateResult,
    commit_decision: &AcceptDecision,
    commit_attempt_id: &str,
) -> Vec<TraceEvent> {
    let mut events = vec![TraceEvent {
        trace_schema_id: "trace:v1".to_string(),
        event_kind: TraceEventKind::AcceptedCandidateResultBuilt,
        formula_stable_id: candidate_result.formula_stable_id.clone(),
        session_id: candidate_result.session_id.clone(),
        candidate_result_id: Some(candidate_result.candidate_result_id.clone()),
        commit_attempt_id: None,
        event_order_key: 1,
        event_payload: TracePayload::CandidateBuilt {
            candidate_result_id: candidate_result.candidate_result_id.clone(),
        },
    }];

    match commit_decision {
        AcceptDecision::Accepted(bundle) => events.push(TraceEvent {
            trace_schema_id: "trace:v1".to_string(),
            event_kind: TraceEventKind::CommitAccepted,
            formula_stable_id: bundle.formula_stable_id.clone(),
            session_id: candidate_result.session_id.clone(),
            candidate_result_id: Some(bundle.candidate_result_id.clone()),
            commit_attempt_id: Some(commit_attempt_id.to_string()),
            event_order_key: 2,
            event_payload: TracePayload::CommitAccepted {
                commit_attempt_id: commit_attempt_id.to_string(),
                candidate_result_id: bundle.candidate_result_id.clone(),
            },
        }),
        AcceptDecision::Rejected(reject) => {
            events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::CommitRejected,
                formula_stable_id: reject.formula_stable_id.clone(),
                session_id: reject.session_id.clone(),
                candidate_result_id: Some(candidate_result.candidate_result_id.clone()),
                commit_attempt_id: Some(commit_attempt_id.to_string()),
                event_order_key: 2,
                event_payload: TracePayload::CommitRejected {
                    commit_attempt_id: commit_attempt_id.to_string(),
                    reject_code: reject.reject_code,
                },
            });
            events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::RejectIssued,
                formula_stable_id: reject.formula_stable_id.clone(),
                session_id: reject.session_id.clone(),
                candidate_result_id: Some(candidate_result.candidate_result_id.clone()),
                commit_attempt_id: Some(commit_attempt_id.to_string()),
                event_order_key: 3,
                event_payload: TracePayload::RejectIssued {
                    reject_code: reject.reject_code,
                },
            });
        }
    }

    events
}

fn value_payload_for_eval_value(
    value: &EvalValue,
) -> (WorksheetValueClass, ValuePayload, Option<Extent>) {
    match value {
        EvalValue::Number(number) => (
            WorksheetValueClass::Scalar,
            ValuePayload::Number(format!("{number}")),
            Some(Extent { rows: 1, cols: 1 }),
        ),
        EvalValue::Text(text) => (
            WorksheetValueClass::Scalar,
            ValuePayload::Text(text.to_string_lossy()),
            Some(Extent { rows: 1, cols: 1 }),
        ),
        EvalValue::Logical(value) => (
            WorksheetValueClass::Scalar,
            ValuePayload::Logical(*value),
            Some(Extent { rows: 1, cols: 1 }),
        ),
        EvalValue::Error(code) => (
            WorksheetValueClass::Error,
            ValuePayload::ErrorCode(format!("{code:?}")),
            Some(Extent { rows: 1, cols: 1 }),
        ),
        EvalValue::Array(array) => (
            WorksheetValueClass::ArrayAnchor,
            ValuePayload::Text(format!(
                "Array({}x{})",
                array.shape().rows,
                array.shape().cols
            )),
            Some(Extent {
                rows: array.shape().rows as u32,
                cols: array.shape().cols as u32,
            }),
        ),
        EvalValue::Reference(reference) => (
            WorksheetValueClass::Scalar,
            ValuePayload::Text(format!("Reference({})", reference.target)),
            Some(Extent { rows: 1, cols: 1 }),
        ),
        EvalValue::Lambda(name) => (
            WorksheetValueClass::Scalar,
            ValuePayload::Text(format!("Lambda({})", name.callable_token)),
            Some(Extent { rows: 1, cols: 1 }),
        ),
    }
}
