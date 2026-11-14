//! Small civil-date helpers. Dates are never converted to UTC (S03 §1).

use chrono::{Duration, NaiveDate};

/// `d + n` days, saturating at the bounds of `NaiveDate` instead of panicking.
pub(crate) fn plus(d: NaiveDate, n: i64) -> NaiveDate {
    let borne = if n >= 0 {
        NaiveDate::MAX
    } else {
        NaiveDate::MIN
    };
    Duration::try_days(n)
        .and_then(|duree| d.checked_add_signed(duree))
        .unwrap_or(borne)
}

/// January 1st of a year.
pub(crate) fn premier_janvier(annee: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(annee, 1, 1).unwrap_or(if annee < 0 {
        NaiveDate::MIN
    } else {
        NaiveDate::MAX
    })
}
