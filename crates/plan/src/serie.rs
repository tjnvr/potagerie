//! Dates of a serie (S03 §2).

use catalogue::Culture;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::days::plus_days;
use crate::error::PlanError;
use crate::ids::PlancheId;
use crate::zone::{Occupation, Zone};

/// How the serie starts (S01 F-PLA-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Sown in pots, then transplanted; `date` is the sowing date.
    Abri,
    /// Bought plants; `date` is the planting date.
    Plants,
    /// Sown directly in the planche; `date` is the sowing date.
    PleineTerre,
}

/// A serie of one crop on a zone of a planche.
///
/// `abri_jours`, `apres_jours` and `duree_jours` are copied from the catalogue
/// when the serie is created, so a catalogue update never shifts it (S03 §2).
/// Only the entered dates are stored; `m`, `r` and `f` are always recomputed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Serie {
    pub culture_id: String,
    pub planche_id: PlancheId,
    pub troncon_debut: u32,
    pub troncon_fin: u32,
    pub mode: Mode,
    /// Sowing date (abri, pleine terre) or planting date (plants).
    pub date: NaiveDate,
    /// Days in pots. For a crop without pots, the days entered by the user.
    pub abri_jours: u32,
    /// Days from planting out to the start of harvest.
    pub apres_jours: u32,
    /// Length of the harvest.
    pub duree_jours: u32,
    /// End modified by hand; it is not recomputed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fin_manuelle: Option<NaiveDate>,
}

/// Calculated dates of a serie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerieDates {
    /// Sowing date; none for bought plants.
    pub sowing: Option<NaiveDate>,
    /// `m`: day the plant goes into the planche.
    pub planting_out: NaiveDate,
    /// `r`: start of harvest.
    pub harvest_start: NaiveDate,
    /// `f`: end of the serie (not occupied).
    pub end: NaiveDate,
}

/// Phases of the timeline (S03 §2). Only growth and harvest occupy the planche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phases {
    /// `[sowing, m[`, for the mode abri only.
    pub pots: Option<Occupation>,
    /// `[m, r[`.
    pub growth: Occupation,
    /// `[r, f[`.
    pub harvest_start: Occupation,
}

impl Serie {
    /// Creates a serie, copying the durations of the catalogue (S03 §2).
    ///
    /// `pot_days` is only read in the mode abri for a crop whose catalogue
    /// `abri_jours` is 0 (carrot, bean): it must then be at least 1.
    pub fn new(
        culture: &Culture,
        planche_id: PlancheId,
        zone: Zone,
        mode: Mode,
        date: NaiveDate,
        pot_days: Option<u32>,
        fin_manuelle: Option<NaiveDate>,
    ) -> Result<Self, PlanError> {
        let serie = Self {
            culture_id: culture.id.clone(),
            planche_id,
            troncon_debut: zone.t1,
            troncon_fin: zone.t2,
            mode,
            date,
            abri_jours: pot_days_for(culture, mode, pot_days)?,
            apres_jours: culture.recolte_apres_jours,
            duree_jours: culture.recolte_duree_jours,
            fin_manuelle,
        };
        serie.check()?;
        Ok(serie)
    }

    /// The sowing date to store when the user enters a transplanting date
    /// `repiquage` instead: `s = x − A` (S03 §2).
    pub fn sowing_from_transplanting(repiquage: NaiveDate, abri_jours: u32) -> NaiveDate {
        plus_days(repiquage, -i64::from(abri_jours))
    }

    /// Switches crop: the durations are copied again from the catalogue (S03 §2).
    pub fn change_culture(
        &mut self,
        culture: &Culture,
        pot_days: Option<u32>,
    ) -> Result<(), PlanError> {
        let mut candidate = self.clone();
        candidate.culture_id = culture.id.clone();
        candidate.apres_jours = culture.recolte_apres_jours;
        candidate.duree_jours = culture.recolte_duree_jours;
        candidate.abri_jours = pot_days_for(culture, self.mode, pot_days)?;
        candidate.check()?;
        *self = candidate;
        Ok(())
    }

