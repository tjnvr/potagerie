//! Typed refusals. The UI translates [`PlanError::code`] (S05 §3, §7).

use chrono::NaiveDate;
use thiserror::Error;

use crate::ids::{PlancheId, SerieId};
use crate::zone::{Occupation, Zone};

/// A refused operation. A refused command changes nothing.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlanError {
    /// The year is before the year of today; past years are read-only (S03 §7).
    #[error("year {year} is in the past (today is {today})")]
    PastYear { year: i32, today: NaiveDate },
    /// More than `max` planches in a year (S01 F-PLN-1).
    #[error("more than {max} planches in {year}")]
    TooManyPlanches { year: i32, max: usize },
    #[error("unknown planche {0}")]
    UnknownPlanche(PlancheId),
    #[error("unknown serie {0}")]
    UnknownSerie(SerieId),
    #[error("planche {0} already exists")]
    PlancheExists(PlancheId),
    #[error("serie {0} already exists")]
    SerieExists(SerieId),
    #[error("unknown culture {0:?}")]
    UnknownCulture(String),
    #[error("invalid zone [{t1}, {t2}]")]
    InvalidZone { t1: u32, t2: u32 },
    /// The planche does not exist in the disposition of a year the serie occupies.
    #[error("planche {planche_id} does not exist in {year}")]
    PlancheMissing { planche_id: PlancheId, year: i32 },
    /// The zone does not fit in the planche (`t2 > n`).
    #[error("zone {zone} does not fit in a planche of {troncons} troncons in {year}")]
    ZoneOutOfPlanche {
        year: i32,
        troncons: u32,
        zone: Zone,
    },
    /// Zones and occupations overlap with another serie of the planche (S03 §5).
    #[error("conflict with serie {with} on troncons {troncons} during {period}")]
    Conflict {
        with: SerieId,
        /// The troncons both series occupy.
        troncons: Zone,
        /// `[max(m), min(f)[`.
        period: Occupation,
    },
    /// Mode "à l'abri" for a crop without pots needs at least 1 day (S03 §2).
    #[error("at least 1 day in pots is required")]
    PotDaysRequired,
    /// A manual end must be after the start of harvest (S03 §2).
    #[error("manual end {end} must be after the start of harvest {harvest_start}")]
    InvalidManualEnd {
        end: NaiveDate,
        harvest_start: NaiveDate,
    },
    #[error("the serie occupies no day")]
    EmptyOccupation,
    /// A planche length is a whole number of cases (S01 F-PLN-1).
    #[error("length {longueur_cm} cm is not a positive multiple of the {case_cm} cm case")]
    InvalidLength { longueur_cm: u32, case_cm: u32 },
}

impl PlanError {
    /// Stable code, translated by the interface.
    pub fn code(&self) -> &'static str {
        match self {
            Self::PastYear { .. } => "annee_passee",
            Self::TooManyPlanches { .. } => "too_many_planches",
            Self::UnknownPlanche(_) => "planche_inconnue",
            Self::UnknownSerie(_) => "serie_inconnue",
            Self::PlancheExists(_) => "planche_existante",
            Self::SerieExists(_) => "serie_existante",
            Self::UnknownCulture(_) => "culture_inconnue",
            Self::InvalidZone { .. } => "zone_invalide",
            Self::PlancheMissing { .. } => "planche_absente",
            Self::ZoneOutOfPlanche { .. } => "zone_hors_planche",
            Self::Conflict { .. } => "conflict",
            Self::PotDaysRequired => "pot_days_required",
            Self::InvalidManualEnd { .. } => "fin_manuelle_invalide",
            Self::EmptyOccupation => "occupation_vide",
            Self::InvalidLength { .. } => "longueur_invalide",
        }
    }
}
