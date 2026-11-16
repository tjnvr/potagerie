//! Placement rules of the editor (S03 §15): plot dimensions, beds
//! inside the plot, no overlap, aisle width between beds.

use crate::error::PlanError;
use crate::ids::BedId;
use crate::model::{Geometry, MAX_NAME_CHARS, MIN_WIDTH_CM, Plan, Plot};

/// Aisle widths offered by S01 F-PLN-2, in cm.
pub const AISLE_CHOICES: [u32; 3] = [30, 50, 80];
/// Smallest and largest plot side, in cm (S03 §15).
pub const MIN_PLOT_CM: u32 = 100;
pub const MAX_PLOT_CM: u32 = 10_000;

impl Plot {
    /// Checks the dimensions and the aisle width (S03 §15).
    pub fn validate(&self) -> Result<(), PlanError> {
        let case = self.cell_cm.cm();
        let side_ok = |side: u32| (MIN_PLOT_CM.max(case)..=MAX_PLOT_CM).contains(&side);
        if !side_ok(self.length_cm) || !side_ok(self.width_cm) {
            return Err(PlanError::InvalidPlot {
                length_cm: self.length_cm,
                width_cm: self.width_cm,
            });
        }
        if !AISLE_CHOICES.contains(&self.aisle_cm) {
            return Err(PlanError::InvalidAisle(self.aisle_cm));
        }
        Ok(())
    }
}

/// Checks a bed name: 1 to 30 characters once trimmed (S01 F-PLN-1).
pub fn check_name(name: &str) -> Result<(), PlanError> {
    let chars = name.trim().chars().count();
    if chars == 0 || chars > MAX_NAME_CHARS {
        return Err(PlanError::InvalidName);
    }
    Ok(())
}

/// Signed gap between two segments `[a, a+la[` and `[b, b+lb[`: negative when they overlap.
fn gap(a: u32, la: u32, b: u32, lb: u32) -> i64 {
    let (a, la, b, lb) = (i64::from(a), i64::from(la), i64::from(b), i64::from(lb));
    (b - (a + la)).max(a - (b + lb))
}

impl Plan {
    /// Checks the layout of every bed, in every year from `from_year` to
    /// the last year with a recorded layout (a layout holds until the next
    /// one, S03 §7): the bed fits in the plot, has a valid width, does
    /// not overlap another one and keeps the aisle width from it.
    ///
    /// Operations call it after changing a layout; the command then does not
    /// save when it fails.
    pub fn check_placements(&self, from_year: i32) -> Result<(), PlanError> {
        match self.placement_errors(from_year).into_iter().next() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Every placement problem from `from_year` on, not only the first one: the
    /// "to resolve" list of an import shows them all (S03 §13).
    pub fn placement_errors(&self, from_year: i32) -> Vec<PlanError> {
        let mut errors = Vec::new();
        let last = self
            .beds
            .values()
            .flat_map(|p| p.layouts.keys().copied())
            .max()
            .unwrap_or(from_year)
            .max(from_year);
        for year in from_year..=last {
            let beds = self.beds_in(year);
            for (id, _, geometry) in &beds {
                errors.extend(self.check_inside(*id, geometry, year).err());
            }
            for (i, (a_id, _, a)) in beds.iter().enumerate() {
                for (b_id, _, b) in &beds[i + 1..] {
                    errors.extend(self.check_apart(*a_id, a, *b_id, b, year).err());
                }
            }
        }
        errors
    }

    fn check_inside(&self, id: BedId, g: &Geometry, year: i32) -> Result<(), PlanError> {
        if g.width_cm < MIN_WIDTH_CM {
            return Err(PlanError::InvalidWidth(g.width_cm));
        }
        let t = &self.plot;
        let fits = u64::from(g.x) + u64::from(g.length_cm) <= u64::from(t.length_cm)
            && u64::from(g.y) + u64::from(g.width_cm) <= u64::from(t.width_cm);
        if !fits {
            return Err(PlanError::OutsidePlot { bed_id: id, year });
        }
        Ok(())
    }

    fn check_apart(
        &self,
        a_id: BedId,
        a: &Geometry,
        b_id: BedId,
        b: &Geometry,
        year: i32,
    ) -> Result<(), PlanError> {
        let dx = gap(a.x, a.length_cm, b.x, b.length_cm);
        let dy = gap(a.y, a.width_cm, b.y, b.width_cm);
        if dx < 0 && dy < 0 {
            return Err(PlanError::BedsOverlap {
                a: a_id,
                b: b_id,
                year,
            });
        }
        let aisle = i64::from(self.plot.aisle_cm);
        if dx < aisle && dy < aisle {
            return Err(PlanError::AisleTooNarrow {
                a: a_id,
                b: b_id,
                year,
                aisle_cm: self.plot.aisle_cm,
            });
        }
        Ok(())
    }
}

impl Plan {
    /// Changes the dimensions and the aisle width; the case size stays (S03 §4).
    /// Refused, with the plan untouched, if the new plot is invalid or no
    /// longer holds the beds of any year: the plot is one for the whole garden
    /// and a past layout cannot be repaired, so a plot that breaks one is refused.
    pub fn set_plot(
        &mut self,
        length_cm: u32,
        width_cm: u32,
        aisle_cm: u32,
        today: chrono::NaiveDate,
    ) -> Result<(), PlanError> {
        let mut after = self.clone();
        after.plot = Plot {
            length_cm,
            width_cm,
            aisle_cm,
            ..self.plot.clone()
        };
        after.plot.validate()?;
        let first = self
            .beds
            .values()
            .flat_map(|b| b.layouts.keys().copied())
            .min()
            .unwrap_or(chrono::Datelike::year(&today))
            .min(chrono::Datelike::year(&today));
        after.check_placements(first)?;
        *self = after;
        Ok(())
    }
}
