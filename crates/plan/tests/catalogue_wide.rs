//! Invariants checked on every crop of the V1 catalogue (67 crops), over many
//! slots: a suggestion must always make a valid planting that stays in its slot.
//! The cases of S03 §12 only cover the 6 reference crops.

mod common;

use catalogue::Catalogue;
use common::*;
use plan::{Mode, Slot};

#[test]
fn every_crop_gives_valid_suggestions_in_any_slot() {
    let (plan, bed) = plan_with_bed();
    let today = d(2027, 1, 10);
    let mut produced = 0;
    for crop in Catalogue::v1().iter() {
        let mut start = d(2027, 3, 1);
        while start <= d(2027, 11, 1) {
            for length in [36_i64, 60, 90, 150, 260] {
                let end = start + chrono::Duration::days(length - 1);
                if end > d(2027, 11, 15) {
                    continue;
                }
                let slot = Slot {
                    bed_id: bed,
                    year: 2027,
                    zone: zone(1, 6),
                    start,
                    end,
                };
                let Some(s) = plan::evaluate(&slot, crop, today) else {
                    continue;
                };
                produced += 1;
                let name = format!("{} {start} +{length}", crop.id);
                let planting = s
                    .into_planting(crop, &slot)
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
                planting.check().unwrap_or_else(|e| panic!("{name}: {e}"));
                assert!(
                    s.planting_out >= start,
                    "{name}: planting out before the slot"
                );
                // Only a slot that reaches 15 November may be overrun (winter crops).
                let open_ended =
                    end == d(2027, 11, 15) && s.harvest_start > end + chrono::Duration::days(1);
                assert!(
                    open_ended || s.end <= end + chrono::Duration::days(1),
                    "{name}: ends after the slot"
                );
                assert!(s.harvest_start <= s.end, "{name}: harvest after the end");
                assert!(s.score() > 0.0 && s.score() <= 1.0, "{name}: score");
                if s.mode == Mode::Indoor {
                    assert!(s.date >= today, "{name}: sowing in the past");
                }
                // Planted alone in an empty bed, it never conflicts.
                let mut alone = plan.clone();
                alone
                    .add_planting(planting_id(1), planting)
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
            }
            start += chrono::Duration::days(5);
        }
    }
    assert!(
        produced > 500,
        "only {produced} suggestions over the whole catalogue"
    );
}

/// The crops no free slot can ever suggest. Winter crops (garlic, spring onion)
/// harvest the next year: they are suggested in the slots that reach 15
/// November, which they may overrun (S03 §9). If this list is not empty, the
/// catalogue or the rules changed: check it is on purpose.
const NEVER_SUGGESTED: &[&str] = &[];

#[test]
fn only_the_known_crops_are_never_suggested() {
    let (_plan, bed) = plan_with_bed();
    let today = d(2027, 1, 10);
    let mut never = Vec::new();
    for crop in Catalogue::v1().iter() {
        let slot = Slot {
            bed_id: bed,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 3, 1),
            end: d(2027, 11, 15),
        };
        let fits = (0..260).any(|n| {
            let start = slot.start + chrono::Duration::days(n);
            plan::evaluate(&Slot { start, ..slot }, crop, today).is_some()
        });
        if !fits {
            never.push(crop.id.as_str());
        }
    }
    assert_eq!(never, NEVER_SUGGESTED);
}

#[test]
fn a_winter_crop_may_run_past_the_end_of_the_search_window() {
    let (_plan, bed) = plan_with_bed();
    let garlic = Catalogue::v1().crop("garlic").unwrap();
    let today = d(2027, 1, 10);
    let slot = Slot {
        bed_id: bed,
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 10, 1),
        end: d(2027, 11, 15),
    };
    let s = plan::evaluate(&slot, garlic, today).expect("garlic is suggested");
    // Planted on 15 October, harvested from the next summer: no cut, no manual end.
    assert_eq!(s.planting_out, d(2027, 10, 15));
    assert!(s.end > d(2028, 1, 1));
    assert_eq!(s.manual_end, None);
    assert!(s.score() > 0.0 && s.score() <= 1.0);
    // A slot that ends earlier is not open: the crop is still discarded.
    let closed = Slot {
        end: d(2027, 11, 1),
        ..slot
    };
    assert!(plan::evaluate(&closed, garlic, today).is_none());
}
