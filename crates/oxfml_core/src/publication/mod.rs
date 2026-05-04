use std::collections::BTreeMap;

use oxfunc_core::locale_format::{LocaleFormatContext, WorkbookDateSystem, ymd_from_excel_serial};
use oxfunc_core::value::{ArrayCellValue, EvalValue, ExcelText, PresentationHint};
use serde_json::{Value, json};

use crate::format::{
    parse_value_text, render_currency, render_visible_number, render_visible_value_text,
    render_with_code, worksheet_error_text,
};
use crate::interface::ReturnedValueSurface;
use crate::seam::{
    DisplayDelta, FormatDelta, FormatDependencyFact, TopologyDelta, WorksheetValueClass,
};
use crate::source::FormulaSourceRecord;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerificationConditionalFormattingRule {
    pub target_ranges: Vec<String>,
    pub rule_kind: String,
    pub operator: Option<String>,
    pub thresholds: Vec<String>,
    pub typed_rule: Option<ConditionalFormattingTypedRule>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub effective_display_text: Option<String>,
    pub applies: Option<bool>,
    pub effective_font_color: Option<String>,
    pub effective_fill_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConditionalFormattingTypedRule {
    pub color_scale: Option<ColorScaleRuleOptions>,
    pub data_bar: Option<DataBarRuleOptions>,
    pub icon_set: Option<IconSetRuleOptions>,
    pub rank: Option<RankRuleOptions>,
    pub average: Option<AverageRuleOptions>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColorScaleRuleOptions {
    pub stops: Vec<ColorScaleRuleStop>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorScaleRuleStop {
    pub position: ConditionalFormattingThreshold,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataBarRuleOptions {
    pub minimum: Option<ConditionalFormattingThreshold>,
    pub maximum: Option<ConditionalFormattingThreshold>,
    pub bar_color: Option<String>,
    pub direction: Option<DataBarDirection>,
    pub show_bar_only: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IconSetRuleOptions {
    pub set_kind: String,
    pub thresholds: Vec<ConditionalFormattingThreshold>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankRuleOptions {
    pub rank: ConditionalFormattingRank,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalFormattingRank {
    Count(usize),
    Percent(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AverageRuleOptions {
    pub include_equal: bool,
    pub stddev_multiplier: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalFormattingThreshold {
    Min,
    Mid,
    Max,
    Percent(f64),
    Percentile(f64),
    Number(f64),
}

impl Eq for ConditionalFormattingTypedRule {}
impl Eq for ColorScaleRuleOptions {}
impl Eq for ColorScaleRuleStop {}
impl Eq for DataBarRuleOptions {}
impl Eq for IconSetRuleOptions {}
impl Eq for RankRuleOptions {}
impl Eq for ConditionalFormattingRank {}
impl Eq for AverageRuleOptions {}
impl Eq for ConditionalFormattingThreshold {}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerificationPublicationContext {
    pub format_profile: Option<String>,
    pub number_format_code: Option<String>,
    pub style_id: Option<String>,
    pub style_hierarchy: Vec<String>,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub conditional_formatting_rules: Vec<VerificationConditionalFormattingRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayCellFormatGrid {
    pub rows: Vec<Vec<ArrayCellFormat>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayCellFormat {
    pub effective_display_text: String,
    pub effective_font_color: Option<String>,
    pub effective_fill_color: Option<String>,
    pub data_bar: Option<DataBarFill>,
    pub icon: Option<CfIcon>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataBarFill {
    pub fill_ratio: f64,
    pub bar_color: String,
    pub direction: DataBarDirection,
    pub show_bar_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBarDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfIcon {
    pub set_kind: String,
    pub icon_index: usize,
}

#[derive(Debug, Clone)]
struct AggregateConditionalFormattingContext {
    numeric_values: Vec<f64>,
    sorted_numeric_values: Vec<f64>,
    min: Option<f64>,
    max: Option<f64>,
    mean: Option<f64>,
    stddev: Option<f64>,
    value_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone)]
struct ArrayVisualizationOutcome {
    effective_fill_color: Option<String>,
    data_bar: Option<DataBarFill>,
    icon: Option<CfIcon>,
}

#[derive(Debug, Clone)]
struct ColorScaleStop {
    position: f64,
    color: RgbColor,
}

#[derive(Debug, Clone, Copy)]
struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
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
    pub array_cell_format: Option<ArrayCellFormatGrid>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerificationComparisonView {
    pub view_family: String,
    pub value: Value,
}

pub fn build_verification_publication_surface(
    source: &FormulaSourceRecord,
    published_worksheet_value: &EvalValue,
    returned_value_surface: &ReturnedValueSurface,
    topology_delta: &TopologyDelta,
    format_delta: Option<&FormatDelta>,
    display_delta: Option<&DisplayDelta>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    now_serial: Option<f64>,
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
    let conditional_evaluation_value = conditional_evaluation_value(published_worksheet_value);
    let conditional_formatting_rules = context
        .map(|value| {
            value
                .conditional_formatting_rules
                .iter()
                .map(|rule| {
                    evaluate_conditional_formatting_rule(
                        rule,
                        &conditional_evaluation_value,
                        &visible_value_text,
                        &base_effective_display_text,
                        locale_ctx,
                        now_serial,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let array_cell_format = context.and_then(|value| {
        build_conditional_cell_format_grid(
            published_worksheet_value,
            returned_value_surface.presentation_hint.as_ref(),
            locale_ctx,
            now_serial,
            value,
        )
    });
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
        array_cell_format,
        conditional_formatting_rules,
    }
}

fn conditional_evaluation_value(value: &EvalValue) -> EvalValue {
    let EvalValue::Array(array) = value else {
        return value.clone();
    };
    if array.shape().rows == 1 && array.shape().cols == 1 {
        return array
            .get(0, 0)
            .map(array_cell_to_eval_value)
            .unwrap_or_else(|| value.clone());
    }
    value.clone()
}

fn build_conditional_cell_format_grid(
    value: &EvalValue,
    presentation_hint: Option<&PresentationHint>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    now_serial: Option<f64>,
    context: &VerificationPublicationContext,
) -> Option<ArrayCellFormatGrid> {
    if let EvalValue::Array(array) = value {
        let aggregate_context = AggregateConditionalFormattingContext::from_array(array);
        let shape = array.shape();
        let mut rows = Vec::with_capacity(shape.rows);
        for row_index in 0..shape.rows {
            let mut row = Vec::with_capacity(shape.cols);
            for col_index in 0..shape.cols {
                let cell = array.get(row_index, col_index)?;
                row.push(build_array_cell_format(
                    cell,
                    presentation_hint,
                    locale_ctx,
                    now_serial,
                    context,
                    &aggregate_context,
                ));
            }
            rows.push(row);
        }

        return Some(ArrayCellFormatGrid { rows });
    }

    if !context
        .conditional_formatting_rules
        .iter()
        .any(is_aggregate_or_visualization_rule)
    {
        return None;
    }

    let aggregate_context = AggregateConditionalFormattingContext::from_values([value.clone()]);
    Some(ArrayCellFormatGrid {
        rows: vec![vec![build_eval_cell_format(
            value.clone(),
            presentation_hint,
            locale_ctx,
            now_serial,
            context,
            &aggregate_context,
        )]],
    })
}

fn build_array_cell_format(
    cell: &ArrayCellValue,
    presentation_hint: Option<&PresentationHint>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    now_serial: Option<f64>,
    context: &VerificationPublicationContext,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> ArrayCellFormat {
    let value = array_cell_to_eval_value(cell);
    build_eval_cell_format(
        value,
        presentation_hint,
        locale_ctx,
        now_serial,
        context,
        aggregate_context,
    )
}

fn build_eval_cell_format(
    value: EvalValue,
    presentation_hint: Option<&PresentationHint>,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    now_serial: Option<f64>,
    context: &VerificationPublicationContext,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> ArrayCellFormat {
    let visible_value_text = render_visible_value_text(&value);
    let base_effective_display_text = render_effective_display_text(
        &value,
        presentation_hint,
        locale_ctx,
        context.number_format_code.as_deref(),
    )
    .unwrap_or_else(|| visible_value_text.clone());
    let mut effective_display_text = base_effective_display_text.clone();
    let mut effective_font_color = None;
    let mut effective_fill_color = None;
    let mut data_bar = None;
    let mut icon = None;

    for rule in &context.conditional_formatting_rules {
        if let Some(outcome) = evaluate_array_visualization_rule(rule, &value, aggregate_context) {
            if let Some(color) = outcome.effective_fill_color {
                effective_fill_color = Some(color);
            }
            if outcome.data_bar.is_some() {
                data_bar = outcome.data_bar;
            }
            if outcome.icon.is_some() {
                icon = outcome.icon;
            }
            continue;
        }

        let evaluated_rule = evaluate_array_conditional_formatting_rule(
            rule,
            &value,
            &visible_value_text,
            &base_effective_display_text,
            locale_ctx,
            now_serial,
            aggregate_context,
        );
        if evaluated_rule.applies != Some(true) {
            continue;
        }
        if let Some(display_text) = evaluated_rule.effective_display_text {
            effective_display_text = display_text;
        }
        if let Some(font_color) = evaluated_rule.effective_font_color {
            effective_font_color = Some(font_color);
        }
        if let Some(fill_color) = evaluated_rule.effective_fill_color {
            effective_fill_color = Some(fill_color);
        }
    }

    ArrayCellFormat {
        effective_display_text,
        effective_font_color,
        effective_fill_color,
        data_bar,
        icon,
    }
}

fn is_aggregate_or_visualization_rule(rule: &VerificationConditionalFormattingRule) -> bool {
    matches!(
        normalized_token(&rule.rule_kind).as_str(),
        "colorscale"
            | "databar"
            | "iconset"
            | "aboveaverage"
            | "belowaverage"
            | "top"
            | "bottom"
            | "uniquevalues"
            | "duplicatevalues"
    )
}

fn array_cell_to_eval_value(cell: &ArrayCellValue) -> EvalValue {
    cell.to_eval_value()
        .unwrap_or_else(|| EvalValue::Text(ExcelText::from_interop_assignment("")))
}

impl AggregateConditionalFormattingContext {
    fn from_array(array: &oxfunc_core::value::EvalArray) -> Self {
        Self::from_values(
            array
                .iter_row_major()
                .map(array_cell_to_eval_value)
                .collect::<Vec<_>>(),
        )
    }

    fn from_values(values: impl IntoIterator<Item = EvalValue>) -> Self {
        let mut numeric_values = Vec::new();
        let mut value_counts = BTreeMap::new();
        for value in values {
            if let EvalValue::Number(number) = &value
                && number.is_finite()
            {
                numeric_values.push(*number);
            }
            let value_text = render_visible_value_text(&value);
            *value_counts.entry(value_text).or_insert(0) += 1;
        }

        let mut sorted_numeric_values = numeric_values.clone();
        sorted_numeric_values.sort_by(f64::total_cmp);
        let min = sorted_numeric_values.first().copied();
        let max = sorted_numeric_values.last().copied();
        let mean = (!numeric_values.is_empty())
            .then(|| numeric_values.iter().sum::<f64>() / numeric_values.len() as f64);
        let stddev = mean.and_then(|mean| {
            (numeric_values.len() > 1).then(|| {
                let variance = numeric_values
                    .iter()
                    .map(|value| (value - mean).powi(2))
                    .sum::<f64>()
                    / numeric_values.len() as f64;
                variance.sqrt()
            })
        });

        Self {
            numeric_values,
            sorted_numeric_values,
            min,
            max,
            mean,
            stddev,
            value_counts,
        }
    }

    fn count_for_visible_value(&self, visible_value_text: &str) -> Option<usize> {
        self.value_counts.get(visible_value_text).copied()
    }

    fn ratio_for_value(&self, value: f64, single_value_ratio: f64) -> Option<f64> {
        let min = self.min?;
        let max = self.max?;
        if (max - min).abs() <= f64::EPSILON {
            return Some(single_value_ratio);
        }
        Some(((value - min) / (max - min)).clamp(0.0, 1.0))
    }
}

pub fn build_verification_comparison_views(
    surface: &VerificationPublicationSurface,
) -> Vec<VerificationComparisonView> {
    let mut views = vec![
        VerificationComparisonView {
            view_family: "comparison_value".to_string(),
            value: comparison_value_json(&surface.published_value),
        },
        VerificationComparisonView {
            view_family: "visible_value_text".to_string(),
            value: Value::String(surface.visible_value_text.clone()),
        },
        VerificationComparisonView {
            view_family: "effective_display_text".to_string(),
            value: Value::String(surface.effective_display_text.clone()),
        },
    ];

    if !surface.has_publication_context {
        return views;
    }

    views.extend([
        VerificationComparisonView {
            view_family: "formatting_view".to_string(),
            value: formatting_view_json(surface),
        },
        VerificationComparisonView {
            view_family: "conditional_formatting_view".to_string(),
            value: conditional_formatting_view_json(surface),
        },
    ]);
    views
}

fn published_worksheet_value_class(value: &EvalValue) -> WorksheetValueClass {
    match value {
        EvalValue::Error(_) => WorksheetValueClass::Error,
        EvalValue::Array(_) => WorksheetValueClass::ArrayAnchor,
        _ => WorksheetValueClass::Scalar,
    }
}

fn comparison_value_json(value: &EvalValue) -> Value {
    match value {
        EvalValue::Number(number) => json!({
            "kind": "number",
            "value": number
        }),
        EvalValue::Text(text) => json!({
            "kind": "text",
            "value": text.to_string_lossy()
        }),
        EvalValue::Logical(value) => json!({
            "kind": "logical",
            "value": value
        }),
        EvalValue::Error(code) => json!({
            "kind": "error",
            "code": format!("{code:?}"),
            "display": worksheet_error_text(*code)
        }),
        EvalValue::Array(array) => json!({
            "kind": "array",
            "shape": {
                "rows": array.shape().rows,
                "cols": array.shape().cols
            },
            "cells": array
                .iter_row_major()
                .map(array_cell_json)
                .collect::<Vec<_>>()
        }),
        EvalValue::Reference(reference) => json!({
            "kind": "reference",
            "reference_kind": format!("{:?}", reference.kind),
            "target": reference.target
        }),
        EvalValue::Lambda(lambda) => json!({
            "kind": "lambda",
            "callable_token": lambda.callable_token,
            "origin_kind": format!("{:?}", lambda.origin_kind),
            "arity_shape": {
                "min": lambda.arity_shape.min,
                "max": lambda.arity_shape.max
            },
            "capture_mode": format!("{:?}", lambda.capture_mode),
            "invocation_contract_ref": lambda.invocation_contract_ref
        }),
    }
}

fn array_cell_json(value: &ArrayCellValue) -> Value {
    match value {
        ArrayCellValue::Number(number) => json!({
            "kind": "number",
            "value": number
        }),
        ArrayCellValue::Text(text) => json!({
            "kind": "text",
            "value": text.to_string_lossy()
        }),
        ArrayCellValue::Logical(value) => json!({
            "kind": "logical",
            "value": value
        }),
        ArrayCellValue::Error(code) => json!({
            "kind": "error",
            "code": format!("{code:?}"),
            "display": worksheet_error_text(*code)
        }),
        ArrayCellValue::EmptyCell => json!({
            "kind": "empty_cell"
        }),
    }
}

fn formatting_view_json(surface: &VerificationPublicationSurface) -> Value {
    if is_spreadsheetml_xml_verification(surface) {
        return json!({
            "number_format_code": surface.number_format_code,
            "style_id": surface.style_id,
            "font_color": surface.font_color,
            "fill_color": surface.fill_color
        });
    }

    json!({
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
    })
}

fn conditional_formatting_view_json(surface: &VerificationPublicationSurface) -> Value {
    if is_spreadsheetml_xml_verification(surface) {
        let rules = surface
            .conditional_formatting_rules
            .iter()
            .map(spreadsheetml_conditional_formatting_rule_json)
            .collect::<Vec<_>>();
        let applied_rule_indexes = surface
            .conditional_formatting_applies
            .iter()
            .enumerate()
            .filter_map(|(index, applies)| applies.and_then(|value| value.then_some(index + 1)))
            .collect::<Vec<_>>();

        return json!({
            "rules": rules,
            "effective_style": {
                "number_format_code": surface.number_format_code,
                "font_color": surface.effective_font_color,
                "fill_color": surface.effective_fill_color,
                "effective_display_text": surface.effective_display_text,
                "applied_rule_indexes": applied_rule_indexes,
                "source_projection": "spreadsheetml_expression_rules_v1"
            }
        });
    }

    json!({
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
        "effective_display": surface.conditional_formatting_effective_display,
        "array_cell_format": surface.array_cell_format.as_ref().map(array_cell_format_grid_json)
    })
}

fn array_cell_format_grid_json(grid: &ArrayCellFormatGrid) -> Value {
    json!({
        "rows": grid.rows
            .iter()
            .map(|row| row.iter().map(array_cell_format_json).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    })
}

fn array_cell_format_json(format: &ArrayCellFormat) -> Value {
    json!({
        "effective_display_text": format.effective_display_text,
        "effective_font_color": format.effective_font_color,
        "effective_fill_color": format.effective_fill_color,
        "data_bar": format.data_bar.as_ref().map(data_bar_fill_json),
        "icon": format.icon.as_ref().map(cf_icon_json)
    })
}

fn data_bar_fill_json(fill: &DataBarFill) -> Value {
    json!({
        "fill_ratio": fill.fill_ratio,
        "bar_color": fill.bar_color,
        "direction": format!("{:?}", fill.direction),
        "show_bar_only": fill.show_bar_only
    })
}

fn cf_icon_json(icon: &CfIcon) -> Value {
    json!({
        "set_kind": icon.set_kind,
        "icon_index": icon.icon_index
    })
}

fn is_spreadsheetml_xml_verification(surface: &VerificationPublicationSurface) -> bool {
    matches!(
        surface.format_profile.as_deref(),
        Some("excel-spreadsheetml-2003-default")
    )
}

fn locale_format_context_json(surface: &LocaleFormatContextSurface) -> Value {
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

fn format_dependency_fact_json(fact: &FormatDependencyFact) -> Value {
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

fn format_delta_json(delta: &FormatDelta) -> Value {
    json!({
        "formula_stable_id": delta.formula_stable_id,
        "target_loci": delta.target_loci.iter().map(locus_json).collect::<Vec<_>>(),
        "format_effect_class": delta.format_effect_class,
        "format_effect_payload": delta.format_effect_payload
    })
}

fn display_delta_json(delta: &DisplayDelta) -> Value {
    json!({
        "formula_stable_id": delta.formula_stable_id,
        "target_loci": delta.target_loci.iter().map(locus_json).collect::<Vec<_>>(),
        "display_effect_class": delta.display_effect_class,
        "display_effect_payload": delta.display_effect_payload
    })
}

fn presentation_hint_json(hint: &PresentationHint) -> Value {
    json!({
        "number_format": hint.number_format.map(|value| format!("{value:?}")),
        "style": hint.style.map(|value| format!("{value:?}"))
    })
}

fn conditional_formatting_rule_json(rule: &VerificationConditionalFormattingRule) -> Value {
    json!({
        "target_ranges": rule.target_ranges,
        "rule_kind": rule.rule_kind,
        "operator": rule.operator,
        "thresholds": rule.thresholds,
        "typed_rule": rule.typed_rule.as_ref().map(typed_conditional_formatting_rule_json),
        "font_color": rule.font_color,
        "fill_color": rule.fill_color,
        "effective_display_text": rule.effective_display_text,
        "applies": rule.applies,
        "effective_font_color": rule.effective_font_color,
        "effective_fill_color": rule.effective_fill_color
    })
}

fn typed_conditional_formatting_rule_json(rule: &ConditionalFormattingTypedRule) -> Value {
    json!({
        "color_scale": rule.color_scale.as_ref().map(color_scale_rule_options_json),
        "data_bar": rule.data_bar.as_ref().map(data_bar_rule_options_json),
        "icon_set": rule.icon_set.as_ref().map(icon_set_rule_options_json),
        "rank": rule.rank.as_ref().map(rank_rule_options_json),
        "average": rule.average.as_ref().map(average_rule_options_json)
    })
}

fn color_scale_rule_options_json(options: &ColorScaleRuleOptions) -> Value {
    json!({
        "stops": options.stops.iter().map(color_scale_rule_stop_json).collect::<Vec<_>>()
    })
}

fn color_scale_rule_stop_json(stop: &ColorScaleRuleStop) -> Value {
    json!({
        "position": conditional_formatting_threshold_json(&stop.position),
        "color": stop.color
    })
}

fn data_bar_rule_options_json(options: &DataBarRuleOptions) -> Value {
    json!({
        "minimum": options.minimum.as_ref().map(conditional_formatting_threshold_json),
        "maximum": options.maximum.as_ref().map(conditional_formatting_threshold_json),
        "bar_color": options.bar_color,
        "direction": options.direction.map(|value| format!("{value:?}")),
        "show_bar_only": options.show_bar_only
    })
}

fn icon_set_rule_options_json(options: &IconSetRuleOptions) -> Value {
    json!({
        "set_kind": options.set_kind,
        "thresholds": options.thresholds.iter().map(conditional_formatting_threshold_json).collect::<Vec<_>>()
    })
}

fn rank_rule_options_json(options: &RankRuleOptions) -> Value {
    match options.rank {
        ConditionalFormattingRank::Count(count) => json!({"kind": "count", "value": count}),
        ConditionalFormattingRank::Percent(percent) => {
            json!({"kind": "percent", "value": percent})
        }
    }
}

fn average_rule_options_json(options: &AverageRuleOptions) -> Value {
    json!({
        "include_equal": options.include_equal,
        "stddev_multiplier": options.stddev_multiplier
    })
}

fn conditional_formatting_threshold_json(threshold: &ConditionalFormattingThreshold) -> Value {
    match threshold {
        ConditionalFormattingThreshold::Min => json!({"kind": "min"}),
        ConditionalFormattingThreshold::Mid => json!({"kind": "mid"}),
        ConditionalFormattingThreshold::Max => json!({"kind": "max"}),
        ConditionalFormattingThreshold::Percent(value) => {
            json!({"kind": "percent", "value": value})
        }
        ConditionalFormattingThreshold::Percentile(value) => {
            json!({"kind": "percentile", "value": value})
        }
        ConditionalFormattingThreshold::Number(value) => json!({"kind": "number", "value": value}),
    }
}

fn spreadsheetml_conditional_formatting_rule_json(
    rule: &VerificationConditionalFormattingRule,
) -> Value {
    let range = rule.target_ranges.first().cloned();
    let is_expression = rule.rule_kind.eq_ignore_ascii_case("expression");
    let formula = is_expression
        .then(|| rule.thresholds.first().cloned())
        .flatten();
    let value1 = (!is_expression)
        .then(|| rule.thresholds.first().cloned())
        .flatten();
    let value2 = (!is_expression)
        .then(|| rule.thresholds.get(1).cloned())
        .flatten();

    json!({
        "range": range,
        "formula": formula,
        "value1": value1,
        "value2": value2,
        "operator": rule.operator,
        "rule_kind": rule.rule_kind.to_ascii_lowercase(),
        "font_color": rule.font_color,
        "fill_color": rule.fill_color
    })
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
        if let Ok(rendered) =
            render_with_code(&locale_ctx.profile, locale_ctx.date_system, *number, code)
        {
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
    use crate::format::{oxfml_en_us_locale_context, render_with_code};
    use crate::interface::ReturnedValueSurface;
    use crate::seam::TopologyDelta;
    use crate::source::FormulaSourceRecord;

    #[test]
    fn number_format_code_heuristics_cover_grouping_percent_date_and_negative_sections() {
        let locale = oxfml_en_us_locale_context();
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
        assert_eq!(
            render_with_code(
                &locale.profile,
                locale.date_system,
                45293.5,
                "m/d/yyyy h:mm"
            ),
            Ok("1/2/2024 12:00".to_string())
        );
    }

    #[test]
    fn verification_publication_surface_applies_evaluable_conditional_formatting() {
        let locale = oxfml_en_us_locale_context();
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
                    typed_rule: None,
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
                    typed_rule: None,
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
            None,
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
    now_serial: Option<f64>,
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
        evaluate_predicate_rule(
            rule,
            value,
            now_serial,
            locale_ctx
                .map(|value| value.date_system)
                .unwrap_or(WorkbookDateSystem::System1900),
        )
    };

    evaluated_conditional_formatting_rule(rule, applies, effective_display_text)
}

fn evaluate_array_conditional_formatting_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    visible_value_text: &str,
    effective_display_text: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
    now_serial: Option<f64>,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> VerificationConditionalFormattingRule {
    if let Some(applies) =
        evaluate_aggregate_rule(rule, value, visible_value_text, aggregate_context)
    {
        return evaluated_conditional_formatting_rule(rule, Some(applies), effective_display_text);
    }

    evaluate_conditional_formatting_rule(
        rule,
        value,
        visible_value_text,
        effective_display_text,
        locale_ctx,
        now_serial,
    )
}

fn evaluate_array_visualization_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<ArrayVisualizationOutcome> {
    match normalized_token(&rule.rule_kind).as_str() {
        "colorscale" => evaluate_color_scale_rule(rule, value, aggregate_context),
        "databar" => evaluate_data_bar_rule(rule, value, aggregate_context),
        "iconset" => evaluate_icon_set_rule(rule, value, aggregate_context),
        _ => None,
    }
}

fn evaluate_color_scale_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<ArrayVisualizationOutcome> {
    let EvalValue::Number(number) = value else {
        return None;
    };
    if !number.is_finite() {
        return None;
    }
    let ratio = aggregate_context.ratio_for_value(*number, 0.5)?;
    let stops = color_scale_stops(rule, aggregate_context)?;
    let color = interpolate_color_scale(&stops, ratio)?;

    Some(ArrayVisualizationOutcome {
        effective_fill_color: Some(color.to_hex()),
        data_bar: None,
        icon: None,
    })
}

fn evaluate_data_bar_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<ArrayVisualizationOutcome> {
    let EvalValue::Number(number) = value else {
        return None;
    };
    if !number.is_finite() {
        return None;
    }
    let typed_options = rule
        .typed_rule
        .as_ref()
        .and_then(|typed| typed.data_bar.as_ref());
    let fill_ratio = data_bar_ratio(rule, typed_options, *number, aggregate_context)?;
    let bar_color = typed_options
        .and_then(|options| options.bar_color.as_deref())
        .or(rule.fill_color.as_deref())
        .and_then(normalize_hex_color)
        .unwrap_or_else(|| "#638EC6".to_string());
    let direction = typed_options
        .and_then(|options| options.direction)
        .unwrap_or_else(|| {
            if rule
                .thresholds
                .iter()
                .any(|threshold| normalized_token(threshold).contains("directionright"))
            {
                DataBarDirection::Right
            } else {
                DataBarDirection::Left
            }
        });
    let show_bar_only = typed_options.map_or_else(
        || {
            rule.thresholds.iter().any(|threshold| {
                let normalized = normalized_token(threshold);
                normalized == "showbaronly" || normalized == "baronly"
            })
        },
        |options| options.show_bar_only,
    );

    Some(ArrayVisualizationOutcome {
        effective_fill_color: None,
        data_bar: Some(DataBarFill {
            fill_ratio,
            bar_color,
            direction,
            show_bar_only,
        }),
        icon: None,
    })
}

fn data_bar_ratio(
    rule: &VerificationConditionalFormattingRule,
    typed_options: Option<&DataBarRuleOptions>,
    value: f64,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<f64> {
    let explicit_min = typed_options
        .and_then(|options| options.minimum.as_ref())
        .and_then(|threshold| typed_threshold_value(threshold, aggregate_context))
        .or_else(|| data_bar_bound(rule, "min"));
    let explicit_max = typed_options
        .and_then(|options| options.maximum.as_ref())
        .and_then(|threshold| typed_threshold_value(threshold, aggregate_context))
        .or_else(|| data_bar_bound(rule, "max"));
    if explicit_min.is_none() && explicit_max.is_none() {
        return aggregate_context.ratio_for_value(value, 1.0);
    }

    let min = explicit_min.or(aggregate_context.min)?;
    let max = explicit_max.or(aggregate_context.max)?;
    if (max - min).abs() <= f64::EPSILON {
        return Some(1.0);
    }
    Some(((value - min) / (max - min)).clamp(0.0, 1.0))
}

fn data_bar_bound(rule: &VerificationConditionalFormattingRule, bound_name: &str) -> Option<f64> {
    for threshold in &rule.thresholds {
        let Some((kind, value)) = threshold.split_once(':') else {
            continue;
        };
        if normalized_token(kind) == bound_name {
            let parsed = value.trim().parse::<f64>().ok()?;
            return parsed.is_finite().then_some(parsed);
        }
    }
    None
}

fn evaluate_icon_set_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<ArrayVisualizationOutcome> {
    let EvalValue::Number(number) = value else {
        return None;
    };
    if !number.is_finite() {
        return None;
    }
    let typed_options = rule
        .typed_rule
        .as_ref()
        .and_then(|typed| typed.icon_set.as_ref());
    let set_kind = typed_options
        .map(|options| options.set_kind.trim())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            rule.thresholds
                .first()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("3Arrows")
        .to_string();
    let icon_count = icon_set_size(&set_kind);
    let ratio = aggregate_context.ratio_for_value(*number, 0.5)?;
    let icon_index = icon_index_for_value(
        rule,
        typed_options,
        icon_count,
        ratio,
        aggregate_context,
        *number,
    )?;

    Some(ArrayVisualizationOutcome {
        effective_fill_color: None,
        data_bar: None,
        icon: Some(CfIcon {
            set_kind,
            icon_index,
        }),
    })
}

fn icon_index_for_value(
    rule: &VerificationConditionalFormattingRule,
    typed_options: Option<&IconSetRuleOptions>,
    icon_count: usize,
    ratio: f64,
    aggregate_context: &AggregateConditionalFormattingContext,
    value: f64,
) -> Option<usize> {
    let thresholds = if let Some(options) = typed_options {
        options
            .thresholds
            .iter()
            .filter_map(|threshold| typed_threshold_value(threshold, aggregate_context))
            .collect::<Vec<_>>()
    } else {
        rule.thresholds
            .iter()
            .skip(1)
            .filter_map(|threshold| icon_threshold_value(threshold, aggregate_context))
            .collect::<Vec<_>>()
    };
    if thresholds.is_empty() {
        let icon_index = (ratio * icon_count as f64).floor() as usize;
        return Some(icon_index.min(icon_count.saturating_sub(1)));
    }

    let mut icon_index = 0usize;
    for threshold in thresholds {
        if value >= threshold {
            icon_index += 1;
        }
    }
    Some(icon_index.min(icon_count.saturating_sub(1)))
}

fn icon_threshold_value(
    threshold: &str,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<f64> {
    let trimmed = threshold.trim();
    if let Some((kind, value)) = trimmed.split_once(':') {
        let parsed = value.trim().trim_end_matches('%').parse::<f64>().ok()?;
        return match normalized_token(kind).as_str() {
            "percent" | "percentile" => {
                let min = aggregate_context.min?;
                let max = aggregate_context.max?;
                let ratio = (parsed / 100.0).clamp(0.0, 1.0);
                Some(min + ratio * (max - min))
            }
            "num" | "number" | "value" => Some(parsed),
            _ => None,
        };
    }

    let parsed = trimmed.trim_end_matches('%').parse::<f64>().ok()?;
    if trimmed.ends_with('%') {
        let min = aggregate_context.min?;
        let max = aggregate_context.max?;
        let ratio = (parsed / 100.0).clamp(0.0, 1.0);
        return Some(min + ratio * (max - min));
    }
    Some(parsed)
}

fn icon_set_size(set_kind: &str) -> usize {
    set_kind
        .chars()
        .find_map(|ch| ch.to_digit(10))
        .map(|value| value as usize)
        .filter(|value| *value >= 2)
        .unwrap_or(3)
}

fn color_scale_stops(
    rule: &VerificationConditionalFormattingRule,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<Vec<ColorScaleStop>> {
    if let Some(options) = rule
        .typed_rule
        .as_ref()
        .and_then(|typed| typed.color_scale.as_ref())
    {
        return typed_color_scale_stops(options, aggregate_context);
    }

    let mut stops = rule
        .thresholds
        .iter()
        .filter_map(|threshold| parse_color_scale_stop(threshold, aggregate_context))
        .collect::<Vec<_>>();
    if stops.is_empty()
        && let (Some(low), Some(high)) = (
            rule.font_color.as_deref().and_then(RgbColor::parse),
            rule.fill_color.as_deref().and_then(RgbColor::parse),
        )
    {
        stops.push(ColorScaleStop {
            position: 0.0,
            color: low,
        });
        stops.push(ColorScaleStop {
            position: 1.0,
            color: high,
        });
    }
    if stops.len() < 2 {
        return None;
    }
    stops.sort_by(|left, right| left.position.total_cmp(&right.position));
    Some(stops)
}

fn typed_color_scale_stops(
    options: &ColorScaleRuleOptions,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<Vec<ColorScaleStop>> {
    let mut stops = options
        .stops
        .iter()
        .filter_map(|stop| {
            Some(ColorScaleStop {
                position: color_scale_position_from_typed(&stop.position, aggregate_context)?,
                color: RgbColor::parse(&stop.color)?,
            })
        })
        .collect::<Vec<_>>();
    if stops.len() < 2 {
        return None;
    }
    stops.sort_by(|left, right| left.position.total_cmp(&right.position));
    Some(stops)
}

fn parse_color_scale_stop(
    threshold: &str,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<ColorScaleStop> {
    let parts = threshold
        .split(':')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let color_text = parts
        .iter()
        .rev()
        .find_map(|part| normalize_hex_color(part))?;
    let color = RgbColor::parse(&color_text)?;
    let position = match parts.as_slice() {
        [color_only] if normalize_hex_color(color_only).is_some() => 0.0,
        [position, _] => color_scale_position(position, aggregate_context)?,
        [kind, value, _] => color_scale_position(&format!("{kind}:{value}"), aggregate_context)?,
        _ => return None,
    };

    Some(ColorScaleStop { position, color })
}

fn color_scale_position_from_typed(
    threshold: &ConditionalFormattingThreshold,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<f64> {
    match threshold {
        ConditionalFormattingThreshold::Min => Some(0.0),
        ConditionalFormattingThreshold::Mid => Some(0.5),
        ConditionalFormattingThreshold::Max => Some(1.0),
        ConditionalFormattingThreshold::Percent(value)
        | ConditionalFormattingThreshold::Percentile(value) => {
            Some((value / 100.0).clamp(0.0, 1.0))
        }
        ConditionalFormattingThreshold::Number(value) => {
            aggregate_context.ratio_for_value(*value, 0.5)
        }
    }
}

fn typed_threshold_value(
    threshold: &ConditionalFormattingThreshold,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<f64> {
    match threshold {
        ConditionalFormattingThreshold::Min => aggregate_context.min,
        ConditionalFormattingThreshold::Mid => {
            let min = aggregate_context.min?;
            let max = aggregate_context.max?;
            Some(min + 0.5 * (max - min))
        }
        ConditionalFormattingThreshold::Max => aggregate_context.max,
        ConditionalFormattingThreshold::Percent(value)
        | ConditionalFormattingThreshold::Percentile(value) => {
            let min = aggregate_context.min?;
            let max = aggregate_context.max?;
            let ratio = (value / 100.0).clamp(0.0, 1.0);
            Some(min + ratio * (max - min))
        }
        ConditionalFormattingThreshold::Number(value) => Some(*value),
    }
}

fn color_scale_position(
    token: &str,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<f64> {
    let trimmed = token.trim();
    let normalized = normalized_token(trimmed);
    match normalized.as_str() {
        "min" => Some(0.0),
        "mid" | "middle" => Some(0.5),
        "max" => Some(1.0),
        _ => {
            if let Some((kind, value)) = trimmed.split_once(':') {
                let parsed = value.trim().trim_end_matches('%').parse::<f64>().ok()?;
                return match normalized_token(kind).as_str() {
                    "percent" | "percentile" => Some((parsed / 100.0).clamp(0.0, 1.0)),
                    "num" | "number" | "value" => aggregate_context.ratio_for_value(parsed, 0.5),
                    _ => None,
                };
            }
            let parsed = trimmed.trim_end_matches('%').parse::<f64>().ok()?;
            if trimmed.ends_with('%') {
                Some((parsed / 100.0).clamp(0.0, 1.0))
            } else {
                Some(parsed.clamp(0.0, 1.0))
            }
        }
    }
}

fn interpolate_color_scale(stops: &[ColorScaleStop], ratio: f64) -> Option<RgbColor> {
    let first = stops.first()?;
    if ratio <= first.position {
        return Some(first.color);
    }
    for pair in stops.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        if ratio <= right.position {
            let width = right.position - left.position;
            let local_ratio = if width.abs() <= f64::EPSILON {
                0.0
            } else {
                (ratio - left.position) / width
            };
            return Some(left.color.interpolate(right.color, local_ratio));
        }
    }
    stops.last().map(|stop| stop.color)
}

fn normalize_hex_color(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('#');
    let rgb = match trimmed.len() {
        6 => trimmed,
        8 => &trimmed[2..],
        _ => return None,
    };
    u32::from_str_radix(rgb, 16).ok()?;
    Some(format!("#{}", rgb.to_ascii_uppercase()))
}

impl RgbColor {
    fn parse(value: &str) -> Option<Self> {
        let normalized = normalize_hex_color(value)?;
        let rgb = normalized.trim_start_matches('#');
        Some(Self {
            r: u8::from_str_radix(&rgb[0..2], 16).ok()?,
            g: u8::from_str_radix(&rgb[2..4], 16).ok()?,
            b: u8::from_str_radix(&rgb[4..6], 16).ok()?,
        })
    }

    fn interpolate(self, other: Self, ratio: f64) -> Self {
        Self {
            r: interpolate_channel(self.r, other.r, ratio),
            g: interpolate_channel(self.g, other.g, ratio),
            b: interpolate_channel(self.b, other.b, ratio),
        }
    }

    fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

fn interpolate_channel(left: u8, right: u8, ratio: f64) -> u8 {
    (left as f64 + (right as f64 - left as f64) * ratio)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn evaluated_conditional_formatting_rule(
    rule: &VerificationConditionalFormattingRule,
    applies: Option<bool>,
    effective_display_text: &str,
) -> VerificationConditionalFormattingRule {
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
        typed_rule: rule.typed_rule.clone(),
        font_color: rule.font_color.clone(),
        fill_color: rule.fill_color.clone(),
        effective_display_text,
        applies,
        effective_font_color,
        effective_fill_color,
    }
}

fn evaluate_aggregate_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    visible_value_text: &str,
    aggregate_context: &AggregateConditionalFormattingContext,
) -> Option<bool> {
    match normalized_token(&rule.rule_kind).as_str() {
        "aboveaverage" => {
            let EvalValue::Number(number) = value else {
                return Some(false);
            };
            evaluate_average_rule(rule, *number, aggregate_context, true)
        }
        "belowaverage" => {
            let EvalValue::Number(number) = value else {
                return Some(false);
            };
            evaluate_average_rule(rule, *number, aggregate_context, false)
        }
        "top" => evaluate_top_bottom_rule(rule, value, aggregate_context, true),
        "bottom" => evaluate_top_bottom_rule(rule, value, aggregate_context, false),
        "uniquevalues" => aggregate_context
            .count_for_visible_value(visible_value_text)
            .map(|count| count == 1),
        "duplicatevalues" => aggregate_context
            .count_for_visible_value(visible_value_text)
            .map(|count| count > 1),
        _ => None,
    }
}

fn evaluate_average_rule(
    rule: &VerificationConditionalFormattingRule,
    number: f64,
    aggregate_context: &AggregateConditionalFormattingContext,
    above: bool,
) -> Option<bool> {
    let mean = aggregate_context.mean?;
    let typed_options = rule
        .typed_rule
        .as_ref()
        .and_then(|typed| typed.average.as_ref());
    let stddev_multiplier = if let Some(options) = typed_options {
        options.stddev_multiplier.unwrap_or(0.0)
    } else {
        average_stddev_multiplier(rule)?
    };
    let threshold = if stddev_multiplier == 0.0 {
        mean
    } else {
        let stddev = aggregate_context.stddev?;
        if above {
            mean + stddev_multiplier * stddev
        } else {
            mean - stddev_multiplier * stddev
        }
    };
    let equal = typed_options.map_or_else(
        || average_includes_equal(rule),
        |options| options.include_equal,
    );
    Some(if above {
        number > threshold || (equal && number == threshold)
    } else {
        number < threshold || (equal && number == threshold)
    })
}

fn average_stddev_multiplier(rule: &VerificationConditionalFormattingRule) -> Option<f64> {
    for threshold in &rule.thresholds {
        let trimmed = threshold.trim();
        if let Some((kind, value)) = trimmed.split_once(':')
            && normalized_token(kind) == "stddev"
        {
            let parsed = value.trim().parse::<f64>().ok()?;
            return parsed.is_finite().then_some(parsed.max(0.0));
        }

        let normalized = normalized_token(trimmed);
        if let Some(prefix) = normalized.strip_suffix("stddev") {
            let parsed = prefix.parse::<f64>().ok()?;
            return parsed.is_finite().then_some(parsed.max(0.0));
        }
    }
    Some(0.0)
}

fn average_includes_equal(rule: &VerificationConditionalFormattingRule) -> bool {
    rule.thresholds.iter().any(|threshold| {
        let normalized = normalized_token(threshold);
        normalized == "equal" || normalized == "equalaverage" || normalized == "includeequal"
    })
}

fn evaluate_top_bottom_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    aggregate_context: &AggregateConditionalFormattingContext,
    top: bool,
) -> Option<bool> {
    let EvalValue::Number(number) = value else {
        return Some(false);
    };
    let count = aggregate_rank_count(rule, aggregate_context.numeric_values.len())?;
    if count == 0 || aggregate_context.sorted_numeric_values.is_empty() {
        return Some(false);
    }

    let sorted = &aggregate_context.sorted_numeric_values;
    let cutoff = if top {
        let index = sorted.len().saturating_sub(count);
        sorted[index]
    } else {
        sorted[count.saturating_sub(1).min(sorted.len() - 1)]
    };
    Some(if top {
        *number >= cutoff
    } else {
        *number <= cutoff
    })
}

fn aggregate_rank_count(
    rule: &VerificationConditionalFormattingRule,
    value_count: usize,
) -> Option<usize> {
    if value_count == 0 {
        return Some(0);
    }
    if let Some(options) = rule
        .typed_rule
        .as_ref()
        .and_then(|typed| typed.rank.as_ref())
    {
        return match options.rank {
            ConditionalFormattingRank::Count(count) => Some(count.min(value_count)),
            ConditionalFormattingRank::Percent(percent) => {
                if !percent.is_finite() || percent <= 0.0 {
                    Some(0)
                } else {
                    Some(((value_count as f64) * percent / 100.0).ceil() as usize)
                }
            }
        };
    }
    let threshold = rule
        .thresholds
        .first()
        .map(String::as_str)
        .unwrap_or("10")
        .trim();
    if let Some(percent) = threshold.strip_suffix('%') {
        let percent = percent.trim().parse::<f64>().ok()?;
        if !percent.is_finite() || percent <= 0.0 {
            return Some(0);
        }
        return Some(((value_count as f64) * percent / 100.0).ceil() as usize);
    }

    let count = threshold.parse::<usize>().ok()?;
    Some(count.min(value_count))
}

fn evaluate_predicate_rule(
    rule: &VerificationConditionalFormattingRule,
    value: &EvalValue,
    now_serial: Option<f64>,
    date_system: WorkbookDateSystem,
) -> Option<bool> {
    match normalized_token(&rule.rule_kind).as_str() {
        "blanks" => Some(is_blank_value(value)),
        "noblanks" => Some(!is_blank_value(value)),
        "errors" => Some(matches!(value, EvalValue::Error(_))),
        "noerrors" => Some(!matches!(value, EvalValue::Error(_))),
        "dates" => {
            evaluate_relative_date_rule(rule.thresholds.first()?, value, now_serial?, date_system)
        }
        _ => None,
    }
}

fn normalized_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn is_blank_value(value: &EvalValue) -> bool {
    matches!(value, EvalValue::Text(text) if text.to_string_lossy().is_empty())
}

fn evaluate_relative_date_rule(
    kind: &str,
    value: &EvalValue,
    now_serial: f64,
    date_system: WorkbookDateSystem,
) -> Option<bool> {
    let EvalValue::Number(value_serial) = value else {
        return Some(false);
    };
    if !value_serial.is_finite() || !now_serial.is_finite() {
        return None;
    }

    let value_day = value_serial.floor() as i64;
    let now_day = now_serial.floor() as i64;
    match normalized_token(kind).as_str() {
        "today" => Some(value_day == now_day),
        "yesterday" => Some(value_day == now_day - 1),
        "tomorrow" => Some(value_day == now_day + 1),
        "last7days" => Some((now_day - 6..=now_day).contains(&value_day)),
        "thisweek" => Some(serial_in_relative_week(value_day, now_day, 0, date_system)),
        "lastweek" => Some(serial_in_relative_week(value_day, now_day, -1, date_system)),
        "nextweek" => Some(serial_in_relative_week(value_day, now_day, 1, date_system)),
        "thismonth" => serial_in_relative_month(value_day, now_day, 0, date_system),
        "lastmonth" => serial_in_relative_month(value_day, now_day, -1, date_system),
        "nextmonth" => serial_in_relative_month(value_day, now_day, 1, date_system),
        _ => None,
    }
}

fn serial_in_relative_week(
    value_day: i64,
    now_day: i64,
    week_offset: i64,
    date_system: WorkbookDateSystem,
) -> bool {
    let current_week_start = now_day - (now_day + sunday_anchor_offset(date_system)).rem_euclid(7);
    let target_week_start = current_week_start + week_offset * 7;
    (target_week_start..=target_week_start + 6).contains(&value_day)
}

fn sunday_anchor_offset(date_system: WorkbookDateSystem) -> i64 {
    match date_system {
        WorkbookDateSystem::System1900 => 0,
        WorkbookDateSystem::System1904 => 5,
    }
}

fn serial_in_relative_month(
    value_day: i64,
    now_day: i64,
    month_offset: i64,
    date_system: WorkbookDateSystem,
) -> Option<bool> {
    let (now_year, now_month, _) = ymd_from_excel_serial(date_system, now_day as f64)?;
    let (target_year, target_month) = add_months(now_year, now_month, month_offset);
    let (value_year, value_month, _) = ymd_from_excel_serial(date_system, value_day as f64)?;
    Some(value_year == target_year && value_month == target_month)
}

fn add_months(year: i64, month: i64, month_offset: i64) -> (i64, i64) {
    let zero_based = year * 12 + (month - 1) + month_offset;
    (zero_based.div_euclid(12), zero_based.rem_euclid(12) + 1)
}

fn evaluate_operator_rule(
    operator: &str,
    thresholds: &[String],
    value: &EvalValue,
    visible_value_text: &str,
    locale_ctx: Option<&LocaleFormatContext<'_>>,
) -> Option<bool> {
    let normalized = normalized_token(operator);
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
