//! Suggestions for a free slot (S03 §9).

use std::cmp::Reverse;

use catalogue::{Catalogue, Crop};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::days::plus_days;
use crate::error::PlanError;
use crate::model::Plan;
use crate::planting::{Mode, Planting};
use crate::slots::Slot;
use crate::window::in_window;
use crate::zone::Zone;

/// At most this many crops are suggested for a slot.
pub const MAX_SUGGESTIONS: usize = 3;

/// A crop proposed for a slot, with the values that prefill the new planting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub crop_id: String,
    pub mode: Mode,
    /// Sowing date (indoor, open ground) or planting date (plants).
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
    pub manual_end: Option<NaiveDate>,
    /// `f − D`: days of the slot the planting fills.
    pub filled_days: i64,
    /// `N = F − D + 1`: length of the slot.
    pub slot_days: i64,
    /// The crop is of the family of the last planting that ended before the
    /// slot on its zone: it comes after the others of the same score.
    pub recent_family: bool,
}

impl Suggestion {
    /// Filling score `(f − D) ÷ N`, between 0 and 1.
    pub fn score(&self) -> f64 {
        self.filled_days as f64 / self.slot_days as f64
    }

    /// The score rounded to a tenth, in tenths (0 to 10), with integer maths.
    pub fn score_tenths(&self) -> i64 {
        (20 * self.filled_days + self.slot_days) / (2 * self.slot_days)
    }

    /// The planting to create when the suggestion is touched.
    pub fn into_planting(&self, crop: &Crop, slot: &Slot) -> Result<Planting, PlanError> {
        Planting::new(
            crop,
            slot.bed_id,
            self.zone,
            self.mode,
            self.date,
            None,
            self.manual_end,
        )
    }
}

/// Evaluates one crop for a slot, without ranking (S03 §9, steps 1 to 5).
///
/// Returns `None` when the crop is discarded: no planting date in the window
/// during the first 28 days of the slot, or not enough room for half of the
/// harvest.
pub fn evaluate(slot: &Slot, crop: &Crop, today: NaiveDate) -> Option<Suggestion> {
    // A perennial is never suggested: a slot is the room of one season (S03 §9).
    if crop.perennial.is_some() {
        return None;
    }
    let start = slot.start;
    let slot_end = slot.end;

    // 1. The first day of [D, D + 27] in the planting window.
    let m = (0..28)
        .map(|i| plus_days(start, i))
        .find(|day| in_window(crop, *day))?;

    // 2. Mode.
    let pot_days = i64::from(crop.indoor_days);
    let (mode, date) = if pot_days > 0 {
        let sowing = plus_days(m, -pot_days);
        if sowing >= today {
            (Mode::Indoor, sowing)
        } else {
            (Mode::Plants, m)
        }
    } else {
        (Mode::OpenGround, m)
    };
    let planting = Planting {
        crop_id: crop.id.clone(),
        bed_id: slot.bed_id,
        segment_start: slot.zone.t1,
        segment_end: slot.zone.t2,
        mode,
        date,
        indoor_days: crop.indoor_days,
        after_days: crop.harvest_after_days,
        duration_days: crop.harvest_duration_days,
        perennial: None,
        manual_end: None,
    };
    let harvest_start = planting.dates().harvest_start;

    // A slot that reaches the end of the search window (15 November) is not
    // closed by another planting: a crop that only starts to harvest after it
    // (garlic, spring onion: the winter crops) may run past it, into the next
    // year, up to its natural end.
    let duration = i64::from(crop.harvest_duration_days);
    let after_slot = plus_days(slot_end, 1);
    let open_ended =
        NaiveDate::from_ymd_opt(slot.year, 11, 15) == Some(slot_end) && harvest_start > after_slot;

    // 3. Minimum harvest: half the harvest duration, rounded up.
    if !open_ended && plus_days(harvest_start, (duration + 1) / 2) > after_slot {
        return None;
    }

    // 4. Proposed end.
    let natural_end = plus_days(harvest_start, duration);
    let end = if open_ended {
        natural_end
    } else {
        after_slot.min(natural_end)
    };

    Some(Suggestion {
        crop_id: crop.id.clone(),
        mode,
        date,
        zone: slot.zone,
        planting_out: m,
        harvest_start,
        end,
        manual_end: (end < natural_end).then_some(end),
        // 5. Filling score.
        filled_days: (end.min(after_slot) - start).num_days(),
        slot_days: slot.length_days(),
        recent_family: false,
    })
}

