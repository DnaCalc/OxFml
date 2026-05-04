use oxfml_core::format::oxfml_en_us_locale_context;
use oxfml_core::{
    AverageRuleOptions, ColorScaleRuleOptions, ColorScaleRuleStop, ConditionalFormattingRank,
    ConditionalFormattingThreshold, ConditionalFormattingTypedRule, DataBarDirection,
    DataBarRuleOptions, FormulaSourceRecord, IconSetRuleOptions, RankRuleOptions,
    ReturnedValueSurface, TopologyDelta, VerificationConditionalFormattingRule,
    VerificationPublicationContext, VerificationPublicationSurface,
    build_verification_publication_surface,
};
use oxfunc_core::value::{
    ArrayCellValue, EvalArray, EvalValue, ExcelText, ExtendedValue, WorksheetErrorCode,
};

fn surface_for_array(
    rows: Vec<Vec<ArrayCellValue>>,
    rules: Vec<VerificationConditionalFormattingRule>,
    now_serial: Option<f64>,
) -> VerificationPublicationSurface {
    let value = EvalValue::Array(EvalArray::from_rows(rows).expect("rectangular array"));
    surface_for_value(value, rules, now_serial)
}

fn surface_for_value(
    value: EvalValue,
    rules: Vec<VerificationConditionalFormattingRule>,
    now_serial: Option<f64>,
) -> VerificationPublicationSurface {
    let locale = oxfml_en_us_locale_context();
    let source = FormulaSourceRecord::new("cf-array", 1, "=A1#");
    let returned_value_surface =
        ReturnedValueSurface::from_extended_value(&ExtendedValue::Core(value.clone()));
    let topology_delta = TopologyDelta {
        formula_stable_id: "cf-array".to_string(),
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

fn rule(
    kind: &str,
    operator: Option<&str>,
    thresholds: Vec<&str>,
    fill_color: &str,
) -> VerificationConditionalFormattingRule {
    VerificationConditionalFormattingRule {
        target_ranges: vec!["A1:C2".to_string()],
        rule_kind: kind.to_string(),
        operator: operator.map(std::string::ToString::to_string),
        thresholds: thresholds
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        typed_rule: None,
        font_color: None,
        fill_color: Some(fill_color.to_string()),
        effective_display_text: None,
        applies: None,
        effective_font_color: None,
        effective_fill_color: None,
    }
}

fn font_rule(
    kind: &str,
    operator: Option<&str>,
    thresholds: Vec<&str>,
    font_color: &str,
) -> VerificationConditionalFormattingRule {
    VerificationConditionalFormattingRule {
        target_ranges: vec!["A1:C2".to_string()],
        rule_kind: kind.to_string(),
        operator: operator.map(std::string::ToString::to_string),
        thresholds: thresholds
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        typed_rule: None,
        font_color: Some(font_color.to_string()),
        fill_color: None,
        effective_display_text: None,
        applies: None,
        effective_font_color: None,
        effective_fill_color: None,
    }
}

fn typed_rule(
    kind: &str,
    typed_rule: ConditionalFormattingTypedRule,
    fill_color: &str,
) -> VerificationConditionalFormattingRule {
    let mut rule = rule(kind, None, Vec::new(), fill_color);
    rule.typed_rule = Some(typed_rule);
    rule
}

#[test]
fn array_cell_value_rule_applies_per_cell() {
    let surface = surface_for_array(
        vec![
            vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Number(3.0),
            ],
            vec![
                ArrayCellValue::Number(4.0),
                ArrayCellValue::Number(5.0),
                ArrayCellValue::Number(6.0),
            ],
        ],
        vec![rule(
            "cell_value",
            Some("greaterThan"),
            vec!["3"],
            "#E6F2D9",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(grid.rows[0][0].effective_fill_color, None);
    assert_eq!(grid.rows[0][1].effective_fill_color, None);
    assert_eq!(grid.rows[0][2].effective_fill_color, None);
    assert_eq!(
        grid.rows[1]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("#E6F2D9"), Some("#E6F2D9"), Some("#E6F2D9")]
    );
}

#[test]
fn array_error_and_blank_predicates_apply_per_cell() {
    let error_surface = surface_for_array(
        vec![
            vec![
                ArrayCellValue::Number(1.0),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
                ArrayCellValue::Error(WorksheetErrorCode::Div0),
            ],
            vec![
                ArrayCellValue::Number(2.0),
                ArrayCellValue::Text(ExcelText::from_interop_assignment("y")),
                ArrayCellValue::Error(WorksheetErrorCode::NA),
            ],
        ],
        vec![rule("errors", None, Vec::new(), "#FFE1E1")],
        None,
    );
    let error_grid = error_surface.array_cell_format.expect("array cell format");
    assert_eq!(error_grid.rows[0][0].effective_fill_color, None);
    assert_eq!(
        error_grid.rows[0][2].effective_fill_color.as_deref(),
        Some("#FFE1E1")
    );
    assert_eq!(
        error_grid.rows[1][2].effective_fill_color.as_deref(),
        Some("#FFE1E1")
    );

    let blank_surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::EmptyCell,
            ArrayCellValue::Text(ExcelText::from_interop_assignment("")),
        ]],
        vec![rule("blanks", None, Vec::new(), "#FFF2CC")],
        None,
    );
    let blank_grid = blank_surface.array_cell_format.expect("array cell format");
    assert_eq!(blank_grid.rows[0][0].effective_fill_color, None);
    assert_eq!(
        blank_grid.rows[0][1].effective_fill_color.as_deref(),
        Some("#FFF2CC")
    );
    assert_eq!(
        blank_grid.rows[0][2].effective_fill_color.as_deref(),
        Some("#FFF2CC")
    );
}

