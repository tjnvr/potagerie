//! Data model (S05 §5): one document, maps indexed by UUID v7, nothing derived.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{AnnotationId, BedId, PlanId, PlantingId};
use crate::planting::Planting;

/// Maximum number of beds in a year (S01 F-PLN-1).
pub const MAX_BEDS: usize = 10;
/// Longest bed name, in characters (S01 F-PLN-1).
pub const MAX_NAME_CHARS: usize = 30;
/// Narrowest bed, in cm (S03 §15).
pub const MIN_WIDTH_CM: u32 = 10;

/// The whole document of a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    /// Bumped by each command that changes the plan (lots 1 and 2 only).
    pub version: u64,
    /// Technical timestamp, in UTC (S03 §1).
    pub updated_at: DateTime<Utc>,
    pub plot: Plot,
    #[serde(default)]
    pub beds: BTreeMap<BedId, Bed>,
    #[serde(default)]
    pub plantings: BTreeMap<PlantingId, Planting>,
    #[serde(default)]
    pub annotations: BTreeMap<AnnotationId, Annotation>,
}

/// Size of a case, fixed when the plot is created (S03 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub enum CellCm {
    C25,
    C50,
    C100,
}

impl CellCm {
    pub const fn cm(self) -> u32 {
        match self {
            Self::C25 => 25,
            Self::C50 => 50,
            Self::C100 => 100,
        }
    }
}

impl TryFrom<u32> for CellCm {
    type Error = String;

    fn try_from(cm: u32) -> Result<Self, Self::Error> {
        match cm {
            25 => Ok(Self::C25),
            50 => Ok(Self::C50),
            100 => Ok(Self::C100),
            other => Err(format!("case size must be 25, 50 or 100 cm, got {other}")),
        }
    }
}

impl From<CellCm> for u32 {
    fn from(case: CellCm) -> Self {
        case.cm()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plot {
    pub length_cm: u32,
    pub width_cm: u32,
    pub cell_cm: CellCm,
    pub aisle_cm: u32,
}

/// A bed. Its layout lives in `layouts`, only for the years where it
/// changes (S03 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bed {
    pub name: String,
    #[serde(default)]
    pub layouts: BTreeMap<i32, Layout>,
}

/// The layout of a bed for one year.
///
/// Serialised with an `state` tag, as in S05 §5: `{"state":"absent"}` for a deletion, or
/// `{"state":"present","type":"bed","x":0,...}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Layout {
    /// The bed no longer exists from this year on.
    #[serde(rename = "absent")]
    Absent,
    #[serde(rename = "present")]
    Present(Geometry),
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeBed {
    Bed,
    Square,
    Mound,
    Planter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub bed_type: TypeBed,
    pub x: u32,
    pub y: u32,
    /// A whole number of cases.
    pub length_cm: u32,
    /// Free, to the centimetre.
    pub width_cm: u32,
}

impl Geometry {
    /// Number of segments `n = length ÷ case` (S03 §4).
    pub fn segments(&self, case: CellCm) -> u32 {
        self.length_cm / case.cm()
    }
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeAnnotation {
    Note,
    Period,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(rename = "type")]
    pub type_annotation: TypeAnnotation,
    pub date_start: NaiveDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_end: Option<NaiveDate>,
    /// Empty means the whole garden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bed_id: Option<BedId>,
    pub text: String,
    #[serde(default)]
    pub hidden: bool,
}

impl Plan {
    /// An empty plan. The timestamp is a parameter: the crate never reads the clock.
    pub fn new(id: PlanId, plot: Plot, now: DateTime<Utc>) -> Self {
        Self {
            id,
            version: 1,
            updated_at: now,
            plot,
            beds: BTreeMap::new(),
            plantings: BTreeMap::new(),
            annotations: BTreeMap::new(),
        }
    }

    /// Records a change: `version + 1` and a new UTC timestamp. Commands call
    /// it after each successful modification (S05 §7).
    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.version += 1;
        self.updated_at = now;
    }
}
