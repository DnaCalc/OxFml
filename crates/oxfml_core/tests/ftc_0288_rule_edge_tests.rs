use oxfunc_core::value::CalcValue;
use std::collections::BTreeMap;

mod common;

use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::eval::{EvaluationContext, evaluate_formula};
use oxfml_core::format::{
    oxfml_en_us_format_profile, oxfml_en_us_locale_context, render_with_code,
};
use oxfml_core::publication::VerificationPublicationContext;
use oxfml_core::{FormulaChannelKind, FormulaSourceRecord, TypedContextQueryBundle};
use oxfunc_core::locale_format::{FormatCodeTokenPolicy, FormatProfile, LocaleFormatContext};
use oxfunc_core::value::ExcelText;

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

fn evaluate_formula_with_locale(
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
        Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
    ));

    evaluate_formula(context).expect("evaluation should succeed")
}

fn text_eval_value(text: &str) -> CalcValue {
    CalcValue::text(ExcelText::from_interop_assignment(text))
}

#[test]
fn evaluator_characterizes_ftc_0288_rule_edges() {
    let contexts = [
        ("comma-thousands", en_us_context_with_separators(".", ",")),
        (
            "nbsp-thousands",
            en_us_context_with_separators(".", "\u{00A0}"),
        ),
        ("decimal-comma", en_us_context_with_separators(",", ".")),
    ];
    let cases = [
        (
            "hash-triple-comma",
            "=TEXT(123456789,\"#,,,\")",
            ["0", "123456789,,,", "123456789,,,"],
        ),
        (
            "zero-triple-comma",
            "=TEXT(123456789,\"0,,,\")",
            ["0", "123456789,,,", "123456789,,,"],
        ),
        (
            "grouped-triple-comma",
            "=TEXT(123456789,\"#,##0,,,\")",
            ["0", "123456,789,,,", "123456,789,,,"],
        ),
        (
            "hash-decimal-zero-comma",
            "=TEXT(1234.5,\"#.0,\")",
            ["1234.5", "1234.5", "1234,5"],
        ),
        (
            "hash-decimal-double-hash",
            "=TEXT(1234567.89,\"#.##\")",
            ["1234567.89", "1234567.89", "1234567,89"],
        ),
    ];

    for (context_index, (context_id, locale)) in contexts.iter().enumerate() {
        for (case_id, formula, expected_texts) in cases {
            let expected_text = expected_texts[context_index];
            let output = evaluate_formula_with_locale(
                &format!("ftc-0288:{context_id}:{case_id}"),
                formula,
                locale,
            );
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
fn formatter_characterizes_ftc_0288_decimal_scaling_suffix_edges() {
    let contexts = [
        ("comma-thousands", en_us_context_with_separators(".", ",")),
        (
            "nbsp-thousands",
            en_us_context_with_separators(".", "\u{00A0}"),
        ),
        ("decimal-comma", en_us_context_with_separators(",", ".")),
    ];

    for (context_id, locale) in contexts {
        let expected_text = match context_id {
            "comma-thousands" => "1234567.9M",
            "nbsp-thousands" => "1234567.9M",
            "decimal-comma" => "1234567,9M",
            _ => unreachable!(),
        };
        assert_eq!(
            render_with_code(
                &locale.profile,
                locale.date_system,
                1234567.89,
                "0.0,,\"M\""
            ),
            Ok(expected_text.to_string()),
            "{context_id} formatter output"
        );
    }
}

#[test]
fn runtime_characterizes_ftc_0288_rule_edges() {
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
        ("decimal-comma", en_us_context_with_separators(",", ".")),
    ];
    let cases = [
        (
            "hash-triple-comma",
            "=TEXT(123456789,\"#,,,\")",
            ["0", "123456789,,,", "123456789,,,"],
        ),
        (
            "zero-triple-comma",
            "=TEXT(123456789,\"0,,,\")",
            ["0", "123456789,,,", "123456789,,,"],
        ),
        (
            "grouped-triple-comma",
            "=TEXT(123456789,\"#,##0,,,\")",
            ["0", "123456,789,,,", "123456,789,,,"],
        ),
        (
            "hash-decimal-zero-comma",
            "=TEXT(1234.5,\"#.0,\")",
            ["1234.5", "1234.5", "1234,5"],
        ),
        (
            "hash-decimal-double-hash",
            "=TEXT(1234567.89,\"#.##\")",
            ["1234567.89", "1234567.89", "1234567,89"],
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
                            format!("runtime:ftc-0288:{context_id}:{case_id}"),
                            1,
                            formula,
                        )
                        .with_formula_channel_kind(FormulaChannelKind::WorksheetA1),
                        TypedContextQueryBundle::new(
                            None,
                            None,
                            Some(locale),
                            Some(46000.0),
                            Some(&oxfml_core::test_support::random::FIXED_RANDOM_PROVIDER_025),
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