#[test]
fn array_relative_date_predicates_use_shared_now_serial_per_cell() {
    let surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(46044.0),
            ArrayCellValue::Number(46045.0),
            ArrayCellValue::Number(46046.0),
        ]],
        vec![rule("dates", None, vec!["today"], "#D9EAD3")],
        Some(46045.0),
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(grid.rows[0][0].effective_fill_color, None);
    assert_eq!(
        grid.rows[0][1].effective_fill_color.as_deref(),
        Some("#D9EAD3")
    );
    assert_eq!(grid.rows[0][2].effective_fill_color, None);
}

#[test]
fn one_by_one_array_cell_format_matches_whole_cell_cf_fields() {
    let surface = surface_for_array(
        vec![vec![ArrayCellValue::Number(4.0)]],
        vec![rule(
            "cell_value",
            Some("greaterThan"),
            vec!["3"],
            "#E6F2D9",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0][0].effective_display_text,
        surface.effective_display_text
    );
    assert_eq!(
        grid.rows[0][0].effective_font_color,
        surface.effective_font_color
    );
    assert_eq!(
        grid.rows[0][0].effective_fill_color,
        surface.effective_fill_color
    );
}

#[test]
fn array_above_and_below_average_rules_use_numeric_aggregate_context() {
    let above_surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(5.0),
        ]],
        vec![rule("aboveAverage", None, Vec::new(), "#D9EAD3")],
        None,
    );
    let above_grid = above_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        above_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![None, None, None, Some("#D9EAD3"), Some("#D9EAD3")]
    );

    let below_surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(5.0),
        ]],
        vec![rule("belowAverage", None, Vec::new(), "#F4CCCC")],
        None,
    );
    let below_grid = below_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        below_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("#F4CCCC"), Some("#F4CCCC"), None, None, None]
    );
}

#[test]
fn array_top_and_bottom_rules_use_ranked_numeric_context() {
    let values = (1..=10)
        .map(|number| ArrayCellValue::Number(number as f64))
        .collect::<Vec<_>>();

    let top_surface = surface_for_array(
        vec![values.clone()],
        vec![rule("top", None, vec!["5"], "#D9EAD3")],
        None,
    );
    let top_grid = top_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        top_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            None,
            None,
            None,
            None,
            None,
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3")
        ]
    );

    let bottom_surface = surface_for_array(
        vec![values],
        vec![rule("bottom", None, vec!["20%"], "#F4CCCC")],
        None,
    );
    let bottom_grid = bottom_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        bottom_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("#F4CCCC"),
            Some("#F4CCCC"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None
        ]
    );
}

