//! Typed refusals. The UI translates [`PlanError::code`] (S05 §3, §7).

use chrono::NaiveDate;
use thiserror::Error;

use crate::ids::{AnnotationId, PlancheId, SerieId};
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
    #[error("unknown annotation {0}")]
    UnknownAnnotation(AnnotationId),
    /// A note or a period has 1 to 80 characters (S01 F-VIS-5).
    #[error("an annotation text has 1 to 80 characters")]
    InvalidAnnotationText,
    /// A period must end on or after its start (S01 F-VIS-5).
    #[error("a period needs an end date on or after its start")]
    InvalidAnnotationDates,
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
    /// Terrain dimensions out of range (S03 §15).
    #[error("terrain {longueur_cm} × {largeur_cm} cm is out of range")]
    InvalidTerrain { longueur_cm: u32, largeur_cm: u32 },
    /// The allée width must be 30, 50 or 80 cm (S01 F-PLN-2).
    #[error("allée width {0} cm is not 30, 50 or 80")]
    InvalidAllee(u32),
    /// A planche width must be at least 10 cm (S03 §15).
    #[error("width {0} cm is not valid")]
    InvalidWidth(u32),
    /// A planche name is 1 to 30 characters (S01 F-PLN-1).
    #[error("a planche name has 1 to 30 characters")]
    InvalidName,
    /// A planche, or the terrain once resized, leaves the terrain (S01 F-PLN-1).
    #[error("planche {planche_id} is outside the terrain in {year}")]
    OutsideTerrain { planche_id: PlancheId, year: i32 },
    /// Two planches overlap (S01 F-PLN-1).
    #[error("planches {a} and {b} overlap in {year}")]
    PlanchesOverlap {
        a: PlancheId,
        b: PlancheId,
        year: i32,
    },
    /// Two planches are closer than the allée width (S01 F-PLN-2).
    #[error("planches {a} and {b} are closer than {allee_cm} cm in {year}")]
    AlleeTooNarrow {
        a: PlancheId,
        b: PlancheId,
        year: i32,
        allee_cm: u32,
    },
}

impl PlanError {
    /// Stable code, translated by the interface.
    pub fn code(&self) -> &'static str {
        match self {
            Self::PastYear { .. } => "annee_passee",
            Self::TooManyPlanches { .. } => "too_many_planches",
            Self::UnknownPlanche(_) => "planche_inconnue",
            Self::UnknownSerie(_) => "serie_inconnue",
            Self::UnknownAnnotation(_) => "annotation_inconnue",
            Self::InvalidAnnotationText => "texte_invalide",
            Self::InvalidAnnotationDates => "annotation_dates_invalides",
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
            Self::InvalidTerrain { .. } => "terrain_invalide",
            Self::InvalidAllee(_) => "allee_invalide",
            Self::InvalidWidth(_) => "largeur_invalide",
            Self::InvalidName => "nom_invalide",
            Self::OutsideTerrain { .. } => "planche_hors_terrain",
            Self::PlanchesOverlap { .. } => "planche_chevauche",
            Self::AlleeTooNarrow { .. } => "allee_trop_etroite",
        }
    }
}
