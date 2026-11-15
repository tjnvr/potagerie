//! Bounds of what the calculations accept (a document can come from a file).
//!
//! The calculations walk day by day and year by year over what a plan holds. A
//! value that no garden ever needs (a planting of 11 million years, a layout in
//! the year 2 000 000 000) would make them run for ever or overflow, so the
//! plans refuse it when it is entered or imported.

use chrono::{Datelike, NaiveDate};

use crate::error::PlanError;

/// The first and the last year a plan may use.
pub const MIN_YEAR: i32 = 1900;
pub const MAX_YEAR: i32 = 2200;
/// The longest duration (pots, time to harvest, harvest), in days: 10 years.
pub const MAX_DAYS: u32 = 3650;

pub(crate) fn check_year(what: &'static str, year: i32) -> Result<(), PlanError> {
    if (MIN_YEAR..=MAX_YEAR).contains(&year) {
        Ok(())
    } else {
        Err(PlanError::OutOfRange {
            what,
            value: year.to_string(),
        })
    }
}

pub(crate) fn check_date(what: &'static str, date: NaiveDate) -> Result<(), PlanError> {
    check_year(what, date.year()).map_err(|_| PlanError::OutOfRange {
        what,
        value: date.to_string(),
    })
}

pub(crate) fn check_days(what: &'static str, days: u32) -> Result<(), PlanError> {
    if days <= MAX_DAYS {
        Ok(())
    } else {
        Err(PlanError::OutOfRange {
            what,
            value: days.to_string(),
        })
    }
}
