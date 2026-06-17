use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::value::ExcelText;

use oxfml_core::binding::{BindContext, BindRequest, NormalizedReference, bind_formula};
use oxfml_core::carrier::{
    CarrierRestrictionCode, CarrierValidationDisposition, ConditionalFormattingCarrierSpec,
    DataValidationCarrierSpec, validate_conditional_formatting_formula,
    validate_data_validation_formula,
};
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::interface::{
    HostProviderOutcomeKind, InMemoryLibraryContextProvider, ReturnedValueSurfaceKind,
    TypedContextQueryBundle,
};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::{
    LibraryAvailabilityState, LibraryContextSnapshot, LibraryContextSnapshotEntry,
    RegistrationSourceKind,
};
use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfml_core::{
    EvaluationBackend, ExecutionOutcomeKind, ExecutionOutcomeStage, LibraryContextSnapshotRef,
};
use oxfunc_core::value::CalcValue;

#[test]
fn r1c1_channel_translates_absolute_relative_and_area_references() {
    let absolute = bind_formula_text(
        FormulaSourceRecord::new("r1c1-abs", 1, "=R2C3")
            .with_formula_channel_kind(FormulaChannelKind::WorksheetR1C1),
        9,
        9,
    );
    match &absolute.normalized_references[0] {
        NormalizedReference::Cell(cell) => {
            assert_eq!(cell.sheet_id, "sheet:default");
            assert_eq!(cell.coord.row, 2);
            assert_eq!(cell.coord.col, 3);
            assert!(cell.address_mode.row_absolute);
            assert!(cell.address_mode.col_absolute);
            assert!(!cell.caller_anchor_used);
        }
        other => panic!("expected R1C1 absolute cell, got {other:?}"),
    }

    let relative = bind_formula_text(
        FormulaSourceRecord::new("r1c1-rel", 1, "=R[-1]C[2]")
            .with_formula_channel_kind(FormulaChannelKind::WorksheetR1C1),
        5,
        3,
    );
    match &relative.normalized_references[0] {
        NormalizedReference::Cell(cell) => {
            assert_eq!(cell.sheet_id, "sheet:default");
            assert_eq!(cell.coord.row, 4);
            assert_eq!(cell.coord.col, 5);
            assert!(!cell.address_mode.row_absolute);
            assert!(!cell.address_mode.col_absolute);
            assert!(cell.caller_anchor_used);
        }
        other => panic!("expected R1C1 relative cell, got {other:?}"),
    }

    let area = bind_formula_text(
        FormulaSourceRecord::new("r1c1-area", 1, "=Sheet2!R1C1:Sheet2!R[1]C[2]")
            .with_formula_channel_kind(FormulaChannelKind::WorksheetR1C1),
        2,
        2,
    );
    assert_eq!(area.normalized_references.len(), 1);
    assert_eq!(area.normalized_references[0].to_string(), "Sheet2!R1C1:3x4");
}

#[test]
fn cf_and_dv_carriers_preserve_host_fields_and_restrictions() {
    let cf_bound = bind_formula_text(
        FormulaSourceRecord::new("cf-admitted", 1, "=A1+1")
            .with_formula_channel_kind(FormulaChannelKind::ConditionalFormatting),
        1,
        1,
    );
    let cf_validation = validate_conditional_formatting_formula(
        &cf_bound,
        &ConditionalFormattingCarrierSpec {
            target_ranges: vec!["A1:A10".to_string()],
            rule_kind: "CellIs".to_string(),
            operator: Some("GreaterThan".to_string()),
            threshold_fields: vec!["cfvo@val".to_string()],
        },
    );
    assert_eq!(
        cf_validation.disposition,
        CarrierValidationDisposition::Admitted
    );
    assert_eq!(
        cf_validation.restriction_profile_id,
        "cf_restricted_not_equal_to_dv"
    );
    assert_eq!(
        cf_validation.host_field_facts,
        vec![
            "target_ranges=A1:A10".to_string(),
            "rule_kind=CellIs".to_string(),
            "operator=GreaterThan".to_string(),
            "threshold_fields=cfvo@val".to_string(),
        ]
    );

    let cf_rejected = bind_formula_text(
        FormulaSourceRecord::new("cf-rejected", 1, "=A1,B1")
            .with_formula_channel_kind(FormulaChannelKind::ConditionalFormatting),
        1,
        1,
    );
    let cf_rejected_validation = validate_conditional_formatting_formula(
        &cf_rejected,
        &ConditionalFormattingCarrierSpec {
            target_ranges: vec!["B1:B10".to_string()],
            rule_kind: "Expression".to_string(),
            operator: None,
            threshold_fields: Vec::new(),
        },
    );
    assert_eq!(
        cf_rejected_validation.disposition,
        CarrierValidationDisposition::Rejected
    );
    assert_eq!(
        cf_rejected_validation.restriction_codes,
        vec![CarrierRestrictionCode::UnionReferenceOperatorNotAdmitted]
    );

    let dv_bound = bind_formula_text(
        FormulaSourceRecord::new("dv-admitted", 1, "=SUM(A1,2)")
            .with_formula_channel_kind(FormulaChannelKind::DataValidation),
        1,
        1,
    );
    let dv_validation = validate_data_validation_formula(
        &dv_bound,
        &DataValidationCarrierSpec {
            target_ranges: vec!["C1:C5".to_string()],
            validation_kind: "Custom".to_string(),
            operator: None,
            formula_slot: "formula1".to_string(),
        },
    );
    assert_eq!(
        dv_validation.disposition,
        CarrierValidationDisposition::Admitted
    );
    assert_eq!(
        dv_validation.restriction_profile_id,
        "dv_restricted_not_equal_to_cf"
    );
    assert_eq!(
        dv_validation.host_field_facts,
        vec![
            "target_ranges=C1:C5".to_string(),
            "validation_kind=Custom".to_string(),
            "formula_slot=formula1".to_string(),
        ]
    );

    let dv_rejected = bind_formula_text(
        FormulaSourceRecord::new("dv-rejected", 1, "=@A1#")
            .with_formula_channel_kind(FormulaChannelKind::DataValidation),
        1,
        1,
    );
    let dv_rejected_validation = validate_data_validation_formula(
        &dv_rejected,
        &DataValidationCarrierSpec {
            target_ranges: vec!["D1:D5".to_string()],
            validation_kind: "Custom".to_string(),
            operator: Some("Between".to_string()),
            formula_slot: "formula2".to_string(),
        },
    );
    assert_eq!(
        dv_rejected_validation.disposition,
        CarrierValidationDisposition::Rejected
    );
    assert_eq!(
        dv_rejected_validation.restriction_codes,
        vec![CarrierRestrictionCode::SpillReferenceOperatorNotAdmitted]
    );
}

