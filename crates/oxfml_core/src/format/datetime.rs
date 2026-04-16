use oxfunc_core::locale_format::{FormatProfile, WorkbookDateSystem, ymd_from_excel_serial};

use crate::format::locale_tables::{month_name, weekday_name};

pub fn looks_like_date_format(section: &str) -> bool {
    let lower = section.to_ascii_lowercase();
    let has_date_tokens = lower.contains('y') || lower.contains('d') || lower.contains('m');
    has_date_tokens
        && !contains_unsupported_time_tokens(&lower)
        && !lower.contains('#')
        && !lower.contains('0')
}

pub fn render_with_date_tokens(
    profile: &FormatProfile,
    date_system: WorkbookDateSystem,
    value: f64,
    section: &str,
) -> Option<String> {
    if contains_unsupported_time_tokens(section) {
        return None;
    }

    let (year, month, day) = ymd_from_excel_serial(date_system, value)?;
    let weekday_index = weekday_from_ymd(year, month, day);
    let mut rendered = section.to_ascii_lowercase();

    rendered = rendered.replace("AM/PM", "AM");
    rendered = rendered.replace("am/pm", "am");
    rendered = rendered.replace("dddd", weekday_name(weekday_index, false));
    rendered = rendered.replace("ddd", weekday_name(weekday_index, true));
    rendered = rendered.replace("yyyy", &format!("{year:04}"));
    rendered = rendered.replace("yy", &format!("{:02}", year.rem_euclid(100)));
    rendered = rendered.replace("mmmm", month_name(month, false));
    rendered = rendered.replace("mmm", month_name(month, true));
    rendered = rendered.replace("mm", &format!("{month:02}"));
    rendered = rendered.replace("dd", &format!("{day:02}"));
    rendered = rendered.replace("m", &month.to_string());
    rendered = rendered.replace("d", &day.to_string());
    rendered = rendered.replace("/", profile.date_separator);
    Some(rendered)
}

fn contains_unsupported_time_tokens(section: &str) -> bool {
    let lower = section.to_ascii_lowercase();
    lower.contains("am/pm") || lower.contains('h') || lower.contains(':')
}

fn weekday_from_ymd(year: i64, month: i64, day: i64) -> usize {
    let mut month = month;
    let mut year = year;
    if month < 3 {
        month += 12;
        year -= 1;
    }
    let k = year.rem_euclid(100);
    let j = year.div_euclid(100);
    let h =
        (day + (13 * (month + 1)).div_euclid(5) + k + k.div_euclid(4) + j.div_euclid(4) + 5 * j)
            .rem_euclid(7);
    match h {
        0 => 6,
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        5 => 4,
        _ => 5,
    }
}
