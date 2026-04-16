use oxfunc_core::locale_format::{FormatFailure, FormatProfile, WorkbookDateSystem};

use crate::format::datetime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedNumericSection {
    pub prefix: String,
    pub suffix: String,
    pub decimals: i32,
    pub use_grouping: bool,
    pub percent_count: i32,
    pub trailing_commas: i32,
    pub negative_parentheses: bool,
    pub is_currency: bool,
    pub scientific_exponent_digits: Option<usize>,
}

pub fn normalize_numeric_text(profile: &FormatProfile, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (negative, body) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else {
        (false, trimmed)
    };

    let mut normalized = body.replace(profile.thousands_separator, "");
    if profile.decimal_separator != "." {
        normalized = normalized.replace(profile.decimal_separator, ".");
    }

    if normalized.matches('.').count() > 1 {
        return None;
    }

    if negative {
        normalized.insert(0, '-');
    }
    Some(normalized)
}

pub fn parse_number_with_profile(profile: &FormatProfile, raw: &str) -> Option<f64> {
    let normalized = normalize_numeric_text(profile, raw)?;
    let parsed = normalized.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}

pub fn render_with_number_format_code(
    profile: &FormatProfile,
    date_system: WorkbookDateSystem,
    value: f64,
    number_format_code: &str,
) -> Result<String, FormatFailure> {
    let section = select_number_format_section(number_format_code, value)
        .ok_or_else(|| FormatFailure::UnsupportedCode(number_format_code.to_string()))?;
    let stripped = strip_condition_and_color_tokens(&section);
    if stripped.chars().all(char::is_whitespace) {
        return Ok(stripped);
    }
    let trimmed = stripped.trim();

    if datetime::looks_like_date_format(trimmed) {
        return datetime::render_with_date_tokens(profile, date_system, value, trimmed)
            .ok_or(FormatFailure::InvalidDateSerial);
    }

    if is_two_digit_integer_code(trimmed) {
        return render_two_digit_integer(value);
    }

    let numeric = parse_numeric_section(trimmed)
        .ok_or_else(|| FormatFailure::UnsupportedCode(number_format_code.to_string()))?;
    let scaled_value = apply_scaling(value, &numeric);

    if let Some(exponent_digits) = numeric.scientific_exponent_digits {
        let rendered = render_scientific(
            scaled_value,
            numeric.decimals.max(0) as usize,
            exponent_digits,
        );
        return Ok(apply_numeric_affixes(rendered, &numeric));
    }

    let base = if numeric.is_currency
        && numeric.prefix == profile.currency_symbol
        && numeric.suffix.is_empty()
    {
        render_currency(profile, scaled_value, numeric.decimals)
    } else {
        render_fixed(
            profile,
            scaled_value.abs(),
            numeric.decimals,
            !numeric.use_grouping,
        )
    };

    let body = if scaled_value.is_sign_negative() && !base.starts_with('-') {
        format!("-{base}")
    } else {
        base
    };
    Ok(apply_numeric_affixes(body, &numeric))
}

pub fn render_currency(profile: &FormatProfile, value: f64, decimals: i32) -> String {
    render_fixed_common(profile, value, decimals, true, profile.currency_symbol)
}

pub fn render_fixed(profile: &FormatProfile, value: f64, decimals: i32, no_commas: bool) -> String {
    render_fixed_common(profile, value, decimals, !no_commas, "")
}

