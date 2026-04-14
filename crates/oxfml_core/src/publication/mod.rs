use oxfunc_core::locale_format::{LocaleFormatContext, WorkbookDateSystem};
use oxfunc_core::value::{EvalValue, PresentationHint};

use crate::format::{
    parse_value_text, render_currency, render_visible_number, render_visible_value_text,
    render_with_code, worksheet_error_text,
};
use crate::interface::ReturnedValueSurface;
use crate::seam::{
    DisplayDelta, FormatDelta, FormatDependencyFact, TopologyDelta, WorksheetValueClass,
};
use crate::source::FormulaSourceRecord;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerificationConditionalFormattingRule {
    pub target_ranges: Vec<String>,
    pub rule_kind: String,
    pub operator: Option<String>,
    pub thresholds: Vec<String>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub effective_display_text: Option<String>,
    pub applies: Option<bool>,
    pub effective_font_color: Option<String>,
    pub effective_fill_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerificationPublicationContext {
    pub format_profile: Option<String>,
    pub number_format_code: Option<String>,
    pub style_id: Option<String>,
    pub style_hierarchy: Vec<String>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub conditional_formatting_rules: Vec<VerificationConditionalFormattingRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleFormatContextSurface {
    pub locale_profile_id: String,
    pub date_system: String,
    pub decimal_separator: String,
    pub thousands_separator: String,
    pub currency_symbol: String,
    pub date_separator: String,
    pub time_separator: String,
}

impl LocaleFormatContextSurface {
    pub fn from_context(locale_ctx: &LocaleFormatContext<'_>) -> Self {
        Self {
            locale_profile_id: format!("{:?}", locale_ctx.profile.id),
            date_system: format!("{:?}", locale_ctx.date_system),
            decimal_separator: locale_ctx.profile.decimal_separator.to_string(),
            thousands_separator: locale_ctx.profile.thousands_separator.to_string(),
            currency_symbol: locale_ctx.profile.currency_symbol.to_string(),
            date_separator: locale_ctx.profile.date_separator.to_string(),
            time_separator: locale_ctx.profile.time_separator.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerificationPublicationSurface {
    pub has_publication_context: bool,
    pub entered_cell_text: String,
    pub published_value: EvalValue,
    pub published_value_class: WorksheetValueClass,
    pub visible_value_text: String,
    pub effective_display_text: String,
    pub format_profile: Option<String>,
    pub locale_format_context: Option<LocaleFormatContextSurface>,
    pub date1904: Option<bool>,
    pub number_format_code: Option<String>,
    pub style_id: Option<String>,
    pub style_hierarchy: Vec<String>,
    pub format_dependency_facts: Vec<FormatDependencyFact>,
    pub format_delta: Option<FormatDelta>,
    pub display_delta: Option<DisplayDelta>,
    pub returned_value_surface: ReturnedValueSurface,
    pub presentation_hint: Option<PresentationHint>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub effective_font_color: Option<String>,
    pub effective_fill_color: Option<String>,
    pub conditional_formatting_rules: Vec<VerificationConditionalFormattingRule>,
    pub conditional_formatting_target_ranges: Vec<Vec<String>>,
    pub conditional_formatting_rule_kind: Vec<String>,
    pub conditional_formatting_operator: Vec<Option<String>>,
    pub conditional_formatting_thresholds: Vec<Vec<String>>,
    pub conditional_formatting_applies: Vec<Option<bool>>,
    pub conditional_formatting_effective_font_color: Vec<Option<String>>,
    pub conditional_formatting_effective_fill_color: Vec<Option<String>>,
    pub conditional_formatting_effective_display: Vec<Option<String>>,
}

pub fn build_verification_publication_surface(
    source: &FormulaSourceRecord,
    published_worksheet_value: &EvalValue,
    returned_value_surface: &ReturnedValueSurface,
    topology_delta: &TopologyDelta,
    format_delta: Option<&FormatDelta>,
    display_delta: Option<&DisplayDelta>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    context: Option<&VerificationPublicationContext>,
) -> VerificationPublicationSurface {
    let visible_value_text = render_visible_value_text(published_worksheet_value);
    let base_effective_display_text = render_effective_display_text(
        published_worksheet_value,
        returned_value_surface.presentation_hint.as_ref(),
        locale_ctx,
        context.and_then(|value| value.number_format_code.as_deref()),
    )
    .unwrap_or_else(|| visible_value_text.clone());
    let base_font_color = context.and_then(|value| value.font_color.clone());
    let base_fill_color = context.and_then(|value| value.fill_color.clone());
    let conditional_formatting_rules = context
        .map(|value| {
            value
                .conditional_formatting_rules
                .iter()
                .map(|rule| {
                    evaluate_conditional_formatting_rule(
                        rule,
                        published_worksheet_value,
                        &visible_value_text,
                        &base_effective_display_text,
                        locale_ctx,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let effective_display_text = conditional_formatting_rules
        .iter()
        .rev()
        .find(|rule| rule.applies == Some(true))
        .and_then(|rule| rule.effective_display_text.clone())
        .unwrap_or_else(|| base_effective_display_text.clone());
    let effective_font_color = conditional_formatting_rules
        .iter()
        .rev()
        .find(|rule| rule.applies == Some(true))
        .and_then(|rule| rule.effective_font_color.clone())
        .or_else(|| base_font_color.clone());
    let effective_fill_color = conditional_formatting_rules
        .iter()
        .rev()
        .find(|rule| rule.applies == Some(true))
        .and_then(|rule| rule.effective_fill_color.clone())
        .or_else(|| base_fill_color.clone());

    VerificationPublicationSurface {
        has_publication_context: context.is_some(),
        entered_cell_text: source.entered_formula_text.clone(),
        published_value: published_worksheet_value.clone(),
        published_value_class: published_worksheet_value_class(published_worksheet_value),
        visible_value_text,
        effective_display_text,
        format_profile: context
            .and_then(|value| value.format_profile.clone())
            .or_else(|| locale_ctx.map(|_| "locale-format-context".to_string())),
        locale_format_context: locale_ctx.map(LocaleFormatContextSurface::from_context),
        date1904: locale_ctx
            .map(|value| matches!(value.date_system, WorkbookDateSystem::System1904)),
        number_format_code: context.and_then(|value| value.number_format_code.clone()),
        style_id: context.and_then(|value| value.style_id.clone()),
        style_hierarchy: context
            .map(|value| value.style_hierarchy.clone())
            .unwrap_or_default(),
        format_dependency_facts: topology_delta.format_dependency_facts.clone(),
        format_delta: format_delta.cloned(),
        display_delta: display_delta.cloned(),
        returned_value_surface: returned_value_surface.clone(),
        presentation_hint: returned_value_surface.presentation_hint,
        font_color: base_font_color,
        fill_color: base_fill_color,
        effective_font_color,
        effective_fill_color,
        conditional_formatting_target_ranges: conditional_formatting_rules
            .iter()
            .map(|rule| rule.target_ranges.clone())
            .collect(),
        conditional_formatting_rule_kind: conditional_formatting_rules
            .iter()
            .map(|rule| rule.rule_kind.clone())
            .collect(),
        conditional_formatting_operator: conditional_formatting_rules
            .iter()
            .map(|rule| rule.operator.clone())
            .collect(),
        conditional_formatting_thresholds: conditional_formatting_rules
            .iter()
            .map(|rule| rule.thresholds.clone())
            .collect(),
        conditional_formatting_applies: conditional_formatting_rules
            .iter()
            .map(|rule| rule.applies)
            .collect(),
        conditional_formatting_effective_font_color: conditional_formatting_rules
            .iter()
            .map(|rule| rule.effective_font_color.clone())
            .collect(),
        conditional_formatting_effective_fill_color: conditional_formatting_rules
            .iter()
            .map(|rule| rule.effective_fill_color.clone())
            .collect(),
        conditional_formatting_effective_display: conditional_formatting_rules
            .iter()
            .map(|rule| {
                if rule.applies == Some(true) {
                    rule.effective_display_text.clone()
                } else {
                    None
                }
            })
            .collect(),
        conditional_formatting_rules,
    }
}

fn published_worksheet_value_class(value: &EvalValue) -> WorksheetValueClass {
    match value {
        EvalValue::Error(_) => WorksheetValueClass::Error,
        EvalValue::Array(_) => WorksheetValueClass::ArrayAnchor,
        _ => WorksheetValueClass::Scalar,
    }
}

fn render_effective_display_text(
    value: &EvalValue,
    presentation_hint: Option<&PresentationHint>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    number_format_code: Option<&str>,
) -> Option<String> {
    let EvalValue::Number(number) = value else {
        return Some(render_visible_value_text(value));
    };

    let locale_ctx = locale_ctx?;
    if let Some(code) = number_format_code {
        if let Ok(rendered) = render_with_code(&locale_ctx.profile, locale_ctx.date_system, *number, code) {
            return Some(rendered);
        }
    }

    let hint = presentation_hint.and_then(|value| value.number_format)?;
    match hint {
        oxfunc_core::value::NumberFormatHint::Currency => render_currency(
            &locale_ctx.profile,
            *number,
            locale_ctx.profile.currency_decimals.into(),
        )
        .ok(),
        oxfunc_core::value::NumberFormatHint::Percentage => {
            render_with_code(&locale_ctx.profile, locale_ctx.date_system, *number, "0%").ok()
        }
        oxfunc_core::value::NumberFormatHint::DateLike => render_with_code(
            &locale_ctx.profile,
            locale_ctx.date_system,
            *number,
            "yyyy-mm-dd",
        )
        .ok(),
        oxfunc_core::value::NumberFormatHint::General
        | oxfunc_core::value::NumberFormatHint::Scientific
        | oxfunc_core::value::NumberFormatHint::Fraction
        | oxfunc_core::value::NumberFormatHint::Custom => Some(render_visible_number(*number)),
    }
}

#[cfg(test)]
mod tests {
    use oxfunc_core::value::{EvalValue, ExtendedValue};

    use super::{
        VerificationConditionalFormattingRule, VerificationPublicationContext,
        build_verification_publication_surface,
    };
    use crate::format::{en_us_context, render_with_code};
    use crate::interface::ReturnedValueSurface;
    use crate::seam::TopologyDelta;
    use crate::source::FormulaSourceRecord;

    #[test]
    fn number_format_code_heuristics_cover_grouping_percent_date_and_negative_sections() {
        let locale = en_us_context();
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, 6.0, "$#,##0.00"),
            Ok("$6.00".to_string())
        );
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, 1234.567, "#,##0.000"),
            Ok("1,234.567".to_string())
        );
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, 0.125, "0.0%"),
            Ok("12.5%".to_string())
        );
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, -1234.5, "($#,##0.00)"),
            Ok("($1,234.50)".to_string())
        );
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, 45293.0, "m/d/yyyy"),
            Ok("1/2/2024".to_string())
        );
        assert!(
            render_with_code(&locale.profile, locale.date_system, 45293.5, "m/d/yyyy h:mm")
                .is_err()
        );
    }

    #[test]
    fn verification_publication_surface_applies_evaluable_conditional_formatting() {
        let locale = en_us_context();
        let source = FormulaSourceRecord::new("publication:test", 1, "=SUM(1,2,3)");
        let returned_value_surface =
            ReturnedValueSurface::from_extended_value(&ExtendedValue::Core(EvalValue::Number(6.0)));
        let context = VerificationPublicationContext {
            format_profile: Some("excel-spreadsheetml-2003-default".to_string()),
            number_format_code: Some("$#,##0.00".to_string()),
            style_id: Some("calc".to_string()),
            style_hierarchy: vec!["calc".to_string()],
            font_color: Some("#112233".to_string()),
            fill_color: Some("#445566".to_string()),
            conditional_formatting_rules: vec![
                VerificationConditionalFormattingRule {
                    target_ranges: vec!["A1".to_string()],
                    rule_kind: "Expression".to_string(),
                    operator: None,
                    thresholds: vec!["=A1>0".to_string()],
                    font_color: Some("#FF0000".to_string()),
                    fill_color: Some("#00FF00".to_string()),
                    effective_display_text: Some("[POS] $6.00".to_string()),
                    applies: None,
                    effective_font_color: None,
                    effective_fill_color: None,
                },
                VerificationConditionalFormattingRule {
                    target_ranges: vec!["A1".to_string()],
                    rule_kind: "CellIs".to_string(),
                    operator: Some("LessThan".to_string()),
                    thresholds: vec!["0".to_string()],
                    font_color: Some("#999999".to_string()),
                    fill_color: Some("#EEEEEE".to_string()),
                    effective_display_text: Some("[NEG] $6.00".to_string()),
                    applies: None,
                    effective_font_color: None,
                    effective_fill_color: None,
                },
            ],
        };

        let surface = build_verification_publication_surface(
            &source,
            &EvalValue::Number(6.0),
            &returned_value_surface,
            &TopologyDelta {
                formula_stable_id: "publication:test".to_string(),
                dependency_additions: Vec::new(),
                dependency_removals: Vec::new(),
                dependency_reclassifications: Vec::new(),
                dependency_consequence_facts: Vec::new(),
                dynamic_reference_facts: Vec::new(),
                spill_facts: Vec::new(),
                format_dependency_facts: Vec::new(),
                capability_effect_facts: Vec::new(),
                candidate_result_id: None,
            },
            None,
            None,
            Some(&locale),
            Some(&context),
        );

        assert_eq!(surface.effective_display_text, "[POS] $6.00");
        assert_eq!(surface.effective_font_color.as_deref(), Some("#FF0000"));
        assert_eq!(surface.effective_fill_color.as_deref(), Some("#00FF00"));
        assert_eq!(
            surface.conditional_formatting_applies,
            vec![Some(true), Some(false)]
        );
        assert_eq!(
            surface.conditional_formatting_effective_display,
            vec![Some("[POS] $6.00".to_string()), None]
        );
        assert_eq!(
            surface.conditional_formatting_effective_font_color,
            vec![Some("#FF0000".to_string()), None]
        );
        assert_eq!(
            surface.conditional_formatting_effective_fill_color,
            vec![Some("#00FF00".to_string()), None]
        );
    }
}

