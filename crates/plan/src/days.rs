//! Small civil-date helpers. Dates are never converted to UTC (S03 §1).

use chrono::{Duration, NaiveDate};

/// `d + n` days, saturating at the bounds of `NaiveDate` instead of panicking.
pub(crate) fn plus_days(d: NaiveDate, n: i64) -> NaiveDate {
    let borne = if n >= 0 {
        NaiveDate::MAX
    } else {
        NaiveDate::MIN
    };
    Duration::try_days(n)
        .and_then(|duration| d.checked_add_signed(duration))
        .unwrap_or(borne)
}

/// January 1st of a year.
pub(crate) fn january_first(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(if year < 0 {
        NaiveDate::MIN
    } else {
        NaiveDate::MAX
    })
}
