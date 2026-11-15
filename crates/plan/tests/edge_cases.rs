//! Edge cases that S03 describes but T1 to T13 do not cover.

mod common;

use catalogue::Catalogue;
use common::*;
use plan::{
    Fraction, Layout, Mode, Occupation, Plan, PlanError, Planting, Slot, TaskType, Zone, evaluate,
};

// ---- §2 dates ----------------------------------------------------------

#[test]
fn bought_plants_are_planted_on_the_entered_day() {
    let s = planting("tomato", bed_id(100), 1, 1, Mode::Plants, d(2027, 5, 5));
    let dates = s.dates();
    assert_eq!(dates.sowing, None);
    assert_eq!(dates.planting_out, d(2027, 5, 5));
    assert_eq!(dates.harvest_start, d(2027, 7, 19));
}

#[test]
fn open_ground_adds_the_pot_phase_of_the_crop() {
    // Tomato: 56 days in pots would have been spent in the ground: 75 + 56.
    let s = planting("tomato", bed_id(100), 1, 1, Mode::OpenGround, d(2027, 5, 5));
    assert_eq!(
        s.dates().harvest_start,
        d(2027, 5, 5) + chrono::Duration::days(131)
    );
}

#[test]
fn entering_the_transplanting_date_stores_the_sowing_date() {
    // Entering the transplanting date instead of the sowing date: s = x − A.
    let sowing = Planting::sowing_from_transplanting(d(2027, 5, 5), 56);
    assert_eq!(sowing, d(2027, 3, 10));
    let s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, sowing);
    assert_eq!(s.dates().planting_out, d(2027, 5, 5));
}

#[test]
fn indoor_without_pot_days_for_a_crop_without_pots_is_refused() {
    for pots in [None, Some(0)] {
        let error = Planting::new(
            crop("green-bean"),
            bed_id(100),
            zone(1, 1),
            Mode::Indoor,
            d(2027, 5, 10),
            pots,
            None,
        )
        .unwrap_err();
        assert_eq!(error, PlanError::PotDaysRequired);
    }
}

#[test]
fn entered_pot_days_ignored_when_the_crop_has_pots() {
    let s = Planting::new(
        crop("tomato"),
        bed_id(100),
        zone(1, 1),
        Mode::Indoor,
        d(2027, 3, 10),
        Some(10),
        None,
    )
    .unwrap();
    assert_eq!(s.indoor_days, 56);
}

#[test]
fn durations_are_copied_at_creation() {
    let mut tomato = crop("tomato").clone();
    let s = Planting::new(
        &tomato,
        bed_id(100),
        zone(1, 1),
        Mode::Indoor,
        d(2027, 3, 10),
        None,
        None,
    )
    .unwrap();
    let before = s.dates();
    // A catalogue update does not move an existing planting.
    tomato.harvest_after_days = 90;
    tomato.harvest_duration_days = 10;
    assert_eq!(s.dates(), before);
    assert_eq!((s.indoor_days, s.after_days, s.duration_days), (56, 75, 70));
}

#[test]
fn changing_crop_copies_the_durations_again() {
    let mut s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, d(2027, 3, 10));
    s.change_crop(crop("lettuce"), None).unwrap();
    assert_eq!(
        (
            s.crop_id.as_str(),
            s.indoor_days,
            s.after_days,
            s.duration_days
        ),
        ("lettuce", 28, 50, 20)
    );
}

#[test]
fn changing_a_carrot_mode_drops_the_entered_pot_days() {
    let mut s = Planting::new(
        crop("carrot"),
        bed_id(100),
        zone(1, 1),
        Mode::Indoor,
        d(2027, 3, 1),
        Some(21),
        None,
    )
    .unwrap();
    s.change_mode(Mode::OpenGround, crop("carrot"), None)
        .unwrap();
    // As T3: nothing added for a carrot sown in the ground.
    assert_eq!(s.dates().harvest_start, d(2027, 6, 9));
}

#[test]
fn manual_end_must_be_after_harvest_start() {
    let tomato = crop("tomato");
    let new = |end| {
        Planting::new(
            tomato,
            bed_id(100),
            zone(1, 1),
            Mode::Indoor,
            d(2027, 3, 10),
            None,
            Some(end),
        )
    };
    assert!(matches!(
        new(d(2027, 7, 19)),
        Err(PlanError::InvalidManualEnd { .. })
    ));
    assert_eq!(new(d(2027, 7, 20)).unwrap().dates().end, d(2027, 7, 20));
}

