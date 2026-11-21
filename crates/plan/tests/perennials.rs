//! Perennials (S03 §2 bis, v0.19): cases T15 to T19 of S03 §12, on the red
//! fruits of S02 §3 ter, and their edge cases.

mod common;

use catalogue::{Catalogue, Crop};
use chrono::NaiveDate;
use common::*;
use plan::{
    Appearance, Mode, Occupation, PlanError, Planting, Slot, Stage, TaskType, YearView,
};

fn fruit(id: &str) -> &'static Crop {
    Catalogue::v1().crop(id).unwrap()
}

fn fruit_planting(id: &str, t1: u32, t2: u32, mode: Mode, date: NaiveDate) -> Planting {
    Planting::new(fruit(id), bed_id(100), zone(t1, t2), mode, date, None, None).unwrap()
}

/// Raspberry T15: plants on 2026-11-20 on [1,2].
fn raspberry() -> Planting {
    fruit_planting("raspberry", 1, 2, Mode::Plants, d(2026, 11, 20))
}

fn harvests(p: &Planting) -> Vec<(NaiveDate, NaiveDate)> {
    p.harvests().iter().map(|h| (h.start, h.end)).collect()
}

/// T15 · Raspberry, plants on 2026-11-20 on [1,2]
/// → `m` 2026-11-20 · `r` 2027-06-25 · `f` 2036-11-20 · 10 harvests
#[test]
fn t15_raspberry_dates_and_harvests() {
    let s = raspberry();
    let dates = s.dates();
    assert_eq!(dates.planting_out, d(2026, 11, 20));
    assert_eq!(dates.harvest_start, d(2027, 6, 25));
    assert_eq!(dates.end, d(2036, 11, 20));
    let h = harvests(&s);
    assert_eq!(h.len(), 10);
    assert_eq!(h[0], (d(2027, 6, 25), d(2027, 8, 9)));
    assert_eq!(h[9], (d(2036, 6, 25), d(2036, 8, 9)));
    for (i, (start, _)) in h.iter().enumerate() {
        assert_eq!(*start, d(2027 + i as i32, 6, 25));
    }
}

/// T15 · Display: bare in winter, young shoot then adult before the first
/// harvest, harvest each summer, adult the rest of the season.
#[test]
fn t15_raspberry_display() {
    let s = raspberry();
    let plant = |stage| Appearance::Plant(stage);
    assert_eq!(s.appearance_on(d(2026, 11, 19)), Appearance::Nothing);
    assert_eq!(s.appearance_on(d(2026, 12, 15)), plant(Stage::Dormant));
    assert_eq!(s.appearance_on(d(2027, 3, 5)), plant(Stage::YoungShoot));
    assert_eq!(s.appearance_on(d(2027, 4, 15)), plant(Stage::Adult));
    assert_eq!(s.appearance_on(d(2027, 7, 1)), plant(Stage::Harvest));
    assert_eq!(s.appearance_on(d(2027, 9, 1)), plant(Stage::Adult));
    assert_eq!(s.appearance_on(d(2028, 1, 10)), plant(Stage::Dormant));
    assert_eq!(s.appearance_on(d(2028, 7, 1)), plant(Stage::Harvest));
    assert_eq!(s.appearance_on(d(2036, 11, 20)), Appearance::Nothing);
    // The bounds of winter: 1 March is in the season, 15 November is not.
    assert_eq!(s.appearance_on(d(2028, 2, 29)), plant(Stage::Dormant));
    assert_eq!(s.appearance_on(d(2028, 3, 1)), plant(Stage::Adult));
    assert_eq!(s.appearance_on(d(2028, 11, 14)), plant(Stage::Adult));
    assert_eq!(s.appearance_on(d(2028, 11, 15)), plant(Stage::Dormant));
}

#[test]
fn t15_changes_agree_with_appearance_on_every_day() {
    let s = raspberry();
    let changes = s.appearance_changes();
    assert_eq!(changes.last().unwrap().appearance, Appearance::Nothing);
    let mut day = d(2026, 11, 1);
    while day < d(2037, 1, 1) {
        let from_changes = changes
            .iter()
            .take_while(|c| c.from <= day)
            .last()
            .map_or(Appearance::Nothing, |c| c.appearance);
        assert_eq!(from_changes, s.appearance_on(day), "{day}");
        day += chrono::Duration::days(1);
    }
}

