use crate::semantics::ExecutionProfileSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locus {
    pub sheet_id: String,
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extent {
    pub rows: u32,
    pub cols: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorksheetValueClass {
    Scalar,
    Error,
    ArrayAnchor,
    BlankLike,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValuePayload {
    Number(String),
    Text(String),
    Logical(bool),
    ErrorCode(String),
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueDelta {
    pub formula_stable_id: String,
    pub primary_locus: Locus,
    pub affected_value_loci: Vec<Locus>,
    pub published_value_class: WorksheetValueClass,
    pub published_payload: ValuePayload,
    pub result_extent: Option<Extent>,
    pub candidate_result_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeOutcomeClass {
    Established,
    Reconfigured,
    Cleared,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeDelta {
    pub formula_stable_id: String,
    pub anchor_locus: Locus,
    pub intended_extent: Extent,
    pub published_extent: Option<Extent>,
    pub blocked_loci: Vec<Locus>,
    pub shape_outcome_class: ShapeOutcomeClass,
    pub candidate_result_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicReferenceFact {
    pub formula_stable_id: String,
    pub discovery_site: String,
    pub reference_identity: Option<String>,
    pub target_extent: Option<Extent>,
    pub resolution_failure_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpillFact {
    pub formula_stable_id: String,
    pub anchor_locus: Locus,
    pub intended_extent: Extent,
    pub published_extent: Option<Extent>,
    pub blocked_loci: Vec<Locus>,
    pub blocked_reason_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatDependencyFact {
    pub formula_stable_id: String,
    pub dependency_token: String,
    pub dependency_class: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityEffectFact {
    pub formula_stable_id: String,
    pub capability_kind: String,
    pub phase_kind: String,
    pub effect_class: String,
    pub fallback_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyDelta {
    pub formula_stable_id: String,
    pub dependency_additions: Vec<String>,
    pub dependency_removals: Vec<String>,
    pub dependency_reclassifications: Vec<String>,
    pub dynamic_reference_facts: Vec<DynamicReferenceFact>,
    pub spill_facts: Vec<SpillFact>,
    pub format_dependency_facts: Vec<FormatDependencyFact>,
    pub capability_effect_facts: Vec<CapabilityEffectFact>,
    pub candidate_result_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatDelta {
    pub formula_stable_id: String,
    pub target_loci: Vec<Locus>,
    pub format_effect_class: String,
    pub format_effect_payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayDelta {
    pub formula_stable_id: String,
    pub target_loci: Vec<Locus>,
    pub display_effect_class: String,
    pub display_effect_payload: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpillEventKind {
    SpillTakeover,
    SpillClearance,
    SpillBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpillEvent {
    pub spill_event_kind: SpillEventKind,
    pub formula_stable_id: String,
    pub anchor_locus: Locus,
    pub intended_extent: Extent,
    pub affected_extent: Option<Extent>,
    pub blocking_loci: Vec<Locus>,
    pub blocking_reason_class: Option<String>,
    pub correlation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceSnapshot {
    pub formula_token: String,
    pub snapshot_epoch: String,
    pub bind_hash: String,
    pub profile_version: String,
    pub capability_view_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedCandidateResult {
    pub formula_stable_id: String,
    pub session_id: Option<String>,
    pub candidate_result_id: String,
    pub fence_snapshot: FenceSnapshot,
    pub value_delta: ValueDelta,
    pub shape_delta: ShapeDelta,
    pub topology_delta: TopologyDelta,
    pub format_delta: Option<FormatDelta>,
    pub display_delta: Option<DisplayDelta>,
    pub spill_events: Vec<SpillEvent>,
    pub execution_profile: Option<ExecutionProfileSummary>,
    pub trace_correlation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitBundle {
    pub formula_stable_id: String,
    pub commit_attempt_id: String,
    pub candidate_result_id: String,
    pub fence_snapshot: FenceSnapshot,
    pub value_delta: ValueDelta,
    pub shape_delta: ShapeDelta,
    pub topology_delta: TopologyDelta,
    pub format_delta: Option<FormatDelta>,
    pub display_delta: Option<DisplayDelta>,
    pub spill_events: Vec<SpillEvent>,
    pub execution_profile: Option<ExecutionProfileSummary>,
    pub trace_correlation_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectCode {
    FenceMismatch,
    CapabilityDenied,
    SessionTerminated,
    BindMismatch,
    StructuralConflict,
    DynamicReferenceFailure,
    ResourceInvariantFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceMismatchContext {
    pub mismatch_member_kind: String,
    pub expected_value: String,
    pub observed_value: String,
    pub mismatch_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDenialContext {
    pub capability_kind: String,
    pub phase_kind: String,
    pub denial_class: String,
    pub fallback_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTerminationContext {
    pub termination_class: String,
    pub session_id: String,
    pub candidate_already_built: bool,
    pub termination_cause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralConflictContext {
    pub conflict_kind: String,
    pub conflicting_loci: Vec<Locus>,
    pub conflicting_extent: Option<Extent>,
    pub retry_admissibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicReferenceFailureContext {
    pub dynamic_reference_family: String,
    pub failure_class: String,
    pub partial_reference_identity: Option<String>,
    pub discovery_site: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInvariantContext {
    pub failure_family: String,
    pub machine_detail_code: String,
    pub resource_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectContext {
    FenceMismatch(FenceMismatchContext),
    CapabilityDenied(CapabilityDenialContext),
    SessionTerminated(SessionTerminationContext),
    StructuralConflict(StructuralConflictContext),
    DynamicReferenceFailure(DynamicReferenceFailureContext),
    ResourceInvariant(ResourceInvariantContext),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectRecord {
    pub formula_stable_id: String,
    pub session_id: Option<String>,
    pub commit_attempt_id: Option<String>,
    pub reject_code: RejectCode,
    pub context: RejectContext,
    pub trace_correlation_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceEventKind {
    SessionOpened,
    CapabilityViewEstablished,
    AcceptedCandidateResultBuilt,
    CommitAccepted,
    CommitRejected,
    RejectIssued,
    SessionAborted,
    SessionExpired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TracePayload {
    SessionOpened {
        session_id: String,
    },
    CapabilityViewEstablished {
        capability_view_key: String,
    },
    CandidateBuilt {
        candidate_result_id: String,
    },
    CommitAccepted {
        commit_attempt_id: String,
        candidate_result_id: String,
    },
    CommitRejected {
        commit_attempt_id: String,
        reject_code: RejectCode,
    },
    RejectIssued {
        reject_code: RejectCode,
    },
    SessionAborted {
        termination_class: String,
    },
    SessionExpired {
        termination_class: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub trace_schema_id: String,
    pub event_kind: TraceEventKind,
    pub formula_stable_id: String,
    pub session_id: Option<String>,
    pub candidate_result_id: Option<String>,
    pub commit_attempt_id: Option<String>,
    pub event_order_key: u64,
    pub event_payload: TracePayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRequest {
    pub candidate_result: AcceptedCandidateResult,
    pub commit_attempt_id: String,
    pub observed_fence: FenceSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptDecision {
    Accepted(CommitBundle),
    Rejected(RejectRecord),
}

pub fn commit_candidate(request: CommitRequest) -> AcceptDecision {
    let candidate = request.candidate_result;
    if let Some(reject) = fence_mismatch_reject(
        &candidate.formula_stable_id,
        candidate.session_id.clone(),
        request.commit_attempt_id.clone(),
        &candidate.fence_snapshot,
        &request.observed_fence,
        &candidate.trace_correlation_id,
    ) {
        return AcceptDecision::Rejected(reject);
    }

    AcceptDecision::Accepted(CommitBundle {
        formula_stable_id: candidate.formula_stable_id,
        commit_attempt_id: request.commit_attempt_id,
        candidate_result_id: candidate.candidate_result_id,
        fence_snapshot: candidate.fence_snapshot,
        value_delta: candidate.value_delta,
        shape_delta: candidate.shape_delta,
        topology_delta: candidate.topology_delta,
        format_delta: candidate.format_delta,
        display_delta: candidate.display_delta,
        spill_events: candidate.spill_events,
        execution_profile: candidate.execution_profile,
        trace_correlation_id: candidate.trace_correlation_id,
    })
}

fn fence_mismatch_reject(
    formula_stable_id: &str,
    session_id: Option<String>,
    commit_attempt_id: String,
    expected: &FenceSnapshot,
    observed: &FenceSnapshot,
    trace_correlation_id: &str,
) -> Option<RejectRecord> {
    let mismatch = if expected.formula_token != observed.formula_token {
        Some(FenceMismatchContext {
            mismatch_member_kind: "formula_token".to_string(),
            expected_value: expected.formula_token.clone(),
            observed_value: observed.formula_token.clone(),
            mismatch_class: "stale".to_string(),
        })
    } else if expected.snapshot_epoch != observed.snapshot_epoch {
        Some(FenceMismatchContext {
            mismatch_member_kind: "snapshot_epoch".to_string(),
            expected_value: expected.snapshot_epoch.clone(),
            observed_value: observed.snapshot_epoch.clone(),
            mismatch_class: "stale".to_string(),
        })
    } else if expected.bind_hash != observed.bind_hash {
        Some(FenceMismatchContext {
            mismatch_member_kind: "bind_hash".to_string(),
            expected_value: expected.bind_hash.clone(),
            observed_value: observed.bind_hash.clone(),
            mismatch_class: "incompatible".to_string(),
        })
    } else if expected.profile_version != observed.profile_version {
        Some(FenceMismatchContext {
            mismatch_member_kind: "profile_version".to_string(),
            expected_value: expected.profile_version.clone(),
            observed_value: observed.profile_version.clone(),
            mismatch_class: "incompatible".to_string(),
        })
    } else if expected.capability_view_key != observed.capability_view_key {
        Some(FenceMismatchContext {
            mismatch_member_kind: "capability_view_key".to_string(),
            expected_value: expected.capability_view_key.clone().unwrap_or_default(),
            observed_value: observed.capability_view_key.clone().unwrap_or_default(),
            mismatch_class: "incompatible".to_string(),
        })
    } else {
        None
    };

    mismatch.map(|context| RejectRecord {
        formula_stable_id: formula_stable_id.to_string(),
        session_id,
        commit_attempt_id: Some(commit_attempt_id),
        reject_code: RejectCode::FenceMismatch,
        context: RejectContext::FenceMismatch(context),
        trace_correlation_id: trace_correlation_id.to_string(),
    })
}
