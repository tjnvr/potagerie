//! Operations that change the plan. Each one checks everything first: an
//! operation that returns an error leaves the plan untouched.

use chrono::{Datelike, NaiveDate};

use crate::days::january_first;
use crate::error::PlanError;
use crate::ids::{BedId, PlantingId};
use crate::model::{Bed, Geometry, Layout, MAX_BEDS, Plan};
use crate::planting::Planting;

impl Plan {
    /// Adds a bed, with its first layout recorded in `year`.
    pub fn add_bed(
        &mut self,
        id: BedId,
        name: String,
        year: i32,
        geometry: Geometry,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        if self.beds.contains_key(&id) {
            return Err(PlanError::BedExists(id));
        }
        self.beds.insert(
            id,
            Bed {
                name,
                layouts: Default::default(),
            },
        );
        let result = self.set_layout(id, year, Layout::Present(geometry), today);
        if result.is_err() {
            self.beds.remove(&id);
        }
        result
    }

    /// Records the layout of a bed in `year`, replacing the one recorded
    /// for that year. It holds for the following years until the next layout
    /// recorded for this same bed (S03 §7).
    ///
    /// Plantings that no longer fit are not refused: ask
    /// [`Plan::plantings_overflowing_if_shortened`] before and warn the user.
    pub fn set_layout(
        &mut self,
        bed_id: BedId,
        year: i32,
        layout: Layout,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        if year < today.year() {
            return Err(PlanError::PastYear { year, today });
        }
        crate::range::check_year("year", year)?;
        if !self.beds.contains_key(&bed_id) {
            return Err(PlanError::UnknownBed(bed_id));
        }
        if let Layout::Present(geometry) = &layout {
            let cell_cm = self.plot.cell_cm.cm();
            if geometry.length_cm == 0 || geometry.length_cm % cell_cm != 0 {
                return Err(PlanError::InvalidLength {
                    length_cm: geometry.length_cm,
                    cell_cm,
                });
            }
        }

        let mut after = self.clone();
        if let Some(bed) = after.beds.get_mut(&bed_id) {
            bed.layouts.insert(year, layout);
        }
        // Years from `year` to the last recorded layout may gain a bed.
        let last = self
            .beds
            .values()
            .flat_map(|p| p.layouts.keys().copied())
            .max()
            .unwrap_or(year)
            .max(year);
        for y in year..=last {
            let existed = self.bed_count_in(y, Some(bed_id)) != self.bed_count_in(y, None);
            let exists = after.bed_count_in(y, Some(bed_id)) != after.bed_count_in(y, None);
            if exists && !existed && after.bed_count_in(y, None) > MAX_BEDS {
                return Err(PlanError::TooManyBeds {
                    year: y,
                    max: MAX_BEDS,
                });
            }
        }
        *self = after;
        Ok(())
    }

    /// Number of beds in `year`, leaving one out if asked.
    fn bed_count_in(&self, year: i32, sauf: Option<BedId>) -> usize {
        self.beds
            .iter()
            .filter(|(id, _)| Some(**id) != sauf)
            .filter(|(_, p)| p.layout_in(year).is_some())
            .count()
    }

    /// The plantings deleted with the bed from `year` on: those whose end `f`
    /// is after January 1st of `year`, including those that began earlier
    /// (S03 §5). Plantings ending on or before that day are kept.
    pub fn plantings_deleted_by(&self, bed_id: BedId, year: i32) -> Vec<PlantingId> {
        let limit = january_first(year);
        self.plantings
            .iter()
            .filter(|(_, s)| s.bed_id == bed_id && s.dates().end > limit)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Deletes a bed from `year` on, with its plantings that touch `year` or
    /// later. Returns the deleted plantings. Layouts recorded after `year` are dropped.
    pub fn delete_bed(
        &mut self,
        bed_id: BedId,
        year: i32,
        today: NaiveDate,
    ) -> Result<Vec<PlantingId>, PlanError> {
        if year < today.year() {
            return Err(PlanError::PastYear { year, today });
        }
        let bed = self
            .beds
            .get_mut(&bed_id)
            .ok_or(PlanError::UnknownBed(bed_id))?;
        bed.layouts.retain(|y, _| *y < year);
        bed.layouts.insert(year, Layout::Absent);
        let deleted = self.plantings_deleted_by(bed_id, year);
        for id in &deleted {
            self.plantings.remove(id);
        }
        Ok(deleted)
    }

    /// Adds a valid planting (S03 §5).
    pub fn add_planting(&mut self, id: PlantingId, planting: Planting) -> Result<(), PlanError> {
        if self.plantings.contains_key(&id) {
            return Err(PlanError::PlantingExists(id));
        }
        self.validate_planting(None, &planting)?;
        self.plantings.insert(id, planting);
        Ok(())
    }

    /// Replaces a planting by a modified one, if the result is valid.
    pub fn replace_planting(
        &mut self,
        id: PlantingId,
        planting: Planting,
    ) -> Result<(), PlanError> {
        if !self.plantings.contains_key(&id) {
            return Err(PlanError::UnknownPlanting(id));
        }
        self.validate_planting(Some(id), &planting)?;
        self.plantings.insert(id, planting);
        Ok(())
    }

    /// Drags a bar by `days` days: the entered date and the manual end shift.
    /// Refused, with the plan untouched, if the moved planting is no longer valid
    /// (S03 §2).
    pub fn shift_planting(&mut self, id: PlantingId, days: i64) -> Result<(), PlanError> {
        let shifted = self
            .plantings
            .get(&id)
            .ok_or(PlanError::UnknownPlanting(id))?
            .shifted(days);
        self.replace_planting(id, shifted)
    }

    /// Deletes a planting.
    pub fn delete_planting(&mut self, id: PlantingId) -> Result<Planting, PlanError> {
        self.plantings
            .remove(&id)
            .ok_or(PlanError::UnknownPlanting(id))
    }
}
