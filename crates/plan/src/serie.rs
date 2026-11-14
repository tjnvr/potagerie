//! Dates of a serie (S03 §2).

use catalogue::Culture;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::PlanError;
use crate::ids::PlancheId;
use crate::jours::plus;
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
    /// Days in pots. For a crop without godets, the days entered by the user.
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
pub struct DatesSerie {
    /// Sowing date; none for bought plants.
    pub semis: Option<NaiveDate>,
    /// `m`: day the plant goes into the planche.
    pub mise_en_place: NaiveDate,
    /// `r`: start of harvest.
    pub recolte: NaiveDate,
    /// `f`: end of the serie (not occupied).
    pub fin: NaiveDate,
}

/// Phases of the timeline (S03 §2). Only growth and harvest occupy the planche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phases {
    /// `[semis, m[`, for the mode abri only.
    pub godets: Option<Occupation>,
    /// `[m, r[`.
    pub croissance: Occupation,
    /// `[r, f[`.
    pub recolte: Occupation,
}

impl Serie {
    /// Creates a serie, copying the durations of the catalogue (S03 §2).
    ///
    /// `godets_jours` is only read in the mode abri for a crop whose catalogue
    /// `abri_jours` is 0 (carrot, bean): it must then be at least 1.
    pub fn nouvelle(
        culture: &Culture,
        planche_id: PlancheId,
        zone: Zone,
        mode: Mode,
        date: NaiveDate,
        godets_jours: Option<u32>,
        fin_manuelle: Option<NaiveDate>,
    ) -> Result<Self, PlanError> {
        let serie = Self {
            culture_id: culture.id.clone(),
            planche_id,
            troncon_debut: zone.t1,
            troncon_fin: zone.t2,
            mode,
            date,
            abri_jours: abri_pour(culture, mode, godets_jours)?,
            apres_jours: culture.recolte_apres_jours,
            duree_jours: culture.recolte_duree_jours,
            fin_manuelle,
        };
        serie.verifier()?;
        Ok(serie)
    }

    /// The sowing date to store when the user enters a transplanting date
    /// `repiquage` instead: `s = x − A` (S03 §2).
    pub fn semis_depuis_repiquage(repiquage: NaiveDate, abri_jours: u32) -> NaiveDate {
        plus(repiquage, -i64::from(abri_jours))
    }

    /// Switches crop: the durations are copied again from the catalogue (S03 §2).
    pub fn changer_culture(
        &mut self,
        culture: &Culture,
        godets_jours: Option<u32>,
    ) -> Result<(), PlanError> {
        let mut candidate = self.clone();
        candidate.culture_id = culture.id.clone();
        candidate.apres_jours = culture.recolte_apres_jours;
        candidate.duree_jours = culture.recolte_duree_jours;
        candidate.abri_jours = abri_pour(culture, self.mode, godets_jours)?;
        candidate.verifier()?;
        *self = candidate;
        Ok(())
    }

    /// Switches the starting mode. The time in pots is taken again from the
    /// catalogue, so a carrot moved from abri to pleine terre adds 0 days.
    pub fn changer_mode(
        &mut self,
        mode: Mode,
        culture: &Culture,
        godets_jours: Option<u32>,
    ) -> Result<(), PlanError> {
        let mut candidate = self.clone();
        candidate.mode = mode;
        candidate.abri_jours = abri_pour(culture, mode, godets_jours)?;
        candidate.verifier()?;
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
    pub fn dates(&self) -> DatesSerie {
        let abri = i64::from(self.abri_jours);
        let apres = i64::from(self.apres_jours);
        let (semis, m, r) = match self.mode {
            Mode::Abri => {
                let m = plus(self.date, abri);
                (Some(self.date), m, plus(m, apres))
            }
            Mode::Plants => (None, self.date, plus(self.date, apres)),
            // The plant spends in the ground the phase it would have spent in pots.
            Mode::PleineTerre => (Some(self.date), self.date, plus(self.date, apres + abri)),
        };
        let fin = self
            .fin_manuelle
            .unwrap_or_else(|| plus(r, i64::from(self.duree_jours)));
        DatesSerie {
            semis,
            mise_en_place: m,
            recolte: r,
            fin,
        }
    }

    /// The days the serie occupies its planche: `[m, f[` (S03 §1).
    pub fn occupation(&self) -> Occupation {
        let dates = self.dates();
        Occupation {
            debut: dates.mise_en_place,
            fin: dates.fin,
        }
    }

    /// Timeline phases (S03 §2).
    pub fn phases(&self) -> Phases {
        let dates = self.dates();
        Phases {
            godets: match (self.mode, dates.semis) {
                (Mode::Abri, Some(semis)) => Some(Occupation {
                    debut: semis,
                    fin: dates.mise_en_place,
                }),
                _ => None,
            },
            croissance: Occupation {
                debut: dates.mise_en_place,
                fin: dates.recolte,
            },
            recolte: Occupation {
                debut: dates.recolte,
                fin: dates.fin,
            },
        }
    }

    /// The same serie moved by `jours`: the entered date and the manual end
    /// both shift (S03 §2).
    pub fn decalee(&self, jours: i64) -> Serie {
        Serie {
            date: plus(self.date, jours),
            fin_manuelle: self.fin_manuelle.map(|fin| plus(fin, jours)),
            ..self.clone()
        }
    }

    /// Checks what the serie alone can say: zone, godets, end, occupation.
    pub fn verifier(&self) -> Result<(), PlanError> {
        Zone::new(self.troncon_debut, self.troncon_fin)?;
        if self.mode == Mode::Abri && self.abri_jours == 0 {
            return Err(PlanError::GodetsRequis);
        }
        let dates = self.dates();
        if self.fin_manuelle.is_some() && dates.fin <= dates.recolte {
            return Err(PlanError::FinManuelleInvalide {
                fin: dates.fin,
                recolte: dates.recolte,
            });
        }
        if self.occupation().est_vide() {
            return Err(PlanError::OccupationVide);
        }
        Ok(())
    }
}

/// `A`: the catalogue `abri_jours` if positive; otherwise, in the mode abri,
/// the days entered by the user (at least 1); otherwise 0.
fn abri_pour(culture: &Culture, mode: Mode, godets_jours: Option<u32>) -> Result<u32, PlanError> {
    if mode == Mode::Abri && culture.abri_jours == 0 {
        match godets_jours {
            Some(jours) if jours >= 1 => Ok(jours),
            _ => Err(PlanError::GodetsRequis),
        }
    } else {
        Ok(culture.abri_jours)
    }
}
