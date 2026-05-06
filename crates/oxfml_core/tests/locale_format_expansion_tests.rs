use oxfml_core::format::{parse_value_text, render_currency, render_with_code};
use oxfunc_core::locale_format::{
    CANONICAL_LOCALE_PROFILE_IDS, LocaleProfileId, WorkbookDateSystem, excel_serial_from_ymd,
    format_profile,
};

fn serial_1900(year: i64, month: i64, day: i64) -> f64 {
    excel_serial_from_ymd(WorkbookDateSystem::System1900, year, month, day).unwrap()
}

#[test]
fn locale_datetime_names_render_from_format_profile() {
    let de = format_profile(LocaleProfileId::DeDe);
    assert_eq!(
        render_with_code(
            &de,
            WorkbookDateSystem::System1900,
            serial_1900(2026, 2, 1),
            "dddd, d. mmmm yyyy",
        ),
        Ok("Sonntag, 1. Februar 2026".to_string())
    );

    let fr = format_profile(LocaleProfileId::FrFr);
    assert_eq!(
        render_with_code(
            &fr,
            WorkbookDateSystem::System1900,
            serial_1900(2026, 1, 1),
            "d mmm yyyy",
        ),
        Ok("1 janv. 2026".to_string())
    );

    let ja = format_profile(LocaleProfileId::JaJp);
    assert_eq!(
        render_with_code(
            &ja,
            WorkbookDateSystem::System1900,
            serial_1900(2026, 5, 4),
            "dddd",
        ),
        Ok("月曜日".to_string())
    );
}

#[test]
fn general_number_rendering_uses_profile_decimal_separator() {
    let de = format_profile(LocaleProfileId::DeDe);
    assert_eq!(
        render_with_code(&de, WorkbookDateSystem::System1900, 1234.5, "General"),
        Ok("1234,5".to_string())
    );
}

#[test]
fn format_code_tokens_are_profile_owned_invariant_excel_tokens() {
    let de = format_profile(LocaleProfileId::DeDe);
    assert_eq!(
        render_with_code(&de, WorkbookDateSystem::System1900, 1234.5, "#,##0.00"),
        Ok("1.234,50".to_string())
    );

    let en_us = format_profile(LocaleProfileId::EnUs);
    assert_eq!(
        render_with_code(
            &en_us,
            WorkbookDateSystem::System1900,
            1234.5,
            "[$-0407]#,##0.00",
        ),
        Ok("1.234,50".to_string())
    );
}

#[test]
fn locale_prefix_format_code_uses_oxfunc_lcid_mapping() {
    let en_us = format_profile(LocaleProfileId::EnUs);
    assert_eq!(
        render_with_code(
            &en_us,
            WorkbookDateSystem::System1900,
            serial_1900(2026, 1, 1),
            "[$-040C]d mmmm yyyy",
        ),
        Ok("1 janvier 2026".to_string())
    );
    assert_eq!(
        render_with_code(
            &en_us,
            WorkbookDateSystem::System1900,
            serial_1900(2026, 5, 4),
            "[$-0411]dddd",
        ),
        Ok("月曜日".to_string())
    );
}

#[test]
fn locale_short_dates_and_currency_use_format_profile_semantics() {
    let de = format_profile(LocaleProfileId::DeDe);
    assert_eq!(
        parse_value_text(&de, WorkbookDateSystem::System1900, "01.02.2026"),
        Ok(serial_1900(2026, 2, 1))
    );
    assert_eq!(
        parse_value_text(&de, WorkbookDateSystem::System1900, "-1.234,50 €"),
        Ok(-1234.5)
    );
    assert_eq!(
        render_currency(&de, -1234.5, 2),
        Ok("-1.234,50 €".to_string())
    );

    let en_us = format_profile(LocaleProfileId::EnUs);
    assert_eq!(
        parse_value_text(&en_us, WorkbookDateSystem::System1900, "01/02/2026"),
        Ok(serial_1900(2026, 1, 2))
    );
    assert_eq!(
        parse_value_text(&en_us, WorkbookDateSystem::System1900, "-$1,234.50"),
        Ok(-1234.5)
    );
    assert_eq!(
        render_currency(&en_us, -1234.5, 2),
        Ok("-$1,234.50".to_string())
    );
}

#[test]
fn canonical_profile_ids_have_date_name_tables() {
    let january = serial_1900(2026, 1, 4);
    for id in CANONICAL_LOCALE_PROFILE_IDS {
        let profile = format_profile(id);
        assert_ne!(
            render_with_code(&profile, WorkbookDateSystem::System1900, january, "mmmm"),
            Ok(String::new()),
            "missing month table for {}",
            id.stable_name()
        );
        assert_ne!(
            render_with_code(&profile, WorkbookDateSystem::System1900, january, "dddd"),
            Ok(String::new()),
            "missing weekday table for {}",
            id.stable_name()
        );
    }
}