#[test]
fn manual_end_is_not_recomputed() {
    let mut s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, d(2027, 3, 10));
    s.manual_end = Some(d(2027, 8, 1));
    s.date = d(2027, 3, 20);
    assert_eq!(s.dates().harvest_start, d(2027, 7, 29));
    assert_eq!(s.dates().end, d(2027, 8, 1));
}

#[test]
fn dragging_shifts_the_date_and_the_manual_end() {
    let mut s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, d(2027, 3, 10));
    s.manual_end = Some(d(2027, 8, 1));
    let shifted = s.shifted(7);
    assert_eq!(shifted.date, d(2027, 3, 17));
    assert_eq!(shifted.manual_end, Some(d(2027, 8, 8)));
    // The duration is kept.
    let (before, after) = (s.occupation(), shifted.occupation());
    assert_eq!(before.end - before.start, after.end - after.start);
}

#[test]
fn timeline_phases() {
    let s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, d(2027, 3, 10));
    let phases = s.phases();
    assert_eq!(
        phases.pots,
        Some(Occupation {
            start: d(2027, 3, 10),
            end: d(2027, 5, 5)
        })
    );
    assert_eq!(
        phases.growth,
        Occupation {
            start: d(2027, 5, 5),
            end: d(2027, 7, 19)
        }
    );
    assert_eq!(
        phases.harvest_start,
        Occupation {
            start: d(2027, 7, 19),
            end: d(2027, 9, 27)
        }
    );
    let plants = planting("tomato", bed_id(100), 1, 1, Mode::Plants, d(2027, 5, 5));
    assert_eq!(plants.phases().pots, None);
}

// ---- §4 zone shortcuts ----------------------------------------------------

#[test]
fn shortcut_sizes() {
    // k = max(1, round(n × fraction)), halves rounded up.
    assert_eq!(
        [
            Fraction::Whole,
            Fraction::Half,
            Fraction::Third,
            Fraction::Quarter
        ]
        .map(|f| f.size(6)),
        [6, 3, 2, 2]
    );
    assert_eq!(Fraction::Half.size(5), 3);
    assert_eq!(Fraction::Quarter.size(5), 1);
    assert_eq!(Fraction::Quarter.size(1), 1);
    assert_eq!(Fraction::Third.size(1), 1);
}

#[test]
fn shortcut_takes_the_first_free_run_of_segments() {
    let (mut plan, bed) = plan_with_bed();
    // [1,2] taken during the whole season.
    let occupied = planting("carrot", bed, 1, 2, Mode::OpenGround, d(2027, 3, 15));
    plan.add_planting(planting_id(1), occupied).unwrap();
    let occupation = Occupation {
        start: d(2027, 4, 1),
        end: d(2027, 6, 1),
    };
    assert_eq!(
        plan.shortcut(bed, Fraction::Half, occupation, None),
        Some(zone(3, 5))
    );
    assert_eq!(
        plan.shortcut(bed, Fraction::Third, occupation, None),
        Some(zone(3, 4))
    );
    // The whole bed is not free.
    assert_eq!(plan.shortcut(bed, Fraction::Whole, occupation, None), None);
    // Free after the carrot ends: back to the first segments.
    let after = Occupation {
        start: d(2027, 8, 2),
        end: d(2027, 9, 1),
    };
    assert_eq!(
        plan.shortcut(bed, Fraction::Whole, after, None),
        Some(zone(1, 6))
    );
    // Editing the carrot itself ignores it.
    assert_eq!(
        plan.shortcut(bed, Fraction::Whole, occupation, Some(planting_id(1))),
        Some(zone(1, 6))
    );
}

#[test]
fn shortcut_greyed_out_without_a_free_run() {
    let (mut plan, bed) = plan_with_bed();
    // Segments 2 and 5 are taken: no run of 3 is left free.
    for (i, t) in [2, 5].into_iter().enumerate() {
        let s = planting("carrot", bed, t, t, Mode::OpenGround, d(2027, 3, 15));
        plan.add_planting(planting_id(i as u128 + 1), s).unwrap();
    }
    let occupation = Occupation {
        start: d(2027, 4, 1),
        end: d(2027, 6, 1),
    };
    assert_eq!(plan.shortcut(bed, Fraction::Half, occupation, None), None);
    assert_eq!(
        plan.shortcut(bed, Fraction::Quarter, occupation, None),
        Some(zone(3, 4))
    );
}