fn evaluate_conditional_formatting_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    visible_value_text: &str,
    effective_display_text: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> VerificationConditionalFormattingRule {
    let applies = if let Some(operator) = rule.operator.as_deref() {
        evaluate_operator_rule(
            operator,
            &rule.thresholds,
            value,
            visible_value_text,
            locale_ctx,
        )
    } else if rule.rule_kind.eq_ignore_ascii_case("expression") {
        rule.thresholds.first().and_then(|formula| {
            evaluate_expression_rule(formula, value, visible_value_text, locale_ctx)
        })
    } else {
        None
    };

    let effective_font_color = if applies == Some(true) {
        rule.font_color.clone()
    } else {
        None
    };
    let effective_fill_color = if applies == Some(true) {
        rule.fill_color.clone()
    } else {
        None
    };
    let effective_display_text = if applies == Some(true) {
        rule.effective_display_text
            .clone()
            .or_else(|| Some(effective_display_text.to_string()))
    } else {
        None
    };

    VerificationConditionalFormattingRule {
        target_ranges: rule.target_ranges.clone(),
        rule_kind: rule.rule_kind.clone(),
        operator: rule.operator.clone(),
        thresholds: rule.thresholds.clone(),
        font_color: rule.font_color.clone(),
        fill_color: rule.fill_color.clone(),
        effective_display_text,
        applies,
        effective_font_color,
        effective_fill_color,
    }
}

