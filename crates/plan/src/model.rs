//! Data model (S05 §5): one document, maps indexed by UUID v7, nothing derived.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{AnnotationId, PlanId, PlancheId, SerieId};
use crate::serie::Serie;

/// Maximum number of planches in a year (S01 F-PLN-1).
pub const MAX_PLANCHES: usize = 10;

/// The whole document of a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    /// Bumped by each command that changes the plan (lots 1 and 2 only).
    pub version: u64,
    /// Technical timestamp, in UTC (S03 §1).
    pub mis_a_jour_le: DateTime<Utc>,
    pub terrain: Terrain,
    #[serde(default)]
    pub planches: BTreeMap<PlancheId, Planche>,
    #[serde(default)]
    pub series: BTreeMap<SerieId, Serie>,
    #[serde(default)]
    pub annotations: BTreeMap<AnnotationId, Annotation>,
}

/// Size of a case, fixed when the terrain is created (S03 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum CaseCm {
    C25,
    C50,
    C100,
}

impl CaseCm {
    pub const fn cm(self) -> u32 {
        match self {
            Self::C25 => 25,
            Self::C50 => 50,
            Self::C100 => 100,
        }
    }
}

impl TryFrom<u32> for CaseCm {
    type Error = String;

    fn try_from(cm: u32) -> Result<Self, Self::Error> {
        match cm {
            25 => Ok(Self::C25),
            50 => Ok(Self::C50),
            100 => Ok(Self::C100),
            autre => Err(format!("case size must be 25, 50 or 100 cm, got {autre}")),
        }
    }
}

impl From<CaseCm> for u32 {
    fn from(case: CaseCm) -> Self {
        case.cm()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terrain {
    pub longueur_cm: u32,
    pub largeur_cm: u32,
    pub case_cm: CaseCm,
    pub allee_cm: u32,
}

/// A planche. Its layout lives in `dispositions`, only for the years where it
/// changes (S03 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planche {
    pub nom: String,
    #[serde(default)]
    pub dispositions: BTreeMap<i32, Disposition>,
}

/// The layout of a planche for one year.
///
/// Serialised with an `etat` tag: `{"etat":"absente"}` for a deletion, or
/// `{"etat":"presente","type":"planche","x":0,...}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "etat", rename_all = "snake_case")]
pub enum Disposition {
    /// The planche no longer exists from this year on.
    Absente,
    Presente(Geometrie),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypePlanche {
    Planche,
    Carre,
    Butte,
    Bac,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geometrie {
    #[serde(rename = "type")]
    pub type_planche: TypePlanche,
    pub x: u32,
    pub y: u32,
    /// A whole number of cases.
    pub longueur_cm: u32,
    /// Free, to the centimetre.
    pub largeur_cm: u32,
}

impl Geometrie {
    /// Number of troncons `n = longueur ÷ case` (S03 §4).
    pub fn troncons(&self, case: CaseCm) -> u32 {
        self.longueur_cm / case.cm()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeAnnotation {
    Note,
    Periode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(rename = "type")]
    pub type_annotation: TypeAnnotation,
    pub date_debut: NaiveDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_fin: Option<NaiveDate>,
    /// Empty means the whole potager.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planche_id: Option<PlancheId>,
    pub texte: String,
    #[serde(default)]
    pub masquee: bool,
}

impl Plan {
    /// An empty plan. The timestamp is a parameter: the crate never reads the clock.
    pub fn nouveau(id: PlanId, terrain: Terrain, maintenant: DateTime<Utc>) -> Self {
        Self {
            id,
            version: 1,
            mis_a_jour_le: maintenant,
            terrain,
            planches: BTreeMap::new(),
            series: BTreeMap::new(),
            annotations: BTreeMap::new(),
        }
    }

    /// Records a change: `version + 1` and a new UTC timestamp. Commands call
    /// it after each successful modification (S05 §7).
    pub fn toucher(&mut self, maintenant: DateTime<Utc>) {
        self.version += 1;
        self.mis_a_jour_le = maintenant;
    }
}