    /// Switches the starting mode. The time in pots is taken again from the
    /// catalogue, so a carrot moved from abri to pleine terre adds 0 days.
    pub fn change_mode(
        &mut self,
        mode: Mode,
        culture: &Culture,
        pot_days: Option<u32>,
    ) -> Result<(), PlanError> {
        let mut candidate = self.clone();
        candidate.mode = mode;
        candidate.abri_jours = pot_days_for(culture, mode, pot_days)?;
        candidate.check()?;
        *self = candidate;
        Ok(())
    }

    /// The zone `[t1, t2]`.
    pub fn zone(&self) -> Zone {
        Zone {
            t1: self.troncon_debut,
            t2: self.troncon_fin,
        }
    }

    /// `m`, `r` and `f` (S03 §2).
    pub fn dates(&self) -> SerieDates {
        let pot_days = i64::from(self.abri_jours);
        let after = i64::from(self.apres_jours);
        let (sowing, m, r) = match self.mode {
            Mode::Abri => {
                let m = plus_days(self.date, pot_days);
                (Some(self.date), m, plus_days(m, after))
            }
            Mode::Plants => (None, self.date, plus_days(self.date, after)),
            // The plant spends in the ground the phase it would have spent in pots.
            Mode::PleineTerre => (
                Some(self.date),
                self.date,
                plus_days(self.date, after + pot_days),
            ),
        };
        let end = self
            .fin_manuelle
            .unwrap_or_else(|| plus_days(r, i64::from(self.duree_jours)));
        SerieDates {
            sowing,
            planting_out: m,
            harvest_start: r,
            end,
        }
    }

    /// The days the serie occupies its planche: `[m, f[` (S03 §1).
    pub fn occupation(&self) -> Occupation {
        let dates = self.dates();
        Occupation {
            start: dates.planting_out,
            end: dates.end,
        }
    }

    /// Timeline phases (S03 §2).
    pub fn phases(&self) -> Phases {
        let dates = self.dates();
        Phases {
            pots: match (self.mode, dates.sowing) {
                (Mode::Abri, Some(sowing)) => Some(Occupation {
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

    /// The same serie moved by `days`: the entered date and the manual end
    /// both shift (S03 §2).
    pub fn shifted(&self, days: i64) -> Serie {
        Serie {
            date: plus_days(self.date, days),
            fin_manuelle: self.fin_manuelle.map(|end| plus_days(end, days)),
            ..self.clone()
        }
    }

    /// Checks what the serie alone can say: zone, pots, end, occupation.
    pub fn check(&self) -> Result<(), PlanError> {
        Zone::new(self.troncon_debut, self.troncon_fin)?;
        if self.mode == Mode::Abri && self.abri_jours == 0 {
            return Err(PlanError::PotDaysRequired);
        }
        let dates = self.dates();
        if self.fin_manuelle.is_some() && dates.end <= dates.harvest_start {
            return Err(PlanError::InvalidManualEnd {
                end: dates.end,
                harvest_start: dates.harvest_start,
            });
        }
        if self.occupation().is_empty() {
            return Err(PlanError::EmptyOccupation);
        }
        Ok(())
    }
}

/// `A`: the catalogue `abri_jours` if positive; otherwise, in the mode abri,
/// the days entered by the user (at least 1); otherwise 0.
fn pot_days_for(culture: &Culture, mode: Mode, pot_days: Option<u32>) -> Result<u32, PlanError> {
    if mode == Mode::Abri && culture.abri_jours == 0 {
        match pot_days {
            Some(days) if days >= 1 => Ok(days),
            _ => Err(PlanError::PotDaysRequired),
        }
    } else {
        Ok(culture.abri_jours)
    }
}
