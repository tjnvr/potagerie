//! Dates of a planting (S03 §2).

use catalogue::{Crop, MonthDay, Perennial};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::days::plus_days;
use crate::error::PlanError;
use crate::ids::BedId;
use crate::zone::{Occupation, Zone};

/// The longest lifespan of a perennial, in years (S03 §17).
pub const MAX_LIFESPAN_YEARS: u32 = 20;

/// How the planting starts (S01 F-PLA-1).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Sown in pots, then transplanted; `date` is the sowing date.
    Indoor,
    /// Bought plants; `date` is the planting date.
    Plants,
    /// Sown directly in the bed; `date` is the sowing date.
    OpenGround,
}

/// A planting of one crop on a zone of a bed.
///
/// `indoor_days`, `after_days` and `duration_days` are copied from the catalogue
/// when the planting is created, so a catalogue update never shifts it (S03 §2).
/// Only the entered dates are stored; `m`, `r` and `f` are always recomputed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planting {
    pub crop_id: String,
    pub bed_id: BedId,
    pub segment_start: u32,
    pub segment_end: u32,
    pub mode: Mode,
    /// Sowing date (indoor, open ground) or planting date (plants).
    pub date: NaiveDate,
    /// Days in pots. For a crop without pots, the days entered by the user.
    pub indoor_days: u32,
    /// Days from planting out to the start of harvest.
    pub after_days: u32,
    /// Length of the harvest (each year, for a perennial).
    pub duration_days: u32,
    /// Lifespan and yearly harvest of a perennial, copied from the catalogue
    /// like the durations (S03 §2 bis). Absent for an annual crop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perennial: Option<Perennial>,
    /// End modified by hand; it is not recomputed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_end: Option<NaiveDate>,
}

/// Calculated dates of a planting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantingDates {
    /// Sowing date; none for bought plants.
    pub sowing: Option<NaiveDate>,
    /// `m`: day the plant goes into the bed.
    pub planting_out: NaiveDate,
    /// `r`: start of harvest.
    pub harvest_start: NaiveDate,
    /// `f`: end of the planting (not occupied).
    pub end: NaiveDate,
}

/// Phases of the timeline (S03 §2). Only growth and harvest occupy the bed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phases {
    /// `[sowing, m[`, for the mode indoor only.
    pub pots: Option<Occupation>,
    /// `[m, r[`.
    pub growth: Occupation,
    /// `[r, f[`.
    pub harvest_start: Occupation,
}

impl Planting {
    /// Creates a planting, copying the durations of the catalogue (S03 §2).
    ///
    /// `pot_days` is only read in the mode indoor for a crop whose catalogue
    /// `indoor_days` is 0 (carrot, bean): it must then be at least 1.
    pub fn new(
        crop: &Crop,
        bed_id: BedId,
        zone: Zone,
        mode: Mode,
        date: NaiveDate,
        pot_days: Option<u32>,
        manual_end: Option<NaiveDate>,
    ) -> Result<Self, PlanError> {
        let planting = Self {
            crop_id: crop.id.clone(),
            bed_id,
            segment_start: zone.t1,
            segment_end: zone.t2,
            mode,
            date,
            indoor_days: pot_days_for(crop, mode, pot_days)?,
            after_days: crop.harvest_after_days,
            duration_days: crop.harvest_duration_days,
            perennial: crop.perennial,
            manual_end,
        };
        planting.check()?;
        Ok(planting)
    }

    /// The sowing date to store when the user enters a transplanting date
    /// `transplanting` instead: `s = x − A` (S03 §2).
    pub fn sowing_from_transplanting(transplanting: NaiveDate, indoor_days: u32) -> NaiveDate {
        plus_days(transplanting, -i64::from(indoor_days))
    }

    /// Switches crop: the durations are copied again from the catalogue (S03 §2).
    pub fn change_crop(&mut self, crop: &Crop, pot_days: Option<u32>) -> Result<(), PlanError> {
        let mut candidate = self.clone();
        candidate.crop_id = crop.id.clone();
        candidate.after_days = crop.harvest_after_days;
        candidate.duration_days = crop.harvest_duration_days;
        candidate.perennial = crop.perennial;
        candidate.indoor_days = pot_days_for(crop, self.mode, pot_days)?;
        candidate.check()?;
        *self = candidate;
        Ok(())
    }

    /// Switches the starting mode. The time in pots is taken again from the
    /// catalogue, so a carrot moved from indoor to open ground adds 0 days.
    pub fn change_mode(
        &mut self,
        mode: Mode,
        crop: &Crop,
        pot_days: Option<u32>,
    ) -> Result<(), PlanError> {
        let mut candidate = self.clone();
        candidate.mode = mode;
        candidate.indoor_days = pot_days_for(crop, mode, pot_days)?;
        candidate.check()?;
        *self = candidate;
        Ok(())
    }

    /// The zone `[t1, t2]`.
    pub fn zone(&self) -> Zone {
        Zone {
            t1: self.segment_start,
            t2: self.segment_end,
        }
    }