#[test]
fn array_unique_and_duplicate_rules_use_visible_value_counts() {
    let unique_surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(3.0),
        ]],
        vec![rule("uniqueValues", None, Vec::new(), "#D9EAD3")],
        None,
    );
    let unique_grid = unique_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        unique_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![None, Some("#D9EAD3"), None, Some("#D9EAD3")]
    );

    let duplicate_surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
            ArrayCellValue::Text(ExcelText::from_interop_assignment("y")),
            ArrayCellValue::Text(ExcelText::from_interop_assignment("x")),
        ]],
        vec![rule("duplicateValues", None, Vec::new(), "#F4CCCC")],
        None,
    );
    let duplicate_grid = duplicate_surface
        .array_cell_format
        .expect("array cell format");
    assert_eq!(
        duplicate_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("#F4CCCC"), None, Some("#F4CCCC")]
    );
}

#[test]
fn array_color_scale_interpolates_fill_colors_from_threshold_stops() {
    let values = (1..=5)
        .map(|number| ArrayCellValue::Number(number as f64))
        .collect::<Vec<_>>();
    let surface = surface_for_array(
        vec![values],
        vec![rule(
            "colorScale",
            None,
            vec!["min:#F8696B", "mid:#FFEB84", "max:#63BE7B"],
            "#000000",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("#F8696B"),
            Some("#FCAA78"),
            Some("#FFEB84"),
            Some("#B1D580"),
            Some("#63BE7B")
        ]
    );
}

#[test]
fn array_data_bar_uses_min_max_fill_ratios() {
    let surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
            ArrayCellValue::Number(40.0),
        ]],
        vec![rule("dataBar", None, Vec::new(), "#638EC6")],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    let ratios = grid.rows[0]
        .iter()
        .map(|cell| {
            let data_bar = cell.data_bar.as_ref().expect("data bar");
            (data_bar.fill_ratio * 1000.0).round() / 1000.0
        })
        .collect::<Vec<_>>();
    assert_eq!(ratios, vec![0.0, 0.333, 0.667, 1.0]);
    assert!(grid.rows[0].iter().all(|cell| cell.icon.is_none()));
}

#[test]
fn array_icon_set_assigns_default_three_bin_indexes() {
    let values = (1..=6)
        .map(|number| ArrayCellValue::Number(number as f64))
        .collect::<Vec<_>>();
    let surface = surface_for_array(
        vec![values],
        vec![rule("iconSet", None, vec!["3Arrows"], "#000000")],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0]
            .iter()
            .map(|cell| cell.icon.as_ref().map(|icon| icon.icon_index))
            .collect::<Vec<_>>(),
        vec![Some(0), Some(0), Some(1), Some(1), Some(2), Some(2)]
    );
    assert!(grid.rows[0].iter().all(|cell| {
        cell.icon
            .as_ref()
            .is_some_and(|icon| icon.set_kind == "3Arrows")
    }));
}

#[test]
fn array_visualization_and_scalar_rules_preserve_per_field_priority() {
    let values = (1..=6)
        .map(|number| ArrayCellValue::Number(number as f64))
        .collect::<Vec<_>>();
    let surface = surface_for_array(
        vec![values],
        vec![
            rule(
                "colorScale",
                None,
                vec!["min:#F8696B", "max:#63BE7B"],
                "#000000",
            ),
            font_rule("cell_value", Some("greaterThan"), vec!["5"], "#FF0000"),
        ],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0][5].effective_font_color.as_deref(),
        Some("#FF0000")
    );
    assert_eq!(
        grid.rows[0][5].effective_fill_color.as_deref(),
        Some("#63BE7B")
    );
    assert_eq!(grid.rows[0][4].effective_font_color, None);
    assert!(grid.rows[0][4].effective_fill_color.is_some());
}