#[test]
fn shortened_bed_flags_plantings_that_no_longer_fit() {
    let (mut plan, bed) = plan_with_bed();
    // 2027: carrots on [1,6]; 2028: lettuce on [1,6]; and a short planting on [1,3] in 2029.
    let carrot = planting("carrot", bed, 1, 6, Mode::OpenGround, d(2027, 3, 15));
    let lettuce = planting("lettuce", bed, 1, 6, Mode::Plants, d(2028, 4, 1));
    let short_one = planting("carrot", bed, 1, 3, Mode::OpenGround, d(2029, 3, 15));
    for (n, s) in [(1, carrot), (2, lettuce), (3, short_one)] {
        plan.add_planting(planting_id(n), s).unwrap();
    }
    // Shortened to 4 segments from 2027: both plantings on [1,6] are flagged, not [1,3].
    assert_eq!(
        plan.plantings_overflowing_if_shortened(bed, 2027, 4),
        vec![planting_id(1), planting_id(2)]
    );
    // From 2028: the carrots ended in 2027.
    assert_eq!(
        plan.plantings_overflowing_if_shortened(bed, 2028, 4),
        vec![planting_id(2)]
    );
    assert!(
        plan.plantings_overflowing_if_shortened(bed, 2029, 4)
            .is_empty()
    );
}

// ---- §5 validity and conflicts -------------------------------------------

#[test]
fn disjoint_zones_same_period_no_conflict() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("carrot", bed, 1, 2, Mode::OpenGround, d(2027, 3, 15)),
    )
    .unwrap();
    plan.add_planting(
        planting_id(2),
        planting("carrot", bed, 3, 4, Mode::OpenGround, d(2027, 3, 15)),
    )
    .unwrap();
    assert!(plan.conflicts().is_empty());
}

#[test]
fn conflict_reports_shared_segments_and_period() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("carrot", bed, 1, 4, Mode::OpenGround, d(2027, 3, 15)),
    )
    .unwrap();
    let other = planting("lettuce", bed, 3, 6, Mode::Plants, d(2027, 7, 1));
    let PlanError::Conflict {
        with,
        segments,
        period,
    } = plan
        .add_planting(planting_id(2), other.clone())
        .unwrap_err()
    else {
        panic!("a conflict is expected");
    };
    assert_eq!(with, planting_id(1));
    assert_eq!(segments, Zone { t1: 3, t2: 4 });
    // Carrot ends 2027-08-02, the lettuce starts 2027-07-01: [max(m), min(f)[.
    assert_eq!(
        period,
        Occupation {
            start: d(2027, 7, 1),
            end: d(2027, 8, 2)
        }
    );
    assert!(!plan.plantings.contains_key(&planting_id(2)));
}

#[test]
fn plantings_of_different_beds_do_not_conflict() {
    let (mut plan, bed) = plan_with_bed();
    let other_bed = bed_id(200);
    plan.add_bed(other_bed, "Bed B".into(), 2026, geometry(6), today())
        .unwrap();
    plan.add_planting(
        planting_id(1),
        planting("carrot", bed, 1, 6, Mode::OpenGround, d(2027, 3, 15)),
    )
    .unwrap();
    plan.add_planting(
        planting_id(2),
        planting("carrot", other_bed, 1, 6, Mode::OpenGround, d(2027, 3, 15)),
    )
    .unwrap();
}

#[test]
fn zone_longer_than_the_bed_is_refused() {
    let (mut plan, bed) = plan_with_bed();
    let s = planting("carrot", bed, 1, 7, Mode::OpenGround, d(2027, 3, 15));
    assert_eq!(
        plan.add_planting(planting_id(1), s).unwrap_err(),
        PlanError::ZoneOutOfBed {
            year: 2027,
            segments: 6,
            zone: zone(1, 7)
        }
    );
}

#[test]
fn planting_on_a_year_without_the_bed_is_refused() {
    let (mut plan, bed) = plan_with_bed();
    plan.delete_bed(bed, 2028, today()).unwrap();
    // The leek runs to 2028-02-05: the bed is absent in 2028.
    let s = planting("leek", bed, 1, 1, Mode::Indoor, d(2027, 5, 1));
    assert_eq!(
        plan.add_planting(planting_id(1), s).unwrap_err(),
        PlanError::BedMissing {
            bed_id: bed,
            year: 2028
        }
    );
    // One that ends on 2028-01-01 does not touch 2028.
    let mut finished = planting("carrot", bed, 1, 1, Mode::OpenGround, d(2027, 3, 15));
    finished.manual_end = Some(d(2028, 1, 1));
    plan.add_planting(planting_id(2), finished).unwrap();
}