    /// `m`, `r` and `f` (S03 §2). A perennial first harvests on the first
    /// `harvest_from` on or after that `r`, and ends after its lifespan (§2 bis).
    pub fn dates(&self) -> PlantingDates {
        let pot_days = i64::from(self.indoor_days);
        let after = i64::from(self.after_days);
        let (sowing, m, r) = match self.mode {
            Mode::Indoor => {
                let m = plus_days(self.date, pot_days);
                (Some(self.date), m, plus_days(m, after))
            }
            Mode::Plants => (None, self.date, plus_days(self.date, after)),
            // The plant spends in the ground the phase it would have spent in pots.
            Mode::OpenGround => (
                Some(self.date),
                self.date,
                plus_days(self.date, after + pot_days),
            ),
        };
        let (r, default_end) = match self.perennial {
            None => (r, plus_days(r, i64::from(self.duration_days))),
            Some(p) => {
                let first = p.harvest_from.in_year(r.year());
                let r = if first >= r {
                    first
                } else {
                    p.harvest_from.in_year(r.year().saturating_add(1))
                };
                let years = i32::try_from(p.lifespan_years).unwrap_or(i32::MAX);
                let end = MonthDay::from_date(m).in_year(m.year().saturating_add(years));
                (r, end)
            }
        };
        let end = self.manual_end.unwrap_or(default_end);
        PlantingDates {
            sowing,
            planting_out: m,
            harvest_start: r,
            end,
        }
    }

    /// The days the planting occupies its bed: `[m, f[` (S03 §1).
    pub fn occupation(&self) -> Occupation {
        let dates = self.dates();
        Occupation {
            start: dates.planting_out,
            end: dates.end,
        }
    }

    /// The harvests, in date order: `[r, f[` for an annual crop; one a year,
    /// cut to `[m, f[`, for a perennial (S03 §2 bis).
    pub fn harvests(&self) -> Vec<Occupation> {
        let dates = self.dates();
        let Some(p) = self.perennial else {
            return vec![Occupation {
                start: dates.harvest_start,
                end: dates.end,
            }];
        };
        let mut harvests = Vec::new();
        let mut year = dates.harvest_start.year();
        loop {
            let start = p.harvest_from.in_year(year);
            if start >= dates.end || year == i32::MAX {
                break;
            }
            let harvest = Occupation {
                start,
                end: plus_days(start, i64::from(self.duration_days)).min(dates.end),
            };
            if start >= dates.harvest_start && !harvest.is_empty() {
                harvests.push(harvest);
            }
            year += 1;
        }
        harvests
    }

    /// Timeline phases (S03 §2).
    pub fn phases(&self) -> Phases {
        let dates = self.dates();
        Phases {
            pots: match (self.mode, dates.sowing) {
                (Mode::Indoor, Some(sowing)) => Some(Occupation {
                    start: sowing,
                    end: dates.planting_out,
                }),
                _ => None,
            },
            growth: Occupation {
                start: dates.planting_out,
                end: dates.harvest_start,
            },
            harvest_start: Occupation {
                start: dates.harvest_start,
                end: dates.end,
            },
        }
    }

    /// The same planting moved by `days`: the entered date and the manual end
    /// both shift (S03 §2).
    pub fn shifted(&self, days: i64) -> Planting {
        Planting {
            date: plus_days(self.date, days),
            manual_end: self.manual_end.map(|end| plus_days(end, days)),
            ..self.clone()
        }
    }

    /// Checks what the planting alone can say: zone, pots, end, occupation.
    pub fn check(&self) -> Result<(), PlanError> {
        Zone::new(self.segment_start, self.segment_end)?;
        crate::range::check_date("planting date", self.date)?;
        if let Some(end) = self.manual_end {
            crate::range::check_date("manual end", end)?;
        }
        crate::range::check_days("days in pots", self.indoor_days)?;
        crate::range::check_days("days to harvest", self.after_days)?;
        crate::range::check_days("harvest length", self.duration_days)?;
        if self.mode == Mode::Indoor && self.indoor_days == 0 {
            return Err(PlanError::PotDaysRequired);
        }
        if let Some(p) = self.perennial {
            if !(1..=MAX_LIFESPAN_YEARS).contains(&p.lifespan_years) {
                return Err(PlanError::OutOfRange {
                    what: "lifespan",
                    value: p.lifespan_years.to_string(),
                });
            }
        }
        let dates = self.dates();
        // A perennial may be ended before its first harvest (S03 §2 bis).
        let bound = match self.perennial {
            None => dates.harvest_start,
            Some(_) => dates.planting_out,
        };
        if self.manual_end.is_some() && dates.end <= bound {
            return Err(PlanError::InvalidManualEnd {
                end: dates.end,
                harvest_start: bound,
            });
        }
        if self.occupation().is_empty() {
            return Err(PlanError::EmptyOccupation);
        }
        Ok(())
    }
}

/// `A`: the catalogue `indoor_days` if positive; otherwise, in the mode indoor,
/// the days entered by the user (at least 1); otherwise 0.
fn pot_days_for(crop: &Crop, mode: Mode, pot_days: Option<u32>) -> Result<u32, PlanError> {
    if mode == Mode::Indoor && crop.indoor_days == 0 {
        match pot_days {
            Some(days) if days >= 1 => Ok(days),
            _ => Err(PlanError::PotDaysRequired),
        }
    } else {
        Ok(crop.indoor_days)
    }
}
