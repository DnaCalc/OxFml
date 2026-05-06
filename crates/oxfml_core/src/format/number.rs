use oxfunc_core::locale_format::{
    CurrencyNegativePattern, CurrencyPlacement, CurrencySpacing, FormatFailure, FormatProfile,
    LocaleProfileId, WorkbookDateSystem, format_profile,
};

use crate::format::datetime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IntegerSeparatorSemantics {
    None,
    RecursiveGrouping,
    LiteralPattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedNumericSection {
    pub prefix: String,
    pub suffix: String,
    pub decimals: i32,
    pub integer_pattern: String,
    pub integer_separator_semantics: IntegerSeparatorSemantics,
    pub percent_count: i32,
    pub scale_commas: i32,
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
    let (stripped, locale_profile_id) = strip_section_tokens(&section);
    let locale_profile = locale_profile_id.map(format_profile);
    let effective_profile = locale_profile.as_ref().unwrap_or(profile);
    if stripped.chars().all(char::is_whitespace) {
        return Ok(stripped);
    }
    let trimmed = stripped.trim();

    if datetime::looks_like_datetime_format(trimmed) {
        return datetime::render_with_datetime_tokens(
            effective_profile,
            date_system,
            value,
            trimmed,
        )
        .ok_or(FormatFailure::InvalidDateSerial);
    }

    if is_two_digit_integer_code(trimmed) {
        return render_two_digit_integer(value);
    }

    if contains_fraction_placeholder_pattern(trimmed) {
        return render_fraction_format(value, trimmed)
            .ok_or_else(|| FormatFailure::UnsupportedCode(number_format_code.to_string()));
    }

    let numeric = parse_numeric_section(trimmed, effective_profile)
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

    let base = render_fixed_with_numeric_section(effective_profile, scaled_value.abs(), &numeric);

    let body = if scaled_value.is_sign_negative() && !base.starts_with('-') {
        format!("-{base}")
    } else {
        base
    };
    Ok(apply_numeric_affixes(body, &numeric))
}

pub(crate) fn render_text_with_number_format_code(
    text: &str,
    number_format_code: &str,
) -> Option<String> {
    let sections = split_format_sections(number_format_code);
    if sections.is_empty() {
        return None;
    }
    let Some(section) = sections.get(3) else {
        return Some(text.to_string());
    };
    let stripped = strip_condition_and_color_tokens(section);
    Some(render_text_format_section(text, &stripped))
}

pub(crate) fn selected_number_format_section_color(
    number_format_code: &str,
    value: f64,
) -> Option<String> {
    let section = select_number_format_section(number_format_code, value)?;
    leading_format_section_color(&section)
}

pub(crate) fn selected_text_format_section_color(number_format_code: &str) -> Option<String> {
    let sections = split_format_sections(number_format_code);
    let section = sections.get(3)?;
    leading_format_section_color(section)
}

pub fn render_currency(profile: &FormatProfile, value: f64, decimals: i32) -> String {
    let magnitude = render_fixed_common(
        profile,
        value.abs(),
        decimals,
        IntegerRenderStyle::RecursiveGrouping,
        "",
    );
    let spacing = currency_spacing_text(profile.currency_spacing);
    let body = match profile.currency_placement {
        CurrencyPlacement::Before => format!("{}{}{}", profile.currency_symbol, spacing, magnitude),
        CurrencyPlacement::After => format!("{}{}{}", magnitude, spacing, profile.currency_symbol),
    };

    if value.is_sign_negative() && value != 0.0 {
        apply_currency_negative_pattern(profile, body, magnitude, spacing)
    } else {
        body
    }
}

pub fn render_fixed(profile: &FormatProfile, value: f64, decimals: i32, no_commas: bool) -> String {
    let integer_render_style = if no_commas {
        IntegerRenderStyle::Plain
    } else {
        IntegerRenderStyle::RecursiveGrouping
    };
    render_fixed_common(profile, value, decimals, integer_render_style, "")
}

pub(crate) fn parse_numeric_section(
    section: &str,
    profile: &FormatProfile,
) -> Option<ParsedNumericSection> {
    let cleaned = expand_literal_tokens(section);
    let first_placeholder = cleaned.find(['#', '0', '?'])?;
    let numeric_region_end = cleaned[first_placeholder..]
        .char_indices()
        .take_while(|(_, ch)| is_numeric_format_token(*ch))
        .last()
        .map(|(index, ch)| first_placeholder + index + ch.len_utf8())?;
    let prefix = cleaned[..first_placeholder].replace('*', "");
    let suffix = cleaned[numeric_region_end..].replace('*', "");
    let numeric_region = &cleaned[first_placeholder..numeric_region_end];
    let decimal_token = format_code_decimal_token(profile);
    let group_token = format_code_group_token(profile);
    let mantissa_region = numeric_region
        .split_once(['E', 'e'])
        .map(|(mantissa, _)| mantissa)
        .unwrap_or(numeric_region);
    let decimals = mantissa_region
        .split_once(decimal_token)
        .map(|(_, fractional)| {
            fractional
                .chars()
                .take_while(|ch| matches!(ch, '0' | '#' | '?'))
                .count() as i32
        })
        .unwrap_or(0);
    let integer_region = mantissa_region
        .split_once(decimal_token)
        .map(|(integer, _)| integer)
        .unwrap_or(mantissa_region);
    let scale_commas = integer_region
        .chars()
        .rev()
        .take_while(|ch| *ch == group_token)
        .count() as i32;
    let integer_pattern = integer_region.trim_end_matches(group_token).to_string();
    let integer_separator_semantics = if integer_pattern.contains(group_token) {
        IntegerSeparatorSemantics::RecursiveGrouping
    } else if group_token != ',' && integer_pattern.contains(',') {
        IntegerSeparatorSemantics::LiteralPattern
    } else {
        IntegerSeparatorSemantics::None
    };
    let scientific_exponent_digits = numeric_region
        .to_ascii_uppercase()
        .split_once('E')
        .map(|(_, exponent)| exponent.chars().filter(|ch| *ch == '0').count())
        .filter(|digits| *digits > 0);
    let percent_count = prefix.matches('%').count() as i32 + suffix.matches('%').count() as i32;
    let negative_parentheses = prefix.contains('(') && suffix.contains(')');
    let is_currency = prefix.contains(profile_currency_tokens())
        || suffix.contains(profile_currency_tokens())
        || prefix.contains(profile.currency_symbol)
        || suffix.contains(profile.currency_symbol);

    Some(ParsedNumericSection {
        prefix,
        suffix,
        decimals,
        integer_pattern,
        integer_separator_semantics,
        percent_count,
        scale_commas,
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
    percent_scaled / 1000f64.powi(numeric.scale_commas)
}

fn render_fixed_with_numeric_section(
    profile: &FormatProfile,
    value: f64,
    numeric: &ParsedNumericSection,
) -> String {
    let integer_render_style = match numeric.integer_separator_semantics {
        IntegerSeparatorSemantics::None => IntegerRenderStyle::Plain,
        IntegerSeparatorSemantics::RecursiveGrouping => IntegerRenderStyle::RecursiveGrouping,
        IntegerSeparatorSemantics::LiteralPattern => {
            IntegerRenderStyle::LiteralPattern(&numeric.integer_pattern)
        }
    };
    let prefix = if numeric.is_currency
        && numeric.prefix == profile.currency_symbol
        && numeric.suffix.is_empty()
    {
        profile.currency_symbol
    } else {
        ""
    };

    render_fixed_common(
        profile,
        value,
        numeric.decimals,
        integer_render_style,
        prefix,
    )
}

enum IntegerRenderStyle<'a> {
    Plain,
    RecursiveGrouping,
    LiteralPattern(&'a str),
}

fn render_fixed_common(
    profile: &FormatProfile,
    value: f64,
    decimals: i32,
    integer_render_style: IntegerRenderStyle<'_>,
    prefix: &str,
) -> String {
    let rounded = format!("{:.*}", decimals.max(0) as usize, value.abs());
    let is_negative = value.is_sign_negative() && value != 0.0;
    let (int_part, frac_part) = match rounded.split_once('.') {
        Some((lhs, rhs)) => (lhs.to_string(), Some(rhs.to_string())),
        None => (rounded, None),
    };
    let grouped = match integer_render_style {
        IntegerRenderStyle::Plain => int_part,
        IntegerRenderStyle::RecursiveGrouping => {
            grouped_integer_string(&int_part, profile.thousands_separator)
        }
        IntegerRenderStyle::LiteralPattern(pattern) => {
            render_integer_pattern_with_literal_commas(&int_part, pattern)
        }
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

fn render_integer_pattern_with_literal_commas(int_part: &str, pattern: &str) -> String {
    let segments: Vec<&str> = pattern.split(',').collect();
    if segments.len() <= 1 {
        return int_part.to_string();
    }

    let digits: Vec<char> = int_part.chars().collect();
    let mut next_digit = digits.len();
    let mut rendered_segments = Vec::with_capacity(segments.len());

    for segment in segments.iter().rev() {
        rendered_segments.push(render_placeholder_segment(
            segment,
            &digits,
            &mut next_digit,
        ));
    }
    rendered_segments.reverse();

    if next_digit > 0 {
        let leading_digits: String = digits[..next_digit].iter().collect();
        if let Some(first) = rendered_segments.first_mut() {
            first.insert_str(0, &leading_digits);
        }
    }

    let first_visible = rendered_segments
        .iter()
        .position(|segment| segment.chars().any(|ch| ch != ' '))
        .unwrap_or(rendered_segments.len().saturating_sub(1));

    rendered_segments[first_visible..].join(",")
}

fn render_placeholder_segment(segment: &str, digits: &[char], next_digit: &mut usize) -> String {
    let mut rendered = String::new();
    for ch in segment.chars().rev() {
        match ch {
            '#' => {
                if *next_digit > 0 {
                    *next_digit -= 1;
                    rendered.push(digits[*next_digit]);
                }
            }
            '0' => {
                if *next_digit > 0 {
                    *next_digit -= 1;
                    rendered.push(digits[*next_digit]);
                } else {
                    rendered.push('0');
                }
            }
            '?' => {
                if *next_digit > 0 {
                    *next_digit -= 1;
                    rendered.push(digits[*next_digit]);
                } else {
                    rendered.push(' ');
                }
            }
            other => rendered.push(other),
        }
    }
    rendered.chars().rev().collect()
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

    let has_explicit_condition = sections
        .iter()
        .any(|section| extract_condition(section).is_some());
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
    strip_section_tokens(section).0
}

fn strip_section_tokens(section: &str) -> (String, Option<LocaleProfileId>) {
    let mut stripped = String::new();
    let mut chars = section.chars().peekable();
    let mut locale_profile_id = None;
    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut token = String::new();
            for next in chars.by_ref() {
                if next == ']' {
                    break;
                }
                token.push(next);
            }
            if is_condition_token(&token)
                || is_format_color_token(&token)
                || is_locale_prefix_token(&token)
                || (token.chars().all(|c| c.is_ascii_alphabetic())
                    && !is_elapsed_time_token(&token))
            {
                locale_profile_id =
                    locale_profile_id.or_else(|| locale_profile_id_from_token(&token));
                continue;
            }
            stripped.push('[');
            stripped.push_str(&token);
            stripped.push(']');
        } else {
            stripped.push(ch);
        }
    }
    (stripped, locale_profile_id)
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

fn render_text_format_section(text: &str, section: &str) -> String {
    let expanded = expand_literal_tokens(section);
    if expanded.contains('@') {
        expanded.replace('@', text)
    } else {
        expanded
    }
}

fn is_numeric_format_token(ch: char) -> bool {
    matches!(ch, '#' | '0' | '?' | ',' | '.' | 'E' | 'e' | '+' | '-')
}

fn extract_condition(section: &str) -> Option<String> {
    let mut remaining = section.trim_start();
    while let Some((token, rest)) = take_leading_bracket_token(remaining) {
        if is_condition_token(token) {
            return Some(token.to_string());
        }
        if is_format_color_token(token)
            || is_locale_prefix_token(token)
            || (token.chars().all(|c| c.is_ascii_alphabetic()) && !is_elapsed_time_token(token))
        {
            remaining = rest.trim_start();
            continue;
        }
        break;
    }
    None
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

fn leading_format_section_color(section: &str) -> Option<String> {
    let mut remaining = section.trim_start();
    while let Some((token, rest)) = take_leading_bracket_token(remaining) {
        if let Some(color) = format_color_token_hex(token) {
            return Some(color.to_string());
        }
        if is_condition_token(token) || is_locale_prefix_token(token) {
            remaining = rest.trim_start();
            continue;
        }
        break;
    }
    None
}

fn take_leading_bracket_token(input: &str) -> Option<(&str, &str)> {
    let remaining = input.strip_prefix('[')?;
    let close = remaining.find(']')?;
    let token = &remaining[..close];
    let rest = &remaining[close + 1..];
    Some((token, rest))
}

fn is_condition_token(token: &str) -> bool {
    token.starts_with('>') || token.starts_with('<') || token.starts_with('=')
}

fn is_format_color_token(token: &str) -> bool {
    format_color_token_hex(token).is_some()
}

fn is_locale_prefix_token(token: &str) -> bool {
    locale_profile_id_from_token(token).is_some()
}

fn locale_profile_id_from_token(token: &str) -> Option<LocaleProfileId> {
    let token = token.trim();
    let body = token.strip_prefix("$-")?;
    let lcid_text = body
        .split_once('-')
        .map(|(lcid, _)| lcid)
        .unwrap_or(body)
        .trim();
    let lcid = u16::from_str_radix(lcid_text, 16).ok()?;
    LocaleProfileId::from_excel_lcid(lcid)
}

fn format_code_decimal_token(profile: &FormatProfile) -> char {
    single_char_token(profile.format_code_decimal_token).unwrap_or('.')
}

fn format_code_group_token(profile: &FormatProfile) -> char {
    single_char_token(profile.format_code_group_token).unwrap_or(',')
}

fn single_char_token(token: &str) -> Option<char> {
    let mut chars = token.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

fn currency_spacing_text(spacing: CurrencySpacing) -> &'static str {
    match spacing {
        CurrencySpacing::None => "",
        CurrencySpacing::Space => " ",
        CurrencySpacing::NarrowNoBreakSpace => "\u{202F}",
    }
}

fn apply_currency_negative_pattern(
    profile: &FormatProfile,
    body: String,
    magnitude: String,
    spacing: &str,
) -> String {
    match profile.currency_negative_pattern {
        CurrencyNegativePattern::LeadingMinus => format!("-{body}"),
        CurrencyNegativePattern::TrailingMinus => format!("{body}-"),
        CurrencyNegativePattern::Parentheses => format!("({body})"),
        CurrencyNegativePattern::MinusBeforeSymbol => match profile.currency_placement {
            CurrencyPlacement::Before => {
                format!("-{}{}{}", profile.currency_symbol, spacing, magnitude)
            }
            CurrencyPlacement::After => format!("-{body}"),
        },
    }
}

fn format_color_token_hex(token: &str) -> Option<&'static str> {
    let normalized = token.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "black" => Some("#000000"),
        "blue" => Some("#0000FF"),
        "cyan" => Some("#00FFFF"),
        "green" => Some("#00FF00"),
        "magenta" => Some("#FF00FF"),
        "red" => Some("#FF0000"),
        "white" => Some("#FFFFFF"),
        "yellow" => Some("#FFFF00"),
        _ => {
            let index = normalized.strip_prefix("color")?.parse::<usize>().ok()?;
            EXCEL_DEFAULT_COLOR_INDEX
                .get(index.checked_sub(1)?)
                .copied()
        }
    }
}

const EXCEL_DEFAULT_COLOR_INDEX: [&str; 56] = [
    "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF",
    "#800000", "#008000", "#000080", "#808000", "#800080", "#008080", "#C0C0C0", "#808080",
    "#9999FF", "#993366", "#FFFFCC", "#CCFFFF", "#660066", "#FF8080", "#0066CC", "#CCCCFF",
    "#000080", "#FF00FF", "#FFFF00", "#00FFFF", "#800080", "#800000", "#008080", "#0000FF",
    "#00CCFF", "#CCFFFF", "#CCFFCC", "#FFFF99", "#99CCFF", "#FF99CC", "#CC99FF", "#FFCC99",
    "#3366FF", "#33CCCC", "#99CC00", "#FFCC00", "#FF9900", "#FF6600", "#666699", "#969696",
    "#003366", "#339966", "#003300", "#333300", "#993300", "#993366", "#333399", "#333333",
];

fn contains_fraction_placeholder_pattern(section: &str) -> bool {
    let expanded = expand_literal_tokens(section);
    expanded
        .split('/')
        .collect::<Vec<_>>()
        .windows(2)
        .any(|parts| {
            let left = parts[0].trim_end();
            let right = parts[1].trim_start();
            left.chars().last().is_some_and(is_fraction_placeholder)
                && right.chars().next().is_some_and(is_fraction_placeholder)
        })
}

fn is_elapsed_time_token(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    !lower.is_empty()
        && lower
            .chars()
            .all(|ch| matches!(ch, 'h' | 'm' | 's') && ch == lower.chars().next().unwrap())
}

fn is_fraction_placeholder(ch: char) -> bool {
    matches!(ch, '#' | '0' | '?')
}

fn render_fraction_format(value: f64, section: &str) -> Option<String> {
    if !value.is_finite() {
        return None;
    }

    let expanded = expand_literal_tokens(section);
    let slash = expanded.find('/')?;
    let left = &expanded[..slash];
    let right = &expanded[slash + 1..];
    let denominator_pattern: String = right
        .chars()
        .take_while(|ch| is_fraction_placeholder(*ch))
        .collect();
    if denominator_pattern.is_empty() {
        return None;
    }
    let denominator_width = denominator_pattern.chars().count();
    let denominator_max = 10_i64.pow(denominator_width as u32) - 1;
    if denominator_max <= 0 {
        return None;
    }

    let numerator_pattern: String = left
        .chars()
        .rev()
        .take_while(|ch| is_fraction_placeholder(*ch))
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if numerator_pattern.is_empty() {
        return None;
    }

    let integer_pattern = left.strip_suffix(&numerator_pattern).unwrap_or(left);
    let has_integer_part = integer_pattern
        .chars()
        .any(|ch| is_fraction_placeholder(ch));
    let negative = value.is_sign_negative() && value != 0.0;
    let abs_value = value.abs();
    let whole = if has_integer_part {
        abs_value.floor() as i64
    } else {
        0
    };
    let fraction_value = if has_integer_part {
        abs_value - whole as f64
    } else {
        abs_value
    };
    let (mut numerator, mut denominator) = approximate_fraction(fraction_value, denominator_max)?;

    let mut whole = whole;
    if has_integer_part && numerator == denominator {
        whole += 1;
        numerator = 0;
        denominator = 1;
    }

    let mut rendered = String::new();
    if negative {
        rendered.push('-');
    }

    if has_integer_part {
        let integer_rendered = render_fraction_integer_part(integer_pattern, whole);
        rendered.push_str(&integer_rendered);
    }

    if numerator != 0 || !has_integer_part {
        rendered.push_str(&render_placeholder_number(
            numerator,
            &numerator_pattern,
            numerator_pattern.contains('0'),
        ));
        rendered.push('/');
        rendered.push_str(&render_placeholder_number(
            denominator,
            &denominator_pattern,
            denominator_pattern.contains('0'),
        ));
        rendered.push_str(&right[denominator_pattern.len()..]);
    } else {
        rendered.push_str(&blank_fraction_tail(
            &numerator_pattern,
            &denominator_pattern,
            &right[denominator_pattern.len()..],
        ));
    }

    Some(rendered)
}

fn approximate_fraction(value: f64, denominator_max: i64) -> Option<(i64, i64)> {
    if value == 0.0 {
        return Some((0, 1));
    }

    let mut best_numerator = 0;
    let mut best_denominator = 1;
    let mut best_error = f64::INFINITY;
    for denominator in 1..=denominator_max {
        let numerator = (value * denominator as f64).round() as i64;
        let error = (value - numerator as f64 / denominator as f64).abs();
        if error < best_error {
            best_error = error;
            best_numerator = numerator;
            best_denominator = denominator;
        }
        if error < f64::EPSILON {
            break;
        }
    }
    Some((best_numerator, best_denominator))
}

fn render_fraction_integer_part(pattern: &str, whole: i64) -> String {
    let mut rendered = String::new();
    let mut digits = whole.to_string();
    let placeholder_count = pattern
        .chars()
        .filter(|ch| is_fraction_placeholder(*ch))
        .count();
    if digits == "0" && !pattern.contains('0') {
        digits.clear();
    }
    if pattern.contains('0') && digits.len() < placeholder_count {
        digits = format!("{digits:0>placeholder_count$}");
    }
    let overflow_digits = digits.len().saturating_sub(placeholder_count);
    rendered.push_str(&digits.chars().take(overflow_digits).collect::<String>());
    let mut digit_chars = digits.chars().skip(overflow_digits);
    for ch in pattern.chars() {
        if is_fraction_placeholder(ch) {
            if let Some(digit) = digit_chars.next() {
                rendered.push(digit);
            } else if ch == '0' {
                rendered.push('0');
            }
        } else {
            rendered.push(ch);
        }
    }
    rendered
}

fn render_placeholder_number(value: i64, pattern: &str, zero_pad: bool) -> String {
    let width = pattern.chars().count();
    if zero_pad {
        format!("{value:0width$}")
    } else {
        format!("{value:>width$}")
    }
}

fn blank_fraction_tail(numerator_pattern: &str, denominator_pattern: &str, suffix: &str) -> String {
    format!(
        "{}/{}{}",
        " ".repeat(numerator_pattern.chars().count()),
        " ".repeat(denominator_pattern.chars().count()),
        suffix
    )
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
