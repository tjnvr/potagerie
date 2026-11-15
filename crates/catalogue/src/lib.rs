//! Potagerie crop catalogue (S02).
//!
//! `crops.json` follows S02 §4 exactly and is embedded in the binary with
//! [`include_str!`]. Use [`Catalogue::v1`] to get the parsed catalogue.

use std::fmt;
use std::sync::OnceLock;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Raw catalogue file, as shipped with the app (S02 §4).
pub const CROPS_JSON: &str = include_str!("../crops.json");

/// Ids of the reference catalogue (S02 §3): the cases of S03 §12 use it.
pub const REFERENCE_IDS: [&str; 6] = [
    "tomato",
    "lettuce",
    "carrot",
    "green-bean",
    "zucchini",
    "leek",
];

/// One crop sheet (S02 §2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Crop {
    /// Stable identifier, lowercase without accents (`tomato`).
    pub id: String,
    /// Display name (`Tomato`).
    pub name: String,
    /// Botanical family, used to rank suggestions.
    pub family: String,
    /// Days in pots before transplanting; 0 if not usually sown under cover.
    pub indoor_days: u32,
    /// Days from planting out to the start of harvest.
    pub harvest_after_days: u32,
    /// Length of the harvest period, in days.
    pub harvest_duration_days: u32,
    /// Ideal planting-out window; may span two years (`start > end`).
    pub planting_out: Period,
    /// Spacing between rows, in cm.
    pub row_cm: u32,
    /// Spacing along the row, in cm.
    pub spacing_cm: u32,
    /// Not used in V1; reserved for frost alerts.
    pub frost_sensitive: bool,
}

/// A `MM-JJ` → `MM-JJ` period of the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Period {
    pub start: MonthDay,
    pub end: MonthDay,
}

impl Period {
    /// True when the period spans two years (`start > end`, S03 §10).
    pub fn spans_two_years(&self) -> bool {
        self.start > self.end
    }
}

/// A month and day without a year (`MM-JJ`), serialised as `"05-01"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonthDay {
    month: u32,
    day: u32,
}

impl MonthDay {
    /// Builds a month-day; `02-29` is accepted.
    pub fn new(month: u32, day: u32) -> Option<Self> {
        // 2024 is a leap year, so every valid MM-JJ exists in it.
        NaiveDate::from_ymd_opt(2024, month, day).map(|_| Self { month, day })
    }

    pub fn month(&self) -> u32 {
        self.month
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    /// Places the month-day in year `year`. `02-29` becomes `02-28` in a
    /// non-leap year (S03 §1).
    pub fn in_year(&self, year: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, self.month, self.day)
            .or_else(|| NaiveDate::from_ymd_opt(year, self.month, self.day - 1))
            .expect("a valid MM-JJ always exists in a year, up to 02-29")
    }

    /// Month-day of a civil date.
    pub fn from_date(date: NaiveDate) -> Self {
        Self {
            month: date.month(),
            day: date.day(),
        }
    }
}

impl fmt::Display for MonthDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}-{:02}", self.month, self.day)
    }
}

impl std::str::FromStr for MonthDay {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("invalid MM-JJ date: {s:?}");
        let (m, j) = s.split_once('-').ok_or_else(invalid)?;
        if m.len() != 2 || j.len() != 2 {
            return Err(invalid());
        }
        let month = m.parse().map_err(|_| invalid())?;
        let day = j.parse().map_err(|_| invalid())?;
        Self::new(month, day).ok_or_else(invalid)
    }
}

impl Serialize for MonthDay {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'of> Deserialize<'of> for MonthDay {
    fn deserialize<D: Deserializer<'of>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

/// The list of crops, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalogue {
    crops: Vec<Crop>,
}

impl Catalogue {
    /// Parses a catalogue in the S02 §4 format.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            crops: serde_json::from_str(json)?,
        })
    }

    /// Builds a catalogue from crops, for tests or a future custom catalogue.
    pub fn from_crops(crops: Vec<Crop>) -> Self {
        Self { crops }
    }

    /// The embedded V1 catalogue (the 6 crops of S02 §3), parsed once.
    pub fn v1() -> &'static Catalogue {
        static V1: OnceLock<Catalogue> = OnceLock::new();
        V1.get_or_init(|| Catalogue::from_json(CROPS_JSON).expect("embedded crops.json is valid"))
    }

    /// The 6 crops of the prototype (S02 §3), on which the cases of S03 §12
    /// are written. Built once from the embedded catalogue.
    pub fn reference() -> &'static Catalogue {
        static REFERENCE: OnceLock<Catalogue> = OnceLock::new();
        REFERENCE.get_or_init(|| {
            Catalogue::from_crops(
                Catalogue::v1()
                    .iter()
                    .filter(|c| REFERENCE_IDS.contains(&c.id.as_str()))
                    .cloned()
                    .collect(),
            )
        })
    }

    /// The crop with this `id`, if any.
    pub fn crop(&self, id: &str) -> Option<&Crop> {
        self.crops.iter().find(|c| c.id == id)
    }

    pub fn crops(&self) -> &[Crop] {
        &self.crops
    }

    pub fn iter(&self) -> impl Iterator<Item = &Crop> {
        self.crops.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_has_unique_ids_and_valid_sheets() {
        let v1 = Catalogue::v1();
        assert!(v1.crops().len() >= 60);
        let mut ids: Vec<&str> = v1.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), v1.crops().len());
        for c in v1.iter() {
            assert!(c.harvest_after_days > 0, "{}", c.id);
            assert!(
                c.harvest_duration_days > 0 && c.row_cm > 0 && c.spacing_cm > 0,
                "{}",
                c.id
            );
            assert!(
                c.id.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-'),
                "{}",
                c.id
            );
        }
    }

    #[test]
    fn reference_has_the_6_crops() {
        let ids: Vec<&str> = Catalogue::reference()
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "tomato",
                "lettuce",
                "carrot",
                "green-bean",
                "zucchini",
                "leek"
            ]
        );
    }

    #[test]
    fn tomato_matches_s02() {
        let tomato = Catalogue::v1().crop("tomato").unwrap();
        assert_eq!(
            *tomato,
            Crop {
                id: "tomato".into(),
                name: "Tomato".into(),
                family: "Solanaceae".into(),
                indoor_days: 56,
                harvest_after_days: 75,
                harvest_duration_days: 70,
                planting_out: Period {
                    start: MonthDay::new(5, 1).unwrap(),
                    end: MonthDay::new(6, 15).unwrap(),
                },
                row_cm: 80,
                spacing_cm: 60,
                frost_sensitive: true,
            }
        );
    }

    #[test]
    fn json_round_trips() {
        let reread: Vec<Crop> =
            serde_json::from_str(&serde_json::to_string(Catalogue::v1().crops()).unwrap()).unwrap();
        assert_eq!(reread, Catalogue::v1().crops());
    }

    #[test]
    fn month_day_feb_29() {
        let d = MonthDay::new(2, 29).unwrap();
        assert_eq!(
            d.in_year(2028),
            NaiveDate::from_ymd_opt(2028, 2, 29).unwrap()
        );
        assert_eq!(
            d.in_year(2027),
            NaiveDate::from_ymd_opt(2027, 2, 28).unwrap()
        );
    }

    #[test]
    fn month_day_rejects_invalid_formats() {
        for s in ["5-01", "13-01", "02-30", "0501", "ab-cd"] {
            assert!(s.parse::<MonthDay>().is_err(), "{s}");
        }
        assert_eq!("09-30".parse::<MonthDay>().unwrap().to_string(), "09-30");
    }
}
