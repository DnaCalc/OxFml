use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::{
    FormulaSourceRecord, ReturnedValueSurface, TopologyDelta,
    VerificationConditionalFormattingRule, VerificationPublicationContext,
    VerificationPublicationSurface, build_verification_publication_surface,
};
use oxfunc_core::value::{CalcValue, ExcelText, FunctionValue, WorksheetErrorCode};

fn surface_for(
    value: FunctionValue,
    rules: Vec<VerificationConditionalFormattingRule>,
    now_serial: Option<f64>,
) -> VerificationPublicationSurface {
    let locale = oxfml_en_us_locale_context();
    let source = FormulaSourceRecord::new("cf-predicate", 1, "=A1");
    let returned_value_surface =
        ReturnedValueSurface::from_calc_value(&CalcValue::from(value.clone()));
    let topology_delta = TopologyDelta {
        formula_stable_id: "cf-predicate".to_string(),
        dependency_additions: Vec::new(),
        dependency_removals: Vec::new(),
        dependency_reclassifications: Vec::new(),
        dependency_consequence_facts: Vec::new(),
        dynamic_reference_facts: Vec::new(),
        spill_facts: Vec::new(),
        format_dependency_facts: Vec::new(),
        capability_effect_facts: Vec::new(),
        candidate_result_id: None,
    };
    let context = VerificationPublicationContext {
        format_profile: Some("en-US".to_string()),
        number_format_code: None,
        style_id: None,
        style_hierarchy: Vec::new(),
        font_color: None,
        fill_color: None,
        conditional_formatting_rules: rules,
    };

    build_verification_publication_surface(
        &source,
        &value,
        &returned_value_surface,
        &topology_delta,
        None,
        None,
        Some(&locale),
        now_serial,
        Some(&context),
    )
}

fn predicate_rule(kind: &str, thresholds: Vec<&str>) -> VerificationConditionalFormattingRule {
    VerificationConditionalFormattingRule {
        target_ranges: vec!["A1".to_string()],
        rule_kind: kind.to_string(),
        operator: None,
        thresholds: thresholds
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        typed_rule: None,
        font_color: Some("#FF0000".to_string()),
        fill_color: None,
        effective_display_text: None,
        applies: None,
        effective_font_color: None,
        effective_fill_color: None,
    }
}

#[test]
fn conditional_formatting_blank_and_error_predicates_publish_applies() {
    let blank_surface = surface_for(
        FunctionValue::Text(ExcelText::from_interop_assignment("")),
        vec![predicate_rule("blanks", Vec::new())],
        None,
    );
    assert_eq!(
        blank_surface.conditional_formatting_applies,
        vec![Some(true)]
    );
    assert_eq!(
        blank_surface.effective_font_color.as_deref(),
        Some("#FF0000")
    );

    let nonblank_surface = surface_for(
        FunctionValue::Number(1.0),
        vec![predicate_rule("noBlanks", Vec::new())],
        None,
    );
    assert_eq!(
        nonblank_surface.conditional_formatting_applies,
        vec![Some(true)]
    );

    let error_surface = surface_for(
        FunctionValue::Error(WorksheetErrorCode::Div0),
        vec![predicate_rule("errors", Vec::new())],
        None,
    );
    assert_eq!(
        error_surface.conditional_formatting_applies,
        vec![Some(true)]
    );

    let no_error_surface = surface_for(
        FunctionValue::Number(1.0),
        vec![predicate_rule("noErrors", Vec::new())],
        None,
    );
    assert_eq!(
        no_error_surface.conditional_formatting_applies,
        vec![Some(true)]
    );
}

#[test]
fn conditional_formatting_relative_date_predicates_use_runtime_now_serial() {
    let now_serial = 46045.5;
    let surface = surface_for(
        FunctionValue::Number(46045.25),
        vec![
            predicate_rule("dates", vec!["today"]),
            predicate_rule("dates", vec!["yesterday"]),
            predicate_rule("dates", vec!["last7Days"]),
            predicate_rule("dates", vec!["thisMonth"]),
        ],
        Some(now_serial),
    );

    assert_eq!(
        surface.conditional_formatting_applies,
        vec![Some(true), Some(false), Some(true), Some(true)]
    );
}

#[test]
fn conditional_formatting_relative_date_predicates_remain_unknown_without_now_serial() {
    let surface = surface_for(
        FunctionValue::Number(46045.0),
        vec![predicate_rule("dates", vec!["today"])],
        None,
    );

    assert_eq!(surface.conditional_formatting_applies, vec![None]);
}

#[test]
fn unknown_conditional_formatting_predicates_stay_unevaluated() {
    let surface = surface_for(
        FunctionValue::Number(1.0),
        vec![predicate_rule("containsBlanks", Vec::new())],
        None,
    );

    assert_eq!(surface.conditional_formatting_applies, vec![None]);
}