#[test]
fn scalar_visualization_rule_populates_single_cell_carrier() {
    let surface = surface_for_value(
        EvalValue::Number(42.0),
        vec![rule(
            "colorScale",
            None,
            vec!["min:#F8696B", "max:#63BE7B"],
            "#000000",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("scalar cell format");
    assert_eq!(grid.rows.len(), 1);
    assert_eq!(grid.rows[0].len(), 1);
    assert_eq!(
        grid.rows[0][0].effective_fill_color.as_deref(),
        Some("#AE9473")
    );
}

#[test]
fn array_data_bar_respects_explicit_min_max_thresholds() {
    let surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
        ]],
        vec![rule(
            "dataBar",
            None,
            vec!["min:0", "max:40", "showBarOnly", "direction:right"],
            "#638EC6",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    let data_bars = grid.rows[0]
        .iter()
        .map(|cell| cell.data_bar.as_ref().expect("data bar"))
        .collect::<Vec<_>>();
    assert_eq!(
        data_bars
            .iter()
            .map(|bar| bar.fill_ratio)
            .collect::<Vec<_>>(),
        vec![0.25, 0.5, 0.75]
    );
    assert!(data_bars.iter().all(|bar| bar.show_bar_only));
    assert!(
        data_bars
            .iter()
            .all(|bar| bar.direction == oxfml_core::DataBarDirection::Right)
    );
}

#[test]
fn array_icon_set_respects_explicit_numeric_thresholds() {
    let surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
            ArrayCellValue::Number(40.0),
        ]],
        vec![rule(
            "iconSet",
            None,
            vec!["3Arrows", "num:20", "num:30"],
            "#000000",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0]
            .iter()
            .map(|cell| cell.icon.as_ref().map(|icon| icon.icon_index))
            .collect::<Vec<_>>(),
        vec![Some(0), Some(1), Some(2), Some(2)]
    );
}

#[test]
fn array_average_rules_respect_equal_and_stddev_thresholds() {
    let values = vec![vec![
        ArrayCellValue::Number(1.0),
        ArrayCellValue::Number(2.0),
        ArrayCellValue::Number(3.0),
        ArrayCellValue::Number(4.0),
        ArrayCellValue::Number(5.0),
    ]];

    let equal_surface = surface_for_array(
        values.clone(),
        vec![rule("aboveAverage", None, vec!["equal"], "#D9EAD3")],
        None,
    );
    let equal_grid = equal_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        equal_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            None,
            None,
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3")
        ]
    );

    let stddev_surface = surface_for_array(
        values,
        vec![rule("aboveAverage", None, vec!["stddev:1"], "#D9EAD3")],
        None,
    );
    let stddev_grid = stddev_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        stddev_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![None, None, None, None, Some("#D9EAD3")]
    );
}

