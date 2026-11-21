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

/// Two beds keep the aisle if they are far enough apart on one axis (S03 §15).
fn keeps_aisle(a: &Geometry, b: &Geometry, aisle_cm: u32) -> bool {
    let aisle = i64::from(aisle_cm);
    gap(a.x, a.length_cm, b.x, b.length_cm) >= aisle
        || gap(a.y, a.width_cm, b.y, b.width_cm) >= aisle
}

impl Plan {
    /// Years from `from_year` to the last year with a recorded layout (a
    /// layout holds until the next one, S03 §7).
    fn checked_years(&self, from_year: i32) -> std::ops::RangeInclusive<i32> {
        let last = self
            .beds
            .values()
            .flat_map(|p| p.layouts.keys().copied())
            .max()
            .unwrap_or(from_year)
            .max(from_year);
        from_year..=last
    }

    /// Checks the layout of every bed, in every year from `from_year` on: the
    /// bed fits in the plot, has a valid width and does not overlap another
    /// one. The aisle is not a rule but a question (S03 §15): see
    /// [`Plan::check_placement_change`].
    pub fn check_placements(&self, from_year: i32) -> Result<(), PlanError> {
        match self.placement_errors(from_year).into_iter().next() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// What a command checks after placing, moving or resizing a bed (S03
    /// §15): the placement rules, then, unless the user chose "Keep here"
    /// (`keep_close`), no pair of beds made closer than the aisle compared
    /// with `before`. The command does not save when it fails.
    pub fn check_placement_change(
        &self,
        before: &Plan,
        from_year: i32,
        keep_close: bool,
    ) -> Result<(), PlanError> {
        self.check_placements(from_year)?;
        if keep_close {
            return Ok(());
        }
        match self.pairs_made_too_close(before, from_year).next() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// The pairs of beds too close in `self` that kept the aisle in `before`,
    /// or did not exist there that year: a pair kept close on purpose earlier
    /// does not count (S03 §15).
    fn pairs_made_too_close<'a>(
        &'a self,
        before: &'a Plan,
        from_year: i32,
    ) -> impl Iterator<Item = PlanError> + 'a {
        self.checked_years(from_year).flat_map(move |year| {
            let beds = self.beds_in(year);
            let previous = before.beds_in(year);
            let mut errors = Vec::new();
            for (i, (a_id, _, a)) in beds.iter().enumerate() {
                for (b_id, _, b) in &beds[i + 1..] {
                    if keeps_aisle(a, b, self.plot.aisle_cm) {
                        continue;
                    }
                    let find = |id: BedId| previous.iter().find(|(p, _, _)| *p == id);
                    let kept_before = match (find(*a_id), find(*b_id)) {
                        (Some((_, _, pa)), Some((_, _, pb))) => {
                            keeps_aisle(pa, pb, before.plot.aisle_cm)
                        }
                        _ => true,
                    };
                    if kept_before {
                        errors.push(PlanError::AisleTooNarrow {
                            a: *a_id,
                            b: *b_id,
                            year,
                            aisle_cm: self.plot.aisle_cm,
                        });
                    }
                }
            }
            errors
        })
    }

    /// Every placement problem from `from_year` on, not only the first one: the
    /// "to resolve" list of an import shows them all (S03 §13).
    pub fn placement_errors(&self, from_year: i32) -> Vec<PlanError> {
        let mut errors = Vec::new();
        for year in self.checked_years(from_year) {
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
        Ok(())
    }
}

impl Plan {
    /// Changes the dimensions and the aisle width; the case size stays (S03 §4).
    /// Refused, with the plan untouched, if the new plot is invalid or no
    /// longer holds the beds of any year: the plot is one for the whole garden
    /// and a past layout cannot be repaired, so a plot that breaks one is refused.
    /// A pair already closer than the old aisle does not count (S03 §15).
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
        after.check_placement_change(self, first, false)?;
        *self = after;
        Ok(())
    }
}