#[test]
fn planting_before_the_first_layout_is_refused() {
    let (mut plan, bed) = plan_with_bed();
    // The bed only exists from 2026.
    let s = planting("carrot", bed, 1, 1, Mode::OpenGround, d(2025, 3, 15));
    assert!(matches!(
        plan.add_planting(planting_id(1), s),
        Err(PlanError::BedMissing { year: 2025, .. })
    ));
}

#[test]
fn replacing_a_planting_does_not_compare_it_with_its_old_self() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("carrot", bed, 1, 6, Mode::OpenGround, d(2027, 3, 15)),
    )
    .unwrap();
    plan.shift_planting(planting_id(1), 3).unwrap();
    assert_eq!(plan.plantings[&planting_id(1)].date, d(2027, 3, 18));
}

#[test]
fn adding_the_same_id_twice_is_refused() {
    let (mut plan, bed) = plan_with_bed();
    let s = planting("carrot", bed, 1, 2, Mode::OpenGround, d(2027, 3, 15));
    plan.add_planting(planting_id(1), s.clone()).unwrap();
    assert_eq!(
        plan.add_planting(planting_id(1), s).unwrap_err(),
        PlanError::PlantingExists(planting_id(1))
    );
}

// ---- §5 deleting a bed -------------------------------------------------

#[test]
fn deletion_keeps_plantings_ending_on_or_before_january_1st() {
    let (mut plan, bed) = plan_with_bed();
    let mut ends_on_the_1st = planting("carrot", bed, 1, 1, Mode::OpenGround, d(2027, 3, 15));
    ends_on_the_1st.manual_end = Some(d(2028, 1, 1));
    let mut ends_on_the_2nd = planting("carrot", bed, 2, 2, Mode::OpenGround, d(2027, 3, 15));
    ends_on_the_2nd.manual_end = Some(d(2028, 1, 2));
    plan.add_planting(planting_id(1), ends_on_the_1st).unwrap();
    // The second one touches 2028, where the bed exists until the deletion.
    plan.add_planting(planting_id(2), ends_on_the_2nd).unwrap();

    let deleted = plan.delete_bed(bed, 2028, today()).unwrap();

    assert_eq!(deleted, vec![planting_id(2)]);
    assert!(plan.plantings.contains_key(&planting_id(1)));
}

#[test]
fn deletion_also_removes_future_plantings() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("carrot", bed, 1, 1, Mode::OpenGround, d(2028, 3, 15)),
    )
    .unwrap();
    let deleted = plan.delete_bed(bed, 2028, today()).unwrap();
    assert_eq!(deleted, vec![planting_id(1)]);
}

#[test]
fn deletion_erases_the_later_layouts_of_the_bed() {
    let (mut plan, bed) = plan_with_bed();
    plan.set_layout(bed, 2030, Layout::Present(geometry(4)), today())
        .unwrap();
    plan.delete_bed(bed, 2028, today()).unwrap();
    let p = &plan.beds[&bed];
    assert!(p.layout_in(2027).is_some());
    for year in [2028, 2029, 2030, 2031] {
        assert!(p.layout_in(year).is_none(), "{year}");
    }
}

#[test]
fn deletion_leaves_other_beds_alone() {
    let (mut plan, bed) = plan_with_bed();
    let other = bed_id(200);
    plan.add_bed(other, "Bed B".into(), 2026, geometry(6), today())
        .unwrap();
    plan.add_planting(
        planting_id(1),
        planting("leek", other, 1, 6, Mode::Indoor, d(2027, 5, 1)),
    )
    .unwrap();
    assert!(plan.delete_bed(bed, 2028, today()).unwrap().is_empty());
    assert!(plan.plantings.contains_key(&planting_id(1)));
    assert!(plan.beds[&other].layout_in(2029).is_some());
}

// ---- §7 layouts ---------------------------------------------------------------

