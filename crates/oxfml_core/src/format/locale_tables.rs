pub fn month_name(month: i64, abbreviated: bool) -> &'static str {
    match (month, abbreviated) {
        (1, true) => "Jan",
        (2, true) => "Feb",
        (3, true) => "Mar",
        (4, true) => "Apr",
        (5, true) => "May",
        (6, true) => "Jun",
        (7, true) => "Jul",
        (8, true) => "Aug",
        (9, true) => "Sep",
        (10, true) => "Oct",
        (11, true) => "Nov",
        (12, true) => "Dec",
        (1, false) => "January",
        (2, false) => "February",
        (3, false) => "March",
        (4, false) => "April",
        (5, false) => "May",
        (6, false) => "June",
        (7, false) => "July",
        (8, false) => "August",
        (9, false) => "September",
        (10, false) => "October",
        (11, false) => "November",
        (12, false) => "December",
        _ => "",
    }
}

pub fn weekday_name(index: usize, abbreviated: bool) -> &'static str {
    match (index, abbreviated) {
        (0, true) => "Sun",
        (1, true) => "Mon",
        (2, true) => "Tue",
        (3, true) => "Wed",
        (4, true) => "Thu",
        (5, true) => "Fri",
        (6, true) => "Sat",
        (0, false) => "Sunday",
        (1, false) => "Monday",
        (2, false) => "Tuesday",
        (3, false) => "Wednesday",
        (4, false) => "Thursday",
        (5, false) => "Friday",
        (6, false) => "Saturday",
        _ => "",
    }
}
