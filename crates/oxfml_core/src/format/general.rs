use oxfunc_core::locale_format::FormatProfile;
use oxfunc_core::value::{ArrayCellValue, EvalValue, WorksheetErrorCode};

use crate::eval::{eval_value_callable_transport_summary, eval_value_is_callable_transport};

pub fn render_visible_value_text(value: &EvalValue) -> String {
    if eval_value_is_callable_transport(value) {
        return format!(
            "Callable({})",
            eval_value_callable_transport_summary(value).unwrap_or_default()
        );
    }
    match value {
        EvalValue::Number(number) => render_visible_number(*number),
        EvalValue::Text(text) => text.to_string_lossy(),
        EvalValue::Logical(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        EvalValue::Error(code) => worksheet_error_text(*code).to_string(),
        EvalValue::Array(array) => array
            .get(0, 0)
            .map(render_array_cell_text)
            .unwrap_or_default(),
        EvalValue::Reference(reference) => reference.target.clone(),
        other => format!("{other:?}"),
    }
}

pub fn render_visible_number(number: f64) -> String {
    if number.fract() == 0.0 {
        format!("{number:.0}")
    } else {
        number.to_string()
    }
}

pub fn render_visible_number_with_profile(profile: &FormatProfile, number: f64) -> String {
    let rendered = render_visible_number(number);
    if profile.decimal_separator == "." {
        rendered
    } else {
        rendered.replace('.', profile.decimal_separator)
    }
}

pub fn worksheet_error_text(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "#NULL!",
        WorksheetErrorCode::Div0 => "#DIV/0!",
        WorksheetErrorCode::Value => "#VALUE!",
        WorksheetErrorCode::Ref => "#REF!",
        WorksheetErrorCode::Name => "#NAME?",
        WorksheetErrorCode::Num => "#NUM!",
        WorksheetErrorCode::NA => "#N/A",
        WorksheetErrorCode::Busy => "#BUSY!",
        WorksheetErrorCode::GettingData => "#GETTING_DATA",
        WorksheetErrorCode::Spill => "#SPILL!",
        WorksheetErrorCode::Calc => "#CALC!",
        WorksheetErrorCode::Field => "#FIELD!",
        WorksheetErrorCode::Blocked => "#BLOCKED!",
        WorksheetErrorCode::Connect => "#CONNECT!",
    }
}

fn render_array_cell_text(value: &ArrayCellValue) -> String {
    match value {
        ArrayCellValue::Number(number) => render_visible_number(*number),
        ArrayCellValue::Text(text) => text.to_string_lossy(),
        ArrayCellValue::Logical(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        ArrayCellValue::Error(code) => worksheet_error_text(*code).to_string(),
        ArrayCellValue::EmptyCell => String::new(),
    }
}
