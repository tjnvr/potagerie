//! Validity of a planting, conflicts and zone shortcuts (S03 §4, §5).

use crate::conflicts::{ConflictDetail, conflict};
use crate::days::january_first;
use crate::error::PlanError;
use crate::ids::{BedId, PlantingId};
use crate::model::Plan;
use crate::planting::Planting;
use crate::zone::{Occupation, Zone};

/// The ½, ⅓ and ¼ zone shortcuts, plus the whole bed (S01 F-PLA-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fraction {
    Whole,
    Half,
    Third,
    Quarter,
}

impl Fraction {
    fn numerator_denominator(self) -> (u32, u32) {
        match self {
            Self::Whole => (1, 1),
            Self::Half => (1, 2),
            Self::Third => (1, 3),
            Self::Quarter => (1, 4),
        }
    }

    /// `k = max(1, rounded(n × fraction))`, rounding halves up (S03 §4).
    pub fn size(self, n: u32) -> u32 {
        let (num, den) = self.numerator_denominator();
        let (n, num, den) = (u64::from(n), u64::from(num), u64::from(den));
        let k = (2 * n * num + den) / (2 * den);
        u32::try_from(k).unwrap_or(u32::MAX).max(1)
    }
}

impl Plan {
    /// Everything that makes `planting` invalid in this plan (S03 §5): a bed
    /// missing in a year it occupies, a zone that does not fit, conflicts.
    ///
    /// `ignore` is the id of the planting being edited, so it is not compared
    /// with its previous self. The list is empty when the planting is valid.
    pub fn planting_problems(
        &self,
        ignore: Option<PlantingId>,
        planting: &Planting,
    ) -> Vec<PlanError> {
        if let Err(error) = planting.check() {
            return vec![error];
        }
        let Some(bed) = self.beds.get(&planting.bed_id) else {
            return vec![PlanError::UnknownBed(planting.bed_id)];
        };
        let zone = planting.zone();
        let mut problems = Vec::new();
        for year in planting.occupation().years() {
            match bed.layout_in(year) {
                None => problems.push(PlanError::BedMissing {
                    bed_id: planting.bed_id,
                    year,
                }),
                Some(geometry) => {
                    let segments = geometry.segments(self.plot.cell_cm);
                    if zone.t2 > segments {
                        problems.push(PlanError::ZoneOutOfBed {
                            year,
                            segments,
                            zone,
                        });
                    }
                }
            }
        }
        for (other_id, other) in &self.plantings {
            if Some(*other_id) == ignore {
                continue;
            }
            if let Some(ConflictDetail { segments, period }) = conflict(planting, other) {
                problems.push(PlanError::Conflict {
                    with: *other_id,
                    segments,
                    period,
                });
            }
        }
        problems
    }

    /// `Ok` if the planting is valid, otherwise the first problem.
    pub fn validate_planting(
        &self,
        ignore: Option<PlantingId>,
        planting: &Planting,
    ) -> Result<(), PlanError> {
        match self.planting_problems(ignore, planting).into_iter().next() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Every pair of conflicting plantings of the plan, each pair once.
    pub fn conflicts(&self) -> Vec<(PlantingId, PlantingId, ConflictDetail)> {
        let mut pairs = Vec::new();
        for (a_id, a) in &self.plantings {
            for (b_id, b) in self.plantings.range(*a_id..).skip(1) {
                if let Some(detail) = conflict(a, b) {
                    pairs.push((*a_id, *b_id, detail));
                }
            }
        }
        pairs
    }

    /// The plantings that would no longer fit if `bed_id` were shortened to
    /// `new_nb_segments` from `year` on (S03 §4): `t2 > n′` and an occupation
    /// that touches `year` or a later year.
    pub fn plantings_overflowing_if_shortened(
        &self,
        bed_id: BedId,
        year: i32,
        new_nb_segments: u32,
    ) -> Vec<PlantingId> {
        let year_start = january_first(year);
        self.plantings
            .iter()
            .filter(|(_, s)| s.bed_id == bed_id)
            .filter(|(_, s)| s.segment_end > new_nb_segments)
            .filter(|(_, s)| s.dates().end > year_start)
            .map(|(id, _)| *id)
            .collect()
    }

    /// The zone proposed by a shortcut: the first run of `k` segments that is
    /// free over the whole `occupation` and fits in the bed every year it
    /// touches. `None` greys the shortcut out (S03 §4).
    ///
    /// `n` is the length of the bed in the year the occupation starts.
    /// `ignore` is the planting being edited, if any.
    pub fn shortcut(
        &self,
        bed_id: BedId,
        fraction: Fraction,
        occupation: Occupation,
        ignore: Option<PlantingId>,
    ) -> Option<Zone> {
        let bed = self.beds.get(&bed_id)?;
        let start_year = chrono::Datelike::year(&occupation.start);
        let n = bed.layout_in(start_year)?.segments(self.plot.cell_cm);
        let k = fraction.size(n);
        let years = occupation.years();
        let fits = |zone: &Zone| {
            years.clone().all(|year| {
                bed.layout_in(year)
                    .is_some_and(|g| g.segments(self.plot.cell_cm) >= zone.t2)
            })
        };
        let free = |zone: &Zone| {
            self.plantings
                .iter()
                .filter(|(id, s)| Some(**id) != ignore && s.bed_id == bed_id)
                .all(|(_, s)| !(s.zone().overlaps(zone) && s.occupation().overlaps(&occupation)))
        };
        (1..=n.checked_sub(k)? + 1)
            .map(|t1| Zone { t1, t2: t1 + k - 1 })
            .find(|zone| fits(zone) && free(zone))
    }
}
