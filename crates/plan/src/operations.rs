//! Operations that change the plan. Each one checks everything first: an
//! operation that returns an error leaves the plan untouched.

use chrono::{Datelike, NaiveDate};

use crate::days::january_first;
use crate::error::PlanError;
use crate::ids::{PlancheId, SerieId};
use crate::model::{Disposition, Geometry, MAX_PLANCHES, Plan, Planche};
use crate::serie::Serie;

impl Plan {
    /// Adds a planche, with its first layout recorded in `year`.
    pub fn add_planche(
        &mut self,
        id: PlancheId,
        nom: String,
        year: i32,
        geometry: Geometry,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        if self.planches.contains_key(&id) {
            return Err(PlanError::PlancheExists(id));
        }
        self.planches.insert(
            id,
            Planche {
                nom,
                dispositions: Default::default(),
            },
        );
        let result = self.set_disposition(id, year, Disposition::Present(geometry), today);
        if result.is_err() {
            self.planches.remove(&id);
        }
        result
    }

    /// Records the layout of a planche in `year`, replacing the one recorded
    /// for that year. It holds for the following years until the next layout
    /// recorded for this same planche (S03 §7).
    ///
    /// Series that no longer fit are not refused: ask
    /// [`Plan::series_overflowing_if_shortened`] before and warn the user.
    pub fn set_disposition(
        &mut self,
        planche_id: PlancheId,
        year: i32,
        disposition: Disposition,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        if year < today.year() {
            return Err(PlanError::PastYear { year, today });
        }
        if !self.planches.contains_key(&planche_id) {
            return Err(PlanError::UnknownPlanche(planche_id));
        }
        if let Disposition::Present(geometry) = &disposition {
            let case_cm = self.terrain.case_cm.cm();
            if geometry.longueur_cm == 0 || geometry.longueur_cm % case_cm != 0 {
                return Err(PlanError::InvalidLength {
                    longueur_cm: geometry.longueur_cm,
                    case_cm,
                });
            }
        }

        let mut after = self.clone();
        if let Some(planche) = after.planches.get_mut(&planche_id) {
            planche.dispositions.insert(year, disposition);
        }
        // Years from `year` to the last recorded layout may gain a planche.
        let last = self
            .planches
            .values()
            .flat_map(|p| p.dispositions.keys().copied())
            .max()
            .unwrap_or(year)
            .max(year);
        for y in year..=last {
            let existed =
                self.planche_count_in(y, Some(planche_id)) != self.planche_count_in(y, None);
            let exists =
                after.planche_count_in(y, Some(planche_id)) != after.planche_count_in(y, None);
            if exists && !existed && after.planche_count_in(y, None) > MAX_PLANCHES {
                return Err(PlanError::TooManyPlanches {
                    year: y,
                    max: MAX_PLANCHES,
                });
            }
        }
        *self = after;
        Ok(())
    }

    /// Number of planches in `year`, leaving one out if asked.
    fn planche_count_in(&self, year: i32, sauf: Option<PlancheId>) -> usize {
        self.planches
            .iter()
            .filter(|(id, _)| Some(**id) != sauf)
            .filter(|(_, p)| p.disposition_in(year).is_some())
            .count()
    }

    /// The series deleted with the planche from `year` on: those whose end `f`
    /// is after January 1st of `year`, including those that began earlier
    /// (S03 §5). Series ending on or before that day are kept.
    pub fn series_deleted_by(&self, planche_id: PlancheId, year: i32) -> Vec<SerieId> {
        let limit = january_first(year);
        self.series
            .iter()
            .filter(|(_, s)| s.planche_id == planche_id && s.dates().end > limit)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Deletes a planche from `year` on, with its series that touch `year` or
    /// later. Returns the deleted series. Layouts recorded after `year` are dropped.
    pub fn delete_planche(
        &mut self,
        planche_id: PlancheId,
        year: i32,
        today: NaiveDate,
    ) -> Result<Vec<SerieId>, PlanError> {
        if year < today.year() {
            return Err(PlanError::PastYear { year, today });
        }
        let planche = self
            .planches
            .get_mut(&planche_id)
            .ok_or(PlanError::UnknownPlanche(planche_id))?;
        planche.dispositions.retain(|y, _| *y < year);
        planche.dispositions.insert(year, Disposition::Absent);
        let deleted = self.series_deleted_by(planche_id, year);
        for id in &deleted {
            self.series.remove(id);
        }
        Ok(deleted)
    }

    /// Adds a valid serie (S03 §5).
    pub fn add_serie(&mut self, id: SerieId, serie: Serie) -> Result<(), PlanError> {
        if self.series.contains_key(&id) {
            return Err(PlanError::SerieExists(id));
        }
        self.validate_serie(None, &serie)?;
        self.series.insert(id, serie);
        Ok(())
    }

    /// Replaces a serie by a modified one, if the result is valid.
    pub fn replace_serie(&mut self, id: SerieId, serie: Serie) -> Result<(), PlanError> {
        if !self.series.contains_key(&id) {
            return Err(PlanError::UnknownSerie(id));
        }
        self.validate_serie(Some(id), &serie)?;
        self.series.insert(id, serie);
        Ok(())
    }

    /// Drags a bar by `days` days: the entered date and the manual end shift.
    /// Refused, with the plan untouched, if the moved serie is no longer valid
    /// (S03 §2).
    pub fn shift_serie(&mut self, id: SerieId, days: i64) -> Result<(), PlanError> {
        let shifted = self
            .series
            .get(&id)
            .ok_or(PlanError::UnknownSerie(id))?
            .shifted(days);
        self.replace_serie(id, shifted)
    }

    /// Deletes a serie.
    pub fn delete_serie(&mut self, id: SerieId) -> Result<Serie, PlanError> {
        self.series.remove(&id).ok_or(PlanError::UnknownSerie(id))
    }
}
