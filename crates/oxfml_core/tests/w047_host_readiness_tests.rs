use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
use oxfml_core::carrier::{
    CarrierRestrictionCode, CarrierValidationDisposition, ConditionalFormattingCarrierSpec,
    DataValidationCarrierSpec, validate_conditional_formatting_formula,
    validate_data_validation_formula,
};
use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::test_support::host::SingleFormulaHost;
use oxfml_core::{ExecutionOutcomeKind, ExecutionOutcomeStage};

// R1C1 absolute/relative/area translation now lives in OxCalc's strict-excel-grid
// profile (see strict_profile_*r1c1* tests), which owns grid reference meaning.

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

// first_host_replay_capture_packet_preserves_snapshot_and_provider_outcomes: grid-reference behavior moved to OxCalc (real strict-excel-grid profile).

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
