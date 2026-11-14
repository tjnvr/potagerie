//! Validity of a serie, conflicts and zone shortcuts (S03 §4, §5).

use crate::conflicts::{ConflictDetail, conflict};
use crate::days::january_first;
use crate::error::PlanError;
use crate::ids::{PlancheId, SerieId};
use crate::model::Plan;
use crate::serie::Serie;
use crate::zone::{Occupation, Zone};

/// The ½, ⅓ and ¼ zone shortcuts, plus the whole planche (S01 F-PLA-1).
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

    /// `k = max(1, arrondi(n × fraction))`, rounding halves up (S03 §4).
    pub fn size(self, n: u32) -> u32 {
        let (num, den) = self.numerator_denominator();
        let (n, num, den) = (u64::from(n), u64::from(num), u64::from(den));
        let k = (2 * n * num + den) / (2 * den);
        u32::try_from(k).unwrap_or(u32::MAX).max(1)
    }
}

impl Plan {
    /// Everything that makes `serie` invalid in this plan (S03 §5): a planche
    /// missing in a year it occupies, a zone that does not fit, conflicts.
    ///
    /// `ignore` is the id of the serie being edited, so it is not compared
    /// with its previous self. The list is empty when the serie is valid.
    pub fn serie_problems(&self, ignore: Option<SerieId>, serie: &Serie) -> Vec<PlanError> {
        if let Err(error) = serie.check() {
            return vec![error];
        }
        let Some(planche) = self.planches.get(&serie.planche_id) else {
            return vec![PlanError::UnknownPlanche(serie.planche_id)];
        };
        let zone = serie.zone();
        let mut problems = Vec::new();
        for year in serie.occupation().years() {
            match planche.disposition_in(year) {
                None => problems.push(PlanError::PlancheMissing {
                    planche_id: serie.planche_id,
                    year,
                }),
                Some(geometry) => {
                    let troncons = geometry.troncons(self.terrain.case_cm);
                    if zone.t2 > troncons {
                        problems.push(PlanError::ZoneOutOfPlanche {
                            year,
                            troncons,
                            zone,
                        });
                    }
                }
            }
        }
        for (other_id, other) in &self.series {
            if Some(*other_id) == ignore {
                continue;
            }
            if let Some(ConflictDetail { troncons, period }) = conflict(serie, other) {
                problems.push(PlanError::Conflict {
                    with: *other_id,
                    troncons,
                    period,
                });
            }
        }
        problems
    }

    /// `Ok` if the serie is valid, otherwise the first problem.
    pub fn validate_serie(&self, ignore: Option<SerieId>, serie: &Serie) -> Result<(), PlanError> {
        match self.serie_problems(ignore, serie).into_iter().next() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Every pair of conflicting series of the plan, each pair once.
    pub fn conflicts(&self) -> Vec<(SerieId, SerieId, ConflictDetail)> {
        let mut pairs = Vec::new();
        for (a_id, a) in &self.series {
            for (b_id, b) in self.series.range(*a_id..).skip(1) {
                if let Some(detail) = conflict(a, b) {
                    pairs.push((*a_id, *b_id, detail));
                }
            }
        }
        pairs
    }

    /// The series that would no longer fit if `planche_id` were shortened to
    /// `new_nb_troncons` from `year` on (S03 §4): `t2 > n′` and an occupation
    /// that touches `year` or a later year.
    pub fn series_overflowing_if_shortened(
        &self,
        planche_id: PlancheId,
        year: i32,
        new_nb_troncons: u32,
    ) -> Vec<SerieId> {
        let year_start = january_first(year);
        self.series
            .iter()
            .filter(|(_, s)| s.planche_id == planche_id)
            .filter(|(_, s)| s.troncon_fin > new_nb_troncons)
            .filter(|(_, s)| s.dates().end > year_start)
            .map(|(id, _)| *id)
            .collect()
    }

    /// The zone proposed by a shortcut: the first run of `k` troncons that is
    /// free over the whole `occupation` and fits in the planche every year it
    /// touches. `None` greys the shortcut out (S03 §4).
    ///
    /// `n` is the length of the planche in the year the occupation starts.
    /// `ignore` is the serie being edited, if any.
    pub fn shortcut(
        &self,
        planche_id: PlancheId,
        fraction: Fraction,
        occupation: Occupation,
        ignore: Option<SerieId>,
    ) -> Option<Zone> {
        let planche = self.planches.get(&planche_id)?;
        let start_year = chrono::Datelike::year(&occupation.start);
        let n = planche
            .disposition_in(start_year)?
            .troncons(self.terrain.case_cm);
        let k = fraction.size(n);
        let years = occupation.years();
        let fits = |zone: &Zone| {
            years.clone().all(|year| {
                planche
                    .disposition_in(year)
                    .is_some_and(|g| g.troncons(self.terrain.case_cm) >= zone.t2)
            })
        };
        let free = |zone: &Zone| {
            self.series
                .iter()
                .filter(|(id, s)| Some(**id) != ignore && s.planche_id == planche_id)
                .all(|(_, s)| !(s.zone().overlaps(zone) && s.occupation().overlaps(&occupation)))
        };
        (1..=n.checked_sub(k)? + 1)
            .map(|t1| Zone { t1, t2: t1 + k - 1 })
            .find(|zone| fits(zone) && free(zone))
    }
}