pub(crate) fn parse_numeric_section(section: &str) -> Option<ParsedNumericSection> {
    let cleaned = expand_literal_tokens(section);
    let first_placeholder = cleaned.find(['#', '0', '?'])?;
    let last_placeholder = cleaned.rfind(['#', '0', '?'])?;
    let prefix = cleaned[..first_placeholder].replace('*', "");
    let suffix = cleaned[last_placeholder + 1..].replace('*', "");
    let placeholder_region = &cleaned[first_placeholder..=last_placeholder];
    let decimals = placeholder_region
        .split_once('.')
        .map(|(_, fractional)| {
            fractional
                .chars()
                .take_while(|ch| matches!(ch, '0' | '#' | '?' | 'E' | 'e' | '+' | '-'))
                .filter(|ch| matches!(ch, '0' | '#' | '?'))
                .count() as i32
        })
        .unwrap_or(0);
    let integer_region = placeholder_region
        .split_once('.')
        .map(|(integer, _)| integer)
        .unwrap_or(placeholder_region);
    let use_grouping = integer_region.contains(',');
    let trailing_commas = integer_region
        .chars()
        .rev()
        .take_while(|ch| *ch == ',')
        .count() as i32;
    let scientific_exponent_digits = placeholder_region
        .to_ascii_uppercase()
        .split_once('E')
        .map(|(_, exponent)| exponent.chars().filter(|ch| *ch == '0').count())
        .filter(|digits| *digits > 0);
    let percent_count = prefix.matches('%').count() as i32 + suffix.matches('%').count() as i32;
    let negative_parentheses = prefix.contains('(') && suffix.contains(')');
    let is_currency =
        prefix.contains(profile_currency_tokens()) || suffix.contains(profile_currency_tokens());

    Some(ParsedNumericSection {
        prefix,
        suffix,
        decimals,
        use_grouping,
        percent_count,
        trailing_commas,
        negative_parentheses,
        is_currency,
        scientific_exponent_digits,
    })
}

fn profile_currency_tokens() -> [char; 2] {
    ['$', 'R']
}

fn apply_scaling(value: f64, numeric: &ParsedNumericSection) -> f64 {
    let percent_scaled = value * 100f64.powi(numeric.percent_count);
    percent_scaled / 1000f64.powi(numeric.trailing_commas)
}

fn render_fixed_common(
    profile: &FormatProfile,
    value: f64,
    decimals: i32,
    use_grouping: bool,
    prefix: &str,
) -> String {
    let rounded = format!("{:.*}", decimals.max(0) as usize, value.abs());
    let is_negative = value.is_sign_negative() && value != 0.0;
    let (int_part, frac_part) = match rounded.split_once('.') {
        Some((lhs, rhs)) => (lhs.to_string(), Some(rhs.to_string())),
        None => (rounded, None),
    };
    let grouped = if use_grouping {
        grouped_integer_string(&int_part, profile.thousands_separator)
    } else {
        int_part
    };

    let mut rendered = String::new();
    if is_negative {
        rendered.push('-');
    }
    rendered.push_str(prefix);
    rendered.push_str(&grouped);
    if let Some(frac) = frac_part {
        if decimals > 0 {
            rendered.push_str(profile.decimal_separator);
            rendered.push_str(&frac);
        }
    }
    rendered
}

fn render_scientific(value: f64, decimals: usize, exponent_digits: usize) -> String {
    if value == 0.0 {
        let mantissa = format!("{:.*}", decimals, 0.0);
        return format!("{mantissa}E+{:0width$}", 0, width = exponent_digits);
    }

    let exponent = value.abs().log10().floor() as i32;
    let mantissa = value / 10f64.powi(exponent);
    let sign = if exponent >= 0 { '+' } else { '-' };
    format!(
        "{:.*}E{}{abs_exponent:0width$}",
        decimals,
        mantissa,
        sign,
        abs_exponent = exponent.unsigned_abs(),
        width = exponent_digits
    )
}

fn grouped_integer_string(int_part: &str, sep: &str) -> String {
    if int_part.len() <= 3 || sep.is_empty() {
        return int_part.to_string();
    }
    let mut out = String::new();
    let first = int_part.len() % 3;
    let mut index = 0;
    if first > 0 {
        out.push_str(&int_part[..first]);
        index = first;
    }
    while index < int_part.len() {
        if !out.is_empty() {
            out.push_str(sep);
        }
        out.push_str(&int_part[index..index + 3]);
        index += 3;
    }
    out
}

pub(crate) fn apply_numeric_affixes(base: String, numeric: &ParsedNumericSection) -> String {
    if numeric.negative_parentheses && base.starts_with('-') {
        let unsigned = base.trim_start_matches('-');
        return format!("{}{}{}", numeric.prefix, unsigned, numeric.suffix);
    }
    if numeric.is_currency
        && numeric.suffix.is_empty()
        && base.starts_with(&numeric.prefix)
        && !numeric.prefix.is_empty()
    {
        return base;
    }
    format!("{}{}{}", numeric.prefix, base, numeric.suffix)
}

