use std::collections::BTreeMap;

use oxfunc_core::value::FunctionValue;

use crate::binding::BindDiagnostic;
use crate::eval::{
    DefinedNameBinding, EvaluationBackend, EvaluationTraceMode, PreparedCall, PreparedResult,
};
use crate::host::{HostRecalcOutput, SingleFormulaHost};
use crate::interface::{
    LibraryContextProvider, LibraryContextSnapshotRef, ReturnedValueSurface, TableCallerRegion,
    TableDescriptor, TableRef, TypedContextQueryBundle, TypedContextQueryBundleSpec,
};
use crate::seam::{AcceptDecision, ExecutionOutcomeSurface, Locus, RejectCode};
use crate::semantics::{ExecutionProfileSummary, FunctionAvailabilitySummary, SemanticDiagnostic};
use crate::source::FormulaChannelKind;
use crate::syntax::token::SyntaxDiagnostic;

#[derive(Clone)]
pub struct OxFuncAdapterRequest<'a> {
    pub fixture_case_id: String,
    pub formula_stable_id: String,
    pub formula_text: String,
    pub formula_channel_kind: FormulaChannelKind,
    pub caller_anchor: Locus,
    pub active_selection_anchor: Option<Locus>,
    pub structure_context_version: String,
    pub cell_fixture: BTreeMap<String, FunctionValue>,
    pub defined_name_bindings: BTreeMap<String, DefinedNameBinding>,
    pub table_catalog: Vec<TableDescriptor>,
    pub enclosing_table_ref: Option<TableRef>,
    pub caller_table_region: Option<TableCallerRegion>,
    pub typed_context_query_bundle: TypedContextQueryBundle<'a>,
    pub library_context_provider: Option<&'a dyn LibraryContextProvider>,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub host_query_capability_profile: Option<String>,
}