impl Plan {
    /// Up to 3 crops for a slot, ranked (S03 §9):
    ///
    /// 1. score rounded to a tenth, highest first;
    /// 2. a family different from the last planting that ended before the slot on a
    ///    segment of its zone;
    /// 3. alphabetical order of the name.
    pub fn suggestions(
        &self,
        slot: &Slot,
        catalogue: &Catalogue,
        today: NaiveDate,
    ) -> Vec<Suggestion> {
        let mut ranked = self.ranked_suggestions(slot, catalogue, today);
        ranked.truncate(MAX_SUGGESTIONS);
        ranked
    }

    /// The first [`MAX_SUGGESTIONS`] of [`Plan::suggestions`], plus the crops tied
    /// with the last of them (same score, same family flag), still in English
    /// alphabetical order. The tie-break by name is the display language's:
    /// the interface sorts these ties again with the names it shows.
    pub fn suggestions_with_ties(
        &self,
        slot: &Slot,
        catalogue: &Catalogue,
        today: NaiveDate,
    ) -> Vec<Suggestion> {
        let ranked = self.ranked_suggestions(slot, catalogue, today);
        let key = |s: &Suggestion| (Reverse(s.score_tenths()), s.recent_family);
        let Some(last) = ranked.get(MAX_SUGGESTIONS - 1).map(key) else {
            return ranked;
        };
        ranked
            .into_iter()
            .enumerate()
            .take_while(|(i, s)| *i < MAX_SUGGESTIONS || key(s) == last)
            .map(|(_, s)| s)
            .collect()
    }

    /// Every suggestion for the slot, ranked (S03 §9).
    fn ranked_suggestions(
        &self,
        slot: &Slot,
        catalogue: &Catalogue,
        today: NaiveDate,
    ) -> Vec<Suggestion> {
        let previous_families = self.families_of_last_finished_planting(slot, catalogue);
        let mut candidates: Vec<(Suggestion, &Crop)> = catalogue
            .iter()
            .filter_map(|c| {
                let mut suggestion = evaluate(slot, c, today)?;
                // A suggestion that runs past the slot (winter crops) must also
                // be free where it goes: the slot does not look beyond 15 November.
                if suggestion.end > plus_days(slot.end, 1) {
                    let planting = suggestion.into_planting(c, slot).ok()?;
                    self.validate_planting(None, &planting).ok()?;
                }
                suggestion.recent_family = previous_families.contains(&c.family);
                Some((suggestion, c))
            })
            .collect();
        candidates.sort_by_key(|(suggestion, crop)| {
            (
                Reverse(suggestion.score_tenths()),
                suggestion.recent_family,
                crop.name.to_lowercase(),
            )
        });
        candidates
            .into_iter()
            .map(|(suggestion, _)| suggestion)
            .collect()
    }

    /// Families of the plantings that ended last (`f ≤ D`) on a segment of the
    /// zone of the slot. Several plantings ending the same day all count.
    fn families_of_last_finished_planting(
        &self,
        slot: &Slot,
        catalogue: &Catalogue,
    ) -> Vec<String> {
        let finished: Vec<(&Planting, NaiveDate)> = self
            .plantings
            .values()
            .filter(|s| s.bed_id == slot.bed_id && s.zone().overlaps(&slot.zone))
            .map(|s| (s, s.dates().end))
            .filter(|(_, end)| *end <= slot.start)
            .collect();
        let Some(last) = finished.iter().map(|(_, end)| *end).max() else {
            return Vec::new();
        };
        finished
            .into_iter()
            .filter(|(_, end)| *end == last)
            .filter_map(|(s, _)| catalogue.crop(&s.crop_id))
            .map(|c| c.family.clone())
            .collect()
    }
}
