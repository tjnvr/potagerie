//! Notes and periods of the timeline (S01 F-VIS-5, S05 §5): the rules that
//! every command checks before it stores one.
//!
//! - a past year is read-only (S03 §7): neither the displayed year nor the
//!   annotation itself may lie entirely before the year of today;
//! - the planche, if any, must exist;
//! - a period ends on or after its start, and has an end; a note has none;
//! - the text is 1 to [`MAX_ANNOTATION_CHARS`] characters once trimmed.

use chrono::{Datelike, NaiveDate};

use crate::error::PlanError;
use crate::ids::AnnotationId;
use crate::model::{Annotation, Plan, TypeAnnotation};

/// Longest text of a note or a period, in characters (S01 F-VIS-5).
pub const MAX_ANNOTATION_CHARS: usize = 80;

impl Annotation {
    /// The last day the annotation touches: its end for a period, its date
    /// for a note.
    pub fn last_day(&self) -> NaiveDate {
        match self.type_annotation {
            TypeAnnotation::Note => self.date_debut,
            TypeAnnotation::Period => self.date_fin.unwrap_or(self.date_debut),
        }
    }

    /// The annotation as the plan stores it: trimmed text, and no end date
    /// on a note. Call [`Plan::check_annotation`] first.
    fn normalised(mut self) -> Self {
        self.texte = self.texte.trim().to_owned();
        if self.type_annotation == TypeAnnotation::Note {
            self.date_fin = None;
        }
        self
    }

    /// What the annotation alone can say: text and dates.
    pub fn check(&self) -> Result<(), PlanError> {
        let chars = self.texte.trim().chars().count();
        if chars == 0 || chars > MAX_ANNOTATION_CHARS {
            return Err(PlanError::InvalidAnnotationText);
        }
        if self.type_annotation == TypeAnnotation::Period {
            match self.date_fin {
                Some(end) if end >= self.date_debut => {}
                _ => return Err(PlanError::InvalidAnnotationDates),
            }
        }
        Ok(())
    }
}

fn check_not_past(year: i32, today: NaiveDate) -> Result<(), PlanError> {
    if year < today.year() {
        return Err(PlanError::PastYear { year, today });
    }
    Ok(())
}

impl Plan {
    /// Checks an annotation to store, shown in `year`.
    pub fn check_annotation(
        &self,
        year: i32,
        annotation: &Annotation,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        check_not_past(year, today)?;
        annotation.check()?;
        check_not_past(annotation.last_day().year(), today)?;
        if let Some(planche_id) = annotation.planche_id {
            if !self.planches.contains_key(&planche_id) {
                return Err(PlanError::UnknownPlanche(planche_id));
            }
        }
        Ok(())
    }

    /// Adds a note or a period.
    pub fn add_annotation(
        &mut self,
        id: AnnotationId,
        year: i32,
        annotation: Annotation,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        self.check_annotation(year, &annotation, today)?;
        self.annotations.insert(id, annotation.normalised());
        Ok(())
    }

    /// Replaces an annotation; hiding it from the timeline is such a change
    /// (`masquee`). The stored annotation must not be past either.
    pub fn replace_annotation(
        &mut self,
        id: AnnotationId,
        year: i32,
        annotation: Annotation,
        today: NaiveDate,
    ) -> Result<(), PlanError> {
        let stored = self
            .annotations
            .get(&id)
            .ok_or(PlanError::UnknownAnnotation(id))?;
        check_not_past(stored.last_day().year(), today)?;
        self.check_annotation(year, &annotation, today)?;
        self.annotations.insert(id, annotation.normalised());
        Ok(())
    }

    /// Deletes an annotation.
    pub fn delete_annotation(
        &mut self,
        id: AnnotationId,
        year: i32,
        today: NaiveDate,
    ) -> Result<Annotation, PlanError> {
        check_not_past(year, today)?;
        let stored = self
            .annotations
            .get(&id)
            .ok_or(PlanError::UnknownAnnotation(id))?;
        check_not_past(stored.last_day().year(), today)?;
        self.annotations
            .remove(&id)
            .ok_or(PlanError::UnknownAnnotation(id))
    }
}
