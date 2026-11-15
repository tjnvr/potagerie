//! Zones (intervals of segments) and half-open occupations (S03 §1, §4).

use std::fmt;
use std::ops::RangeInclusive;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::days::{january_first, plus_days};
use crate::error::PlanError;

/// An interval `[t1, t2]` of segments, numbered from 1 (S03 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Zone {
    pub t1: u32,
    pub t2: u32,
}

impl Zone {
    /// A zone with `1 ≤ t1 ≤ t2`. The upper bound depends on the bed
    /// and is checked by the plan.
    pub fn new(t1: u32, t2: u32) -> Result<Self, PlanError> {
        if t1 >= 1 && t1 <= t2 {
            Ok(Self { t1, t2 })
        } else {
            Err(PlanError::InvalidZone { t1, t2 })
        }
    }

    /// Number of segments in the zone.
    pub fn segment_count(&self) -> u32 {
        self.t2.saturating_sub(self.t1) + 1
    }

    /// True when the zones share at least one segment (S03 §5).
    pub fn overlaps(&self, other: &Zone) -> bool {
        self.t1 <= other.t2 && other.t1 <= self.t2
    }

    /// The segments the two zones have in common.
    pub fn intersection(&self, other: &Zone) -> Option<Zone> {
        let t1 = self.t1.max(other.t1);
        let t2 = self.t2.min(other.t2);
        (t1 <= t2).then_some(Zone { t1, t2 })
    }

    /// True when `other` lies entirely inside `self`.
    pub fn contains(&self, other: &Zone) -> bool {
        self.t1 <= other.t1 && other.t2 <= self.t2
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.t1, self.t2)
    }
}

/// A half-open period `[start, end[` (S03 §1): `start` is occupied, `end` is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Occupation {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl Occupation {
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// `a.start < b.end and b.start < a.end` (S03 §5).
    pub fn overlaps(&self, other: &Occupation) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// `[max(debuts), min(fins)[`, if it is not empty.
    pub fn intersection(&self, other: &Occupation) -> Option<Occupation> {
        let commune = Occupation {
            start: self.start.max(other.start),
            end: self.end.min(other.end),
        };
        (!commune.is_empty()).then_some(commune)
    }

    /// True if the day is occupied.
    pub fn contains(&self, day: NaiveDate) -> bool {
        self.start <= day && day < self.end
    }

    /// The calendar years the occupation touches. A plantings ending on
    /// January 1st does not touch the new year.
    pub fn years(&self) -> RangeInclusive<i32> {
        let last = if self.is_empty() {
            self.start
        } else {
            plus_days(self.end, -1)
        };
        self.start.year()..=last.year()
    }

    /// True if at least one occupied day falls in `year`.
    pub fn touches_year(&self, year: i32) -> bool {
        !self.is_empty()
            && self.start < january_first(year.saturating_add(1))
            && self.end > january_first(year)
    }
}

impl fmt::Display for Occupation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}[", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    #[test]
    fn zone_rejects_invalid_bounds() {
        assert!(Zone::new(0, 3).is_err());
        assert!(Zone::new(4, 3).is_err());
        assert_eq!(Zone::new(2, 2).unwrap().segment_count(), 1);
    }

    #[test]
    fn adjacent_zones_do_not_overlap() {
        let a = Zone::new(1, 2).unwrap();
        let b = Zone::new(3, 4).unwrap();
        assert!(!a.overlaps(&b));
        assert!(a.overlaps(&Zone::new(2, 4).unwrap()));
        assert_eq!(
            a.intersection(&Zone::new(2, 4).unwrap()),
            Zone::new(2, 2).ok()
        );
    }

    #[test]
    fn half_open_occupation() {
        let a = Occupation {
            start: d(2027, 5, 5),
            end: d(2027, 9, 27),
        };
        let b = Occupation {
            start: d(2027, 9, 27),
            end: d(2027, 12, 6),
        };
        assert!(!a.overlaps(&b));
        assert!(a.contains(d(2027, 9, 26)));
        assert!(!a.contains(d(2027, 9, 27)));
    }

    #[test]
    fn occupation_ending_on_january_1st_does_not_touch_the_new_year() {
        let o = Occupation {
            start: d(2027, 11, 1),
            end: d(2028, 1, 1),
        };
        assert_eq!(o.years(), 2027..=2027);
        assert!(o.touches_year(2027));
        assert!(!o.touches_year(2028));
        let o = Occupation {
            start: d(2027, 11, 1),
            end: d(2028, 1, 2),
        };
        assert_eq!(o.years(), 2027..=2028);
        assert!(o.touches_year(2028));
    }
}
