use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
use oxfml_core::format::{oxfml_en_us_locale_context, render_with_code};
use oxfml_core::publication::{
    VerificationPublicationContext, build_verification_publication_surface,
};
use oxfml_core::seam::TopologyDelta;
use oxfml_core::{FormulaSourceRecord, ReturnedValueSurface, TypedContextQueryBundle};
use oxfunc_core::value::{CalcValue, CoreValue, FunctionValue, NumberFormatHint, PresentationHint};

#[test]
fn format_engine_renders_time_tokens_and_ampm_modes() {
    let locale = oxfml_en_us_locale_context();
    let cases = [
        (0.625, "h:mm", "15:00"),
        (0.625, "hh:mm", "15:00"),
        (0.625, "HH:mm:ss", "15:00:00"),
        (0.625, "h:mm AM/PM", "3:00 PM"),
        (0.0, "h:mm AM/PM", "12:00 AM"),
        (0.5, "h:mm AM/PM", "12:00 PM"),
    ];

    for (value, code, expected) in cases {
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, value, code),
            Ok(expected.to_string()),
            "{code}"
        );
    }
}

#[test]
fn format_engine_renders_datetime_composites_and_elapsed_time() {
    let locale = oxfml_en_us_locale_context();

    assert_eq!(
        render_with_code(
            &locale.profile,
            locale.date_system,
            45293.625,
            "yyyy-mm-dd hh:mm:ss",
        ),
        Ok("2024-01-02 15:00:00".to_string())
    );
    assert_eq!(
        render_with_code(&locale.profile, locale.date_system, 1.625, "[h]:mm:ss"),
        Ok("39:00:00".to_string())
    );
    assert_eq!(
        render_with_code(&locale.profile, locale.date_system, 1.625, "[m]:ss"),
        Ok("2340:00".to_string())
    );
    assert_eq!(
        render_with_code(&locale.profile, locale.date_system, 1.625, "[s]"),
        Ok("140400".to_string())
    );
}

#[test]
fn format_engine_renders_simple_fraction_codes() {
    let locale = oxfml_en_us_locale_context();
    let cases = [
        (0.25, "?/?", "1/4"),
        (0.25, "# ?/?", " 1/4"),
        (1.25, "# ?/?", "1 1/4"),
        (1.25, "# ??/??", "1  1/ 4"),
        (0.125, "# ??/??", "  1/ 8"),
        (0.25, "0/0", "1/4"),
    ];

    for (value, code, expected) in cases {
        assert_eq!(
            render_with_code(&locale.profile, locale.date_system, value, code),
            Ok(expected.to_string()),
            "{code}"
        );
    }
}

#[test]
fn format_engine_renders_accounting_parentheses_patterns() {
    let locale = oxfml_en_us_locale_context();

    assert_eq!(
        render_with_code(
            &locale.profile,
            locale.date_system,
            -1234.5,
            "$#,##0.00;($#,##0.00)",
        ),
        Ok("($1,234.50)".to_string())
    );
    assert_eq!(
        render_with_code(
            &locale.profile,
            locale.date_system,
            1234.5,
            "$#,##0.00;($#,##0.00)",
        ),
        Ok("$1,234.50".to_string())
    );
    assert_eq!(
        render_with_code(
            &locale.profile,
            locale.date_system,
            -1234.5,
            "_($* #,##0.00_);_($* (#,##0.00);_($* \"-\"??_);_(@_)",
        ),
        Ok(" $ (1,234.50)".to_string())
    );
}

#[test]
fn format_engine_time_fraction_edge_inputs_do_not_panic() {
    let locale = oxfml_en_us_locale_context();

    assert_eq!(
        render_with_code(&locale.profile, locale.date_system, -0.0, "HH:mm:ss"),
        Ok("00:00:00".to_string())
    );
    assert!(render_with_code(&locale.profile, locale.date_system, f64::NAN, "HH:mm:ss").is_err());
    assert!(
        render_with_code(
            &locale.profile,
            locale.date_system,
            1.0e12,
            "yyyy-mm-dd hh:mm:ss"
        )
        .is_ok()
    );
    assert!(render_with_code(&locale.profile, locale.date_system, f64::NAN, "# ?/?").is_err());
}

#[test]
fn publication_surface_respects_user_supplied_time_format_code() {
    let locale = oxfml_en_us_locale_context();
    let source = FormulaSourceRecord::new("publication:time-format", 1, "=NOW()");
    let returned_value_surface =
        ReturnedValueSurface::from_calc_value(&CalcValue::with_presentation(
            CoreValue::Number(45293.625),
            PresentationHint::number_format(NumberFormatHint::DateLike),
        ));
    let context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: Some("HH:mm:ss".to_string()),
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: Vec::new(),
    };

    let surface = build_verification_publication_surface(
        &source,
        &FunctionValue::Number(45293.625),
        &returned_value_surface,
        &TopologyDelta {
            formula_stable_id: "publication:time-format".to_string(),
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

    assert_eq!(surface.effective_display_text, "15:00:00");
}

#[test]
fn runtime_text_uses_fraction_format_code() {
    let locale = oxfml_en_us_locale_context();
    let result = RuntimeEnvironment::new()
        .execute(RuntimeFormulaRequest::new(
            FormulaSourceRecord::new("runtime:fraction-format", 1, "=TEXT(0.25,\"# ?/?\")"),
            TypedContextQueryBundle::new(None, None, Some(&locale), None, None),
        ))
        .expect("runtime execution should succeed");

    assert_eq!(
        result.published_worksheet_value,
        FunctionValue::Text(oxfunc_core::value::ExcelText::from_interop_assignment(
            " 1/4"
        ))
    );
}