#[test]
fn first_host_replay_capture_packet_preserves_snapshot_and_provider_outcomes() {
    let provider = InMemoryLibraryContextProvider::new(snapshot_with_entry("INFO"));
    let locale = oxfml_en_us_locale_context();
    let bundle = TypedContextQueryBundle::new(
        Some(&W047HostInfoProvider),
        None,
        Some(&locale),
        Some(46000.0),
        Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
    );
    let mut info_host = SingleFormulaHost::new("host-info", "=INFO(\"system\")");
    let info_output = info_host
        .recalc_with_interfaces(EvaluationBackend::OxFuncBacked, bundle, Some(&provider))
        .expect("info host recalc");
    let info_packet = info_output.to_first_host_replay_capture_packet();
    assert_eq!(info_packet.adapter_id, "oxfml.replay_adapter.v1");
    assert_eq!(
        info_packet.library_context_snapshot_ref,
        Some(LibraryContextSnapshotRef::new(
            "oxfunc:runtime",
            "2026-03-23"
        ))
    );
    assert_eq!(
        info_packet.typed_query_bundle_spec.families,
        vec![
            oxfml_core::TypedContextQueryFamily::CellInfo,
            oxfml_core::TypedContextQueryFamily::Info,
            oxfml_core::TypedContextQueryFamily::Image,
            oxfml_core::TypedContextQueryFamily::FormulaText,
            oxfml_core::TypedContextQueryFamily::SheetIndex,
            oxfml_core::TypedContextQueryFamily::SheetCount,
            oxfml_core::TypedContextQueryFamily::AggregateReferenceContext,
            oxfml_core::TypedContextQueryFamily::WidthConversionMode,
            oxfml_core::TypedContextQueryFamily::Translate,
            oxfml_core::TypedContextQueryFamily::NowSerial,
            oxfml_core::TypedContextQueryFamily::RandomProvider,
            oxfml_core::TypedContextQueryFamily::LocaleFormatContext,
        ]
    );
    assert_eq!(
        info_packet.returned_value_surface.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    );
    assert_eq!(
        info_packet
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .map(|surface| surface.outcome_kind),
        Some(HostProviderOutcomeKind::UnsupportedQuery)
    );
    assert_eq!(info_packet.commit_decision_kind, "accepted");
    assert_eq!(
        info_packet.execution_outcome_surface.outcome_kind,
        ExecutionOutcomeKind::ExecutedResult
    );
    assert_eq!(
        info_packet.execution_outcome_surface.outcome_stage,
        ExecutionOutcomeStage::Executed
    );
    assert_eq!(
        info_packet.trace_event_kinds,
        vec![
            "AcceptedCandidateResultBuilt".to_string(),
            "CommitAccepted".to_string()
        ]
    );

    let mut cell_host = SingleFormulaHost::new("host-cell", "=CELL(\"filename\",A1)");
    let cell_output = cell_host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::new(
                Some(&W047HostInfoProvider),
                None,
                Some(&locale),
                Some(46000.0),
                Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
            ),
            Some(&provider),
        )
        .expect("cell host recalc");
    let cell_packet = cell_output.to_first_host_replay_capture_packet();
    assert_eq!(
        cell_packet
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .map(|surface| surface.outcome_kind),
        Some(HostProviderOutcomeKind::ProviderFailure)
    );

    let mut rtd_host = SingleFormulaHost::new("host-rtd", "=RTD(\"prog\",\"server\",\"topic\")");
    let rtd_output = rtd_host
        .recalc_with_interfaces(
            EvaluationBackend::OxFuncBacked,
            TypedContextQueryBundle::new(
                Some(&W047HostInfoProvider),
                Some(&CapabilityDeniedRtdProvider),
                Some(&locale),
                Some(46000.0),
                Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
            ),
            Some(&provider),
        )
        .expect("rtd host recalc");
    let rtd_packet = rtd_output.to_first_host_replay_capture_packet();
    assert_eq!(
        rtd_packet
            .returned_value_surface
            .host_provider_outcome
            .as_ref()
            .map(|surface| surface.outcome_kind),
        Some(HostProviderOutcomeKind::CapabilityDenied)
    );
}

