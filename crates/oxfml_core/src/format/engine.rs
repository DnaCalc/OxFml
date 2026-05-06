use oxfunc_core::locale_format::{
    DateComponentOrder, FormatCodeEngine, FormatFailure, FormatProfile, LocaleFormatContext,
    LocaleProfileId, LocaleValueParser, ParseFailure, WorkbookDateSystem, excel_serial_from_ymd,
    format_profile,
};
use oxfunc_core::value::ExcelText;

use crate::format::general::render_visible_number_with_profile;
use crate::format::number::{
    parse_number_with_profile, render_currency as render_currency_text,
    render_fixed as render_fixed_text, render_with_number_format_code,
};

pub struct OxFmlLocaleValueParser;
pub struct OxFmlFormatCodeEngine;

pub static OXFML_LOCALE_VALUE_PARSER: OxFmlLocaleValueParser = OxFmlLocaleValueParser;
pub static OXFML_FORMAT_CODE_ENGINE: OxFmlFormatCodeEngine = OxFmlFormatCodeEngine;

pub const fn oxfml_en_us_format_profile() -> FormatProfile {
    format_profile(LocaleProfileId::EnUs)
}

pub const fn oxfml_current_excel_host_format_profile() -> FormatProfile {
    format_profile(LocaleProfileId::CurrentExcelHost)
}

pub fn oxfml_locale_context(
    profile: FormatProfile,
    date_system: WorkbookDateSystem,
) -> LocaleFormatContext<'static> {
    LocaleFormatContext {
        profile,
        date_system,
        parser: &OXFML_LOCALE_VALUE_PARSER,
        formatter: &OXFML_FORMAT_CODE_ENGINE,
    }
}

pub fn oxfml_en_us_locale_context() -> LocaleFormatContext<'static> {
    oxfml_locale_context(oxfml_en_us_format_profile(), WorkbookDateSystem::System1900)
}

pub fn oxfml_current_excel_host_locale_context() -> LocaleFormatContext<'static> {
    oxfml_locale_context(
        oxfml_current_excel_host_format_profile(),
        WorkbookDateSystem::System1900,
    )
}

pub fn canonicalize_locale_context<'a>(
    locale_ctx: &'a LocaleFormatContext<'a>,
) -> LocaleFormatContext<'a> {
    LocaleFormatContext {
        profile: locale_ctx.profile,
        date_system: locale_ctx.date_system,
        parser: &OXFML_LOCALE_VALUE_PARSER,
        formatter: &OXFML_FORMAT_CODE_ENGINE,
    }
}

pub fn parse_value_text(
    profile: &FormatProfile,
    date_system: WorkbookDateSystem,
    text: &str,
) -> Result<f64, ParseFailure> {
    OXFML_LOCALE_VALUE_PARSER.parse_value_text(profile, date_system, text)
}

pub fn render_with_code(
    profile: &FormatProfile,
    date_system: WorkbookDateSystem,
    value: f64,
    code: &str,
) -> Result<String, FormatFailure> {
    OXFML_FORMAT_CODE_ENGINE
        .render_with_code(profile, date_system, value, code)
        .map(|text| text.to_string_lossy())
}

pub fn render_currency(
    profile: &FormatProfile,
    value: f64,
    decimals: i32,
) -> Result<String, FormatFailure> {
    OXFML_FORMAT_CODE_ENGINE
        .render_currency(profile, value, decimals)
        .map(|text| text.to_string_lossy())
}

pub fn render_fixed(
    profile: &FormatProfile,
    value: f64,
    decimals: i32,
    no_commas: bool,
) -> Result<String, FormatFailure> {
    OXFML_FORMAT_CODE_ENGINE
        .render_fixed(profile, value, decimals, no_commas)
        .map(|text| text.to_string_lossy())
}

impl LocaleValueParser for OxFmlLocaleValueParser {
    fn parse_value_text(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        text: &str,
    ) -> Result<f64, ParseFailure> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        if let Some(stripped) = trimmed.strip_suffix('%') {
            return parse_number_with_profile(profile, stripped)
                .map(|value| value / 100.0)
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        if let Some(parsed) = parse_currency_with_profile(profile, trimmed) {
            return Ok(parsed);
        }

        if let Some((year, month, day)) = parse_iso_ymd(trimmed) {
            return excel_serial_from_ymd(date_system, year, month, day)
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        if let Some((year, month, day)) = parse_profile_short_date(profile, trimmed) {
            return excel_serial_from_ymd(date_system, year, month, day)
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        parse_number_with_profile(profile, trimmed)
            .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()))
    }
}