pub(crate) fn select_number_format_section(code: &str, value: f64) -> Option<String> {
    let sections = split_format_sections(code);
    if sections.is_empty() {
        return None;
    }

    let has_explicit_condition = sections.iter().any(|section| extract_condition(section).is_some());
    if has_explicit_condition {
        let mut fallback = None;
        for section in &sections {
            if let Some(condition) = extract_condition(section) {
                if condition_matches(&condition, value) {
                    return Some(section.clone());
                }
            } else if fallback.is_none() {
                fallback = Some(section.clone());
            }
        }
        return fallback;
    }

    match sections.len() {
        1 => sections.first().cloned(),
        2 => {
            if value < 0.0 {
                sections
                    .get(1)
                    .cloned()
                    .or_else(|| sections.first().cloned())
            } else {
                sections.first().cloned()
            }
        }
        _ => {
            if value > 0.0 {
                sections.first().cloned()
            } else if value < 0.0 {
                sections
                    .get(1)
                    .cloned()
                    .or_else(|| sections.first().cloned())
            } else {
                sections
                    .get(2)
                    .cloned()
                    .or_else(|| sections.first().cloned())
            }
        }
    }
}

fn split_format_sections(code: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for ch in code.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => {
                current.push(ch);
                escaped = true;
            }
            '"' => {
                current.push(ch);
                in_quotes = !in_quotes;
            }
            ';' if !in_quotes => {
                sections.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() || code.ends_with(';') {
        sections.push(current);
    }
    sections
        .into_iter()
        .filter(|section| !section.trim().is_empty())
        .collect()
}

pub(crate) fn strip_condition_and_color_tokens(section: &str) -> String {
    let mut stripped = String::new();
    let mut chars = section.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut token = String::new();
            for next in chars.by_ref() {
                if next == ']' {
                    break;
                }
                token.push(next);
            }
            if token.starts_with('>')
                || token.starts_with('<')
                || token.starts_with('=')
                || token.chars().all(|c| c.is_ascii_alphabetic())
            {
                continue;
            }
            stripped.push('[');
            stripped.push_str(&token);
            stripped.push(']');
        } else {
            stripped.push(ch);
        }
    }
    stripped
}

pub(crate) fn expand_literal_tokens(section: &str) -> String {
    let mut result = String::new();
    let mut chars = section.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '\\' => {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
            '_' => {
                let _ = chars.next();
                result.push(' ');
            }
            '*' => {}
            _ if in_quotes => result.push(ch),
            _ => result.push(ch),
        }
    }

    result
}

fn extract_condition(section: &str) -> Option<String> {
    let trimmed = section.trim_start();
    let remaining = trimmed.strip_prefix('[')?;
    let token = remaining.split(']').next()?;
    if token.starts_with('>') || token.starts_with('<') || token.starts_with('=') {
        Some(token.to_string())
    } else {
        None
    }
}

fn condition_matches(condition: &str, value: f64) -> bool {
    let operator = if let Some(rest) = condition.strip_prefix(">=") {
        (">=", rest)
    } else if let Some(rest) = condition.strip_prefix("<=") {
        ("<=", rest)
    } else if let Some(rest) = condition.strip_prefix("<>") {
        ("<>", rest)
    } else if let Some(rest) = condition.strip_prefix('>') {
        (">", rest)
    } else if let Some(rest) = condition.strip_prefix('<') {
        ("<", rest)
    } else if let Some(rest) = condition.strip_prefix('=') {
        ("=", rest)
    } else {
        return false;
    };
    let Ok(threshold) = operator.1.trim().parse::<f64>() else {
        return false;
    };
    match operator.0 {
        ">" => value > threshold,
        ">=" => value >= threshold,
        "<" => value < threshold,
        "<=" => value <= threshold,
        "=" => (value - threshold).abs() < f64::EPSILON,
        "<>" => (value - threshold).abs() >= f64::EPSILON,
        _ => false,
    }
}

fn is_two_digit_integer_code(section: &str) -> bool {
    section == "00"
}

fn render_two_digit_integer(value: f64) -> Result<String, FormatFailure> {
    if !value.is_finite() {
        return Err(FormatFailure::UnsupportedCode("00".to_string()));
    }

    let rounded = value.round();
    let magnitude = rounded.abs() as i64;
    if rounded.is_sign_negative() && rounded != 0.0 {
        Ok(format!("-{magnitude:02}"))
    } else {
        Ok(format!("{magnitude:02}"))
    }
}
