//! Validation of a received plan (S03 §13, applied to imports).
//!
//! Two levels:
//! - [`Plan::check_structure`]: what would break the calculations (plot,
//!   layouts, plantings that are not valid on their own). The import is refused.
//! - [`Plan::incoherences`]: what breaks the rules of the plan without
//!   breaking the calculations (too many beds, orphan plantings, overlaps,
//!   placement, bad annotations). Nothing is deleted or corrected: the import
//!   is accepted and the list is shown "to resolve" (S03 §13 default).

use std::collections::BTreeSet;

use crate::error::PlanError;
use crate::ids::{AnnotationId, BedId, PlantingId};
use crate::model::{Layout, MAX_BEDS, Plan};
use crate::zone::Zone;

/// One thing to resolve in a received plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoherence {
    /// More than 10 beds in `year`.
    TooManyBeds { year: i32, count: usize },
    /// The bed of the planting is unknown, or absent in a year it occupies.
    OrphanPlanting {
        planting_id: PlantingId,
        bed_id: BedId,
        year: Option<i32>,
    },
    /// The zone of the planting exceeds its bed in `year`.
    ZoneOutOfBed {
        planting_id: PlantingId,
        bed_id: BedId,
        year: i32,
        segments: u32,
        zone: Zone,
    },
    /// Two plantings share segments and days (S03 §5).
    Conflict { a: PlantingId, b: PlantingId },
    /// A bed breaks a placement rule (S03 §15).
    Placement(PlanError),
    /// The planting uses a crop that is not in the catalogue.
    UnknownCrop {
        planting_id: PlantingId,
        crop_id: String,
    },
    /// The annotation points to a bed that does not exist.
    AnnotationUnknownBed {
        annotation_id: AnnotationId,
        bed_id: BedId,
    },
    /// The text or the dates of the annotation are not valid.
    AnnotationInvalid { annotation_id: AnnotationId },
}

impl Incoherence {
    /// Stable code, translated by the interface.
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooManyBeds { .. } => "too_many_beds",
            Self::OrphanPlanting { .. } => "planting_orphan",
            Self::ZoneOutOfBed { .. } => "zone_outside_bed",
            Self::Conflict { .. } => "overlap",
            Self::Placement(e) => e.code(),
            Self::UnknownCrop { .. } => "crop_unknown",
            Self::AnnotationUnknownBed { .. } => "note_bed_unknown",
            Self::AnnotationInvalid { .. } => "note_invalid",
        }
    }
}

fn crop_missing(crop_id: &str) -> bool {
    catalogue::Catalogue::v1().crop(crop_id).is_none()
}

impl Plan {
    /// Refuses a plan whose calculations would not be safe to run.
    pub fn check_structure(&self) -> Result<(), PlanError> {
        self.plot.validate()?;
        let cell_cm = self.plot.cell_cm.cm();
        for bed in self.beds.values() {
            for (year, layout) in &bed.layouts {
                crate::range::check_year("layout year", *year)?;
                if let Layout::Present(g) = layout {
                    if g.length_cm == 0 || g.length_cm % cell_cm != 0 {
                        return Err(PlanError::InvalidLength {
                            length_cm: g.length_cm,
                            cell_cm,
                        });
                    }
                }
            }
        }
        for planting in self.plantings.values() {
            planting.check()?;
        }
        Ok(())
    }

    /// Every incoherence of the plan, in a stable order. Empty when valid.
    pub fn incoherences(&self) -> Vec<Incoherence> {
        let mut out = Vec::new();
        let years: BTreeSet<i32> = self
            .beds
            .values()
            .flat_map(|p| p.layouts.keys().copied())
            .collect();
        if let (Some(&first), Some(&last)) = (years.first(), years.last()) {
            for year in first..=last {
                let count = self.beds_in(year).len();
                if count > MAX_BEDS {
                    out.push(Incoherence::TooManyBeds { year, count });
                }
            }
            if let Err(e) = self.check_placements(first) {
                out.push(Incoherence::Placement(e));
            }
        }
        for (id, planting) in &self.plantings {
            if crop_missing(&planting.crop_id) {
                out.push(Incoherence::UnknownCrop {
                    planting_id: *id,
                    crop_id: planting.crop_id.clone(),
                });
            }
            let Some(bed) = self.beds.get(&planting.bed_id) else {
                out.push(Incoherence::OrphanPlanting {
                    planting_id: *id,
                    bed_id: planting.bed_id,
                    year: None,
                });
                continue;
            };
            let zone = planting.zone();
            for year in planting.occupation().years() {
                match bed.layout_in(year) {
                    None => out.push(Incoherence::OrphanPlanting {
                        planting_id: *id,
                        bed_id: planting.bed_id,
                        year: Some(year),
                    }),
                    Some(g) => {
                        let segments = g.segments(self.plot.cell_cm);
                        if zone.t2 > segments {
                            out.push(Incoherence::ZoneOutOfBed {
                                planting_id: *id,
                                bed_id: planting.bed_id,
                                year,
                                segments,
                                zone,
                            });
                        }
                    }
                }
            }
        }
        for (a, b, _) in self.conflicts() {
            out.push(Incoherence::Conflict { a, b });
        }
        for (id, a) in &self.annotations {
            if a.check().is_err() {
                out.push(Incoherence::AnnotationInvalid { annotation_id: *id });
            }
            if let Some(bed_id) = a.bed_id {
                if !self.beds.contains_key(&bed_id) {
                    out.push(Incoherence::AnnotationUnknownBed {
                        annotation_id: *id,
                        bed_id,
                    });
                }
            }
        }
        out
    }
}