#[test]
fn modification_holds_until_the_next_recorded_layout() {
    let (mut plan, bed) = plan_with_bed();
    plan.set_layout(bed, 2029, Layout::Present(geometry(4)), today())
        .unwrap();
    // Modified in 2027: holds for 2028, but 2029 stays as it was.
    plan.set_layout(bed, 2027, Layout::Present(geometry(8)), today())
        .unwrap();
    let nb = |year| plan.segment_count(bed, year);
    assert_eq!(
        [nb(2026), nb(2027), nb(2028), nb(2029), nb(2030)],
        [Some(6), Some(8), Some(8), Some(4), Some(4)]
    );
}

#[test]
fn other_beds_are_not_touched_by_a_modification() {
    let (mut plan, bed) = plan_with_bed();
    let other = bed_id(200);
    plan.add_bed(other, "Bed B".into(), 2026, geometry(6), today())
        .unwrap();
    plan.set_layout(bed, 2027, Layout::Present(geometry(8)), today())
        .unwrap();
    assert_eq!(plan.segment_count(other, 2027), Some(6));
}

#[test]
fn past_year_is_read_only() {
    let (mut plan, bed) = plan_with_bed();
    let before = plan.clone();
    let error = plan
        .set_layout(bed, 2025, Layout::Present(geometry(4)), today())
        .unwrap_err();
    assert_eq!(
        error,
        PlanError::PastYear {
            year: 2025,
            today: today()
        }
    );
    assert!(matches!(
        plan.delete_bed(bed, 2025, today()),
        Err(PlanError::PastYear { .. })
    ));
    // The year of today can still be modified.
    plan.set_layout(bed, 2026, Layout::Present(geometry(4)), today())
        .unwrap();
    assert_ne!(plan, before);
}

#[test]
fn eleventh_bed_refused() {
    let (mut plan, _) = plan_with_bed();
    for n in 1..10u128 {
        plan.add_bed(bed_id(200 + n), format!("P{n}"), 2026, geometry(4), today())
            .unwrap();
    }
    assert_eq!(plan.beds_in(2026).len(), 10);
    let before = plan.clone();
    let error = plan
        .add_bed(bed_id(999), "Too".into(), 2026, geometry(4), today())
        .unwrap_err();
    assert_eq!(
        error,
        PlanError::TooManyBeds {
            year: 2026,
            max: 10
        }
    );
    assert_eq!(plan, before);
    // A bed deleted from 2027 frees a place from 2027 on.
    plan.delete_bed(bed_id(100), 2027, today()).unwrap();
    plan.add_bed(bed_id(999), "Ok".into(), 2027, geometry(4), today())
        .unwrap();
}

#[test]
fn length_not_a_multiple_of_the_case_refused() {
    let (mut plan, bed) = plan_with_bed();
    let mut geo = geometry(4);
    geo.length_cm = 210;
    assert_eq!(
        plan.set_layout(bed, 2027, Layout::Present(geo), today())
            .unwrap_err(),
        PlanError::InvalidLength {
            length_cm: 210,
            cell_cm: 50
        }
    );
}

// ---- §8 free slots -----------------------------------------------------------

/// A bed with a planting of one segment ending when the slot starts: builds
/// slots of an exact length.
fn plan_with_occupation(
    occupied_zone: (u32, u32),
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> (Plan, plan::BedId) {
    let (mut plan, bed) = plan_with_bed();
    let mut s = planting(
        "carrot",
        bed,
        occupied_zone.0,
        occupied_zone.1,
        Mode::OpenGround,
        start,
    );
    s.manual_end = Some(end);
    plan.add_planting(planting_id(1), s).unwrap();
    (plan, bed)
}

#[test]
fn slot_of_35_days_dropped_and_36_days_kept() {
    // Occupied from 2027-04-05: the free period 03-01..04-04 is 35 days (31 + 4).
    let (plan, _) = plan_with_occupation((1, 6), d(2027, 4, 5), d(2027, 11, 20));
    assert!(plan.slots(2027).is_empty());
    // One day later: 03-01..04-05 is 36 days.
    let (plan, bed) = plan_with_occupation((1, 6), d(2027, 4, 6), d(2027, 11, 20));
    assert_eq!(
        plan.slots(2027),
        vec![Slot {
            bed_id: bed,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 3, 1),
            end: d(2027, 4, 5)
        }]
    );
}

#[test]
fn slot_bounded_by_the_march_1st_to_november_15th_window() {
    let (plan, bed) = plan_with_bed();
    assert_eq!(
        plan.slots(2027),
        vec![Slot {
            bed_id: bed,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 3, 1),
            end: d(2027, 11, 15)
        }]
    );
}