impl<'a> OxFuncAdapterRequest<'a> {
    pub fn new(
        fixture_case_id: impl Into<String>,
        formula_stable_id: impl Into<String>,
        formula_text: impl Into<String>,
        caller_anchor: Locus,
        typed_context_query_bundle: TypedContextQueryBundle<'a>,
    ) -> Self {
        Self {
            fixture_case_id: fixture_case_id.into(),
            formula_stable_id: formula_stable_id.into(),
            formula_text: formula_text.into(),
            formula_channel_kind: FormulaChannelKind::WorksheetA1,
            caller_anchor,
            active_selection_anchor: None,
            structure_context_version: "w049.adapter.v1".to_string(),
            cell_fixture: BTreeMap::new(),
            defined_name_bindings: BTreeMap::new(),
            table_catalog: Vec::new(),
            enclosing_table_ref: None,
            caller_table_region: None,
            typed_context_query_bundle,
            library_context_provider: None,
            library_context_snapshot_ref: None,
            host_query_capability_profile: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxFuncPreparationArtifact {
    pub fixture_case_id: String,
    pub formula_stable_id: String,
    pub formula_channel_kind: FormulaChannelKind,
    pub caller_anchor: Locus,
    pub active_selection_anchor: Option<Locus>,
    pub structure_context_version: String,
    pub host_query_capability_profile: Option<String>,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub typed_query_bundle_spec: TypedContextQueryBundleSpec,
    pub syntax_diagnostics: Vec<SyntaxDiagnostic>,
    pub bind_diagnostics: Vec<BindDiagnostic>,
    pub semantic_diagnostics: Vec<SemanticDiagnostic>,
    pub semantic_plan_key: String,
    pub bind_hash: String,
    pub execution_profile: ExecutionProfileSummary,
    pub capability_requirements: Vec<String>,
    pub availability_summaries: Vec<FunctionAvailabilitySummary>,
    pub prepared_calls: Vec<PreparedCall>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxFuncEvaluationArtifact {
    pub fixture_case_id: String,
    pub library_context_snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub evaluation_result: PreparedResult,
    pub worksheet_value: FunctionValue,
    pub returned_value_surface: ReturnedValueSurface,
    pub execution_outcome_surface: ExecutionOutcomeSurface,
    pub candidate_result_id: String,
    pub commit_decision_kind: String,
    pub reject_code: Option<RejectCode>,
    pub trace_event_kinds: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxFuncMismatchOwnerGuess {
    OxFml,
    OxFunc,
    SharedFreezeNeeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxFuncMismatchArtifact {
    pub fixture_case_id: String,
    pub snapshot_ref: Option<LibraryContextSnapshotRef>,
    pub expected_seam_family: String,
    pub failing_packet_family: String,
    pub owner_guess: OxFuncMismatchOwnerGuess,
    pub expected_summary: String,
    pub actual_summary: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxFuncAdapterRun {
    pub host_output: HostRecalcOutput,
    pub preparation_artifact: OxFuncPreparationArtifact,
    pub evaluation_artifact: OxFuncEvaluationArtifact,
}

impl OxFuncAdapterRun {
    pub fn mismatch_artifact(
        &self,
        expected_seam_family: impl Into<String>,
        failing_packet_family: impl Into<String>,
        owner_guess: OxFuncMismatchOwnerGuess,
        expected_summary: impl Into<String>,
        actual_summary: impl Into<String>,
        detail: Option<String>,
    ) -> OxFuncMismatchArtifact {
        OxFuncMismatchArtifact {
            fixture_case_id: self.preparation_artifact.fixture_case_id.clone(),
            snapshot_ref: self
                .preparation_artifact
                .library_context_snapshot_ref
                .clone(),
            expected_seam_family: expected_seam_family.into(),
            failing_packet_family: failing_packet_family.into(),
            owner_guess,
            expected_summary: expected_summary.into(),
            actual_summary: actual_summary.into(),
            detail,
        }
    }
}

pub fn run_oxfunc_preparation_adapter(
    request: OxFuncAdapterRequest<'_>,
) -> Result<OxFuncAdapterRun, String> {
    let OxFuncAdapterRequest {
        fixture_case_id,
        formula_stable_id,
        formula_text,
        formula_channel_kind,
        caller_anchor,
        active_selection_anchor,
        structure_context_version,
        cell_fixture,
        defined_name_bindings,
        table_catalog,
        enclosing_table_ref,
        caller_table_region,
        typed_context_query_bundle,
        library_context_provider,
        library_context_snapshot_ref,
        host_query_capability_profile,
    } = request;

    let mut host = SingleFormulaHost::new(formula_stable_id.clone(), formula_text);
    host.set_formula_channel_kind(formula_channel_kind);
    host.structure_context_version = structure_context_version.clone();
    host.caller_row = caller_anchor.row;
    host.caller_col = caller_anchor.col;
    host.primary_locus = caller_anchor.clone();
    host.set_trace_mode(EvaluationTraceMode::PreparedCalls);
    host.set_table_catalog(table_catalog);
    host.set_enclosing_table_ref(enclosing_table_ref);
    host.set_caller_table_region(caller_table_region);

    for (target, value) in cell_fixture {
        host.set_cell_value(target, value);
    }
    for (name, binding) in defined_name_bindings {
        match binding {
            DefinedNameBinding::Value(value) => host.set_defined_name_value(name, value),
            DefinedNameBinding::Reference(reference) => {
                host.set_defined_name_reference(name, reference)
            }
            DefinedNameBinding::Callable(callable) => {
                host.set_defined_name_callable(name, callable)
            }
        }
    }

    let host_output = host.recalc_with_interfaces_and_snapshot_ref(
        EvaluationBackend::OxFuncBacked,
        typed_context_query_bundle,
        library_context_provider,
        library_context_snapshot_ref.as_ref(),
    )?;

    let preparation_artifact = OxFuncPreparationArtifact {
        fixture_case_id: fixture_case_id.clone(),
        formula_stable_id: host_output.source.formula_stable_id.0.clone(),
        formula_channel_kind: host_output.source.formula_channel_kind,
        caller_anchor,
        active_selection_anchor,
        structure_context_version,
        host_query_capability_profile,
        library_context_snapshot_ref: host_output.library_context_snapshot_ref.clone(),
        typed_query_bundle_spec: host_output.typed_query_bundle_spec.clone(),
        syntax_diagnostics: host_output.syntax_diagnostics.clone(),
        bind_diagnostics: host_output.bind_diagnostics.clone(),
        semantic_diagnostics: host_output.semantic_plan.diagnostics.clone(),
        semantic_plan_key: host_output.semantic_plan.semantic_plan_key.clone(),
        bind_hash: host_output.semantic_plan.bind_hash.clone(),
        execution_profile: host_output.semantic_plan.execution_profile.clone(),
        capability_requirements: host_output.semantic_plan.capability_requirements.clone(),
        availability_summaries: host_output.semantic_plan.availability_summaries.clone(),
        prepared_calls: host_output.evaluation.trace.prepared_calls.clone(),
    };

    let evaluation_artifact = OxFuncEvaluationArtifact {
        fixture_case_id,
        library_context_snapshot_ref: host_output.library_context_snapshot_ref.clone(),
        evaluation_result: host_output.evaluation.result.clone(),
        worksheet_value: host_output.published_worksheet_value.clone(),
        returned_value_surface: host_output.returned_value_surface.clone(),
        execution_outcome_surface: host_output.execution_outcome_surface.clone(),
        candidate_result_id: host_output.candidate_result.candidate_result_id.clone(),
        commit_decision_kind: match &host_output.commit_decision {
            AcceptDecision::Accepted(_) => "accepted".to_string(),
            AcceptDecision::Rejected(_) => "rejected".to_string(),
        },
        reject_code: match &host_output.commit_decision {
            AcceptDecision::Accepted(_) => None,
            AcceptDecision::Rejected(reject) => Some(reject.reject_code),
        },
        trace_event_kinds: host_output
            .trace_events
            .iter()
            .map(|event| format!("{:?}", event.event_kind))
            .collect(),
    };

    Ok(OxFuncAdapterRun {
        host_output,
        preparation_artifact,
        evaluation_artifact,
    })
}