fn evaluate_operator_rule(
    operator: &str,
    thresholds: &[String],
    value: &EvalValue,
    visible_value_text: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> Option<bool> {
    let normalized = operator
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "greater" | "greaterthan" => {
            compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)
                .map(|ordering| ordering.is_gt())
        }
        "greaterorequal" | "greaterthanorequal" | "greaterequal" => {
            compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)
                .map(|ordering| ordering.is_gt() || ordering.is_eq())
        }
        "less" | "lessthan" => {
            compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)
                .map(|ordering| ordering.is_lt())
        }
        "lessorequal" | "lessthanorequal" | "lessequal" => {
            compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)
                .map(|ordering| ordering.is_lt() || ordering.is_eq())
        }
        "equal" => compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)
            .map(|ordering| ordering.is_eq()),
        "notequal" => compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)
            .map(|ordering| !ordering.is_eq()),
        "between" => {
            let lower =
                compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)?;
            let upper =
                compare_threshold(value, visible_value_text, thresholds.get(1)?, locale_ctx)?;
            Some((lower.is_gt() || lower.is_eq()) && (upper.is_lt() || upper.is_eq()))
        }
        "notbetween" => {
            let lower =
                compare_threshold(value, visible_value_text, thresholds.first()?, locale_ctx)?;
            let upper =
                compare_threshold(value, visible_value_text, thresholds.get(1)?, locale_ctx)?;
            Some(!(lower.is_gt() || lower.is_eq()) || !(upper.is_lt() || upper.is_eq()))
        }
        "containstext" => Some(
            visible_value_text
                .to_ascii_lowercase()
                .contains(&thresholds.first()?.to_ascii_lowercase()),
        ),
        "notcontainstext" => Some(
            !visible_value_text
                .to_ascii_lowercase()
                .contains(&thresholds.first()?.to_ascii_lowercase()),
        ),
        "beginswith" => Some(
            visible_value_text
                .to_ascii_lowercase()
                .starts_with(&thresholds.first()?.to_ascii_lowercase()),
        ),
        "endswith" => Some(
            visible_value_text
                .to_ascii_lowercase()
                .ends_with(&thresholds.first()?.to_ascii_lowercase()),
        ),
        _ => None,
    }
}

