//! Validation of a received plan (S03 §13, applied to imports).
//!
//! Two levels:
//! - [`Plan::check_structure`]: what would break the calculations (terrain,
//!   layouts, series that are not valid on their own). The import is refused.
//! - [`Plan::incoherences`]: what breaks the rules of the plan without
//!   breaking the calculations (too many planches, orphan series, overlaps,
//!   placement, bad annotations). Nothing is deleted or corrected: the import
//!   is accepted and the list is shown "à résoudre" (S03 §13 default).

use std::collections::BTreeSet;

use crate::error::PlanError;
use crate::ids::{AnnotationId, PlancheId, SerieId};
use crate::model::{Disposition, MAX_PLANCHES, Plan};
use crate::zone::Zone;

/// One thing to resolve in a received plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoherence {
    /// More than 10 planches in `year`.
    TooManyPlanches { year: i32, count: usize },
    /// The planche of the serie is unknown, or absent in a year it occupies.
    OrphanSerie {
        serie_id: SerieId,
        planche_id: PlancheId,
        year: Option<i32>,
    },
    /// The zone of the serie exceeds its planche in `year`.
    ZoneOutOfPlanche {
        serie_id: SerieId,
        planche_id: PlancheId,
        year: i32,
        troncons: u32,
        zone: Zone,
    },
    /// Two series share troncons and days (S03 §5).
    Conflict { a: SerieId, b: SerieId },
    /// A planche breaks a placement rule (S03 §15).
    Placement(PlanError),
    /// The serie uses a crop that is not in the catalogue.
    UnknownCulture {
        serie_id: SerieId,
        culture_id: String,
    },
    /// The annotation points to a planche that does not exist.
    AnnotationUnknownPlanche {
        annotation_id: AnnotationId,
        planche_id: PlancheId,
    },
    /// The text or the dates of the annotation are not valid.
    AnnotationInvalid { annotation_id: AnnotationId },
}

impl Incoherence {
    /// Stable code, translated by the interface.
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooManyPlanches { .. } => "trop_de_planches",
            Self::OrphanSerie { .. } => "serie_orpheline",
            Self::ZoneOutOfPlanche { .. } => "zone_hors_planche",
            Self::Conflict { .. } => "chevauchement",
            Self::Placement(e) => e.code(),
            Self::UnknownCulture { .. } => "culture_inconnue",
            Self::AnnotationUnknownPlanche { .. } => "note_planche_inconnue",
            Self::AnnotationInvalid { .. } => "note_invalide",
        }
    }
}

fn culture_missing(culture_id: &str) -> bool {
    catalogue::Catalogue::v1().culture(culture_id).is_none()
}

impl Plan {
    /// Refuses a plan whose calculations would not be safe to run.
    pub fn check_structure(&self) -> Result<(), PlanError> {
        self.terrain.validate()?;
        let case_cm = self.terrain.case_cm.cm();
        for planche in self.planches.values() {
            for disposition in planche.dispositions.values() {
                if let Disposition::Present(g) = disposition {
                    if g.longueur_cm == 0 || g.longueur_cm % case_cm != 0 {
                        return Err(PlanError::InvalidLength {
                            longueur_cm: g.longueur_cm,
                            case_cm,
                        });
                    }
                }
            }
        }
        for serie in self.series.values() {
            serie.check()?;
        }
        Ok(())
    }

    /// Every incoherence of the plan, in a stable order. Empty when valid.
    pub fn incoherences(&self) -> Vec<Incoherence> {
        let mut out = Vec::new();
        let years: BTreeSet<i32> = self
            .planches
            .values()
            .flat_map(|p| p.dispositions.keys().copied())
            .collect();
        if let (Some(&first), Some(&last)) = (years.first(), years.last()) {
            for year in first..=last {
                let count = self.planches_in(year).len();
                if count > MAX_PLANCHES {
                    out.push(Incoherence::TooManyPlanches { year, count });
                }
            }
            if let Err(e) = self.check_placements(first) {
                out.push(Incoherence::Placement(e));
            }
        }
        for (id, serie) in &self.series {
            if culture_missing(&serie.culture_id) {
                out.push(Incoherence::UnknownCulture {
                    serie_id: *id,
                    culture_id: serie.culture_id.clone(),
                });
            }
            let Some(planche) = self.planches.get(&serie.planche_id) else {
                out.push(Incoherence::OrphanSerie {
                    serie_id: *id,
                    planche_id: serie.planche_id,
                    year: None,
                });
                continue;
            };
            let zone = serie.zone();
            for year in serie.occupation().years() {
                match planche.disposition_in(year) {
                    None => out.push(Incoherence::OrphanSerie {
                        serie_id: *id,
                        planche_id: serie.planche_id,
                        year: Some(year),
                    }),
                    Some(g) => {
                        let troncons = g.troncons(self.terrain.case_cm);
                        if zone.t2 > troncons {
                            out.push(Incoherence::ZoneOutOfPlanche {
                                serie_id: *id,
                                planche_id: serie.planche_id,
                                year,
                                troncons,
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
            if let Some(planche_id) = a.planche_id {
                if !self.planches.contains_key(&planche_id) {
                    out.push(Incoherence::AnnotationUnknownPlanche {
                        annotation_id: *id,
                        planche_id,
                    });
                }
            }
        }
        out
    }
}
