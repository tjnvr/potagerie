//! Typed refusals. The UI translates [`PlanError::code`] (S05 §3, §7).

use chrono::NaiveDate;
use thiserror::Error;

use crate::ids::{PlancheId, SerieId};
use crate::zone::{Occupation, Zone};

/// A refused operation. A refused command changes nothing.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlanError {
    /// The year is before the year of today; past years are read-only (S03 §7).
    #[error("year {annee} is in the past (today is {aujourd_hui})")]
    AnneePassee { annee: i32, aujourd_hui: NaiveDate },
    /// More than `max` planches in a year (S01 F-PLN-1).
    #[error("more than {max} planches in {annee}")]
    TropDePlanches { annee: i32, max: usize },
    #[error("unknown planche {0}")]
    PlancheInconnue(PlancheId),
    #[error("unknown serie {0}")]
    SerieInconnue(SerieId),
    #[error("planche {0} already exists")]
    PlancheExistante(PlancheId),
    #[error("serie {0} already exists")]
    SerieExistante(SerieId),
    #[error("unknown culture {0:?}")]
    CultureInconnue(String),
    #[error("invalid zone [{t1}, {t2}]")]
    ZoneInvalide { t1: u32, t2: u32 },
    /// The planche does not exist in the disposition of a year the serie occupies.
    #[error("planche {planche_id} does not exist in {annee}")]
    PlancheAbsente { planche_id: PlancheId, annee: i32 },
    /// The zone does not fit in the planche (`t2 > n`).
    #[error("zone {zone} does not fit in a planche of {troncons} troncons in {annee}")]
    ZoneHorsPlanche {
        annee: i32,
        troncons: u32,
        zone: Zone,
    },
    /// Zones and occupations overlap with another serie of the planche (S03 §5).
    #[error("conflict with serie {avec} on troncons {troncons} during {periode}")]
    Conflit {
        avec: SerieId,
        /// The troncons both series occupy.
        troncons: Zone,
        /// `[max(m), min(f)[`.
        periode: Occupation,
    },
    /// Mode "à l'abri" for a crop without godets needs at least 1 day (S03 §2).
    #[error("a godets duration of at least 1 day is required")]
    GodetsRequis,
    /// A manual end must be after the start of harvest (S03 §2).
    #[error("manual end {fin} must be after the start of harvest {recolte}")]
    FinManuelleInvalide { fin: NaiveDate, recolte: NaiveDate },
    #[error("the serie occupies no day")]
    OccupationVide,
    /// A planche length is a whole number of cases (S01 F-PLN-1).
    #[error("length {longueur_cm} cm is not a positive multiple of the {case_cm} cm case")]
    LongueurInvalide { longueur_cm: u32, case_cm: u32 },
}

impl PlanError {
    /// Stable code, translated by the interface.
    pub fn code(&self) -> &'static str {
        match self {
            Self::AnneePassee { .. } => "annee_passee",
            Self::TropDePlanches { .. } => "trop_de_planches",
            Self::PlancheInconnue(_) => "planche_inconnue",
            Self::SerieInconnue(_) => "serie_inconnue",
            Self::PlancheExistante(_) => "planche_existante",
            Self::SerieExistante(_) => "serie_existante",
            Self::CultureInconnue(_) => "culture_inconnue",
            Self::ZoneInvalide { .. } => "zone_invalide",
            Self::PlancheAbsente { .. } => "planche_absente",
            Self::ZoneHorsPlanche { .. } => "zone_hors_planche",
            Self::Conflit { .. } => "conflit",
            Self::GodetsRequis => "godets_requis",
            Self::FinManuelleInvalide { .. } => "fin_manuelle_invalide",
            Self::OccupationVide => "occupation_vide",
            Self::LongueurInvalide { .. } => "longueur_invalide",
        }
    }
}
