use crate::consumer::runtime::{
    RuntimeFormulaResult, RuntimeManagedCommitResult, RuntimeManagedExecutionResult,
    RuntimeManagedOpenResult, RuntimeManagedSessionSnapshot, RuntimeManagedTerminationResult,
};
use crate::host::{FirstHostReplayCapturePacket, HostRecalcOutput};
use crate::interface::{LibraryContextSnapshotRef, TypedContextQueryBundleSpec};
use crate::session::{SessionPhase, SessionRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayProjectionFamily {
    FirstHostCapture,
    SessionLifecycle,
    FixtureFamily,
    RetainedWitness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayFixtureFamilySource {
    pub source_schema_id: String,
    pub source_fixture_family: String,
    pub source_case_ids: Vec<String>,
    pub registry_pin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRetainedWitnessSource {
    pub witness_id: String,
    pub source_fixture_family: String,
    pub source_case_ids: Vec<String>,
    pub witness_lifecycle_state: String,
    pub retention_policy_id: Option<String>,
    pub registry_pin: Option<String>,
    pub source_bundle_ref: Option<String>,
    pub reduction_manifest_ref: Option<String>,
}

pub enum ReplayProjectionSource<'a> {
    RuntimeResult(&'a RuntimeFormulaResult),
    RuntimeManagedOpen(&'a RuntimeManagedOpenResult),
    RuntimeManagedExecution(&'a RuntimeManagedExecutionResult),
    RuntimeManagedCommit(&'a RuntimeManagedCommitResult),
    RuntimeManagedTermination(&'a RuntimeManagedTerminationResult),
    RuntimeManagedSession(&'a RuntimeManagedSessionSnapshot),
    HostResult(&'a HostRecalcOutput),
    SessionRecord(&'a SessionRecord),
    FixtureFamily(&'a ReplayFixtureFamilySource),
    RetainedWitness(&'a ReplayRetainedWitnessSource),
}

pub struct ReplayProjectionRequest<'a> {
    source: ReplayProjectionSource<'a>,
    projection_family: ReplayProjectionFamily,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
}

impl<'a> ReplayProjectionRequest<'a> {
    pub fn runtime_result(result: &'a RuntimeFormulaResult) -> Self {
        Self {
            source: ReplayProjectionSource::RuntimeResult(result),
            projection_family: ReplayProjectionFamily::FirstHostCapture,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn host_result(result: &'a HostRecalcOutput) -> Self {
        Self {
            source: ReplayProjectionSource::HostResult(result),
            projection_family: ReplayProjectionFamily::FirstHostCapture,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn runtime_managed_open(result: &'a RuntimeManagedOpenResult) -> Self {
        Self {
            source: ReplayProjectionSource::RuntimeManagedOpen(result),
            projection_family: ReplayProjectionFamily::SessionLifecycle,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn runtime_managed_execution(result: &'a RuntimeManagedExecutionResult) -> Self {
        Self {
            source: ReplayProjectionSource::RuntimeManagedExecution(result),
            projection_family: ReplayProjectionFamily::SessionLifecycle,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn runtime_managed_commit(result: &'a RuntimeManagedCommitResult) -> Self {
        Self {
            source: ReplayProjectionSource::RuntimeManagedCommit(result),
            projection_family: ReplayProjectionFamily::SessionLifecycle,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn runtime_managed_termination(result: &'a RuntimeManagedTerminationResult) -> Self {
        Self {
            source: ReplayProjectionSource::RuntimeManagedTermination(result),
            projection_family: ReplayProjectionFamily::SessionLifecycle,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn runtime_managed_session(result: &'a RuntimeManagedSessionSnapshot) -> Self {
        Self {
            source: ReplayProjectionSource::RuntimeManagedSession(result),
            projection_family: ReplayProjectionFamily::SessionLifecycle,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn session_record(record: &'a SessionRecord) -> Self {
        Self {
            source: ReplayProjectionSource::SessionRecord(record),
            projection_family: ReplayProjectionFamily::SessionLifecycle,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn fixture_family(source: &'a ReplayFixtureFamilySource) -> Self {
        Self {
            source: ReplayProjectionSource::FixtureFamily(source),
            projection_family: ReplayProjectionFamily::FixtureFamily,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn retained_witness(source: &'a ReplayRetainedWitnessSource) -> Self {
        Self {
            source: ReplayProjectionSource::RetainedWitness(source),
            projection_family: ReplayProjectionFamily::RetainedWitness,
            source_case_id: None,
            shared_scenario_alias: None,
        }
    }

    pub fn with_projection_family(mut self, projection_family: ReplayProjectionFamily) -> Self {
        self.projection_family = projection_family;
        self
    }

    pub fn with_source_case_id(mut self, source_case_id: impl Into<String>) -> Self {
        self.source_case_id = Some(source_case_id.into());
        self
    }

    pub fn with_shared_scenario_alias(mut self, shared_scenario_alias: impl Into<String>) -> Self {
        self.shared_scenario_alias = Some(shared_scenario_alias.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayProjectionResult {
    pub source_artifact_family: String,
    pub source_schema_id: Option<String>,
    pub source_fixture_family: Option<String>,
    pub source_case_id: Option<String>,
    pub source_case_ids: Vec<String>,
    pub shared_scenario_alias: Option<String>,
    pub formula_stable_id: String,
    pub session_id: Option<String>,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub typed_query_bundle_spec: Option<TypedContextQueryBundleSpec>,
    pub registry_pin: Option<String>,
    pub witness_id: Option<String>,
    pub witness_lifecycle_state: Option<String>,
    pub retention_policy_id: Option<String>,
    pub source_bundle_ref: Option<String>,
    pub reduction_manifest_ref: Option<String>,
    pub phase: Option<String>,
    pub candidate_result_id: Option<String>,
    pub commit_decision_kind: Option<String>,
    pub trace_event_kinds: Vec<String>,
    pub first_host_replay_capture_packet: Option<FirstHostReplayCapturePacket>,
}

pub struct ReplayProjectionService;

impl ReplayProjectionService {
    pub fn project(request: ReplayProjectionRequest<'_>) -> ReplayProjectionResult {
        match request.source {
            ReplayProjectionSource::RuntimeResult(result) => project_runtime_result(
                result,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
            ReplayProjectionSource::RuntimeManagedOpen(result) => project_runtime_managed_open(
                result,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
            ReplayProjectionSource::RuntimeManagedExecution(result) => {
                project_runtime_managed_execution(
                    result,
                    request.source_case_id,
                    request.shared_scenario_alias,
                )
            }
            ReplayProjectionSource::RuntimeManagedCommit(result) => project_runtime_managed_commit(
                result,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
            ReplayProjectionSource::RuntimeManagedTermination(result) => {
                project_runtime_managed_termination(
                    result,
                    request.source_case_id,
                    request.shared_scenario_alias,
                )
            }
            ReplayProjectionSource::RuntimeManagedSession(result) => {
                project_runtime_managed_session(
                    result,
                    request.source_case_id,
                    request.shared_scenario_alias,
                )
            }
            ReplayProjectionSource::HostResult(result) => project_host_result(
                result,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
            ReplayProjectionSource::SessionRecord(record) => project_session_record(
                record,
                request.projection_family,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
            ReplayProjectionSource::FixtureFamily(source) => project_fixture_family(
                source,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
            ReplayProjectionSource::RetainedWitness(source) => project_retained_witness(
                source,
                request.source_case_id,
                request.shared_scenario_alias,
            ),
        }
    }
}

fn project_runtime_result(
    result: &RuntimeFormulaResult,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "runtime_formula_result".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.source.formula_stable_id.0.clone(),
        session_id: result.candidate_result.session_id.clone(),
        library_context_snapshot_ref: result.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: Some(result.typed_query_bundle_spec.clone()),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some("CommittedOrRejected".to_string()),
        candidate_result_id: Some(result.candidate_result.candidate_result_id.clone()),
        commit_decision_kind: Some(match result.commit_decision {
            crate::seam::AcceptDecision::Accepted(_) => "accepted".to_string(),
            crate::seam::AcceptDecision::Rejected(_) => "rejected".to_string(),
        }),
        trace_event_kinds: result
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: Some(result.first_host_replay_capture_packet.clone()),
    }
}

fn project_host_result(
    result: &HostRecalcOutput,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    let packet = result.to_first_host_replay_capture_packet();
    ReplayProjectionResult {
        source_artifact_family: "host_recalc_output".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.source.formula_stable_id.0.clone(),
        session_id: result.candidate_result.session_id.clone(),
        library_context_snapshot_ref: result.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: Some(result.typed_query_bundle_spec.clone()),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some("CommittedOrRejected".to_string()),
        candidate_result_id: Some(result.candidate_result.candidate_result_id.clone()),
        commit_decision_kind: Some(match result.commit_decision {
            crate::seam::AcceptDecision::Accepted(_) => "accepted".to_string(),
            crate::seam::AcceptDecision::Rejected(_) => "rejected".to_string(),
        }),
        trace_event_kinds: result
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: Some(packet),
    }
}

fn project_runtime_managed_open(
    result: &RuntimeManagedOpenResult,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "runtime_managed_open".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.semantic_plan.formula_stable_id.clone(),
        session_id: Some(result.session_id.clone()),
        library_context_snapshot_ref: result.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: None,
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some("Open".to_string()),
        candidate_result_id: None,
        commit_decision_kind: None,
        trace_event_kinds: Vec::new(),
        first_host_replay_capture_packet: None,
    }
}

fn project_runtime_managed_execution(
    result: &RuntimeManagedExecutionResult,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "runtime_managed_execution".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.formula_stable_id.clone(),
        session_id: Some(result.session_id.clone()),
        library_context_snapshot_ref: result.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: Some(result.typed_query_bundle_spec.clone()),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some("Executed".to_string()),
        candidate_result_id: Some(result.candidate_result.candidate_result_id.clone()),
        commit_decision_kind: None,
        trace_event_kinds: result
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: None,
    }
}

fn project_runtime_managed_session(
    result: &RuntimeManagedSessionSnapshot,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "runtime_managed_session".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.formula_stable_id.clone(),
        session_id: Some(result.session_id.clone()),
        library_context_snapshot_ref: result.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: result.typed_query_bundle_spec.clone(),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some(
            match result.phase {
                crate::consumer::runtime::RuntimeManagedSessionPhase::Open => "Open",
                crate::consumer::runtime::RuntimeManagedSessionPhase::CapabilityViewEstablished => {
                    "CapabilityViewEstablished"
                }
                crate::consumer::runtime::RuntimeManagedSessionPhase::Executed => "Executed",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Committed => "Committed",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Rejected => "Rejected",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Aborted => "Aborted",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Expired => "Expired",
            }
            .to_string(),
        ),
        candidate_result_id: result.candidate_result_id.clone(),
        commit_decision_kind: None,
        trace_event_kinds: result
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: None,
    }
}

fn project_runtime_managed_commit(
    result: &RuntimeManagedCommitResult,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "runtime_managed_commit".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.session.formula_stable_id.clone(),
        session_id: Some(result.session.session_id.clone()),
        library_context_snapshot_ref: result.session.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: result.session.typed_query_bundle_spec.clone(),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some(
            match result.session.phase {
                crate::consumer::runtime::RuntimeManagedSessionPhase::Committed => "Committed",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Rejected => "Rejected",
                _ => "CommitAttempted",
            }
            .to_string(),
        ),
        candidate_result_id: result.session.candidate_result_id.clone(),
        commit_decision_kind: Some(match &result.commit_decision {
            crate::seam::AcceptDecision::Accepted(_) => "accepted".to_string(),
            crate::seam::AcceptDecision::Rejected(_) => "rejected".to_string(),
        }),
        trace_event_kinds: result
            .session
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: None,
    }
}

fn project_runtime_managed_termination(
    result: &RuntimeManagedTerminationResult,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "runtime_managed_termination".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: result.session.formula_stable_id.clone(),
        session_id: Some(result.session.session_id.clone()),
        library_context_snapshot_ref: result.session.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: result.session.typed_query_bundle_spec.clone(),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some(
            match result.session.phase {
                crate::consumer::runtime::RuntimeManagedSessionPhase::Aborted => "Aborted",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Expired => "Expired",
                crate::consumer::runtime::RuntimeManagedSessionPhase::Rejected => "Rejected",
                _ => "Terminated",
            }
            .to_string(),
        ),
        candidate_result_id: result.session.candidate_result_id.clone(),
        commit_decision_kind: Some("rejected".to_string()),
        trace_event_kinds: result
            .session
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: None,
    }
}

fn project_session_record(
    record: &SessionRecord,
    _projection_family: ReplayProjectionFamily,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "session_record".to_string(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: record.prepared.source.formula_stable_id.0.clone(),
        session_id: Some(record.session_id.clone()),
        library_context_snapshot_ref: record.prepared.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: record.typed_query_bundle_spec.clone(),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some(session_phase_name(&record.phase).to_string()),
        candidate_result_id: record
            .candidate_result
            .as_ref()
            .map(|candidate| candidate.candidate_result_id.clone()),
        commit_decision_kind: record
            .candidate_result
            .as_ref()
            .map(|_| match record.phase {
                SessionPhase::Committed => "accepted".to_string(),
                SessionPhase::Rejected => "rejected".to_string(),
                _ => "pending".to_string(),
            }),
        trace_event_kinds: record
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
        first_host_replay_capture_packet: None,
    }
}

fn project_fixture_family(
    source: &ReplayFixtureFamilySource,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "fixture_family".to_string(),
        source_schema_id: Some(source.source_schema_id.clone()),
        source_fixture_family: Some(source.source_fixture_family.clone()),
        source_case_id,
        source_case_ids: source.source_case_ids.clone(),
        shared_scenario_alias,
        formula_stable_id: String::new(),
        session_id: None,
        library_context_snapshot_ref: None,
        typed_query_bundle_spec: None,
        registry_pin: source.registry_pin.clone(),
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: None,
        candidate_result_id: None,
        commit_decision_kind: None,
        trace_event_kinds: Vec::new(),
        first_host_replay_capture_packet: None,
    }
}

fn project_retained_witness(
    source: &ReplayRetainedWitnessSource,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: "retained_witness".to_string(),
        source_schema_id: None,
        source_fixture_family: Some(source.source_fixture_family.clone()),
        source_case_id,
        source_case_ids: source.source_case_ids.clone(),
        shared_scenario_alias,
        formula_stable_id: String::new(),
        session_id: None,
        library_context_snapshot_ref: None,
        typed_query_bundle_spec: None,
        registry_pin: source.registry_pin.clone(),
        witness_id: Some(source.witness_id.clone()),
        witness_lifecycle_state: Some(source.witness_lifecycle_state.clone()),
        retention_policy_id: source.retention_policy_id.clone(),
        source_bundle_ref: source.source_bundle_ref.clone(),
        reduction_manifest_ref: source.reduction_manifest_ref.clone(),
        phase: None,
        candidate_result_id: None,
        commit_decision_kind: None,
        trace_event_kinds: Vec::new(),
        first_host_replay_capture_packet: None,
    }
}

fn session_phase_name(phase: &SessionPhase) -> &'static str {
    match phase {
        SessionPhase::Open => "Open",
        SessionPhase::CapabilityViewEstablished => "CapabilityViewEstablished",
        SessionPhase::Executed => "Executed",
        SessionPhase::Committed => "Committed",
        SessionPhase::Rejected => "Rejected",
        SessionPhase::Aborted => "Aborted",
        SessionPhase::Expired => "Expired",
    }
}
