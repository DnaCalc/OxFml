use oxfunc_core::locale_format::FormatProfile;
use oxfunc_core::value::WorksheetErrorCode;

use crate::eval::{
    FunctionArrayCell, FunctionValue, eval_value_callable_summary, eval_value_is_callable,
};

pub fn render_visible_value_text(value: &FunctionValue) -> String {
    if eval_value_is_callable(value) {
        return format!(
            "Callable({})",
            eval_value_callable_summary(value).unwrap_or_default()
        );
    }
    match value {
        FunctionValue::Number(number) => render_visible_number(*number),
        FunctionValue::Text(text) => text.to_string_lossy(),
        FunctionValue::Logical(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        FunctionValue::Error(code) => worksheet_error_text(*code).to_string(),
        FunctionValue::Array(array) => array
            .get(0, 0)
            .map(render_array_cell_text)
            .unwrap_or_default(),
        FunctionValue::Reference(reference) => reference.target().to_string(),
        FunctionValue::Callable(callable) => format!("Callable({})", callable.summary),
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

fn render_array_cell_text(value: &FunctionArrayCell) -> String {
    match value {
        FunctionArrayCell::Number(number) => render_visible_number(*number),
        FunctionArrayCell::Text(text) => text.to_string_lossy(),
        FunctionArrayCell::Logical(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        FunctionArrayCell::Error(code) => worksheet_error_text(*code).to_string(),
        FunctionArrayCell::EmptyCell => String::new(),
        FunctionArrayCell::Callable(callable) => format!("Callable({})", callable.summary),
    }
}
