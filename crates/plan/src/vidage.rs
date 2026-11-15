//! "Vider le plan" (S01 F-PLN-4, S05 §7).

use chrono::{Datelike, NaiveDate};

use crate::days::january_first;
use crate::error::PlanError;
use crate::ids::{AnnotationId, PlancheId, SerieId};
use crate::model::Plan;

/// What [`Plan::clear_from`] removed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cleared {
    pub planches: Vec<PlancheId>,
    pub series: Vec<SerieId>,
    pub annotations: Vec<AnnotationId>,
}

impl Plan {
    /// Empties the plan from `year` on: every planche that exists in `year`
    /// is deleted as `delete_planche` does (with its series that touch `year`
    /// or later), and so are the annotations attached to those planches that
    /// end on or after January 1st of `year`. The terrain, past years, series
    /// already finished, and the annotations of the whole potager stay.
    pub fn clear_from(&mut self, year: i32, today: NaiveDate) -> Result<Cleared, PlanError> {
        if year < today.year() {
            return Err(PlanError::PastYear { year, today });
        }
        let mut cleared = Cleared::default();
        let ids: Vec<PlancheId> = self
            .planches_in(year)
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        for id in ids {
            cleared.series.extend(self.delete_planche(id, year, today)?);
            cleared.planches.push(id);
        }
        let start = january_first(year);
        let gone: Vec<AnnotationId> = self
            .annotations
            .iter()
            .filter(|(_, a)| a.last_day() >= start)
            .filter(|(_, a)| a.planche_id.is_some_and(|p| cleared.planches.contains(&p)))
            .map(|(id, _)| *id)
            .collect();
        for id in gone {
            self.annotations.remove(&id);
            cleared.annotations.push(id);
        }
        Ok(cleared)
    }
}
