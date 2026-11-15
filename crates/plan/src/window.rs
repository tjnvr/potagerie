//! Planting-out window of a crop (S03 §10).

use catalogue::Crop;
use chrono::{Datelike, NaiveDate};

/// The window `[start, end]` that starts in year `year`.
///
/// A window within one year is `[Y-start, Y-end]`; a window across two years
/// (`start > end`, garlic for example) is `[Y-start, (Y+1)-end]`. `start` and
/// `end` are the fields of the catalogue (S02 §4).
pub fn window_starting_in(crop: &Crop, year: i32) -> (NaiveDate, NaiveDate) {
    let period = &crop.planting_out;
    let end_year = if period.spans_two_years() {
        year + 1
    } else {
        year
    };
    (period.start.in_year(year), period.end.in_year(end_year))
}

/// True if `date` falls in the window of the crop, whichever year it started in.
pub fn in_window(crop: &Crop, date: NaiveDate) -> bool {
    [date.year() - 1, date.year()].into_iter().any(|year| {
        let (start, end) = window_starting_in(crop, year);
        start <= date && date <= end
    })
}

#[cfg(test)]
mod tests {
    use catalogue::{Catalogue, MonthDay, Period};

    use super::*;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    #[test]
    fn window_within_one_year_bounds_included() {
        let tomato = Catalogue::reference().crop("tomato").unwrap();
        assert!(!in_window(tomato, d(2027, 4, 30)));
        assert!(in_window(tomato, d(2027, 5, 1)));
        assert!(in_window(tomato, d(2027, 6, 15)));
        assert!(!in_window(tomato, d(2027, 6, 16)));
    }

    #[test]
    fn window_across_two_years() {
        let mut garlic = Catalogue::reference().crop("tomato").unwrap().clone();
        garlic.planting_out = Period {
            start: MonthDay::new(10, 15).unwrap(),
            end: MonthDay::new(2, 15).unwrap(),
        };
        assert_eq!(
            window_starting_in(&garlic, 2027),
            (d(2027, 10, 15), d(2028, 2, 15))
        );
        assert!(in_window(&garlic, d(2027, 10, 15)));
        assert!(in_window(&garlic, d(2027, 12, 31)));
        assert!(in_window(&garlic, d(2028, 1, 1)));
        assert!(in_window(&garlic, d(2028, 2, 15)));
        assert!(!in_window(&garlic, d(2028, 2, 16)));
        assert!(!in_window(&garlic, d(2027, 10, 14)));
    }

    #[test]
    fn window_ending_on_feb_29_in_a_non_leap_year() {
        let mut crop = Catalogue::reference().crop("tomato").unwrap().clone();
        crop.planting_out = Period {
            start: MonthDay::new(2, 1).unwrap(),
            end: MonthDay::new(2, 29).unwrap(),
        };
        assert!(in_window(&crop, d(2027, 2, 28)));
        assert!(!in_window(&crop, d(2027, 3, 1)));
        assert!(in_window(&crop, d(2028, 2, 29)));
    }
}
