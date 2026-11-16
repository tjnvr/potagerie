//! "Clear le plan" (S01 F-PLN-4, S05 §7).

use chrono::{Datelike, NaiveDate};

use crate::days::january_first;
use crate::error::PlanError;
use crate::ids::{AnnotationId, BedId, PlantingId};
use crate::model::{Layout, Plan};

/// What [`Plan::clear_from`] removed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cleared {
    pub beds: Vec<BedId>,
    pub plantings: Vec<PlantingId>,
    pub annotations: Vec<AnnotationId>,
}

impl Plan {
    /// Empties the plan from `year` on: every bed that exists in `year` or later
    /// is deleted as `delete_bed` does (with its plantings that touch `year`
    /// or later), and so are the annotations attached to those beds that
    /// end on or after January 1st of `year`. The plot, past years, plantings
    /// already finished, and the annotations of the whole garden stay.
    pub fn clear_from(&mut self, year: i32, today: NaiveDate) -> Result<Cleared, PlanError> {
        if year < today.year() {
            return Err(PlanError::PastYear { year, today });
        }
        let mut cleared = Cleared::default();
        // Every bed that exists in `year`, and those created in a later year only.
        let mut ids: Vec<BedId> = self.beds_in(year).iter().map(|(id, _, _)| *id).collect();
        for (id, bed) in &self.beds {
            let later = bed
                .layouts
                .iter()
                .any(|(y, l)| *y > year && !matches!(l, Layout::Absent));
            if later && !ids.contains(id) {
                ids.push(*id);
            }
        }
        for id in ids {
            cleared.plantings.extend(self.delete_bed(id, year, today)?);
            cleared.beds.push(id);
        }
        let start = january_first(year);
        let gone: Vec<AnnotationId> = self
            .annotations
            .iter()
            .filter(|(_, a)| a.last_day() >= start)
            .filter(|(_, a)| a.bed_id.is_some_and(|p| cleared.beds.contains(&p)))
            .map(|(id, _)| *id)
            .collect();
        for id in gone {
            self.annotations.remove(&id);
            cleared.annotations.push(id);
        }
        Ok(cleared)
    }
}
