//! Rules of S03 that the numbered cases T1 to T13 (see `s03_cas.rs`) do not
//! pin down: each test asserts the exact value written in the spec.

mod common;

use catalogue::Catalogue;
use common::*;
use plan::{
    Appearance, CellCm, Fraction, Layout, Mode, Occupation, PlanError, Slot, Stage, quantity,
};

fn slot(bed: plan::BedId, start: chrono::NaiveDate, end: chrono::NaiveDate) -> Slot {
    Slot {
        bed_id: bed,
        year: 2027,
        zone: zone(1, 6),
        start,
        end,
    }
}

/// S03 §12, after T13: with an empty garden the slot of T10 gives leek (1.0),
/// then the 0.7 group (carrot, tomato, zucchini).
#[test]
fn t10_ranking_on_an_empty_garden() {
    let (plan, bed) = plan_with_bed();
    let s = slot(bed, d(2027, 5, 1), d(2027, 11, 15));
    let today = d(2027, 4, 20);
    let all = plan.suggestions_with_ties(&s, Catalogue::reference(), today);
    let scored: Vec<(&str, i64)> = all
        .iter()
        .map(|x| (x.crop_id.as_str(), x.score_tenths()))
        .collect();
    assert_eq!(scored[0], ("leek", 10));
    for id in ["tomato", "carrot", "zucchini"] {
        assert!(scored.contains(&(id, 7)), "{id}: {scored:?}");
    }
    let top: Vec<String> = plan
        .suggestions(&s, Catalogue::reference(), today)
        .into_iter()
        .map(|x| x.crop_id)
        .collect();
    assert_eq!(top, ["leek", "carrot", "tomato"]);
}

/// §3: the stage thresholds on a real planting.
#[test]
fn stages_follow_p_on_a_real_planting() {
    let p = planting("tomato", bed_id(1), 1, 1, Mode::OpenGround, d(2027, 5, 5));
    let dates = p.dates();
    let span = (dates.harvest_start - dates.planting_out).num_days();
    let at = |n: i64| p.appearance_on(dates.planting_out + chrono::Duration::days(n));
    // Each range includes its lower bound.
    let first_young = (0..span)
        .find(|n| at(*n) == Appearance::Plant(Stage::YoungShoot))
        .unwrap();
    let first_adult = (0..span)
        .find(|n| at(*n) == Appearance::Plant(Stage::Adult))
        .unwrap();
    assert!(first_young * 100 >= span * 15);
    assert!((first_young - 1) * 100 < span * 15);
    assert!(first_adult * 100 >= span * 50);
    assert!((first_adult - 1) * 100 < span * 50);
    assert_eq!(at(span), Appearance::Plant(Stage::Harvest));
    assert_eq!(at(0), Appearance::Plant(Stage::Sowing));
}

/// §3: exact bounds p = 0.15 and p = 0.50 belong to the upper stage; bought
/// plants never go below young shoot.
#[test]
fn stage_bounds_belong_to_the_upper_stage() {
    let mut p = planting("tomato", bed_id(1), 1, 1, Mode::OpenGround, d(2027, 5, 1));
    p.indoor_days = 0;
    p.after_days = 20; // p = 0.15 at day 3, 0.5 at day 10
    p.duration_days = 30;
    let m = d(2027, 5, 1);
    let at = |s: &plan::Planting, n: i64| s.appearance_on(m + chrono::Duration::days(n));
    assert_eq!(at(&p, 2), Appearance::Plant(Stage::Sowing));
    assert_eq!(at(&p, 3), Appearance::Plant(Stage::YoungShoot));
    assert_eq!(at(&p, 9), Appearance::Plant(Stage::YoungShoot));
    assert_eq!(at(&p, 10), Appearance::Plant(Stage::Adult));
    assert_eq!(at(&p, 20), Appearance::Plant(Stage::Harvest));
    p.mode = Mode::Plants;
    assert_eq!(at(&p, 0), Appearance::Plant(Stage::YoungShoot));
    assert_eq!(at(&p, 10), Appearance::Plant(Stage::Adult));
}

/// §6: at least 1 row and 1 plant, and the formula floors.
#[test]
fn quantity_minimums() {
    let carrot = quantity(5, 1, CellCm::C25, crop("zucchini"));
    assert_eq!((carrot.rows, carrot.plants_per_row), (1, 1));
    let q = quantity(120, 2, CellCm::C50, crop("carrot"));
    assert_eq!((q.rows, q.plants_per_row, q.total()), (4, 25, 100));
}

/// §4: n = 6, quarter gives 2 (1.5 rounded up); sizes never fall under 1.
#[test]
fn shortcut_rounding() {
    assert_eq!(Fraction::Quarter.size(6), 2);
    assert_eq!(Fraction::Half.size(1), 1);
    assert_eq!(Fraction::Quarter.size(2), 1);
    assert_eq!(Fraction::Third.size(4), 1);
    assert_eq!(Fraction::Third.size(5), 2);
    assert_eq!(Fraction::Quarter.size(10), 3);
}

/// §4: the shortcut fits in the bed every year the occupation touches.
#[test]
fn shortcut_fits_every_year_touched() {
    let (mut plan, bed) = plan_with_bed();
    plan.set_layout(bed, 2028, Layout::Present(geometry(2)), today())
        .unwrap();
    let across = Occupation {
        start: d(2027, 11, 1),
        end: d(2028, 2, 1),
    };
    assert_eq!(plan.shortcut(bed, Fraction::Whole, across, None), None);
    assert_eq!(
        plan.shortcut(bed, Fraction::Quarter, across, None),
        Some(zone(1, 2))
    );
}