fn evaluate_expression_rule(
    formula: &str,
    value: &EvalValue,
    visible_value_text: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> Option<bool> {
    let trimmed = formula.trim().trim_start_matches('=').trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("and(") && trimmed.ends_with(')') {
        return split_formula_arguments(&trimmed[4..trimmed.len() - 1])
            .into_iter()
            .map(|arg| {
                evaluate_expression_rule(&format!("={arg}"), value, visible_value_text, locale_ctx)
            })
            .try_fold(true, |acc, next| Some(acc && next?));
    }
    if lower.starts_with("or(") && trimmed.ends_with(')') {
        return split_formula_arguments(&trimmed[3..trimmed.len() - 1])
            .into_iter()
            .map(|arg| {
                evaluate_expression_rule(&format!("={arg}"), value, visible_value_text, locale_ctx)
            })
            .try_fold(false, |acc, next| Some(acc || next?));
    }

    let (lhs, operator, rhs) = split_binary_expression(trimmed)?;
    if !is_current_cell_reference(lhs.trim()) {
        return None;
    }
    let threshold = if is_current_cell_reference(rhs.trim()) {
        visible_value_text.to_string()
    } else {
        rhs.trim().to_string()
    };
    let ordering = compare_threshold(value, visible_value_text, &threshold, locale_ctx)?;
    Some(match operator {
        ">" => ordering.is_gt(),
        ">=" => ordering.is_gt() || ordering.is_eq(),
        "<" => ordering.is_lt(),
        "<=" => ordering.is_lt() || ordering.is_eq(),
        "=" => ordering.is_eq(),
        "<>" => !ordering.is_eq(),
        _ => return None,
    })
}

fn split_formula_arguments(args: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    for ch in args.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn split_binary_expression(expression: &str) -> Option<(&str, &str, &str)> {
    for operator in [">=", "<=", "<>", ">", "<", "="] {
        if let Some(index) = expression.find(operator) {
            let lhs = &expression[..index];
            let rhs = &expression[index + operator.len()..];
            return Some((lhs, operator, rhs));
        }
    }
    None
}

fn is_current_cell_reference(token: &str) -> bool {
    let trimmed = token.trim().trim_matches('$');
    if trimmed.is_empty() {
        return false;
    }
    let letters = trimmed
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .count();
    letters > 0
        && letters < trimmed.len()
        && trimmed[letters..].chars().all(|ch| ch.is_ascii_digit())
}

fn compare_threshold(
    value: &EvalValue,
    visible_value_text: &str,
    threshold: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> Option<std::cmp::Ordering> {
    match value {
        EvalValue::Number(number) => {
            let threshold_value = parse_threshold_number(threshold, locale_ctx)?;
            number.partial_cmp(&threshold_value)
        }
        EvalValue::Text(text) => Some(
            text.to_string_lossy()
                .to_ascii_lowercase()
                .cmp(&strip_threshold_quotes(threshold).to_ascii_lowercase()),
        ),
        EvalValue::Logical(logical) => {
            let threshold_value = parse_threshold_bool(threshold)?;
            Some(logical.cmp(&threshold_value))
        }
        EvalValue::Error(code) => {
            Some(worksheet_error_text(*code).cmp(strip_threshold_quotes(threshold)))
        }
        _ => Some(visible_value_text.cmp(strip_threshold_quotes(threshold))),
    }
}

fn parse_threshold_number(
    threshold: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> Option<f64> {
    let stripped = strip_threshold_quotes(threshold);
    stripped.parse::<f64>().ok().or_else(|| {
        locale_ctx.and_then(|ctx| parse_value_text(&ctx.profile, ctx.date_system, stripped).ok())
    })
}

fn parse_threshold_bool(threshold: &str) -> Option<bool> {
    match strip_threshold_quotes(threshold)
        .to_ascii_lowercase()
        .as_str()
    {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn strip_threshold_quotes(threshold: &str) -> &str {
    threshold.trim().trim_matches('"')
}