#[test]
fn slots_of_different_zones_maximal_in_width_and_time() {
    // [1,2] busy [04-01, 07-01[ and [5,6] busy [06-01, 09-01[.
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        occupying("carrot", bed, 1, 2, d(2027, 4, 1), d(2027, 7, 1)),
    )
    .unwrap();
    plan.add_planting(
        planting_id(2),
        occupying("carrot", bed, 5, 6, d(2027, 6, 1), d(2027, 9, 1)),
    )
    .unwrap();
    let expected = |t1, t2, start, end| Slot {
        bed_id: bed,
        year: 2027,
        zone: zone(t1, t2),
        start,
        end,
    };
    // Dropped: [1,4] and [1,6] 03-01..03-31 (31 days, below the minimum), and
    // [3,6] 09-01..11-15 (it can be widened to [1,6] over that period).
    assert_eq!(
        plan.slots(2027),
        vec![
            expected(1, 4, d(2027, 7, 1), d(2027, 11, 15)),
            expected(1, 6, d(2027, 9, 1), d(2027, 11, 15)),
            expected(3, 4, d(2027, 3, 1), d(2027, 11, 15)),
            expected(3, 6, d(2027, 3, 1), d(2027, 5, 31)),
        ]
    );
}

#[test]
fn slots_computed_for_each_year_the_bed_exists() {
    let (mut plan, bed) = plan_with_bed();
    plan.delete_bed(bed, 2028, today()).unwrap();
    assert!(!plan.slots(2027).is_empty());
    assert!(plan.slots(2028).is_empty(), "the bed no longer exists");
    assert!(plan.slots(2025).is_empty(), "the bed does not exist yet");
}

#[test]
fn a_planting_from_the_previous_year_occupies_the_start_of_the_year() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("leek", bed, 1, 6, Mode::Indoor, d(2027, 5, 1)),
    )
    .unwrap();
    // The leek runs until 2028-02-05: the 2028 window opens on 02-05... which is before 03-01.
    assert_eq!(
        plan.slots(2028),
        vec![Slot {
            bed_id: bed,
            year: 2028,
            zone: zone(1, 6),
            start: d(2028, 3, 1),
            end: d(2028, 11, 15)
        }]
    );
    // In 2027 the leek starts on 07-10: free 03-01..07-09.
    assert_eq!(plan.slots(2027)[0].end, d(2027, 7, 9));
}

// ---- §9 suggestions ------------------------------------------------------------

fn slot(start: chrono::NaiveDate, end: chrono::NaiveDate) -> Slot {
    Slot {
        bed_id: bed_id(100),
        year: start.year_(),
        zone: zone(1, 6),
        start,
        end,
    }
}

trait Year {
    fn year_(&self) -> i32;
}
impl Year for chrono::NaiveDate {
    fn year_(&self) -> i32 {
        chrono::Datelike::year(self)
    }
}

#[test]
fn crop_discarded_when_the_minimal_harvest_does_not_fit() {
    // Lettuce, harvest 20 days, minimum 10: r = 2027-09-21 for m = 08-02.
    let enough = slot(d(2027, 8, 2), d(2027, 9, 30)); // r + 10 = 10-01 = F + 1: fits
    assert!(evaluate(&enough, crop("lettuce"), today()).is_some());
    let too_short = slot(d(2027, 8, 2), d(2027, 9, 29)); // F + 1 = 09-30 < r + 10
    assert_eq!(evaluate(&too_short, crop("lettuce"), today()), None);
}

#[test]
fn planting_out_is_searched_in_the_first_28_days() {
    // Tomato window opens 05-01: reachable from 04-04 (D + 27), not from 04-03.
    let ok = slot(d(2027, 4, 4), d(2027, 11, 15));
    assert_eq!(
        evaluate(&ok, crop("tomato"), d(2027, 1, 1))
            .unwrap()
            .planting_out,
        d(2027, 5, 1)
    );
    let too_early = slot(d(2027, 4, 3), d(2027, 11, 15));
    assert_eq!(evaluate(&too_early, crop("tomato"), d(2027, 1, 1)), None);
}

#[test]
fn sowing_under_cover_today_is_still_possible() {
    let c = slot(d(2027, 5, 1), d(2027, 11, 15));
    // Tomato sowing would be 03-06.
    assert_eq!(
        evaluate(&c, crop("tomato"), d(2027, 3, 6)).unwrap().mode,
        Mode::Indoor
    );
    assert_eq!(
        evaluate(&c, crop("tomato"), d(2027, 3, 7)).unwrap().mode,
        Mode::Plants
    );
}

