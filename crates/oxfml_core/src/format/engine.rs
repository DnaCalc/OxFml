use oxfunc_core::locale_format::{
    FormatCodeEngine, FormatFailure, FormatProfile, LocaleFormatContext, LocaleProfileId,
    LocaleValueParser, ParseFailure, WorkbookDateSystem, excel_serial_from_ymd, format_profile,
};
use oxfunc_core::value::ExcelText;

use crate::format::general::render_visible_number;
use crate::format::number::{
    parse_number_with_profile, render_currency as render_currency_text,
    render_fixed as render_fixed_text, render_with_number_format_code,
};

pub struct OxFmlLocaleValueParser;
pub struct OxFmlFormatCodeEngine;

pub static OXFML_LOCALE_VALUE_PARSER: OxFmlLocaleValueParser = OxFmlLocaleValueParser;
pub static OXFML_FORMAT_CODE_ENGINE: OxFmlFormatCodeEngine = OxFmlFormatCodeEngine;

pub fn en_us_context() -> LocaleFormatContext<'static> {
    LocaleFormatContext {
        profile: format_profile(LocaleProfileId::EnUs),
        date_system: WorkbookDateSystem::System1900,
        parser: &OXFML_LOCALE_VALUE_PARSER,
        formatter: &OXFML_FORMAT_CODE_ENGINE,
    }
}

pub fn current_excel_host_context() -> LocaleFormatContext<'static> {
    LocaleFormatContext {
        profile: format_profile(LocaleProfileId::CurrentExcelHost),
        date_system: WorkbookDateSystem::System1900,
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

        let (negative, body) = if let Some(rest) = trimmed.strip_prefix('-') {
            (true, rest.trim_start())
        } else {
            (false, trimmed)
        };
        if let Some(rest) = body.strip_prefix(profile.currency_symbol) {
            let parsed = parse_number_with_profile(profile, rest.trim_start())
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()))?;
            return Ok(if negative { -parsed } else { parsed });
        }

        if let Some((year, month, day)) = parse_iso_ymd(trimmed) {
            return excel_serial_from_ymd(date_system, year, month, day)
                .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
        }

        if profile.id == LocaleProfileId::EnUs {
            if let Some((year, month, day)) = parse_en_us_slash_date(trimmed) {
                return excel_serial_from_ymd(date_system, year, month, day)
                    .ok_or_else(|| ParseFailure::UnsupportedText(trimmed.to_string()));
            }
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
            render_visible_number(value)
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

fn parse_en_us_slash_date(text: &str) -> Option<(i64, i64, i64)> {
    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[2].parse::<i64>().ok()?,
        parts[0].parse::<i64>().ok()?,
        parts[1].parse::<i64>().ok()?,
    ))
}
