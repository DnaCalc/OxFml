use crate::consumer::runtime::{
    RuntimeFormulaResult, RuntimeManagedCommitResult, RuntimeManagedExecutionResult,
    RuntimeManagedOpenResult, RuntimeManagedSessionSnapshot, RuntimeManagedTerminationResult,
};
use crate::host::FirstHostReplayCapturePacket;
use crate::interface::{LibraryContextSnapshotRef, TypedContextQueryBundleSpec};
use crate::publication::VerificationPublicationSurface;
use serde_json::{Value, json};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayFirstHostCaptureSource {
    pub source_artifact_family: String,
    pub session_id: Option<String>,
    pub packet: FirstHostReplayCapturePacket,
}

enum ReplayProjectionSource<'a> {
    RuntimeResult(&'a RuntimeFormulaResult),
    RuntimeManagedOpen(&'a RuntimeManagedOpenResult),
    RuntimeManagedExecution(&'a RuntimeManagedExecutionResult),
    RuntimeManagedCommit(&'a RuntimeManagedCommitResult),
    RuntimeManagedTermination(&'a RuntimeManagedTerminationResult),
    RuntimeManagedSession(&'a RuntimeManagedSessionSnapshot),
    FirstHostCapture(&'a ReplayFirstHostCaptureSource),
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

    pub fn first_host_capture(source: &'a ReplayFirstHostCaptureSource) -> Self {
        Self {
            source: ReplayProjectionSource::FirstHostCapture(source),
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

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayComparisonView {
    pub view_family: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub comparison_views: Option<Vec<ReplayComparisonView>>,
    pub verification_publication_surface: Option<VerificationPublicationSurface>,
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
            ReplayProjectionSource::FirstHostCapture(source) => project_first_host_capture(
                source,
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
        comparison_views: Some(build_comparison_views(
            &result.verification_publication_surface,
        )),
        verification_publication_surface: Some(result.verification_publication_surface.clone()),
        first_host_replay_capture_packet: Some(result.first_host_replay_capture_packet.clone()),
    }
}

fn project_first_host_capture(
    source: &ReplayFirstHostCaptureSource,
    source_case_id: Option<String>,
    shared_scenario_alias: Option<String>,
) -> ReplayProjectionResult {
    ReplayProjectionResult {
        source_artifact_family: source.source_artifact_family.clone(),
        source_schema_id: None,
        source_fixture_family: None,
        source_case_id,
        source_case_ids: Vec::new(),
        shared_scenario_alias,
        formula_stable_id: source.packet.formula_stable_id.clone(),
        session_id: source.session_id.clone(),
        library_context_snapshot_ref: source.packet.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: Some(source.packet.typed_query_bundle_spec.clone()),
        registry_pin: None,
        witness_id: None,
        witness_lifecycle_state: None,
        retention_policy_id: None,
        source_bundle_ref: None,
        reduction_manifest_ref: None,
        phase: Some("CommittedOrRejected".to_string()),
        candidate_result_id: Some(source.packet.candidate_result_id.clone()),
        commit_decision_kind: Some(source.packet.commit_decision_kind.clone()),
        trace_event_kinds: source.packet.trace_event_kinds.clone(),
        comparison_views: Some(build_comparison_views(
            &source.packet.verification_publication_surface,
        )),
        verification_publication_surface: Some(
            source.packet.verification_publication_surface.clone(),
        ),
        first_host_replay_capture_packet: Some(source.packet.clone()),
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
        comparison_views: None,
        verification_publication_surface: None,
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
        comparison_views: None,
        verification_publication_surface: None,
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
        comparison_views: None,
        verification_publication_surface: None,
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
        comparison_views: None,
        verification_publication_surface: None,
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
        comparison_views: None,
        verification_publication_surface: None,
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
        comparison_views: None,
        verification_publication_surface: None,
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
        comparison_views: None,
        verification_publication_surface: None,
        first_host_replay_capture_packet: None,
    }
}

fn build_comparison_views(surface: &VerificationPublicationSurface) -> Vec<ReplayComparisonView> {
    vec![
        ReplayComparisonView {
            view_family: "visible_value".to_string(),
            value: Value::String(surface.visible_value_text.clone()),
        },
        ReplayComparisonView {
            view_family: "effective_display_text".to_string(),
            value: Value::String(surface.effective_display_text.clone()),
        },
        ReplayComparisonView {
            view_family: "formatting_view".to_string(),
            value: json!({
                "format_profile": surface.format_profile,
                "locale_format_context": surface.locale_format_context.as_ref().map(locale_format_context_json),
                "date1904": surface.date1904,
                "number_format_code": surface.number_format_code,
                "style_id": surface.style_id,
                "style_hierarchy": surface.style_hierarchy,
                "format_dependency_facts": surface
                    .format_dependency_facts
                    .iter()
                    .map(format_dependency_fact_json)
                    .collect::<Vec<_>>(),
                "format_delta": surface.format_delta.as_ref().map(format_delta_json),
                "display_delta": surface.display_delta.as_ref().map(display_delta_json),
                "presentation_hint": surface.presentation_hint.as_ref().map(presentation_hint_json),
                "font_color": surface.font_color,
                "fill_color": surface.fill_color,
                "effective_font_color": surface.effective_font_color,
                "effective_fill_color": surface.effective_fill_color
            }),
        },
        ReplayComparisonView {
            view_family: "conditional_formatting_view".to_string(),
            value: json!({
                "rules": surface
                    .conditional_formatting_rules
                    .iter()
                    .map(conditional_formatting_rule_json)
                    .collect::<Vec<_>>(),
                "target_ranges": surface.conditional_formatting_target_ranges,
                "rule_kind": surface.conditional_formatting_rule_kind,
                "operator": surface.conditional_formatting_operator,
                "thresholds": surface.conditional_formatting_thresholds,
                "applies": surface.conditional_formatting_applies,
                "effective_font_color": surface.conditional_formatting_effective_font_color,
                "effective_fill_color": surface.conditional_formatting_effective_fill_color,
                "effective_display": surface.conditional_formatting_effective_display
            }),
        },
    ]
}

fn locale_format_context_json(surface: &crate::publication::LocaleFormatContextSurface) -> Value {
    json!({
        "locale_profile_id": surface.locale_profile_id,
        "date_system": surface.date_system,
        "decimal_separator": surface.decimal_separator,
        "thousands_separator": surface.thousands_separator,
        "currency_symbol": surface.currency_symbol,
        "date_separator": surface.date_separator,
        "time_separator": surface.time_separator
    })
}

fn format_dependency_fact_json(fact: &crate::seam::FormatDependencyFact) -> Value {
    json!({
        "formula_stable_id": fact.formula_stable_id,
        "dependency_token": fact.dependency_token,
        "dependency_class": fact.dependency_class,
        "scope": fact.scope
    })
}

fn locus_json(locus: &crate::seam::Locus) -> Value {
    json!({
        "sheet_id": locus.sheet_id,
        "row": locus.row,
        "col": locus.col
    })
}

fn format_delta_json(delta: &crate::seam::FormatDelta) -> Value {
    json!({
        "formula_stable_id": delta.formula_stable_id,
        "target_loci": delta.target_loci.iter().map(locus_json).collect::<Vec<_>>(),
        "format_effect_class": delta.format_effect_class,
        "format_effect_payload": delta.format_effect_payload
    })
}

fn display_delta_json(delta: &crate::seam::DisplayDelta) -> Value {
    json!({
        "formula_stable_id": delta.formula_stable_id,
        "target_loci": delta.target_loci.iter().map(locus_json).collect::<Vec<_>>(),
        "display_effect_class": delta.display_effect_class,
        "display_effect_payload": delta.display_effect_payload
    })
}

fn presentation_hint_json(hint: &oxfunc_core::value::PresentationHint) -> Value {
    json!({
        "number_format": hint.number_format.map(|value| format!("{value:?}")),
        "style": hint.style.map(|value| format!("{value:?}"))
    })
}

fn conditional_formatting_rule_json(
    rule: &crate::publication::VerificationConditionalFormattingRule,
) -> Value {
    json!({
        "target_ranges": rule.target_ranges,
        "rule_kind": rule.rule_kind,
        "operator": rule.operator,
        "thresholds": rule.thresholds,
        "font_color": rule.font_color,
        "fill_color": rule.fill_color,
        "effective_display_text": rule.effective_display_text,
        "applies": rule.applies,
        "effective_font_color": rule.effective_font_color,
        "effective_fill_color": rule.effective_fill_color
    })
}
