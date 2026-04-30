use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::{oxfml_en_us_format_profile, oxfml_en_us_locale_context};
use oxfml_core::publication::VerificationPublicationContext;
use oxfml_core::{FormulaChannelKind, FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::locale_format::{FormatProfile, LocaleFormatContext};
use oxfunc_core::value::{EvalValue, ExcelText};

fn en_us_profile_with_separators(
    decimal_separator: &'static str,
    thousands_separator: &'static str,
) -> FormatProfile {
    FormatProfile {
        decimal_separator,
        thousands_separator,
        ..oxfml_en_us_format_profile()
    }
}

fn en_us_context_with_separators(
    decimal_separator: &'static str,
    thousands_separator: &'static str,
) -> LocaleFormatContext<'static> {
    let base = oxfml_en_us_locale_context();
    LocaleFormatContext {
        profile: en_us_profile_with_separators(decimal_separator, thousands_separator),
        ..base
    }
}

fn evaluate_text_formula(locale_ctx: &LocaleFormatContext<'_>) -> oxfml_core::EvaluationOutput {
    let compiled = common::compile_formula(
        "ftc-0288-separator-context",
        "=TEXT(1234567.89,\"#,##0.00\")",
        BTreeMap::new(),
        "eval-struct-v1",
        "oxfunc:test",
    );

    let mut context = EvaluationContext::new(&compiled.bound_formula, &compiled.semantic_plan);
    context.apply_typed_context_query_bundle(TypedContextQueryBundle::new(
        None,
        None,
        Some(locale_ctx),
        Some(46000.0),
        Some(0.25),
    ));

    evaluate_formula(context).expect("evaluation should succeed")
}

fn text_eval_value(text: &str) -> EvalValue {
    EvalValue::Text(ExcelText::from_interop_assignment(text))
}

#[test]
fn evaluator_respects_separator_context_for_text_grouping_ftc_0288() {
    let cases = [
        (
            "forced-comma-thousands",
            en_us_context_with_separators(".", ","),
            "1,234,567.89",
        ),
        (
            "nbsp-thousands",
            en_us_context_with_separators(".", "\u{00A0}"),
            "1234,567.89",
        ),
    ];

    for (case_id, locale, expected_text) in cases {
        let output = evaluate_text_formula(&locale);
        assert_eq!(
            output.oxfunc_value,
            text_eval_value(expected_text),
            "{case_id} oxfunc_value"
        );
        assert_eq!(
            output.result.payload_summary,
            format!("Text({expected_text})"),
            "{case_id} payload_summary"
        );
        assert_eq!(
            output.result.format_hint.as_deref(),
            Some("locale_format_semantics"),
            "{case_id} format_hint"
        );
    }
}

#[test]
fn runtime_environment_respects_separator_context_for_text_grouping_ftc_0288() {
    let verification_context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: None,
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: Vec::new(),
    };
    let cases = [
        (
            "forced-comma-thousands",
            en_us_context_with_separators(".", ","),
            "1,234,567.89",
        ),
        (
            "nbsp-thousands",
            en_us_context_with_separators(".", "\u{00A0}"),
            "1234,567.89",
        ),
    ];

    for (case_id, locale, expected_text) in cases {
        let expected_value = text_eval_value(expected_text);
        let result = RuntimeEnvironment::new()
            .execute(
                RuntimeFormulaRequest::new(
                    FormulaSourceRecord::new(
                        &format!("runtime:ftc-0288:{case_id}"),
                        1,
                        "=TEXT(1234567.89,\"#,##0.00\")",
                    )
                    .with_formula_channel_kind(FormulaChannelKind::WorksheetA1),
                    TypedContextQueryBundle::new(
                        None,
                        None,
                        Some(&locale),
                        Some(46000.0),
                        Some(0.25),
                    ),
                )
                .with_verification_publication_context(verification_context.clone()),
            )
            .unwrap_or_else(|error| panic!("{case_id} runtime execution should succeed: {error}"));

        assert_eq!(
            result.published_worksheet_value, expected_value,
            "{case_id} published_worksheet_value"
        );
        assert_eq!(
            result.verification_publication_surface.published_value, expected_value,
            "{case_id} verification_publication_surface.published_value"
        );
        assert_eq!(
            result.verification_publication_surface.visible_value_text, expected_text,
            "{case_id} visible_value_text"
        );
    }
}