/// T16 · Raspberry T15; lettuce plants on 2028-04-01 on [2,3]
/// → Conflict in year 2: segments [2,2], period [2028-04-01, 2028-06-10[
#[test]
fn t16_conflict_with_an_annual_crop_in_year_2() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(planting_id(1), raspberry()).unwrap();
    let lettuce = planting("lettuce", bed, 2, 3, Mode::Plants, d(2028, 4, 1));
    assert_eq!(
        plan.add_planting(planting_id(2), lettuce).unwrap_err(),
        PlanError::Conflict {
            with: planting_id(1),
            segments: zone(2, 2),
            period: Occupation {
                start: d(2028, 4, 1),
                end: d(2028, 6, 10),
            },
        }
    );
    let beside = planting("lettuce", bed, 3, 4, Mode::Plants, d(2028, 4, 1));
    plan.add_planting(planting_id(3), beside).unwrap();
}

/// T17 · Raspberry T15 alone on the bed, year 2028
/// → One slot: [3,6] from 2028-03-01 to 2028-11-15; no raspberry suggested
#[test]
fn t17_free_slot_beside_the_perennial() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(planting_id(1), raspberry()).unwrap();
    let slots = plan.slots(2028);
    assert_eq!(
        slots,
        vec![Slot {
            bed_id: bed,
            year: 2028,
            zone: zone(3, 6),
            start: d(2028, 3, 1),
            end: d(2028, 11, 15),
        }]
    );
    let mut crops = Catalogue::reference().crops().to_vec();
    crops.push(fruit("raspberry").clone());
    let catalogue = Catalogue::from_crops(crops);
    // In November, the raspberry is in its planting window: it would fit.
    let late = Slot {
        start: d(2028, 11, 1),
        ..slots[0]
    };
    for slot in [slots[0], late] {
        let suggestions = plan.suggestions(&slot, &catalogue, today());
        assert!(!suggestions.is_empty() || slot == late);
        assert!(suggestions.iter().all(|s| s.crop_id != "raspberry"));
        assert!(plan::evaluate(&slot, fruit("raspberry"), today()).is_none());
    }
}

/// T18 · Raspberry T15 ended by hand on 2029-09-30 (stored `f` 2029-10-01)
/// → 3 harvests; year 2030: slot [1,6] from 2030-03-01 to 2030-11-15
#[test]
fn t18_ending_a_perennial() {
    let (mut plan, bed) = plan_with_bed();
    let mut s = raspberry();
    s.manual_end = Some(d(2029, 10, 1));
    s.check().unwrap();
    assert_eq!(harvests(&s).len(), 3);
    assert_eq!(harvests(&s)[2], (d(2029, 6, 25), d(2029, 8, 9)));
    plan.add_planting(planting_id(1), s).unwrap();
    assert_eq!(
        plan.slots(2030),
        vec![Slot {
            bed_id: bed,
            year: 2030,
            zone: zone(1, 6),
            start: d(2030, 3, 1),
            end: d(2030, 11, 15),
        }]
    );

    // Before the first harvest: accepted, no harvest.
    let mut early = raspberry();
    early.manual_end = Some(d(2027, 1, 1));
    early.check().unwrap();
    assert!(early.harvests().is_empty());
    // On the day of planting out ("19 November" read, `f` = `m`): refused.
    let mut none = raspberry();
    none.manual_end = Some(d(2026, 11, 20));
    assert!(matches!(
        none.check(),
        Err(PlanError::InvalidManualEnd { .. })
    ));
}

