//! Placement rules of the atelier (S03 §15): terrain dimensions, planches
//! inside the terrain, no overlap, allée width between planches.

use crate::error::PlanError;
use crate::ids::PlancheId;
use crate::model::{Geometry, MAX_NAME_CHARS, MIN_WIDTH_CM, Plan, Terrain};

/// Allée widths offered by S01 F-PLN-2, in cm.
pub const ALLEE_CHOICES: [u32; 3] = [30, 50, 80];
/// Smallest and largest terrain side, in cm (S03 §15).
pub const MIN_TERRAIN_CM: u32 = 100;
pub const MAX_TERRAIN_CM: u32 = 10_000;

impl Terrain {
    /// Checks the dimensions and the allée width (S03 §15).
    pub fn validate(&self) -> Result<(), PlanError> {
        let case = self.case_cm.cm();
        let side_ok = |side: u32| (MIN_TERRAIN_CM.max(case)..=MAX_TERRAIN_CM).contains(&side);
        if !side_ok(self.longueur_cm) || !side_ok(self.largeur_cm) {
            return Err(PlanError::InvalidTerrain {
                longueur_cm: self.longueur_cm,
                largeur_cm: self.largeur_cm,
            });
        }
        if !ALLEE_CHOICES.contains(&self.allee_cm) {
            return Err(PlanError::InvalidAllee(self.allee_cm));
        }
        Ok(())
    }
}

/// Checks a planche name: 1 to 30 characters once trimmed (S01 F-PLN-1).
pub fn check_name(nom: &str) -> Result<(), PlanError> {
    let chars = nom.trim().chars().count();
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
    /// Checks the layout of every planche, in every year from `from_year` to
    /// the last year with a recorded layout (a layout holds until the next
    /// one, S03 §7): the planche fits in the terrain, has a valid width, does
    /// not overlap another one and keeps the allée width from it.
    ///
    /// Operations call it after changing a layout; the command then does not
    /// save when it fails.
    pub fn check_placements(&self, from_year: i32) -> Result<(), PlanError> {
        let last = self
            .planches
            .values()
            .flat_map(|p| p.dispositions.keys().copied())
            .max()
            .unwrap_or(from_year)
            .max(from_year);
        for year in from_year..=last {
            let planches = self.planches_in(year);
            for (id, _, geometry) in &planches {
                self.check_inside(*id, geometry, year)?;
            }
            for (i, (a_id, _, a)) in planches.iter().enumerate() {
                for (b_id, _, b) in &planches[i + 1..] {
                    self.check_apart(*a_id, a, *b_id, b, year)?;
                }
            }
        }
        Ok(())
    }

    fn check_inside(&self, id: PlancheId, g: &Geometry, year: i32) -> Result<(), PlanError> {
        if g.largeur_cm < MIN_WIDTH_CM {
            return Err(PlanError::InvalidWidth(g.largeur_cm));
        }
        let t = &self.terrain;
        let fits = u64::from(g.x) + u64::from(g.longueur_cm) <= u64::from(t.longueur_cm)
            && u64::from(g.y) + u64::from(g.largeur_cm) <= u64::from(t.largeur_cm);
        if !fits {
            return Err(PlanError::OutsideTerrain {
                planche_id: id,
                year,
            });
        }
        Ok(())
    }

    fn check_apart(
        &self,
        a_id: PlancheId,
        a: &Geometry,
        b_id: PlancheId,
        b: &Geometry,
        year: i32,
    ) -> Result<(), PlanError> {
        let dx = gap(a.x, a.longueur_cm, b.x, b.longueur_cm);
        let dy = gap(a.y, a.largeur_cm, b.y, b.largeur_cm);
        if dx < 0 && dy < 0 {
            return Err(PlanError::PlanchesOverlap {
                a: a_id,
                b: b_id,
                year,
            });
        }
        let allee = i64::from(self.terrain.allee_cm);
        if dx < allee && dy < allee {
            return Err(PlanError::AlleeTooNarrow {
                a: a_id,
                b: b_id,
                year,
                allee_cm: self.terrain.allee_cm,
            });
        }
        Ok(())
    }
}

impl Plan {
    /// Changes the dimensions and the allée width; the case size stays (S03 §4).
    /// Refused, with the plan untouched, if the new terrain is invalid or no
    /// longer holds the planches of the current year and the following ones.
    pub fn set_terrain(
        &mut self,
        longueur_cm: u32,
        largeur_cm: u32,
        allee_cm: u32,
        today: chrono::NaiveDate,
    ) -> Result<(), PlanError> {
        let mut after = self.clone();
        after.terrain = Terrain {
            longueur_cm,
            largeur_cm,
            allee_cm,
            ..self.terrain.clone()
        };
        after.terrain.validate()?;
        after.check_placements(chrono::Datelike::year(&today))?;
        *self = after;
        Ok(())
    }
}