impl FormatCodeEngine for OxFmlFormatCodeEngine {
    fn render_with_code(
        &self,
        profile: &FormatProfile,
        date_system: WorkbookDateSystem,
        value: f64,
        code: &str,
    ) -> Result<ExcelText, FormatFailure> {
        let trimmed = code.trim();
        let rendered = if trimmed.eq_ignore_ascii_case("general") {
            render_visible_number_with_profile(profile, value)
        } else {
            render_with_number_format_code(profile, date_system, value, trimmed)?
        };
        Ok(text_from_string(rendered))
    }

    fn render_currency(
        &self,
        profile: &FormatProfile,
        value: f64,
        decimals: i32,
    ) -> Result<ExcelText, FormatFailure> {
        Ok(text_from_string(render_currency_text(
            profile, value, decimals,
        )))
    }

    fn render_fixed(
        &self,
        profile: &FormatProfile,
        value: f64,
        decimals: i32,
        no_commas: bool,
    ) -> Result<ExcelText, FormatFailure> {
        Ok(text_from_string(render_fixed_text(
            profile, value, decimals, no_commas,
        )))
    }
}

fn text_from_string(text: String) -> ExcelText {
    ExcelText::from_utf16_code_units(text.encode_utf16().collect())
}

fn parse_iso_ymd(text: &str) -> Option<(i64, i64, i64)> {
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse::<i64>().ok()?,
        parts[1].parse::<i64>().ok()?,
        parts[2].parse::<i64>().ok()?,
    ))
}

fn parse_profile_short_date(profile: &FormatProfile, text: &str) -> Option<(i64, i64, i64)> {
    if !text.contains(profile.date_separator) {
        return None;
    }

    let parts: Vec<&str> = text
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }
    let first = parts[0].parse::<i64>().ok()?;
    let second = parts[1].parse::<i64>().ok()?;
    let third = parts[2].parse::<i64>().ok()?;

    let (year, month, day) = match profile.short_date_order {
        DateComponentOrder::Mdy => (third, first, second),
        DateComponentOrder::Dmy => (third, second, first),
        DateComponentOrder::Ymd => (first, second, third),
    };
    Some((expand_two_digit_year(profile, year), month, day))
}

fn expand_two_digit_year(profile: &FormatProfile, year: i64) -> i64 {
    if !(0..=99).contains(&year) {
        return year;
    }
    let Some(pivot) = profile.two_digit_year_pivot else {
        return year;
    };
    let pivot = i64::from(pivot);
    let century = pivot - pivot.rem_euclid(100);
    let candidate = century + year;
    if candidate > pivot {
        candidate - 100
    } else {
        candidate
    }
}

fn parse_currency_with_profile(profile: &FormatProfile, text: &str) -> Option<f64> {
    let (negative_from_parens, body) = if let Some(inner) = text
        .trim()
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    {
        (true, inner.trim())
    } else {
        (false, text.trim())
    };

    let (negative_from_prefix, body) = if let Some(rest) = body.strip_prefix('-') {
        (true, rest.trim_start())
    } else if let Some(rest) = body.strip_prefix('+') {
        (false, rest.trim_start())
    } else {
        (false, body)
    };

    let (body, negative_from_suffix) = if let Some(rest) = body.strip_suffix('-') {
        (rest.trim_end(), true)
    } else {
        (body, false)
    };

    let amount = if let Some(rest) = body.strip_prefix(profile.currency_symbol) {
        rest.trim_start()
            .trim_start_matches('\u{00A0}')
            .trim_start_matches('\u{202F}')
    } else if let Some(rest) = body.strip_suffix(profile.currency_symbol) {
        rest.trim_end()
            .trim_end_matches('\u{00A0}')
            .trim_end_matches('\u{202F}')
    } else {
        return None;
    };

    let parsed = parse_number_with_profile(profile, amount)?;
    if negative_from_parens || negative_from_prefix || negative_from_suffix {
        Some(-parsed)
    } else {
        Some(parsed)
    }
}
