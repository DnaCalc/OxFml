use std::collections::BTreeMap;

use oxfunc_core::host_info::HostInfoProvider;
use oxfunc_core::locale_format::LocaleFormatContext;
use oxfunc_core::value::EvalValue;

use crate::binding::BoundFormula;
use crate::eval::{
    DefinedNameBinding, EvaluationBackend, EvaluationContext, EvaluationOutput, evaluate_formula,
};
use crate::scheduler::{ExecutionRestriction, build_execution_contract};
use crate::seam::{
    AcceptDecision, AcceptedCandidateResult, CapabilityDenialContext, CapabilityEffectFact,
    CommitRequest, DependencyConsequenceFact, DisplayDelta, DynamicReferenceFact, Extent,
    FenceSnapshot, FormatDelta, FormatDependencyFact, Locus, RejectCode, RejectContext,
    RejectRecord, ResourceInvariantContext, SessionTerminationContext, ShapeDelta,
    ShapeOutcomeClass, SpillEvent, SpillEventKind, TopologyDelta, TraceEvent, TraceEventKind,
    TracePayload, ValueDelta, ValuePayload, WorksheetValueClass, commit_candidate,
};
use crate::semantics::SemanticPlan;
use crate::source::FormulaSourceRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareRequest {
    pub source: FormulaSourceRecord,
    pub bound_formula: BoundFormula,
    pub semantic_plan: SemanticPlan,
    pub primary_locus: Locus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedSession {
    pub source: FormulaSourceRecord,
    pub bound_formula: BoundFormula,
    pub semantic_plan: SemanticPlan,
    pub primary_locus: Locus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionPhase {
    Open,
    CapabilityViewEstablished,
    Executed,
    Committed,
    Rejected,
    Aborted,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityViewSpec {
    pub host_query_enabled: bool,
    pub locale_format_enabled: bool,
    pub caller_context_enabled: bool,
    pub external_provider_enabled: bool,
}

impl Default for CapabilityViewSpec {
    fn default() -> Self {
        Self {
            host_query_enabled: false,
            locale_format_enabled: false,
            caller_context_enabled: true,
            external_provider_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityView {
    pub capability_view_key: String,
    pub spec: CapabilityViewSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenSessionResult {
    pub session_id: String,
    pub fence_snapshot: FenceSnapshot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionRecord {
    pub session_id: String,
    pub prepared: PreparedSession,
    pub phase: SessionPhase,
    pub capability_view: Option<CapabilityView>,
    pub candidate_result: Option<AcceptedCandidateResult>,
    pub last_reject: Option<RejectRecord>,
    pub trace_events: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayEntry {
    pub overlay_entry_id: String,
    pub overlay_scope_key: String,
    pub overlay_family: String,
    pub session_id: String,
    pub formula_stable_id: String,
}

pub struct ExecuteRequest<'a> {
    pub session_id: String,
    pub backend: EvaluationBackend,
    pub caller_row: usize,
    pub caller_col: usize,
    pub cell_values: BTreeMap<String, EvalValue>,
    pub defined_names: BTreeMap<String, DefinedNameBinding>,
    pub locale_ctx: Option<&'a LocaleFormatContext<'a>>,
    pub host_info: Option<&'a dyn HostInfoProvider>,
    pub now_serial: Option<f64>,
    pub random_value: Option<f64>,
}

#[derive(Debug, Default)]
pub struct SessionService {
    next_session_id: u64,
    sessions: BTreeMap<String, SessionRecord>,
    active_locus_claims: BTreeMap<String, String>,
    session_overlays: BTreeMap<String, Vec<OverlayEntry>>,
}

impl SessionService {
    pub fn new() -> Self {
        Self {
            next_session_id: 1,
            sessions: BTreeMap::new(),
            active_locus_claims: BTreeMap::new(),
            session_overlays: BTreeMap::new(),
        }
    }

    pub fn prepare(&self, request: PrepareRequest) -> Result<PreparedSession, RejectRecord> {
        if request.source.formula_stable_id.0 != request.bound_formula.formula_stable_id
            || request.source.formula_stable_id.0 != request.semantic_plan.formula_stable_id
        {
            return Err(prepare_mismatch_reject(
                &request.source.formula_stable_id.0,
                "formula_stable_id_mismatch",
            ));
        }

        if request.bound_formula.bind_hash != request.semantic_plan.bind_hash {
            return Err(prepare_mismatch_reject(
                &request.source.formula_stable_id.0,
                "bind_hash_mismatch",
            ));
        }

        Ok(PreparedSession {
            source: request.source,
            bound_formula: request.bound_formula,
            semantic_plan: request.semantic_plan,
            primary_locus: request.primary_locus,
        })
    }

    pub fn open_session(&mut self, prepared: PreparedSession) -> OpenSessionResult {
        let session_id = format!("session:{:04}", self.next_session_id);
        self.next_session_id += 1;
        let fence_snapshot = fence_snapshot_for_prepared(&prepared, None);

        self.sessions.insert(
            session_id.clone(),
            SessionRecord {
                session_id: session_id.clone(),
                prepared: prepared.clone(),
                phase: SessionPhase::Open,
                capability_view: None,
                candidate_result: None,
                last_reject: None,
                trace_events: vec![TraceEvent {
                    trace_schema_id: "trace:v1".to_string(),
                    event_kind: TraceEventKind::SessionOpened,
                    formula_stable_id: prepared.source.formula_stable_id.0.clone(),
                    session_id: Some(session_id.clone()),
                    candidate_result_id: None,
                    commit_attempt_id: None,
                    event_order_key: 1,
                    event_payload: TracePayload::SessionOpened {
                        session_id: session_id.clone(),
                    },
                }],
            },
        );

        OpenSessionResult {
            session_id,
            fence_snapshot,
        }
    }

    pub fn establish_capability_view(
        &mut self,
        session_id: &str,
        spec: CapabilityViewSpec,
    ) -> Result<CapabilityView, RejectRecord> {
        let record = self
            .sessions
            .get_mut(session_id)
            .expect("session should exist for capability view");

        if matches!(record.phase, SessionPhase::Aborted | SessionPhase::Expired) {
            return Err(session_terminated_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "terminated_before_capability_view",
                matches!(record.phase, SessionPhase::Expired),
                None,
            ));
        }
        if matches!(
            record.phase,
            SessionPhase::Executed | SessionPhase::Committed
        ) {
            let reject = structural_conflict_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "capability_view_after_execution",
                "not_admissible",
            );
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(trace_reject_event(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                order,
                reject.reject_code,
            ));
            return Err(reject);
        }
        if matches!(record.phase, SessionPhase::Rejected) {
            return Err(record
                .last_reject
                .clone()
                .expect("rejected session should carry reject"));
        }

        if let Some(reject) = capability_denial_for_spec(session_id, record, &spec) {
            record.phase = SessionPhase::Rejected;
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(trace_reject_event(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                order,
                reject.reject_code,
            ));
            return Err(reject);
        }

        let view = CapabilityView {
            capability_view_key: capability_view_key_for_spec(&spec),
            spec,
        };
        record.capability_view = Some(view.clone());
        record.phase = SessionPhase::CapabilityViewEstablished;
        let order = record.trace_events.len() as u64 + 1;
        record.trace_events.push(TraceEvent {
            trace_schema_id: "trace:v1".to_string(),
            event_kind: TraceEventKind::CapabilityViewEstablished,
            formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
            session_id: Some(session_id.to_string()),
            candidate_result_id: None,
            commit_attempt_id: None,
            event_order_key: order,
            event_payload: TracePayload::CapabilityViewEstablished {
                capability_view_key: view.capability_view_key.clone(),
            },
        });
        Ok(view)
    }

    pub fn execute(
        &mut self,
        request: ExecuteRequest<'_>,
    ) -> Result<AcceptedCandidateResult, RejectRecord> {
        let should_claim = {
            let record = self
                .sessions
                .get_mut(&request.session_id)
                .expect("session should exist for execute");
            match record.phase {
                SessionPhase::Aborted | SessionPhase::Expired => {
                    let reject = session_terminated_reject(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        "terminated_before_execute",
                        matches!(record.phase, SessionPhase::Expired),
                        None,
                    );
                    record.last_reject = Some(reject.clone());
                    return Err(reject);
                }
                SessionPhase::Rejected => {
                    return Err(record
                        .last_reject
                        .clone()
                        .expect("rejected session should carry reject"));
                }
                SessionPhase::Executed => {
                    let reject = structural_conflict_reject(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        "candidate_already_built",
                        "not_admissible",
                    );
                    record.phase = SessionPhase::Rejected;
                    record.last_reject = Some(reject.clone());
                    let order = record.trace_events.len() as u64 + 1;
                    record.trace_events.push(trace_reject_event(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        order,
                        reject.reject_code,
                    ));
                    return Err(reject);
                }
                SessionPhase::Committed => {
                    let reject = structural_conflict_reject(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        "execute_after_commit",
                        "not_admissible",
                    );
                    record.last_reject = Some(reject.clone());
                    let order = record.trace_events.len() as u64 + 1;
                    record.trace_events.push(trace_reject_event(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        order,
                        reject.reject_code,
                    ));
                    return Err(reject);
                }
                SessionPhase::Open => {
                    let reject = capability_denial_reject(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        "capability_view",
                        "execute",
                        "not_established",
                    );
                    record.phase = SessionPhase::Rejected;
                    record.last_reject = Some(reject.clone());
                    return Err(reject);
                }
                SessionPhase::CapabilityViewEstablished => {}
            }
            requires_single_locus_claim(&record.prepared.semantic_plan)
        };

        let (formula_stable_id, primary_locus) = {
            let record = self
                .sessions
                .get(&request.session_id)
                .expect("session should exist for execute");
            (
                record.prepared.source.formula_stable_id.0.clone(),
                record.prepared.primary_locus.clone(),
            )
        };
        let locus_claim_key = locus_claim_key(&primary_locus);
        if let Some(owner_session_id) = self.active_locus_claims.get(&locus_claim_key) {
            if owner_session_id != &request.session_id {
                let reject = contention_conflict_reject(
                    &formula_stable_id,
                    &request.session_id,
                    &primary_locus,
                    owner_session_id,
                );
                let record = self
                    .sessions
                    .get_mut(&request.session_id)
                    .expect("session should exist for execute");
                record.phase = SessionPhase::Rejected;
                record.last_reject = Some(reject.clone());
                let order = record.trace_events.len() as u64 + 1;
                record.trace_events.push(trace_reject_event(
                    &formula_stable_id,
                    &request.session_id,
                    order,
                    reject.reject_code,
                ));
                return Err(reject);
            }
        } else if should_claim {
            self.active_locus_claims
                .insert(locus_claim_key.clone(), request.session_id.clone());
        }

        let candidate = {
            let record = self
                .sessions
                .get_mut(&request.session_id)
                .expect("session should exist for execute");
            let mut evaluation_context = EvaluationContext::new(
                &record.prepared.bound_formula,
                &record.prepared.semantic_plan,
            );
            evaluation_context.backend = request.backend;
            evaluation_context.caller_row = request.caller_row;
            evaluation_context.caller_col = request.caller_col;
            evaluation_context.cell_values = request.cell_values;
            evaluation_context.defined_names = request.defined_names;
            evaluation_context.locale_ctx = request.locale_ctx;
            evaluation_context.host_info = request.host_info;
            evaluation_context.now_serial = request.now_serial;
            evaluation_context.random_value = request.random_value;

            let evaluation = match evaluate_formula(evaluation_context) {
                Ok(evaluation) => evaluation,
                Err(error) => {
                    let reject = RejectRecord {
                        formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
                        session_id: Some(request.session_id.clone()),
                        commit_attempt_id: None,
                        reject_code: RejectCode::ResourceInvariantFailure,
                        context: RejectContext::ResourceInvariant(ResourceInvariantContext {
                            failure_family: "execute_failure".to_string(),
                            machine_detail_code: error.message,
                            resource_class: Some("evaluation".to_string()),
                        }),
                        trace_correlation_id: format!("trace:{}", request.session_id),
                    };
                    record.phase = SessionPhase::Rejected;
                    record.last_reject = Some(reject.clone());
                    let order = record.trace_events.len() as u64 + 1;
                    record.trace_events.push(trace_reject_event(
                        &record.prepared.source.formula_stable_id.0,
                        &request.session_id,
                        order,
                        reject.reject_code,
                    ));
                    return Err(reject);
                }
            };

            let candidate = build_candidate_result(
                &record.prepared.source,
                &record.prepared.semantic_plan,
                &evaluation,
                &record.prepared.primary_locus,
                &request.session_id,
                record
                    .capability_view
                    .as_ref()
                    .map(|view| view.capability_view_key.as_str()),
            );
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::AcceptedCandidateResultBuilt,
                formula_stable_id: candidate.formula_stable_id.clone(),
                session_id: Some(request.session_id.clone()),
                candidate_result_id: Some(candidate.candidate_result_id.clone()),
                commit_attempt_id: None,
                event_order_key: order,
                event_payload: TracePayload::CandidateBuilt {
                    candidate_result_id: candidate.candidate_result_id.clone(),
                },
            });
            record.phase = SessionPhase::Executed;
            record.candidate_result = Some(candidate.clone());
            candidate
        };
        self.session_overlays.insert(
            request.session_id.clone(),
            overlay_entries_for_candidate(&candidate),
        );
        Ok(candidate)
    }

    pub fn commit(
        &mut self,
        session_id: &str,
        commit_attempt_id: impl Into<String>,
        observed_fence: FenceSnapshot,
    ) -> AcceptDecision {
        let record = self
            .sessions
            .get_mut(session_id)
            .expect("session should exist for commit");

        if matches!(record.phase, SessionPhase::Aborted | SessionPhase::Expired) {
            let reject = session_terminated_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "terminated_before_commit",
                matches!(record.phase, SessionPhase::Expired),
                None,
            );
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(trace_reject_event(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                order,
                reject.reject_code,
            ));
            return AcceptDecision::Rejected(reject);
        }
        if matches!(record.phase, SessionPhase::Committed) {
            let reject = structural_conflict_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "commit_after_commit",
                "not_admissible",
            );
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::CommitRejected,
                formula_stable_id: reject.formula_stable_id.clone(),
                session_id: reject.session_id.clone(),
                candidate_result_id: record
                    .candidate_result
                    .as_ref()
                    .map(|candidate| candidate.candidate_result_id.clone()),
                commit_attempt_id: Some(commit_attempt_id.into()),
                event_order_key: order,
                event_payload: TracePayload::CommitRejected {
                    commit_attempt_id: "duplicate_commit".to_string(),
                    reject_code: reject.reject_code,
                },
            });
            return AcceptDecision::Rejected(reject);
        }

        let Some(candidate_result) = record.candidate_result.clone() else {
            let reject = structural_conflict_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "commit_without_candidate",
                "execute_required",
            );
            record.phase = SessionPhase::Rejected;
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::CommitRejected,
                formula_stable_id: reject.formula_stable_id.clone(),
                session_id: reject.session_id.clone(),
                candidate_result_id: None,
                commit_attempt_id: Some("commit_without_candidate".to_string()),
                event_order_key: order,
                event_payload: TracePayload::CommitRejected {
                    commit_attempt_id: "commit_without_candidate".to_string(),
                    reject_code: reject.reject_code,
                },
            });
            return AcceptDecision::Rejected(reject);
        };

        let commit_attempt_id = commit_attempt_id.into();
        let decision = commit_candidate(CommitRequest {
            candidate_result,
            commit_attempt_id: commit_attempt_id.clone(),
            observed_fence,
        });
        let order = record.trace_events.len() as u64 + 1;

        let should_release = {
            match &decision {
                AcceptDecision::Accepted(bundle) => {
                    record.phase = SessionPhase::Committed;
                    record.trace_events.push(TraceEvent {
                        trace_schema_id: "trace:v1".to_string(),
                        event_kind: TraceEventKind::CommitAccepted,
                        formula_stable_id: bundle.formula_stable_id.clone(),
                        session_id: Some(session_id.to_string()),
                        candidate_result_id: Some(bundle.candidate_result_id.clone()),
                        commit_attempt_id: Some(commit_attempt_id.clone()),
                        event_order_key: order,
                        event_payload: TracePayload::CommitAccepted {
                            commit_attempt_id,
                            candidate_result_id: bundle.candidate_result_id.clone(),
                        },
                    });
                    true
                }
                AcceptDecision::Rejected(reject) => {
                    record.phase = SessionPhase::Rejected;
                    record.last_reject = Some(reject.clone());
                    record.trace_events.push(TraceEvent {
                        trace_schema_id: "trace:v1".to_string(),
                        event_kind: TraceEventKind::CommitRejected,
                        formula_stable_id: reject.formula_stable_id.clone(),
                        session_id: reject.session_id.clone(),
                        candidate_result_id: record
                            .candidate_result
                            .as_ref()
                            .map(|candidate| candidate.candidate_result_id.clone()),
                        commit_attempt_id: Some(commit_attempt_id.clone()),
                        event_order_key: order,
                        event_payload: TracePayload::CommitRejected {
                            commit_attempt_id,
                            reject_code: reject.reject_code,
                        },
                    });
                    true
                }
            }
        };
        if should_release {
            self.release_runtime_state(session_id);
        }
        decision
    }

    pub fn abort_session(&mut self, session_id: &str, cause: Option<String>) -> RejectRecord {
        let reject = {
            let record = self
                .sessions
                .get_mut(session_id)
                .expect("session should exist for abort");
            record.phase = SessionPhase::Aborted;
            let reject = session_terminated_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "session_aborted",
                false,
                cause,
            );
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::SessionAborted,
                formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
                session_id: Some(session_id.to_string()),
                candidate_result_id: None,
                commit_attempt_id: None,
                event_order_key: order,
                event_payload: TracePayload::SessionAborted {
                    termination_class: "aborted".to_string(),
                },
            });
            reject
        };
        self.release_runtime_state(session_id);
        reject
    }

    pub fn expire_session(&mut self, session_id: &str, cause: Option<String>) -> RejectRecord {
        let reject = {
            let record = self
                .sessions
                .get_mut(session_id)
                .expect("session should exist for expire");
            record.phase = SessionPhase::Expired;
            let reject = session_terminated_reject(
                &record.prepared.source.formula_stable_id.0,
                session_id,
                "session_expired",
                true,
                cause,
            );
            record.last_reject = Some(reject.clone());
            let order = record.trace_events.len() as u64 + 1;
            record.trace_events.push(TraceEvent {
                trace_schema_id: "trace:v1".to_string(),
                event_kind: TraceEventKind::SessionExpired,
                formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
                session_id: Some(session_id.to_string()),
                candidate_result_id: None,
                commit_attempt_id: None,
                event_order_key: order,
                event_payload: TracePayload::SessionExpired {
                    termination_class: "expired".to_string(),
                },
            });
            reject
        };
        self.release_runtime_state(session_id);
        reject
    }

    pub fn session(&self, session_id: &str) -> Option<&SessionRecord> {
        self.sessions.get(session_id)
    }

    pub fn overlay_entries(&self, session_id: &str) -> &[OverlayEntry] {
        self.session_overlays
            .get(session_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn active_locus_claim_owner(&self, locus: &Locus) -> Option<&str> {
        self.active_locus_claims
            .get(&locus_claim_key(locus))
            .map(String::as_str)
    }

    fn release_runtime_state(&mut self, session_id: &str) {
        self.session_overlays.remove(session_id);
        self.active_locus_claims
            .retain(|_, owner_session_id| owner_session_id != session_id);
    }
}

fn capability_denial_for_spec(
    session_id: &str,
    record: &SessionRecord,
    spec: &CapabilityViewSpec,
) -> Option<RejectRecord> {
    let profile = &record.prepared.semantic_plan.execution_profile;
    if profile.requires_host_query && !spec.host_query_enabled {
        Some(capability_denial_reject(
            &record.prepared.source.formula_stable_id.0,
            session_id,
            "host_query",
            "capability_view",
            "unavailable",
        ))
    } else if profile.requires_locale && !spec.locale_format_enabled {
        Some(capability_denial_reject(
            &record.prepared.source.formula_stable_id.0,
            session_id,
            "locale_format_context",
            "capability_view",
            "unavailable",
        ))
    } else if profile.requires_caller_context && !spec.caller_context_enabled {
        Some(capability_denial_reject(
            &record.prepared.source.formula_stable_id.0,
            session_id,
            "caller_context",
            "capability_view",
            "unavailable",
        ))
    } else if profile.requires_async_coupling && !spec.external_provider_enabled {
        Some(capability_denial_reject(
            &record.prepared.source.formula_stable_id.0,
            session_id,
            "external_provider",
            "capability_view",
            "unavailable",
        ))
    } else {
        None
    }
}

fn prepare_mismatch_reject(formula_stable_id: &str, detail: &str) -> RejectRecord {
    RejectRecord {
        formula_stable_id: formula_stable_id.to_string(),
        session_id: None,
        commit_attempt_id: None,
        reject_code: RejectCode::BindMismatch,
        context: RejectContext::ResourceInvariant(ResourceInvariantContext {
            failure_family: "prepare_mismatch".to_string(),
            machine_detail_code: detail.to_string(),
            resource_class: Some("prepare_request".to_string()),
        }),
        trace_correlation_id: format!("trace:prepare:{formula_stable_id}"),
    }
}

fn fence_snapshot_for_prepared(
    prepared: &PreparedSession,
    capability_view_key: Option<&str>,
) -> FenceSnapshot {
    FenceSnapshot {
        formula_token: prepared.source.formula_token().0,
        snapshot_epoch: format!("epoch:{}", prepared.source.formula_text_version.0),
        bind_hash: prepared.semantic_plan.bind_hash.clone(),
        profile_version: prepared
            .semantic_plan
            .locale_profile
            .clone()
            .unwrap_or_else(|| "profile:default".to_string()),
        capability_view_key: capability_view_key.map(ToString::to_string),
    }
}

fn capability_view_key_for_spec(spec: &CapabilityViewSpec) -> String {
    format!(
        "cap:hq:{}:loc:{}:ctx:{}:ext:{}",
        spec.host_query_enabled as u8,
        spec.locale_format_enabled as u8,
        spec.caller_context_enabled as u8,
        spec.external_provider_enabled as u8
    )
}

fn capability_denial_reject(
    formula_stable_id: &str,
    session_id: &str,
    capability_kind: &str,
    phase_kind: &str,
    denial_class: &str,
) -> RejectRecord {
    RejectRecord {
        formula_stable_id: formula_stable_id.to_string(),
        session_id: Some(session_id.to_string()),
        commit_attempt_id: None,
        reject_code: RejectCode::CapabilityDenied,
        context: RejectContext::CapabilityDenied(CapabilityDenialContext {
            capability_kind: capability_kind.to_string(),
            phase_kind: phase_kind.to_string(),
            denial_class: denial_class.to_string(),
            fallback_available: false,
        }),
        trace_correlation_id: format!("trace:{session_id}"),
    }
}

fn structural_conflict_reject(
    formula_stable_id: &str,
    session_id: &str,
    conflict_kind: &str,
    retry_admissibility: &str,
) -> RejectRecord {
    RejectRecord {
        formula_stable_id: formula_stable_id.to_string(),
        session_id: Some(session_id.to_string()),
        commit_attempt_id: None,
        reject_code: RejectCode::StructuralConflict,
        context: RejectContext::StructuralConflict(crate::seam::StructuralConflictContext {
            conflict_kind: conflict_kind.to_string(),
            conflicting_loci: Vec::new(),
            conflicting_extent: None,
            retry_admissibility: retry_admissibility.to_string(),
        }),
        trace_correlation_id: format!("trace:{session_id}"),
    }
}

fn contention_conflict_reject(
    formula_stable_id: &str,
    session_id: &str,
    locus: &Locus,
    owner_session_id: &str,
) -> RejectRecord {
    RejectRecord {
        formula_stable_id: formula_stable_id.to_string(),
        session_id: Some(session_id.to_string()),
        commit_attempt_id: None,
        reject_code: RejectCode::StructuralConflict,
        context: RejectContext::StructuralConflict(crate::seam::StructuralConflictContext {
            conflict_kind: "locus_busy".to_string(),
            conflicting_loci: vec![locus.clone()],
            conflicting_extent: Some(Extent { rows: 1, cols: 1 }),
            retry_admissibility: format!("retry_after_release:{owner_session_id}"),
        }),
        trace_correlation_id: format!("trace:{session_id}"),
    }
}

fn session_terminated_reject(
    formula_stable_id: &str,
    session_id: &str,
    termination_class: &str,
    expired: bool,
    cause: Option<String>,
) -> RejectRecord {
    RejectRecord {
        formula_stable_id: formula_stable_id.to_string(),
        session_id: Some(session_id.to_string()),
        commit_attempt_id: None,
        reject_code: RejectCode::SessionTerminated,
        context: RejectContext::SessionTerminated(SessionTerminationContext {
            termination_class: termination_class.to_string(),
            session_id: session_id.to_string(),
            candidate_already_built: false,
            termination_cause: cause.or_else(|| {
                Some(if expired {
                    "expired".to_string()
                } else {
                    "aborted".to_string()
                })
            }),
        }),
        trace_correlation_id: format!("trace:{session_id}"),
    }
}

fn trace_reject_event(
    formula_stable_id: &str,
    session_id: &str,
    event_order_key: u64,
    reject_code: RejectCode,
) -> TraceEvent {
    TraceEvent {
        trace_schema_id: "trace:v1".to_string(),
        event_kind: TraceEventKind::RejectIssued,
        formula_stable_id: formula_stable_id.to_string(),
        session_id: Some(session_id.to_string()),
        candidate_result_id: None,
        commit_attempt_id: None,
        event_order_key,
        event_payload: TracePayload::RejectIssued { reject_code },
    }
}

fn requires_single_locus_claim(plan: &SemanticPlan) -> bool {
    let contract = build_execution_contract(plan);
    contract.restrictions.iter().any(|restriction| {
        matches!(
            restriction,
            ExecutionRestriction::ThreadAffine
                | ExecutionRestriction::SerialOnly
                | ExecutionRestriction::SingleFlightAdvisable
                | ExecutionRestriction::HostSerialized
                | ExecutionRestriction::NotThreadSafe
        )
    })
}

fn overlay_entries_for_candidate(candidate: &AcceptedCandidateResult) -> Vec<OverlayEntry> {
    let mut overlays = Vec::new();
    let Some(session_id) = candidate.session_id.as_deref() else {
        return overlays;
    };
    let overlay_scope_key = format!(
        "{}:{}:{}:{}",
        candidate.fence_snapshot.snapshot_epoch,
        candidate.fence_snapshot.formula_token,
        candidate.fence_snapshot.bind_hash,
        candidate.fence_snapshot.profile_version
    );

    overlays.push(OverlayEntry {
        overlay_entry_id: format!("overlay:{session_id}:dependency"),
        overlay_scope_key: overlay_scope_key.clone(),
        overlay_family: "dependency_overlay".to_string(),
        session_id: session_id.to_string(),
        formula_stable_id: candidate.formula_stable_id.clone(),
    });
    if !candidate.spill_events.is_empty() {
        overlays.push(OverlayEntry {
            overlay_entry_id: format!("overlay:{session_id}:spill"),
            overlay_scope_key: overlay_scope_key.clone(),
            overlay_family: "spill_overlay".to_string(),
            session_id: session_id.to_string(),
            formula_stable_id: candidate.formula_stable_id.clone(),
        });
    }
    if !candidate.topology_delta.format_dependency_facts.is_empty() {
        overlays.push(OverlayEntry {
            overlay_entry_id: format!("overlay:{session_id}:format"),
            overlay_scope_key,
            overlay_family: "format_dependency_overlay".to_string(),
            session_id: session_id.to_string(),
            formula_stable_id: candidate.formula_stable_id.clone(),
        });
    }
    if candidate
        .topology_delta
        .capability_effect_facts
        .iter()
        .any(|fact| fact.capability_kind == "async_coupling")
    {
        overlays.push(OverlayEntry {
            overlay_entry_id: format!("overlay:{session_id}:runtime_async"),
            overlay_scope_key: format!(
                "{}:{}:{}:{}",
                candidate.fence_snapshot.snapshot_epoch,
                candidate.fence_snapshot.formula_token,
                candidate.fence_snapshot.bind_hash,
                candidate.fence_snapshot.profile_version
            ),
            overlay_family: "runtime_async_overlay".to_string(),
            session_id: session_id.to_string(),
            formula_stable_id: candidate.formula_stable_id.clone(),
        });
    }
    if candidate.format_delta.is_some() || candidate.display_delta.is_some() {
        overlays.push(OverlayEntry {
            overlay_entry_id: format!("overlay:{session_id}:publication_surface"),
            overlay_scope_key: format!(
                "{}:{}:{}:{}",
                candidate.fence_snapshot.snapshot_epoch,
                candidate.fence_snapshot.formula_token,
                candidate.fence_snapshot.bind_hash,
                candidate.fence_snapshot.profile_version
            ),
            overlay_family: "publication_surface_overlay".to_string(),
            session_id: session_id.to_string(),
            formula_stable_id: candidate.formula_stable_id.clone(),
        });
    }
    overlays
}

fn locus_claim_key(locus: &Locus) -> String {
    format!("{}:{}:{}", locus.sheet_id, locus.row, locus.col)
}

fn build_candidate_result(
    source: &FormulaSourceRecord,
    semantic_plan: &SemanticPlan,
    evaluation: &EvaluationOutput,
    primary_locus: &Locus,
    session_id: &str,
    capability_view_key: Option<&str>,
) -> AcceptedCandidateResult {
    let candidate_result_id = format!("candidate:{}:{session_id}", source.formula_text_version.0);
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

    let capability_effect_facts = capability_effect_facts_for_plan(source, semantic_plan);

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
        fence_snapshot: fence_snapshot_for_prepared(
            &PreparedSession {
                source: source.clone(),
                bound_formula: BoundFormula {
                    formula_stable_id: source.formula_stable_id.0.clone(),
                    green_tree_key: String::new(),
                    structure_context_version: String::new(),
                    bind_context_fingerprint: String::new(),
                    bind_hash: semantic_plan.bind_hash.clone(),
                    root: crate::binding::BoundExpr::NumberLiteral("0".to_string()),
                    normalized_references: Vec::new(),
                    dependency_seeds: Vec::new(),
                    unresolved_references: Vec::new(),
                    capability_requirements: Vec::new(),
                    diagnostics: Vec::new(),
                },
                semantic_plan: semantic_plan.clone(),
                primary_locus: primary_locus.clone(),
            },
            capability_view_key,
        ),
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
            dependency_reclassifications: dependency_reclassifications_for_plan(semantic_plan),
            dependency_consequence_facts: dependency_consequence_facts_for_plan(
                source,
                semantic_plan,
            ),
            dynamic_reference_facts: dynamic_reference_facts_for_plan(source, semantic_plan),
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

fn capability_effect_facts_for_plan(
    source: &FormulaSourceRecord,
    semantic_plan: &SemanticPlan,
) -> Vec<CapabilityEffectFact> {
    let mut facts = Vec::new();
    let formula_stable_id = source.formula_stable_id.0.clone();

    if semantic_plan.execution_profile.requires_host_query {
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: "host_query".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "admitted".to_string(),
            fallback_class: None,
        });
    }
    if semantic_plan.execution_profile.requires_locale {
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: "locale_format_context".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "admitted".to_string(),
            fallback_class: None,
        });
    }
    if semantic_plan.execution_profile.requires_caller_context {
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: "caller_context".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "admitted".to_string(),
            fallback_class: None,
        });
    }
    if semantic_plan.execution_profile.contains_pseudo_random {
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: "random_provider".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "consumed".to_string(),
            fallback_class: None,
        });
    }
    if semantic_plan.execution_profile.contains_time_dependence {
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: "time_provider".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "consumed".to_string(),
            fallback_class: None,
        });
    }
    if semantic_plan.execution_profile.requires_async_coupling
        || semantic_plan
            .execution_profile
            .contains_external_event_dependence
    {
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: "external_provider".to_string(),
            phase_kind: "evaluate".to_string(),
            effect_class: "admitted".to_string(),
            fallback_class: Some("deferred_without_provider".to_string()),
        });
    }

    let contract = build_execution_contract(semantic_plan);
    for restriction in contract.restrictions {
        let (capability_kind, phase_kind) = match restriction {
            ExecutionRestriction::ThreadAffine => ("thread_affinity", "schedule"),
            ExecutionRestriction::SerialOnly => ("serial_scheduler_lane", "schedule"),
            ExecutionRestriction::SingleFlightAdvisable => ("single_flight", "schedule"),
            ExecutionRestriction::HostSerialized => ("host_serialized", "schedule"),
            ExecutionRestriction::NotThreadSafe => ("not_thread_safe", "schedule"),
            ExecutionRestriction::AsyncCoupled => ("async_coupling", "schedule"),
            _ => continue,
        };
        facts.push(CapabilityEffectFact {
            formula_stable_id: formula_stable_id.clone(),
            capability_kind: capability_kind.to_string(),
            phase_kind: phase_kind.to_string(),
            effect_class: "required".to_string(),
            fallback_class: None,
        });
    }

    facts
}

fn dynamic_reference_facts_for_plan(
    source: &FormulaSourceRecord,
    semantic_plan: &SemanticPlan,
) -> Vec<DynamicReferenceFact> {
    let mut facts = Vec::new();

    if semantic_plan
        .capability_requirements
        .iter()
        .any(|item| item == "external_reference")
    {
        facts.push(DynamicReferenceFact {
            formula_stable_id: source.formula_stable_id.0.clone(),
            discovery_site: "semantic_plan.external_reference".to_string(),
            reference_identity: Some("external_reference".to_string()),
            target_extent: None,
            resolution_failure_class: Some("external_reference_deferred".to_string()),
        });
    }

    facts
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

fn dependency_reclassifications_for_plan(semantic_plan: &SemanticPlan) -> Vec<String> {
    let mut reclassifications = Vec::new();

    if semantic_plan
        .capability_requirements
        .iter()
        .any(|item| item == "external_reference")
    {
        reclassifications.push("reference_lane:external_reference_deferred".to_string());
    }

    reclassifications
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
            ValuePayload::Text(format!("Lambda({name})")),
            Some(Extent { rows: 1, cols: 1 }),
        ),
    }
}
