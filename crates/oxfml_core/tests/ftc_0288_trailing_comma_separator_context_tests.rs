use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::{oxfml_en_us_format_profile, oxfml_en_us_locale_context};
use oxfml_core::publication::VerificationPublicationContext;
use oxfml_core::{FormulaChannelKind, FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::locale_format::{FormatCodeTokenPolicy, FormatProfile, LocaleFormatContext};
use oxfunc_core::value::{EvalValue, ExcelText};

fn en_us_profile_with_separators(
    decimal_separator: &'static str,
    thousands_separator: &'static str,
) -> FormatProfile {
    FormatProfile {
        decimal_separator,
        thousands_separator,
        format_code_decimal_token: ".",
        format_code_group_token: if thousands_separator == "," {
            ","
        } else {
            "\u{00A0}"
        },
        format_code_token_policy: FormatCodeTokenPolicy::LocalizedExcel,
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

fn evaluate_text_formula(
    formula_stable_id: &str,
    formula: &str,
    locale_ctx: &LocaleFormatContext<'_>,
) -> oxfml_core::EvaluationOutput {
    let compiled = common::compile_formula(
        formula_stable_id,
        formula,
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
fn evaluator_characterizes_trailing_comma_scaling_patterns_across_separator_contexts() {
    let contexts = [
        ("comma-thousands", en_us_context_with_separators(".", ",")),
        (
            "nbsp-thousands",
            en_us_context_with_separators(".", "\u{00A0}"),
        ),
    ];
    let cases = [
        (
            "hash-double-comma-small",
            "=TEXT(1234,\"#,,\")",
            ["0", "1234,,"],
        ),
        (
            "hash-double-comma-mid",
            "=TEXT(12345,\"#,,\")",
            ["0", "12345,,"],
        ),
        (
            "hash-double-comma-large",
            "=TEXT(1234567.89,\"#,,\")",
            ["1", "1234568,,"],
        ),
        (
            "zero-double-comma-small",
            "=TEXT(1234,\"0,,\")",
            ["0", "1234,,"],
        ),
        (
            "zero-double-comma-mid",
            "=TEXT(12345,\"0,,\")",
            ["0", "12345,,"],
        ),
        (
            "zero-double-comma-large",
            "=TEXT(1234567.89,\"0,,\")",
            ["1", "1234568,,"],
        ),
        (
            "grouped-double-comma-small",
            "=TEXT(1234,\"#,##0,,\")",
            ["0", "1,234,,"],
        ),
        (
            "grouped-double-comma-mid",
            "=TEXT(12345,\"#,##0,,\")",
            ["0", "12,345,,"],
        ),
        (
            "grouped-double-comma-large",
            "=TEXT(1234567.89,\"#,##0,,\")",
            ["1", "1234,568,,"],
        ),
    ];

    for (context_index, (context_id, locale)) in contexts.iter().enumerate() {
        for (case_id, formula, expected_texts) in cases {
            let expected_text = expected_texts[context_index];
            let output =
                evaluate_text_formula(&format!("ftc-0288:{context_id}:{case_id}"), formula, locale);
            assert_eq!(
                output.oxfunc_value,
                text_eval_value(expected_text),
                "{context_id}/{case_id} oxfunc_value"
            );
            assert_eq!(
                output.result.payload_summary,
                format!("Text({expected_text})"),
                "{context_id}/{case_id} payload_summary"
            );
            assert_eq!(
                output.result.format_hint.as_deref(),
                Some("locale_format_semantics"),
                "{context_id}/{case_id} format_hint"
            );
        }
    }
}

#[test]
fn runtime_characterizes_trailing_comma_scaling_patterns_across_separator_contexts() {
    let verification_context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: None,
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: Vec::new(),
    };
    let contexts = [
        ("comma-thousands", en_us_context_with_separators(".", ",")),
        (
            "nbsp-thousands",
            en_us_context_with_separators(".", "\u{00A0}"),
        ),
    ];
    let cases = [
        (
            "hash-double-comma-small",
            "=TEXT(1234,\"#,,\")",
            ["0", "1234,,"],
        ),
        (
            "hash-double-comma-mid",
            "=TEXT(12345,\"#,,\")",
            ["0", "12345,,"],
        ),
        (
            "hash-double-comma-large",
            "=TEXT(1234567.89,\"#,,\")",
            ["1", "1234568,,"],
        ),
        (
            "zero-double-comma-small",
            "=TEXT(1234,\"0,,\")",
            ["0", "1234,,"],
        ),
        (
            "zero-double-comma-mid",
            "=TEXT(12345,\"0,,\")",
            ["0", "12345,,"],
        ),
        (
            "zero-double-comma-large",
            "=TEXT(1234567.89,\"0,,\")",
            ["1", "1234568,,"],
        ),
        (
            "grouped-double-comma-small",
            "=TEXT(1234,\"#,##0,,\")",
            ["0", "1,234,,"],
        ),
        (
            "grouped-double-comma-mid",
            "=TEXT(12345,\"#,##0,,\")",
            ["0", "12,345,,"],
        ),
        (
            "grouped-double-comma-large",
            "=TEXT(1234567.89,\"#,##0,,\")",
            ["1", "1234,568,,"],
        ),
    ];

    for (context_index, (context_id, locale)) in contexts.iter().enumerate() {
        for (case_id, formula, expected_texts) in cases {
            let expected_text = expected_texts[context_index];
            let expected_value = text_eval_value(expected_text);
            let result = RuntimeEnvironment::new()
                .execute(
                    RuntimeFormulaRequest::new(
                        FormulaSourceRecord::new(
                            &format!("runtime:ftc-0288:{context_id}:{case_id}"),
                            1,
                            formula,
                        )
                        .with_formula_channel_kind(FormulaChannelKind::WorksheetA1),
                        TypedContextQueryBundle::new(
                            None,
                            None,
                            Some(locale),
                            Some(46000.0),
                            Some(0.25),
                        ),
                    )
                    .with_verification_publication_context(verification_context.clone()),
                )
                .unwrap_or_else(|error| {
                    panic!("{context_id}/{case_id} runtime execution should succeed: {error}")
                });

            assert_eq!(
                result.published_worksheet_value, expected_value,
                "{context_id}/{case_id} published_worksheet_value"
            );
            assert_eq!(
                result.verification_publication_surface.published_value, expected_value,
                "{context_id}/{case_id} verification_publication_surface.published_value"
            );
            assert_eq!(
                result.verification_publication_surface.visible_value_text, expected_text,
                "{context_id}/{case_id} visible_value_text"
            );
        }
    }
}