/// §5: a planting ending on 1 January does not touch the new year.
#[test]
fn a_planting_ending_on_january_first_does_not_touch_the_new_year() {
    let (mut plan, bed) = plan_with_bed();
    plan.delete_bed(bed, 2028, today()).unwrap();
    let p = occupying("lettuce", bed, 1, 2, d(2027, 11, 1), d(2028, 1, 1));
    plan.add_planting(planting_id(1), p).unwrap();
    let q = occupying("lettuce", bed, 3, 4, d(2027, 11, 1), d(2028, 1, 2));
    assert!(matches!(
        plan.add_planting(planting_id(2), q),
        Err(PlanError::BedMissing { year: 2028, .. })
    ));
}

/// §5: deletion from Y keeps a planting ending on 1 January of Y and deletes
/// one ending the day after.
#[test]
fn deleting_a_bed_boundary_is_1_january() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        occupying("lettuce", bed, 1, 2, d(2027, 11, 1), d(2028, 1, 1)),
    )
    .unwrap();
    plan.add_planting(
        planting_id(2),
        occupying("lettuce", bed, 3, 4, d(2027, 11, 1), d(2028, 1, 2)),
    )
    .unwrap();
    let deleted = plan.delete_bed(bed, 2028, today()).unwrap();
    assert_eq!(deleted, vec![planting_id(2)]);
    assert!(plan.plantings.contains_key(&planting_id(1)));
}

/// §7: bringing back an absent bed that would bring a year to 11 is refused.
#[test]
fn bringing_back_a_bed_cannot_make_an_eleventh() {
    let (mut plan, bed) = plan_with_bed();
    plan.delete_bed(bed, 2027, today()).unwrap();
    for n in 1..=10u128 {
        plan.add_bed(bed_id(200 + n), format!("P{n}"), 2027, geometry(4), today())
            .unwrap();
    }
    let error = plan
        .set_layout(bed, 2027, Layout::Present(geometry(4)), today())
        .unwrap_err();
    assert_eq!(
        error,
        PlanError::TooManyBeds {
            year: 2027,
            max: 10
        }
    );
}

/// §7: the layout of a bed in Y is its last layout saved for a year <= Y.
#[test]
fn layout_of_a_year_is_the_last_one_saved_before() {
    let (mut plan, bed) = plan_with_bed();
    plan.set_layout(bed, 2028, Layout::Present(geometry(4)), today())
        .unwrap();
    assert_eq!(plan.segment_count(bed, 2025), None);
    assert_eq!(plan.segment_count(bed, 2026), Some(6));
    assert_eq!(plan.segment_count(bed, 2027), Some(6));
    assert_eq!(plan.segment_count(bed, 2028), Some(4));
    assert_eq!(plan.segment_count(bed, 2031), Some(4));
}

/// §9 steps 4 and 5: the end is cut to the slot (entered end only then) and
/// the score is (f - D) / N.
#[test]
fn proposed_end_and_score() {
    let (_, bed) = plan_with_bed();
    let s = slot(bed, d(2027, 8, 2), d(2027, 10, 5));
    let l = plan::evaluate(&s, crop("lettuce"), today()).unwrap();
    assert_eq!(l.end, d(2027, 10, 6));
    assert_eq!(l.manual_end, Some(d(2027, 10, 6)));
    assert_eq!(l.score(), 1.0);
    let wide = slot(bed, d(2027, 8, 2), d(2027, 11, 15));
    let l = plan::evaluate(&wide, crop("lettuce"), today()).unwrap();
    assert_eq!(l.manual_end, None);
    assert_eq!(l.end, d(2027, 10, 11));
    // (10-11 - 08-02) = 70 days over N = 106.
    assert_eq!(l.score_tenths(), 7);
}

/// §17: a shortened bed leaves a planting that overflows; it is listed and
/// cannot be edited until it fits.
#[test]
fn an_overflowing_planting_is_listed_and_cannot_be_edited() {
    let (mut plan, bed) = plan_with_bed();
    let p = planting("carrot", bed, 1, 6, Mode::OpenGround, d(2027, 3, 15));
    plan.add_planting(planting_id(1), p.clone()).unwrap();
    plan.set_layout(bed, 2027, Layout::Present(geometry(4)), today())
        .unwrap();
    assert!(
        plan.incoherences()
            .iter()
            .any(|i| i.code() == "zone_outside_bed"),
        "{:?}",
        plan.incoherences()
            .iter()
            .map(|i| i.code())
            .collect::<Vec<_>>()
    );
    assert!(matches!(
        plan.replace_planting(planting_id(1), p.shifted(1)),
        Err(PlanError::ZoneOutOfBed { .. })
    ));
    let mut fits = p.clone();
    fits.segment_end = 4;
    plan.replace_planting(planting_id(1), fits).unwrap();
}

/// §17: plantings of a past year stay editable.
#[test]
fn plantings_of_a_past_year_stay_editable() {
    let (mut plan, bed) = plan_with_bed();
    let p = planting("lettuce", bed, 1, 2, Mode::Plants, d(2026, 3, 1));
    plan.add_planting(planting_id(1), p.clone()).unwrap();
    plan.replace_planting(planting_id(1), p.shifted(3)).unwrap();
    plan.shift_planting(planting_id(1), -3).unwrap();
}