#[test]
fn crop_without_pots_is_sown_in_open_ground() {
    let c = slot(d(2027, 5, 10), d(2027, 11, 15));
    let bean = evaluate(&c, crop("green-bean"), today()).unwrap();
    assert_eq!((bean.mode, bean.date), (Mode::OpenGround, d(2027, 5, 10)));
}

#[test]
fn at_most_3_suggestions_ranked_by_filling_then_name() {
    let (plan, bed) = plan_with_bed();
    let c = Slot {
        bed_id: bed,
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 5, 1),
        end: d(2027, 11, 15),
    };
    let suggestions = plan.suggestions(&c, Catalogue::reference(), d(2027, 4, 20));
    assert_eq!(suggestions.len(), 3);
    let ids: Vec<&str> = suggestions.iter().map(|s| s.crop_id.as_str()).collect();
    // Leek fills the slot (score 1.0, end cut to the slot); then the 0.7 group by name:
    // Carrot, Tomato, Zucchini (cut off by the limit of 3).
    assert_eq!(ids, ["leek", "carrot", "tomato"]);
    assert_eq!(suggestions[0].manual_end, Some(d(2027, 11, 16)));
    assert_eq!(suggestions[0].score_tenths(), 10);
}

#[test]
fn on_equal_score_a_different_family_from_the_last_planting_comes_first() {
    let ids = |plan: &Plan, bed| -> Vec<String> {
        let c = Slot {
            bed_id: bed,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 5, 1),
            end: d(2027, 11, 15),
        };
        plan.suggestions(&c, Catalogue::reference(), d(2027, 4, 20))
            .into_iter()
            .map(|s| s.crop_id)
            .collect()
    };
    // The last planting before the slot is a tomato (Solanaceae): in the 0.7 group
    // (carrot, zucchini, tomato) it was already last by name.
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        occupying("tomato", bed, 1, 6, d(2027, 3, 1), d(2027, 5, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, bed), ["leek", "carrot", "zucchini"]);

    // It is a carrot (Apiaceae): the carrot goes behind the other crops of its score.
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        occupying("carrot", bed, 1, 6, d(2027, 3, 1), d(2027, 5, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, bed), ["leek", "tomato", "zucchini"]);

    // An earlier planting does not count once a later one has ended.
    plan.add_planting(
        planting_id(2),
        occupying("tomato", bed, 1, 6, d(2027, 1, 1), d(2027, 3, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, bed), ["leek", "tomato", "zucchini"]);
}

#[test]
fn suggestion_into_planting_prefills_the_creation() {
    let c = slot(d(2027, 8, 2), d(2027, 11, 15));
    let suggestion = evaluate(&c, crop("lettuce"), today()).unwrap();
    let planting = suggestion.into_planting(crop("lettuce"), &c).unwrap();
    assert_eq!(
        (planting.mode, planting.date, planting.zone()),
        (Mode::Indoor, d(2027, 7, 5), zone(1, 6))
    );
    assert_eq!(planting.dates().planting_out, suggestion.planting_out);
    assert_eq!(planting.dates().end, suggestion.end);
}

// ---- §11 tasks -------------------------------------------------------------

#[test]
fn tasks_by_mode() {
    let types = |mode| -> Vec<TaskType> {
        planting("tomato", bed_id(100), 1, 1, mode, d(2027, 3, 10))
            .tasks(planting_id(1))
            .into_iter()
            .map(|t| t.task_type)
            .collect()
    };
    assert_eq!(
        types(Mode::Indoor),
        [
            TaskType::SowUnderCover,
            TaskType::Transplant,
            TaskType::Harvest
        ]
    );
    assert_eq!(types(Mode::Plants), [TaskType::Plant, TaskType::Harvest]);
    assert_eq!(
        types(Mode::OpenGround),
        [TaskType::SowInGround, TaskType::Harvest]
    );
}

#[test]
fn task_dates_of_an_indoor_planting() {
    let s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, d(2027, 3, 10));
    let dates: Vec<_> = s
        .tasks(planting_id(1))
        .into_iter()
        .map(|t| t.date)
        .collect();
    assert_eq!(dates, [d(2027, 3, 10), d(2027, 5, 5), d(2027, 7, 19)]);
}

