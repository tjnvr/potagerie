//! Visual stages in the garden view (S03 §3).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::serie::{Mode, Serie};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Sowing,
    YoungShoot,
    Adult,
    Harvest,
}

/// What the garden view draws on the zone at a given day.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "appearance", content = "stage", rename_all = "snake_case")]
pub enum Appearance {
    Nothing,
    /// A label planted on the zone: sown under cover, not yet transplanted.
    Label,
    Plant(Stage),
}

impl Serie {
    /// The display at day `d` (S03 §3).
    ///
    /// `p = (d − m) / (r − m)`. Thresholds are compared with integers; the
    /// bands are `[0, 0.15[`, `[0.15, 0.50[`, `[0.50, 1[` and `[1, ∞[`.
    pub fn appearance_on(&self, d: NaiveDate) -> Appearance {
        let dates = self.dates();
        if d < dates.planting_out {
            return match (self.mode, dates.sowing) {
                (Mode::Abri, Some(sowing)) if d >= sowing => Appearance::Label,
                _ => Appearance::Nothing,
            };
        }
        if d >= dates.end {
            return Appearance::Nothing;
        }
        let elapsed = (d - dates.planting_out).num_days();
        let duration = (dates.harvest_start - dates.planting_out).num_days();
        let stade = if duration <= 0 || elapsed >= duration {
            Stage::Harvest
        } else if elapsed * 100 < duration * 15 {
            Stage::Sowing
        } else if elapsed * 100 < duration * 50 {
            Stage::YoungShoot
        } else {
            Stage::Adult
        };
        // Bought plants are never smaller than a young plant.
        let stade = if self.mode == Mode::Plants {
            stade.max(Stage::YoungShoot)
        } else {
            stade
        };
        Appearance::Plant(stade)
    }
}

#[cfg(test)]
mod tests {
    use catalogue::Catalogue;

    use super::*;
    use crate::ids::PlancheId;
    use crate::zone::Zone;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    /// Tomato sown under cover on 2027-03-10: m 05-05, r 07-19 (75 days), f 09-27.
    fn tomate(mode: Mode, date: NaiveDate) -> Serie {
        let culture = Catalogue::v1().culture("tomate").unwrap();
        Serie::new(
            culture,
            PlancheId::generate(),
            Zone::new(1, 1).unwrap(),
            mode,
            date,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn label_from_sowing_to_planting_out() {
        let s = tomate(Mode::Abri, d(2027, 3, 10));
        assert_eq!(s.appearance_on(d(2027, 3, 9)), Appearance::Nothing);
        assert_eq!(s.appearance_on(d(2027, 3, 10)), Appearance::Label);
        assert_eq!(s.appearance_on(d(2027, 5, 4)), Appearance::Label);
        assert_eq!(
            s.appearance_on(d(2027, 5, 5)),
            Appearance::Plant(Stage::Sowing)
        );
    }

    #[test]
    fn nothing_from_the_end() {
        let s = tomate(Mode::Abri, d(2027, 3, 10));
        assert_eq!(
            s.appearance_on(d(2027, 9, 26)),
            Appearance::Plant(Stage::Harvest)
        );
        assert_eq!(s.appearance_on(d(2027, 9, 27)), Appearance::Nothing);
    }

    #[test]
    fn stage_thresholds() {
        // 100 days from m to r: p = 0.15 at day 15, 0.50 at day 50, 1 at day 100.
        let mut s = tomate(Mode::PleineTerre, d(2027, 5, 1));
        s.abri_jours = 0;
        s.apres_jours = 100;
        s.duree_jours = 30;
        let m = d(2027, 5, 1);
        let day = |n: i64| m + chrono::Duration::days(n);
        assert_eq!(s.appearance_on(day(14)), Appearance::Plant(Stage::Sowing));
        assert_eq!(
            s.appearance_on(day(15)),
            Appearance::Plant(Stage::YoungShoot)
        );
        assert_eq!(
            s.appearance_on(day(49)),
            Appearance::Plant(Stage::YoungShoot)
        );
        assert_eq!(s.appearance_on(day(50)), Appearance::Plant(Stage::Adult));
        assert_eq!(s.appearance_on(day(99)), Appearance::Plant(Stage::Adult));
        assert_eq!(s.appearance_on(day(100)), Appearance::Plant(Stage::Harvest));
    }

    #[test]
    fn bought_plants_never_below_young_shoot() {
        let s = tomate(Mode::Plants, d(2027, 5, 5));
        assert_eq!(
            s.appearance_on(d(2027, 5, 5)),
            Appearance::Plant(Stage::YoungShoot)
        );
        assert_eq!(s.appearance_on(d(2027, 5, 4)), Appearance::Nothing);
    }

    #[test]
    fn pleine_terre_has_no_label() {
        let s = tomate(Mode::PleineTerre, d(2027, 5, 5));
        assert_eq!(s.appearance_on(d(2027, 5, 4)), Appearance::Nothing);
    }

    #[test]
    fn immediate_harvest_without_delay() {
        let mut s = tomate(Mode::Plants, d(2027, 5, 5));
        s.apres_jours = 0;
        assert_eq!(
            s.appearance_on(d(2027, 5, 5)),
            Appearance::Plant(Stage::Harvest)
        );
    }
}