/// T19 · Strawberry, plants on 2027-08-20 on [1,6]
/// → `m` 2027-08-20 · `r` 2028-05-15 · `f` 2030-08-20 · 3 harvests · tasks
#[test]
fn t19_strawberry_harvests_and_tasks() {
    let s = fruit_planting("strawberry", 1, 6, Mode::Plants, d(2027, 8, 20));
    let dates = s.dates();
    assert_eq!(dates.planting_out, d(2027, 8, 20));
    assert_eq!(dates.harvest_start, d(2028, 5, 15));
    assert_eq!(dates.end, d(2030, 8, 20));
    assert_eq!(
        harvests(&s),
        [2028, 2029, 2030].map(|y| (d(y, 5, 15), d(y, 6, 24)))
    );
    let id = planting_id(7);
    let tasks: Vec<(String, TaskType, NaiveDate)> = s
        .tasks(id)
        .into_iter()
        .map(|t| (t.id, t.task_type, t.date))
        .collect();
    assert_eq!(
        tasks,
        vec![
            (format!("{id}/plant"), TaskType::Plant, d(2027, 8, 20)),
            (format!("{id}/harvest"), TaskType::Harvest, d(2028, 5, 15)),
            (format!("{id}/harvest-2"), TaskType::Harvest, d(2029, 5, 15)),
            (format!("{id}/harvest-3"), TaskType::Harvest, d(2030, 5, 15)),
        ]
    );
}

#[test]
fn the_lifespan_is_copied_and_follows_a_change_of_crop() {
    let mut s = raspberry();
    assert_eq!(s.perennial, fruit("raspberry").perennial);
    s.change_crop(crop("lettuce"), None).unwrap();
    assert_eq!(s.perennial, None);
    assert_eq!(harvests(&s).len(), 1);
    s.change_crop(fruit("strawberry"), None).unwrap();
    assert_eq!(s.dates().end, d(2029, 11, 20));
}

#[test]
fn a_lifespan_out_of_range_is_refused() {
    for years in [0, 21] {
        let mut s = raspberry();
        s.perennial.as_mut().unwrap().lifespan_years = years;
        assert!(matches!(s.check(), Err(PlanError::OutOfRange { .. })), "{years}");
    }
}

#[test]
fn a_perennial_planted_on_29_february_ends_on_28_february() {
    let s = fruit_planting("strawberry", 1, 1, Mode::Plants, d(2028, 2, 29));
    assert_eq!(s.dates().end, d(2031, 2, 28));
}

#[test]
fn a_perennial_harvests_the_same_year_when_planted_early_enough() {
    // Open ground on 2027-01-01: `m + 180` is 2027-06-30, after the 15 May of
    // the strawberry, so the first harvest is the next year.
    let s = fruit_planting("strawberry", 1, 1, Mode::OpenGround, d(2027, 1, 1));
    assert_eq!(s.dates().harvest_start, d(2028, 5, 15));
    // A raspberry planted on 2026-12-01: 180 days is 2027-05-30, before 25 June.
    let r = fruit_planting("raspberry", 1, 1, Mode::Plants, d(2026, 12, 1));
    assert_eq!(r.dates().harvest_start, d(2027, 6, 25));
}

#[test]
fn a_perennial_shows_in_the_year_view_of_each_year_it_lives() {
    let (mut plan, _) = plan_with_bed();
    plan.add_planting(planting_id(1), raspberry()).unwrap();
    for year in [2026, 2027, 2031, 2036] {
        let view = YearView::build(Some(&plan), year, today(), Catalogue::v1());
        assert_eq!(view.plantings.len(), 1, "{year}");
        assert_eq!(view.plantings[0].harvests.len(), 10);
    }
    let view = YearView::build(Some(&plan), 2037, today(), Catalogue::v1());
    assert!(view.plantings.is_empty());
    assert_eq!(view.history.len(), 1);
    let tasks = YearView::build(Some(&plan), 2030, today(), Catalogue::v1()).tasks;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].date, d(2030, 6, 25));
}

#[test]
fn deleting_the_bed_in_year_5_deletes_the_perennial() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(planting_id(1), raspberry()).unwrap();
    plan.delete_bed(bed, 2031, today()).unwrap();
    assert!(plan.plantings.is_empty());
}

#[test]
fn an_old_document_without_perennial_reads_as_annual() {
    let s = planting("tomato", bed_id(100), 1, 1, Mode::Plants, d(2027, 5, 1));
    let json = serde_json::to_value(&s).unwrap();
    assert!(json.get("perennial").is_none());
    let back: Planting = serde_json::from_value(json).unwrap();
    assert_eq!(back, s);
    let r = raspberry();
    let back: Planting = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
    assert_eq!(back, r);
}