#[test]
fn task_id_is_stable_when_the_date_moves() {
    let id = planting_id(7);
    let s = planting("tomato", bed_id(100), 1, 1, Mode::Indoor, d(2027, 3, 10));
    let before = s.tasks(id);
    let after = s.shifted(10).tasks(id);
    let ids = |t: &[plan::Task]| t.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&before), ids(&after));
    assert_ne!(before[0].date, after[0].date);
    assert_eq!(before[0].id, format!("{id}/sow-indoor"));
    assert_eq!(before[2].id, format!("{id}/harvest"));
}

#[test]
fn task_calendar_by_month() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("tomato", bed, 1, 3, Mode::Indoor, d(2027, 3, 10)),
    )
    .unwrap();
    let calendar = plan.calendar_by_month();
    let month: Vec<_> = calendar.keys().copied().collect();
    assert_eq!(month, [(2027, 3), (2027, 5), (2027, 7)]);
    assert_eq!(calendar[&(2027, 5)][0].task_type, TaskType::Transplant);
}

#[test]
fn families_of_the_two_previous_seasons() {
    let (mut plan, bed) = plan_with_bed();
    // 2027: tomato (Solanaceae) and lettuce; 2028: carrot; 2026 would be Y−2 of 2028.
    plan.add_planting(
        planting_id(1),
        planting("tomato", bed, 1, 3, Mode::Indoor, d(2027, 3, 10)),
    )
    .unwrap();
    plan.add_planting(
        planting_id(2),
        planting("carrot", bed, 4, 6, Mode::OpenGround, d(2028, 3, 15)),
    )
    .unwrap();
    plan.add_planting(
        planting_id(3),
        planting("lettuce", bed, 4, 6, Mode::Plants, d(2026, 4, 1)),
    )
    .unwrap();
    let families = |year| plan.previous_families(bed, year, Catalogue::reference());
    assert_eq!(families(2028), ["Asteraceae", "Solanaceae"]);
    assert_eq!(families(2029), ["Apiaceae", "Solanaceae"]);
    assert_eq!(families(2030), ["Apiaceae"]);
    assert!(families(2031).is_empty());
}

// ---- §1 and S05 §5: model --------------------------------------------------------

#[test]
fn plan_serialises_without_derived_data_and_reads_back() {
    let (mut plan, bed) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("tomato", bed, 1, 3, Mode::Indoor, d(2027, 3, 10)),
    )
    .unwrap();
    plan.set_layout(bed, 2028, Layout::Absent, today()).unwrap();

    let json = serde_json::to_value(&plan).unwrap();
    // Collections are maps indexed by id, never lists.
    assert!(json["beds"].is_object());
    assert!(json["plantings"].is_object());
    assert!(json["annotations"].is_object());
    assert!(json["beds"][bed.to_string()]["layouts"]["2026"].is_object());
    // Only entered values are stored: no m, r, f, quantity, slot or task.
    let planting_json = &json["plantings"][planting_id(1).to_string()];
    let mut keys: Vec<_> = planting_json.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "after_days",
            "bed_id",
            "crop_id",
            "date",
            "duration_days",
            "indoor_days",
            "mode",
            "segment_end",
            "segment_start",
        ]
    );
    assert_eq!(planting_json["mode"], "indoor");
    assert_eq!(planting_json["date"], "2027-03-10");

    let reread: Plan = serde_json::from_value(json).unwrap();
    assert_eq!(reread, plan);
}

#[test]
fn technical_timestamp_in_utc() {
    let (mut plan, _) = plan_with_bed();
    assert_eq!(plan.version, 1);
    plan.touch(now() + chrono::Duration::hours(1));
    assert_eq!(plan.version, 2);
    let json = serde_json::to_value(&plan).unwrap();
    assert!(json["updated_at"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn generated_ids_are_distinct_uuid_v7() {
    let (a, b) = (plan::PlantingId::generate(), plan::PlantingId::generate());
    assert_ne!(a, b);
    assert_eq!(a.uuid().get_version_num(), 7);
}

#[test]
fn stable_error_codes_for_the_ui() {
    assert_eq!(PlanError::PotDaysRequired.code(), "pot_days_required");
    assert_eq!(
        PlanError::TooManyBeds {
            year: 2027,
            max: 10
        }
        .code(),
        "too_many_beds"
    );
}

#[test]
fn zone_type_is_reexported() {
    assert_eq!(Zone::new(1, 2).unwrap().segment_count(), 2);
}
