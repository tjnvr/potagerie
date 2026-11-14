//! Suggestions for a free slot (S03 §9).

use std::cmp::Reverse;

use catalogue::{Catalogue, Culture};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::creneaux::Creneau;
use crate::days::plus_days;
use crate::error::PlanError;
use crate::model::Plan;
use crate::serie::{Mode, Serie};
use crate::window::in_window;
use crate::zone::Zone;

/// At most this many crops are suggested for a slot.
pub const MAX_SUGGESTIONS: usize = 3;

/// A crop proposed for a slot, with the values that prefill the new serie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub culture_id: String,
    pub mode: Mode,
    /// Sowing date (abri, pleine terre) or planting date (plants).
    pub date: NaiveDate,
    pub zone: Zone,
    /// `m`.
    pub planting_out: NaiveDate,
    /// `r`.
    pub harvest_start: NaiveDate,
    /// Proposed end `f = min(F + 1, r + duration)`.
    pub end: NaiveDate,
    /// Set when the end was shortened to fit the slot: it then counts as an end
    /// modified by hand and is not recomputed.
    pub fin_manuelle: Option<NaiveDate>,
    /// `f − D`: days of the slot the serie fills.
    pub filled_days: i64,
    /// `N = F − D + 1`: length of the slot.
    pub slot_days: i64,
}

impl Suggestion {
    /// Filling score `(f − D) ÷ N`, between 0 and 1.
    pub fn score(&self) -> f64 {
        self.filled_days as f64 / self.slot_days as f64
    }

    /// The score rounded to a tenth, in tenths (0 to 10), with integer maths.
    pub fn score_dixiemes(&self) -> i64 {
        (20 * self.filled_days + self.slot_days) / (2 * self.slot_days)
    }

    /// The serie to create when the suggestion is touched.
    pub fn into_serie(&self, culture: &Culture, creneau: &Creneau) -> Result<Serie, PlanError> {
        Serie::new(
            culture,
            creneau.planche_id,
            self.zone,
            self.mode,
            self.date,
            None,
            self.fin_manuelle,
        )
    }
}

/// Evaluates one crop for a slot, without ranking (S03 §9, steps 1 to 5).
///
/// Returns `None` when the crop is discarded: no planting date in the window
/// during the first 28 days of the slot, or not enough room for half of the
/// harvest.
pub fn evaluate(creneau: &Creneau, culture: &Culture, today: NaiveDate) -> Option<Suggestion> {
    let start = creneau.start;
    let creneau_end = creneau.end;

    // 1. The first day of [D, D + 27] in the planting window.
    let m = (0..28)
        .map(|i| plus_days(start, i))
        .find(|day| in_window(culture, *day))?;

    // 2. Mode.
    let pot_days = i64::from(culture.abri_jours);
    let (mode, date) = if pot_days > 0 {
        let sowing = plus_days(m, -pot_days);
        if sowing >= today {
            (Mode::Abri, sowing)
        } else {
            (Mode::Plants, m)
        }
    } else {
        (Mode::PleineTerre, m)
    };
    let serie = Serie {
        culture_id: culture.id.clone(),
        planche_id: creneau.planche_id,
        troncon_debut: creneau.zone.t1,
        troncon_fin: creneau.zone.t2,
        mode,
        date,
        abri_jours: culture.abri_jours,
        apres_jours: culture.recolte_apres_jours,
        duree_jours: culture.recolte_duree_jours,
        fin_manuelle: None,
    };
    let harvest_start = serie.dates().harvest_start;

    // 3. Minimal harvest: half the harvest duration, rounded up.
    let duration = i64::from(culture.recolte_duree_jours);
    let after_creneau = plus_days(creneau_end, 1);
    if plus_days(harvest_start, (duration + 1) / 2) > after_creneau {
        return None;
    }

    // 4. Proposed end.
    let natural_end = plus_days(harvest_start, duration);
    let end = after_creneau.min(natural_end);

    Some(Suggestion {
        culture_id: culture.id.clone(),
        mode,
        date,
        zone: creneau.zone,
        planting_out: m,
        harvest_start,
        end,
        fin_manuelle: (end < natural_end).then_some(end),
        // 5. Filling score.
        filled_days: (end - start).num_days(),
        slot_days: creneau.length_days(),
    })
}

impl Plan {
    /// Up to 3 crops for a slot, ranked (S03 §9):
    ///
    /// 1. score rounded to a tenth, highest first;
    /// 2. a family different from the last serie that ended before the slot on a
    ///    troncon of its zone;
    /// 3. alphabetical order of the name.
    pub fn suggestions(
        &self,
        creneau: &Creneau,
        catalogue: &Catalogue,
        today: NaiveDate,
    ) -> Vec<Suggestion> {
        let previous_families = self.families_of_last_finished_serie(creneau, catalogue);
        let mut candidates: Vec<(Suggestion, &Culture)> = catalogue
            .iter()
            .filter_map(|c| Some((evaluate(creneau, c, today)?, c)))
            .collect();
        candidates.sort_by_key(|(suggestion, culture)| {
            (
                Reverse(suggestion.score_dixiemes()),
                previous_families.contains(&culture.famille),
                culture.nom.to_lowercase(),
            )
        });
        candidates
            .into_iter()
            .take(MAX_SUGGESTIONS)
            .map(|(suggestion, _)| suggestion)
            .collect()
    }

    /// Families of the series that ended last (`f ≤ D`) on a troncon of the
    /// zone of the slot. Several series ending the same day all count.
    fn families_of_last_finished_serie(
        &self,
        creneau: &Creneau,
        catalogue: &Catalogue,
    ) -> Vec<String> {
        let finished: Vec<(&Serie, NaiveDate)> = self
            .series
            .values()
            .filter(|s| s.planche_id == creneau.planche_id && s.zone().overlaps(&creneau.zone))
            .map(|s| (s, s.dates().end))
            .filter(|(_, end)| *end <= creneau.start)
            .collect();
        let Some(last) = finished.iter().map(|(_, end)| *end).max() else {
            return Vec::new();
        };
        finished
            .into_iter()
            .filter(|(_, end)| *end == last)
            .filter_map(|(s, _)| catalogue.culture(&s.culture_id))
            .map(|c| c.famille.clone())
            .collect()
    }
}