#[test]
fn typed_color_scale_payload_matches_bounded_threshold_convention() {
    let values = (1..=5)
        .map(|number| ArrayCellValue::Number(number as f64))
        .collect::<Vec<_>>();
    let surface = surface_for_array(
        vec![values],
        vec![typed_rule(
            "colorScale",
            ConditionalFormattingTypedRule {
                color_scale: Some(ColorScaleRuleOptions {
                    stops: vec![
                        ColorScaleRuleStop {
                            position: ConditionalFormattingThreshold::Min,
                            color: "#F8696B".to_string(),
                        },
                        ColorScaleRuleStop {
                            position: ConditionalFormattingThreshold::Mid,
                            color: "#FFEB84".to_string(),
                        },
                        ColorScaleRuleStop {
                            position: ConditionalFormattingThreshold::Max,
                            color: "#63BE7B".to_string(),
                        },
                    ],
                }),
                ..Default::default()
            },
            "#000000",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("#F8696B"),
            Some("#FCAA78"),
            Some("#FFEB84"),
            Some("#B1D580"),
            Some("#63BE7B")
        ]
    );
}

#[test]
fn typed_data_bar_payload_controls_bounds_direction_and_bar_only() {
    let surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
        ]],
        vec![typed_rule(
            "dataBar",
            ConditionalFormattingTypedRule {
                data_bar: Some(DataBarRuleOptions {
                    minimum: Some(ConditionalFormattingThreshold::Number(0.0)),
                    maximum: Some(ConditionalFormattingThreshold::Number(40.0)),
                    bar_color: Some("#638EC6".to_string()),
                    direction: Some(DataBarDirection::Right),
                    show_bar_only: true,
                }),
                ..Default::default()
            },
            "#000000",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    let data_bars = grid.rows[0]
        .iter()
        .map(|cell| cell.data_bar.as_ref().expect("data bar"))
        .collect::<Vec<_>>();
    assert_eq!(
        data_bars
            .iter()
            .map(|bar| bar.fill_ratio)
            .collect::<Vec<_>>(),
        vec![0.25, 0.5, 0.75]
    );
    assert!(
        data_bars
            .iter()
            .all(|bar| bar.direction == DataBarDirection::Right)
    );
    assert!(data_bars.iter().all(|bar| bar.show_bar_only));
    assert!(data_bars.iter().all(|bar| bar.bar_color == "#638EC6"));
}

#[test]
fn typed_icon_set_payload_uses_explicit_thresholds() {
    let surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(10.0),
            ArrayCellValue::Number(20.0),
            ArrayCellValue::Number(30.0),
            ArrayCellValue::Number(40.0),
        ]],
        vec![typed_rule(
            "iconSet",
            ConditionalFormattingTypedRule {
                icon_set: Some(IconSetRuleOptions {
                    set_kind: "3Arrows".to_string(),
                    thresholds: vec![
                        ConditionalFormattingThreshold::Number(20.0),
                        ConditionalFormattingThreshold::Number(30.0),
                    ],
                }),
                ..Default::default()
            },
            "#000000",
        )],
        None,
    );

    let grid = surface.array_cell_format.expect("array cell format");
    assert_eq!(
        grid.rows[0]
            .iter()
            .map(|cell| cell.icon.as_ref().map(|icon| icon.icon_index))
            .collect::<Vec<_>>(),
        vec![Some(0), Some(1), Some(2), Some(2)]
    );
}

#[test]
fn typed_rank_and_average_payloads_replace_threshold_parsing() {
    let top_values = (1..=10)
        .map(|number| ArrayCellValue::Number(number as f64))
        .collect::<Vec<_>>();
    let top_surface = surface_for_array(
        vec![top_values],
        vec![typed_rule(
            "top",
            ConditionalFormattingTypedRule {
                rank: Some(RankRuleOptions {
                    rank: ConditionalFormattingRank::Count(5),
                }),
                ..Default::default()
            },
            "#D9EAD3",
        )],
        None,
    );
    let top_grid = top_surface.array_cell_format.expect("array cell format");
    assert_eq!(
        top_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            None,
            None,
            None,
            None,
            None,
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3")
        ]
    );

    let average_surface = surface_for_array(
        vec![vec![
            ArrayCellValue::Number(1.0),
            ArrayCellValue::Number(2.0),
            ArrayCellValue::Number(3.0),
            ArrayCellValue::Number(4.0),
            ArrayCellValue::Number(5.0),
        ]],
        vec![typed_rule(
            "aboveAverage",
            ConditionalFormattingTypedRule {
                average: Some(AverageRuleOptions {
                    include_equal: true,
                    stddev_multiplier: None,
                }),
                ..Default::default()
            },
            "#D9EAD3",
        )],
        None,
    );
    let average_grid = average_surface
        .array_cell_format
        .expect("array cell format");
    assert_eq!(
        average_grid.rows[0]
            .iter()
            .map(|cell| cell.effective_fill_color.as_deref())
            .collect::<Vec<_>>(),
        vec![
            None,
            None,
            Some("#D9EAD3"),
            Some("#D9EAD3"),
            Some("#D9EAD3")
        ]
    );
}
