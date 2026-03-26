use crate::binding::{BoundExpr, BoundFormula, NormalizedReference, ReferenceExpr};
use crate::source::FormulaChannelKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierValidationDisposition {
    Admitted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierRestrictionCode {
    UnionReferenceOperatorNotAdmitted,
    IntersectionReferenceOperatorNotAdmitted,
    SpillReferenceOperatorNotAdmitted,
    ExternalReferenceNotAdmitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalFormattingCarrierSpec {
    pub target_ranges: Vec<String>,
    pub rule_kind: String,
    pub operator: Option<String>,
    pub threshold_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataValidationCarrierSpec {
    pub target_ranges: Vec<String>,
    pub validation_kind: String,
    pub operator: Option<String>,
    pub formula_slot: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaCarrierValidation {
    pub formula_channel_kind: FormulaChannelKind,
    pub restriction_profile_id: String,
    pub disposition: CarrierValidationDisposition,
    pub restriction_codes: Vec<CarrierRestrictionCode>,
    pub host_field_facts: Vec<String>,
}

pub fn validate_conditional_formatting_formula(
    bound_formula: &BoundFormula,
    spec: &ConditionalFormattingCarrierSpec,
) -> FormulaCarrierValidation {
    let restriction_codes = collect_restrictions(bound_formula);
    FormulaCarrierValidation {
        formula_channel_kind: FormulaChannelKind::ConditionalFormatting,
        restriction_profile_id: "cf_restricted_not_equal_to_dv".to_string(),
        disposition: if restriction_codes.is_empty() {
            CarrierValidationDisposition::Admitted
        } else {
            CarrierValidationDisposition::Rejected
        },
        restriction_codes,
        host_field_facts: build_cf_host_field_facts(spec),
    }
}

pub fn validate_data_validation_formula(
    bound_formula: &BoundFormula,
    spec: &DataValidationCarrierSpec,
) -> FormulaCarrierValidation {
    let restriction_codes = collect_restrictions(bound_formula);
    FormulaCarrierValidation {
        formula_channel_kind: FormulaChannelKind::DataValidation,
        restriction_profile_id: "dv_restricted_not_equal_to_cf".to_string(),
        disposition: if restriction_codes.is_empty() {
            CarrierValidationDisposition::Admitted
        } else {
            CarrierValidationDisposition::Rejected
        },
        restriction_codes,
        host_field_facts: build_dv_host_field_facts(spec),
    }
}

fn build_cf_host_field_facts(spec: &ConditionalFormattingCarrierSpec) -> Vec<String> {
    let mut facts = vec![
        format!("target_ranges={}", spec.target_ranges.join(",")),
        format!("rule_kind={}", spec.rule_kind),
    ];
    if let Some(operator) = &spec.operator {
        facts.push(format!("operator={operator}"));
    }
    if !spec.threshold_fields.is_empty() {
        facts.push(format!(
            "threshold_fields={}",
            spec.threshold_fields.join(",")
        ));
    }
    facts
}

fn build_dv_host_field_facts(spec: &DataValidationCarrierSpec) -> Vec<String> {
    let mut facts = vec![
        format!("target_ranges={}", spec.target_ranges.join(",")),
        format!("validation_kind={}", spec.validation_kind),
        format!("formula_slot={}", spec.formula_slot),
    ];
    if let Some(operator) = &spec.operator {
        facts.push(format!("operator={operator}"));
    }
    facts
}

fn collect_restrictions(bound_formula: &BoundFormula) -> Vec<CarrierRestrictionCode> {
    let mut restrictions = Vec::new();
    collect_expr_restrictions(&bound_formula.root, &mut restrictions);
    if bound_formula
        .normalized_references
        .iter()
        .any(|reference| matches!(reference, NormalizedReference::External(_)))
    {
        push_unique(
            &mut restrictions,
            CarrierRestrictionCode::ExternalReferenceNotAdmitted,
        );
    }
    restrictions
}

fn collect_expr_restrictions(expr: &BoundExpr, restrictions: &mut Vec<CarrierRestrictionCode>) {
    match expr {
        BoundExpr::Binary { left, right, .. } => {
            collect_expr_restrictions(left, restrictions);
            collect_expr_restrictions(right, restrictions);
        }
        BoundExpr::FunctionCall { args, .. } | BoundExpr::Invocation { args, .. } => {
            for arg in args {
                collect_expr_restrictions(arg, restrictions);
            }
        }
        BoundExpr::Reference(reference) => {
            collect_reference_restrictions(reference, restrictions);
        }
        BoundExpr::ImplicitIntersection(inner) => collect_expr_restrictions(inner, restrictions),
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::ArrayLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_) => {}
    }
}

fn collect_reference_restrictions(
    reference: &ReferenceExpr,
    restrictions: &mut Vec<CarrierRestrictionCode>,
) {
    match reference {
        ReferenceExpr::Union { left, right } => {
            push_unique(
                restrictions,
                CarrierRestrictionCode::UnionReferenceOperatorNotAdmitted,
            );
            collect_reference_restrictions(left, restrictions);
            collect_reference_restrictions(right, restrictions);
        }
        ReferenceExpr::Intersection { left, right } => {
            push_unique(
                restrictions,
                CarrierRestrictionCode::IntersectionReferenceOperatorNotAdmitted,
            );
            collect_reference_restrictions(left, restrictions);
            collect_reference_restrictions(right, restrictions);
        }
        ReferenceExpr::Spill { anchor } => {
            push_unique(
                restrictions,
                CarrierRestrictionCode::SpillReferenceOperatorNotAdmitted,
            );
            collect_reference_restrictions(anchor, restrictions);
        }
        ReferenceExpr::Range { start, end } => {
            collect_reference_restrictions(start, restrictions);
            collect_reference_restrictions(end, restrictions);
        }
        ReferenceExpr::Atom(_) => {}
    }
}

fn push_unique(
    restrictions: &mut Vec<CarrierRestrictionCode>,
    restriction: CarrierRestrictionCode,
) {
    if !restrictions.contains(&restriction) {
        restrictions.push(restriction);
    }
}
