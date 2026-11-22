//! Typed refusals. The UI translates [`PlanError::code`] (S05 §3, §7).

use chrono::NaiveDate;
use thiserror::Error;

use crate::ids::{AnnotationId, BedId, PlantingId};
use crate::zone::{Occupation, Zone};

/// A refused operation. A refused command changes nothing.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlanError {
    /// The year is before the year of today; past years are read-only (S03 §7).
    #[error("year {year} is in the past (today is {today})")]
    PastYear { year: i32, today: NaiveDate },
    /// More than `max` beds in a year (S01 F-PLN-1).
    #[error("more than {max} beds in {year}")]
    TooManyBeds { year: i32, max: usize },
    #[error("unknown bed {0}")]
    UnknownBed(BedId),
    #[error("unknown planting {0}")]
    UnknownPlanting(PlantingId),
    #[error("unknown annotation {0}")]
    UnknownAnnotation(AnnotationId),
    /// A note or a period has 1 to 80 characters (S01 F-VIS-5).
    #[error("an annotation text has 1 to 80 characters")]
    InvalidAnnotationText,
    /// A period must end on or after its start (S01 F-VIS-5).
    #[error("a period needs an end date on or after its start")]
    InvalidAnnotationDates,
    #[error("bed {0} already exists")]
    BedExists(BedId),
    #[error("planting {0} already exists")]
    PlantingExists(PlantingId),
    #[error("unknown crop {0:?}")]
    UnknownCrop(String),
    #[error("invalid zone [{t1}, {t2}]")]
    InvalidZone { t1: u32, t2: u32 },
    /// The bed does not exist in the layout of a year the planting occupies.
    #[error("bed {bed_id} does not exist in {year}")]
    BedMissing { bed_id: BedId, year: i32 },
    /// The zone does not fit in the bed (`t2 > n`).
    #[error("zone {zone} does not fit in a bed of {segments} segments in {year}")]
    ZoneOutOfBed {
        year: i32,
        segments: u32,
        zone: Zone,
    },
    /// Zones and occupations overlap with another planting of the bed (S03 §5).
    #[error("conflict with planting {with} on segments {segments} during {period}")]
    Conflict {
        with: PlantingId,
        /// The segments both plantings occupy.
        segments: Zone,
        /// `[max(m), min(f)[`.
        period: Occupation,
    },
    /// A year, a date or a duration outside the supported range (see
    /// [`crate::MIN_YEAR`], [`crate::MAX_YEAR`], [`crate::MAX_DAYS`]).
    #[error("{what} is out of range: {value}")]
    OutOfRange { what: &'static str, value: String },
    /// Mode "indoor" for a crop without pots needs at least 1 day (S03 §2).
    #[error("at least 1 day in pots is required")]
    PotDaysRequired,
    /// A manual end must be after the start of harvest (S03 §2), or after the
    /// planting out for a perennial (S03 §2 bis).
    #[error("manual end {end} must be after the start of harvest {harvest_start}")]
    InvalidManualEnd {
        end: NaiveDate,
        harvest_start: NaiveDate,
    },
    #[error("the planting occupies no day")]
    EmptyOccupation,
    /// A bed length is a whole number of cases (S01 F-PLN-1).
    #[error("length {length_cm} cm is not a positive multiple of the {cell_cm} cm case")]
    InvalidLength { length_cm: u32, cell_cm: u32 },
    /// Plot dimensions out of range (S03 §15).
    #[error("plot {length_cm} × {width_cm} cm is out of range")]
    InvalidPlot { length_cm: u32, width_cm: u32 },
    /// The aisle width must be 30, 50 or 80 cm (S01 F-PLN-2).
    #[error("aisle width {0} cm is not 30, 50 or 80")]
    InvalidAisle(u32),
    /// A bed width must be at least 10 cm (S03 §15).
    #[error("width {0} cm is not valid")]
    InvalidWidth(u32),
    /// A bed name is 1 to 30 characters (S01 F-PLN-1).
    #[error("a bed name has 1 to 30 characters")]
    InvalidName,
    /// A bed, or the plot once resized, leaves the plot (S01 F-PLN-1).
    #[error("bed {bed_id} is outside the plot in {year}")]
    OutsidePlot { bed_id: BedId, year: i32 },
    /// Two beds overlap (S01 F-PLN-1).
    #[error("beds {a} and {b} overlap in {year}")]
    BedsOverlap { a: BedId, b: BedId, year: i32 },
    /// Two beds are closer than the aisle width (S01 F-PLN-2).
    #[error("beds {a} and {b} are closer than {aisle_cm} cm in {year}")]
    AisleTooNarrow {
        a: BedId,
        b: BedId,
        year: i32,
        aisle_cm: u32,
    },
    /// No spot of the plot holds the copy of a bed while keeping the aisle (S03 §15).
    #[error("no free spot for the copy in {year}")]
    NoFreeSpot { year: i32 },
}

impl PlanError {
    /// Stable code, translated by the interface.
    pub fn code(&self) -> &'static str {
        match self {
            Self::PastYear { .. } => "year_past",
            Self::TooManyBeds { .. } => "too_many_beds",
            Self::UnknownBed(_) => "bed_unknown",
            Self::UnknownPlanting(_) => "planting_unknown",
            Self::UnknownAnnotation(_) => "annotation_unknown",
            Self::InvalidAnnotationText => "text_invalid",
            Self::InvalidAnnotationDates => "annotation_dates_invalid",
            Self::BedExists(_) => "bed_existing",
            Self::PlantingExists(_) => "planting_existing",
            Self::UnknownCrop(_) => "crop_unknown",
            Self::InvalidZone { .. } => "zone_invalid",
            Self::BedMissing { .. } => "bed_absent",
            Self::ZoneOutOfBed { .. } => "zone_outside_bed",
            Self::Conflict { .. } => "conflict",
            Self::OutOfRange { .. } => "out_of_range",
            Self::PotDaysRequired => "pot_days_required",
            Self::InvalidManualEnd { .. } => "manual_end_invalid",
            Self::EmptyOccupation => "occupation_empty",
            Self::InvalidLength { .. } => "length_invalid",
            Self::InvalidPlot { .. } => "plot_invalid",
            Self::InvalidAisle(_) => "aisle_invalid",
            Self::InvalidWidth(_) => "width_invalid",
            Self::InvalidName => "name_invalid",
            Self::OutsidePlot { .. } => "bed_outside_plot",
            Self::BedsOverlap { .. } => "bed_overlaps",
            Self::AisleTooNarrow { .. } => "aisle_too_narrow",
            Self::NoFreeSpot { .. } => "no_free_spot",
        }
    }
}