#[test]
fn first_host_replay_capture_packet_surfaces_bind_boundary_execution_outcome() {
    let locale = oxfml_en_us_locale_context();
    let mut host = SingleFormulaHost::new("host-bind-reject", "={\"x\",LAMBDA(100)}");
    let output = host
        .recalc(None, Some(&locale))
        .expect("bind-boundary host recalc should still project output");
    let packet = output.to_first_host_replay_capture_packet();

    assert_eq!(packet.commit_decision_kind, "rejected");
    assert_eq!(
        packet.execution_outcome_surface.outcome_kind,
        ExecutionOutcomeKind::Rejected
    );
    assert_eq!(
        packet.execution_outcome_surface.outcome_stage,
        ExecutionOutcomeStage::BindBoundary
    );
    assert_eq!(
        packet.execution_outcome_surface.class_id,
        "bind_boundary_reject"
    );
    assert_eq!(
        packet.execution_outcome_surface.lane_reason_code.as_deref(),
        Some("BindMismatch")
    );
    assert_eq!(
        packet.execution_outcome_surface.raw_detail.as_deref(),
        Some("LAMBDA cannot appear inside array constants")
    );
}

fn bind_formula_text(
    source: FormulaSourceRecord,
    caller_row: u32,
    caller_col: u32,
) -> oxfml_core::BoundFormula {
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source,
        green_tree: parse.green_tree,
        red_projection: red,
        context: BindContext {
            caller_row,
            caller_col,
            structure_context_version: StructureContextVersion("w047-test".to_string()),
            ..BindContext::default()
        },

        reference_bind_profile: None,
    });
    bind.bound_formula
}

fn snapshot_with_entry(surface_name: &str) -> LibraryContextSnapshot {
    LibraryContextSnapshot {
        snapshot_id: "oxfunc:runtime".to_string(),
        snapshot_version: "2026-03-23".to_string(),
        entries: vec![LibraryContextSnapshotEntry {
            surface_name: surface_name.to_string(),
            canonical_id: Some(format!("FUNC.{surface_name}")),
            surface_stable_id: Some(format!("surface:{surface_name}")),
            name_resolution_table_ref: Some("name-table:v1".to_string()),
            semantic_trait_profile_ref: Some("traits:v1".to_string()),
            gating_profile_ref: Some("gating:v1".to_string()),
            metadata_status: Some("runtime".to_string()),
            special_interface_kind: None,
            admission_interface_kind: Some("ordinary".to_string()),
            preparation_owner: Some("oxfunc".to_string()),
            runtime_boundary_kind: Some("host_query".to_string()),
            interface_contract_ref: Some("iface:v1".to_string()),
            registration_source_kind: RegistrationSourceKind::BuiltIn,
            parse_bind_state: LibraryAvailabilityState::CatalogKnown,
            semantic_plan_state: LibraryAvailabilityState::CatalogKnown,
            runtime_capability_state: Some(LibraryAvailabilityState::CatalogKnown),
            post_dispatch_state: Some(LibraryAvailabilityState::CatalogKnown),
        }],
    }
}

struct W047HostInfoProvider;

impl HostInfoProvider for W047HostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&oxfunc_core::value::ReferenceLike>,
    ) -> Result<CalcValue, HostInfoError> {
        match query {
            CellInfoQuery::Filename => Err(HostInfoError::ProviderFailure {
                detail: "filename_unavailable".to_string(),
            }),
            _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
        }
    }

    fn query_info(&self, query: InfoQuery) -> Result<CalcValue, HostInfoError> {
        match query {
            InfoQuery::Directory => Ok(CalcValue::from(CalcValue::text(
                ExcelText::from_utf16_code_units("C:\\Work".encode_utf16().collect()),
            ))),
            _ => Err(HostInfoError::UnsupportedInfoQuery(query)),
        }
    }
}

struct CapabilityDeniedRtdProvider;

impl RtdProvider for CapabilityDeniedRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        RtdProviderResult::CapabilityDenied
    }
}
