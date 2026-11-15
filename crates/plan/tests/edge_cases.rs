//! Edge cases that S03 describes but T1 to T13 do not cover.

mod common;

use catalogue::Catalogue;
use common::*;
use plan::{
    Creneau, Disposition, Fraction, Mode, Occupation, Plan, PlanError, Serie, TaskType, Zone,
    evaluate,
};

// ---- §2 dates ----------------------------------------------------------

#[test]
fn bought_plants_are_planted_on_the_entered_day() {
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Plants, d(2027, 5, 5));
    let dates = s.dates();
    assert_eq!(dates.sowing, None);
    assert_eq!(dates.planting_out, d(2027, 5, 5));
    assert_eq!(dates.harvest_start, d(2027, 7, 19));
}

#[test]
fn pleine_terre_adds_the_pot_phase_of_the_culture() {
    // Tomato: 56 days in pots would have been spent in the ground: 75 + 56.
    let s = serie(
        "tomate",
        planche_id(100),
        1,
        1,
        Mode::PleineTerre,
        d(2027, 5, 5),
    );
    assert_eq!(
        s.dates().harvest_start,
        d(2027, 5, 5) + chrono::Duration::days(131)
    );
}

#[test]
fn entering_the_transplanting_date_stores_the_sowing_date() {
    // Entering the transplanting date instead of the sowing date: s = x − A.
    let sowing = Serie::sowing_from_transplanting(d(2027, 5, 5), 56);
    assert_eq!(sowing, d(2027, 3, 10));
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, sowing);
    assert_eq!(s.dates().planting_out, d(2027, 5, 5));
}

#[test]
fn abri_without_pot_days_for_a_crop_without_pots_is_refused() {
    for pots in [None, Some(0)] {
        let error = Serie::new(
            culture("haricot-vert"),
            planche_id(100),
            zone(1, 1),
            Mode::Abri,
            d(2027, 5, 10),
            pots,
            None,
        )
        .unwrap_err();
        assert_eq!(error, PlanError::PotDaysRequired);
    }
}

#[test]
fn entered_pot_days_ignored_when_the_culture_has_pots() {
    let s = Serie::new(
        culture("tomate"),
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 10),
        Some(10),
        None,
    )
    .unwrap();
    assert_eq!(s.abri_jours, 56);
}

#[test]
fn durations_are_copied_at_creation() {
    let mut tomate = culture("tomate").clone();
    let s = Serie::new(
        &tomate,
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 10),
        None,
        None,
    )
    .unwrap();
    let before = s.dates();
    // A catalogue update does not move an existing serie.
    tomate.recolte_apres_jours = 90;
    tomate.recolte_duree_jours = 10;
    assert_eq!(s.dates(), before);
    assert_eq!((s.abri_jours, s.apres_jours, s.duree_jours), (56, 75, 70));
}

#[test]
fn changing_culture_copies_the_durations_again() {
    let mut s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    s.change_culture(culture("laitue"), None).unwrap();
    assert_eq!(
        (
            s.culture_id.as_str(),
            s.abri_jours,
            s.apres_jours,
            s.duree_jours
        ),
        ("laitue", 28, 50, 20)
    );
}

#[test]
fn changing_a_carrot_mode_drops_the_entered_pot_days() {
    let mut s = Serie::new(
        culture("carotte"),
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 1),
        Some(21),
        None,
    )
    .unwrap();
    s.change_mode(Mode::PleineTerre, culture("carotte"), None)
        .unwrap();
    // As T3: nothing added for a carrot sown in the ground.
    assert_eq!(s.dates().harvest_start, d(2027, 6, 9));
}

#[test]
fn manual_end_must_be_after_harvest_start() {
    let tomate = culture("tomate");
    let new = |end| {
        Serie::new(
            tomate,
            planche_id(100),
            zone(1, 1),
            Mode::Abri,
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
    let mut s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    s.fin_manuelle = Some(d(2027, 8, 1));
    s.date = d(2027, 3, 20);
    assert_eq!(s.dates().harvest_start, d(2027, 7, 29));
    assert_eq!(s.dates().end, d(2027, 8, 1));
}

#[test]
fn dragging_shifts_the_date_and_the_manual_end() {
    let mut s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    s.fin_manuelle = Some(d(2027, 8, 1));
    let shifted = s.shifted(7);
    assert_eq!(shifted.date, d(2027, 3, 17));
    assert_eq!(shifted.fin_manuelle, Some(d(2027, 8, 8)));
    // The duration is kept.
    let (before, after) = (s.occupation(), shifted.occupation());
    assert_eq!(before.end - before.start, after.end - after.start);
}

#[test]
fn timeline_phases() {
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
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
    let plants = serie("tomate", planche_id(100), 1, 1, Mode::Plants, d(2027, 5, 5));
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
fn shortcut_takes_the_first_free_run_of_troncons() {
    let (mut plan, planche) = plan_with_planche();
    // [1,2] taken during the whole season.
    let occupied = serie("carotte", planche, 1, 2, Mode::PleineTerre, d(2027, 3, 15));
    plan.add_serie(serie_id(1), occupied).unwrap();
    let occupation = Occupation {
        start: d(2027, 4, 1),
        end: d(2027, 6, 1),
    };
    assert_eq!(
        plan.shortcut(planche, Fraction::Half, occupation, None),
        Some(zone(3, 5))
    );
    assert_eq!(
        plan.shortcut(planche, Fraction::Third, occupation, None),
        Some(zone(3, 4))
    );
    // The whole planche is not free.
    assert_eq!(
        plan.shortcut(planche, Fraction::Whole, occupation, None),
        None
    );
    // Free after the carrot ends: back to the first troncons.
    let after = Occupation {
        start: d(2027, 8, 2),
        end: d(2027, 9, 1),
    };
    assert_eq!(
        plan.shortcut(planche, Fraction::Whole, after, None),
        Some(zone(1, 6))
    );
    // Editing the carrot itself ignores it.
    assert_eq!(
        plan.shortcut(planche, Fraction::Whole, occupation, Some(serie_id(1))),
        Some(zone(1, 6))
    );
}

#[test]
fn shortcut_greyed_out_without_a_free_run() {
    let (mut plan, planche) = plan_with_planche();
    // Troncons 2 and 5 are taken: no run of 3 is left free.
    for (i, t) in [2, 5].into_iter().enumerate() {
        let s = serie("carotte", planche, t, t, Mode::PleineTerre, d(2027, 3, 15));
        plan.add_serie(serie_id(i as u128 + 1), s).unwrap();
    }
    let occupation = Occupation {
        start: d(2027, 4, 1),
        end: d(2027, 6, 1),
    };
    assert_eq!(
        plan.shortcut(planche, Fraction::Half, occupation, None),
        None
    );
    assert_eq!(
        plan.shortcut(planche, Fraction::Quarter, occupation, None),
        Some(zone(3, 4))
    );
}

#[test]
fn shortened_planche_flags_series_that_no_longer_fit() {
    let (mut plan, planche) = plan_with_planche();
    // 2027: carrots on [1,6]; 2028: lettuce on [1,6]; and a short serie on [1,3] in 2029.
    let carotte = serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15));
    let laitue = serie("laitue", planche, 1, 6, Mode::Plants, d(2028, 4, 1));
    let short_one = serie("carotte", planche, 1, 3, Mode::PleineTerre, d(2029, 3, 15));
    for (n, s) in [(1, carotte), (2, laitue), (3, short_one)] {
        plan.add_serie(serie_id(n), s).unwrap();
    }
    // Shortened to 4 troncons from 2027: both series on [1,6] are flagged, not [1,3].
    assert_eq!(
        plan.series_overflowing_if_shortened(planche, 2027, 4),
        vec![serie_id(1), serie_id(2)]
    );
    // From 2028: the carrots ended in 2027.
    assert_eq!(
        plan.series_overflowing_if_shortened(planche, 2028, 4),
        vec![serie_id(2)]
    );
    assert!(
        plan.series_overflowing_if_shortened(planche, 2029, 4)
            .is_empty()
    );
}

// ---- §5 validity and conflicts -------------------------------------------

#[test]
fn disjoint_zones_same_period_no_conflict() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("carotte", planche, 1, 2, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    plan.add_serie(
        serie_id(2),
        serie("carotte", planche, 3, 4, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    assert!(plan.conflicts().is_empty());
}

#[test]
fn conflict_reports_shared_troncons_and_period() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("carotte", planche, 1, 4, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    let other = serie("laitue", planche, 3, 6, Mode::Plants, d(2027, 7, 1));
    let PlanError::Conflict {
        with,
        troncons,
        period,
    } = plan.add_serie(serie_id(2), other.clone()).unwrap_err()
    else {
        panic!("a conflict is expected");
    };
    assert_eq!(with, serie_id(1));
    assert_eq!(troncons, Zone { t1: 3, t2: 4 });
    // Carrot ends 2027-08-02, the lettuce starts 2027-07-01: [max(m), min(f)[.
    assert_eq!(
        period,
        Occupation {
            start: d(2027, 7, 1),
            end: d(2027, 8, 2)
        }
    );
    assert!(!plan.series.contains_key(&serie_id(2)));
}

#[test]
fn series_of_different_planches_do_not_conflict() {
    let (mut plan, planche) = plan_with_planche();
    let other_planche = planche_id(200);
    plan.add_planche(
        other_planche,
        "Planche B".into(),
        2026,
        geometry(6),
        today(),
    )
    .unwrap();
    plan.add_serie(
        serie_id(1),
        serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    plan.add_serie(
        serie_id(2),
        serie(
            "carotte",
            other_planche,
            1,
            6,
            Mode::PleineTerre,
            d(2027, 3, 15),
        ),
    )
    .unwrap();
}

#[test]
fn zone_longer_than_the_planche_is_refused() {
    let (mut plan, planche) = plan_with_planche();
    let s = serie("carotte", planche, 1, 7, Mode::PleineTerre, d(2027, 3, 15));
    assert_eq!(
        plan.add_serie(serie_id(1), s).unwrap_err(),
        PlanError::ZoneOutOfPlanche {
            year: 2027,
            troncons: 6,
            zone: zone(1, 7)
        }
    );
}

#[test]
fn serie_on_a_year_without_the_planche_is_refused() {
    let (mut plan, planche) = plan_with_planche();
    plan.delete_planche(planche, 2028, today()).unwrap();
    // The poireau runs to 2028-02-05: the planche is absent in 2028.
    let s = serie("poireau", planche, 1, 1, Mode::Abri, d(2027, 5, 1));
    assert_eq!(
        plan.add_serie(serie_id(1), s).unwrap_err(),
        PlanError::PlancheMissing {
            planche_id: planche,
            year: 2028
        }
    );
    // One that ends on 2028-01-01 does not touch 2028.
    let mut finished = serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2027, 3, 15));
    finished.fin_manuelle = Some(d(2028, 1, 1));
    plan.add_serie(serie_id(2), finished).unwrap();
}

#[test]
fn serie_before_the_first_disposition_is_refused() {
    let (mut plan, planche) = plan_with_planche();
    // The planche only exists from 2026.
    let s = serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2025, 3, 15));
    assert!(matches!(
        plan.add_serie(serie_id(1), s),
        Err(PlanError::PlancheMissing { year: 2025, .. })
    ));
}

#[test]
fn replacing_a_serie_does_not_compare_it_with_its_old_self() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15)),
    )
    .unwrap();
    plan.shift_serie(serie_id(1), 3).unwrap();
    assert_eq!(plan.series[&serie_id(1)].date, d(2027, 3, 18));
}

#[test]
fn adding_the_same_id_twice_is_refused() {
    let (mut plan, planche) = plan_with_planche();
    let s = serie("carotte", planche, 1, 2, Mode::PleineTerre, d(2027, 3, 15));
    plan.add_serie(serie_id(1), s.clone()).unwrap();
    assert_eq!(
        plan.add_serie(serie_id(1), s).unwrap_err(),
        PlanError::SerieExists(serie_id(1))
    );
}

// ---- §5 deleting a planche -------------------------------------------------

#[test]
fn deletion_keeps_series_ending_on_or_before_january_1st() {
    let (mut plan, planche) = plan_with_planche();
    let mut ends_on_the_1st = serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2027, 3, 15));
    ends_on_the_1st.fin_manuelle = Some(d(2028, 1, 1));
    let mut ends_on_the_2nd = serie("carotte", planche, 2, 2, Mode::PleineTerre, d(2027, 3, 15));
    ends_on_the_2nd.fin_manuelle = Some(d(2028, 1, 2));
    plan.add_serie(serie_id(1), ends_on_the_1st).unwrap();
    // The second one touches 2028, where the planche exists until the deletion.
    plan.add_serie(serie_id(2), ends_on_the_2nd).unwrap();

    let deleted = plan.delete_planche(planche, 2028, today()).unwrap();

    assert_eq!(deleted, vec![serie_id(2)]);
    assert!(plan.series.contains_key(&serie_id(1)));
}

#[test]
fn deletion_also_removes_future_series() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("carotte", planche, 1, 1, Mode::PleineTerre, d(2028, 3, 15)),
    )
    .unwrap();
    let deleted = plan.delete_planche(planche, 2028, today()).unwrap();
    assert_eq!(deleted, vec![serie_id(1)]);
}

#[test]
fn deletion_erases_the_later_dispositions_of_the_planche() {
    let (mut plan, planche) = plan_with_planche();
    plan.set_disposition(planche, 2030, Disposition::Present(geometry(4)), today())
        .unwrap();
    plan.delete_planche(planche, 2028, today()).unwrap();
    let p = &plan.planches[&planche];
    assert!(p.disposition_in(2027).is_some());
    for year in [2028, 2029, 2030, 2031] {
        assert!(p.disposition_in(year).is_none(), "{year}");
    }
}

#[test]
fn deletion_leaves_other_planches_alone() {
    let (mut plan, planche) = plan_with_planche();
    let other = planche_id(200);
    plan.add_planche(other, "Planche B".into(), 2026, geometry(6), today())
        .unwrap();
    plan.add_serie(
        serie_id(1),
        serie("poireau", other, 1, 6, Mode::Abri, d(2027, 5, 1)),
    )
    .unwrap();
    assert!(
        plan.delete_planche(planche, 2028, today())
            .unwrap()
            .is_empty()
    );
    assert!(plan.series.contains_key(&serie_id(1)));
    assert!(plan.planches[&other].disposition_in(2029).is_some());
}

// ---- §7 layouts ---------------------------------------------------------------

#[test]
fn modification_holds_until_the_next_recorded_disposition() {
    let (mut plan, planche) = plan_with_planche();
    plan.set_disposition(planche, 2029, Disposition::Present(geometry(4)), today())
        .unwrap();
    // Modified in 2027: holds for 2028, but 2029 stays as it was.
    plan.set_disposition(planche, 2027, Disposition::Present(geometry(8)), today())
        .unwrap();
    let nb = |year| plan.troncon_count(planche, year);
    assert_eq!(
        [nb(2026), nb(2027), nb(2028), nb(2029), nb(2030)],
        [Some(6), Some(8), Some(8), Some(4), Some(4)]
    );
}

#[test]
fn other_planches_are_not_touched_by_a_modification() {
    let (mut plan, planche) = plan_with_planche();
    let other = planche_id(200);
    plan.add_planche(other, "Planche B".into(), 2026, geometry(6), today())
        .unwrap();
    plan.set_disposition(planche, 2027, Disposition::Present(geometry(8)), today())
        .unwrap();
    assert_eq!(plan.troncon_count(other, 2027), Some(6));
}

#[test]
fn past_year_is_read_only() {
    let (mut plan, planche) = plan_with_planche();
    let before = plan.clone();
    let error = plan
        .set_disposition(planche, 2025, Disposition::Present(geometry(4)), today())
        .unwrap_err();
    assert_eq!(
        error,
        PlanError::PastYear {
            year: 2025,
            today: today()
        }
    );
    assert!(matches!(
        plan.delete_planche(planche, 2025, today()),
        Err(PlanError::PastYear { .. })
    ));
    // The year of today can still be modified.
    plan.set_disposition(planche, 2026, Disposition::Present(geometry(4)), today())
        .unwrap();
    assert_ne!(plan, before);
}

#[test]
fn eleventh_planche_refused() {
    let (mut plan, _) = plan_with_planche();
    for n in 1..10u128 {
        plan.add_planche(
            planche_id(200 + n),
            format!("P{n}"),
            2026,
            geometry(4),
            today(),
        )
        .unwrap();
    }
    assert_eq!(plan.planches_in(2026).len(), 10);
    let before = plan.clone();
    let error = plan
        .add_planche(planche_id(999), "Trop".into(), 2026, geometry(4), today())
        .unwrap_err();
    assert_eq!(
        error,
        PlanError::TooManyPlanches {
            year: 2026,
            max: 10
        }
    );
    assert_eq!(plan, before);
    // A planche deleted from 2027 frees a place from 2027 on.
    plan.delete_planche(planche_id(100), 2027, today()).unwrap();
    plan.add_planche(planche_id(999), "Ok".into(), 2027, geometry(4), today())
        .unwrap();
}

#[test]
fn length_not_a_multiple_of_the_case_refused() {
    let (mut plan, planche) = plan_with_planche();
    let mut geo = geometry(4);
    geo.longueur_cm = 210;
    assert_eq!(
        plan.set_disposition(planche, 2027, Disposition::Present(geo), today())
            .unwrap_err(),
        PlanError::InvalidLength {
            longueur_cm: 210,
            case_cm: 50
        }
    );
}

// ---- §8 free slots -----------------------------------------------------------

/// A planche with a serie of one troncon ending when the slot starts: builds
/// slots of an exact length.
fn plan_with_occupation(
    occupied_zone: (u32, u32),
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> (Plan, plan::PlancheId) {
    let (mut plan, planche) = plan_with_planche();
    let mut s = serie(
        "carotte",
        planche,
        occupied_zone.0,
        occupied_zone.1,
        Mode::PleineTerre,
        start,
    );
    s.fin_manuelle = Some(end);
    plan.add_serie(serie_id(1), s).unwrap();
    (plan, planche)
}

#[test]
fn creneau_of_35_days_dropped_and_36_days_kept() {
    // Occupied from 2027-04-05: the free period 03-01..04-04 is 35 days (31 + 4).
    let (plan, _) = plan_with_occupation((1, 6), d(2027, 4, 5), d(2027, 11, 20));
    assert!(plan.creneaux(2027).is_empty());
    // One day later: 03-01..04-05 is 36 days.
    let (plan, planche) = plan_with_occupation((1, 6), d(2027, 4, 6), d(2027, 11, 20));
    assert_eq!(
        plan.creneaux(2027),
        vec![Creneau {
            planche_id: planche,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 3, 1),
            end: d(2027, 4, 5)
        }]
    );
}

#[test]
fn creneau_bounded_by_the_march_1st_to_november_15th_window() {
    let (plan, planche) = plan_with_planche();
    assert_eq!(
        plan.creneaux(2027),
        vec![Creneau {
            planche_id: planche,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 3, 1),
            end: d(2027, 11, 15)
        }]
    );
}

#[test]
fn creneaux_of_different_zones_maximal_in_width_and_time() {
    // [1,2] busy [04-01, 07-01[ and [5,6] busy [06-01, 09-01[.
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        occupying("carotte", planche, 1, 2, d(2027, 4, 1), d(2027, 7, 1)),
    )
    .unwrap();
    plan.add_serie(
        serie_id(2),
        occupying("carotte", planche, 5, 6, d(2027, 6, 1), d(2027, 9, 1)),
    )
    .unwrap();
    let expected = |t1, t2, start, end| Creneau {
        planche_id: planche,
        year: 2027,
        zone: zone(t1, t2),
        start,
        end,
    };
    // Dropped: [1,4] and [1,6] 03-01..03-31 (31 days, below the minimum), and
    // [3,6] 09-01..11-15 (it can be widened to [1,6] over that period).
    assert_eq!(
        plan.creneaux(2027),
        vec![
            expected(1, 4, d(2027, 7, 1), d(2027, 11, 15)),
            expected(1, 6, d(2027, 9, 1), d(2027, 11, 15)),
            expected(3, 4, d(2027, 3, 1), d(2027, 11, 15)),
            expected(3, 6, d(2027, 3, 1), d(2027, 5, 31)),
        ]
    );
}

#[test]
fn creneaux_computed_for_each_year_the_planche_exists() {
    let (mut plan, planche) = plan_with_planche();
    plan.delete_planche(planche, 2028, today()).unwrap();
    assert!(!plan.creneaux(2027).is_empty());
    assert!(
        plan.creneaux(2028).is_empty(),
        "the planche no longer exists"
    );
    assert!(
        plan.creneaux(2025).is_empty(),
        "the planche does not exist yet"
    );
}

#[test]
fn a_serie_from_the_previous_year_occupies_the_start_of_the_year() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("poireau", planche, 1, 6, Mode::Abri, d(2027, 5, 1)),
    )
    .unwrap();
    // The leek runs until 2028-02-05: the 2028 window opens on 02-05... which is before 03-01.
    assert_eq!(
        plan.creneaux(2028),
        vec![Creneau {
            planche_id: planche,
            year: 2028,
            zone: zone(1, 6),
            start: d(2028, 3, 1),
            end: d(2028, 11, 15)
        }]
    );
    // In 2027 the leek starts on 07-10: free 03-01..07-09.
    assert_eq!(plan.creneaux(2027)[0].end, d(2027, 7, 9));
}

// ---- §9 suggestions ------------------------------------------------------------

fn creneau(start: chrono::NaiveDate, end: chrono::NaiveDate) -> Creneau {
    Creneau {
        planche_id: planche_id(100),
        year: start.year_(),
        zone: zone(1, 6),
        start,
        end,
    }
}

trait Annee {
    fn year_(&self) -> i32;
}
impl Annee for chrono::NaiveDate {
    fn year_(&self) -> i32 {
        chrono::Datelike::year(self)
    }
}

#[test]
fn culture_discarded_when_the_minimal_harvest_does_not_fit() {
    // Lettuce, harvest 20 days, minimum 10: r = 2027-09-21 for m = 08-02.
    let enough = creneau(d(2027, 8, 2), d(2027, 9, 30)); // r + 10 = 10-01 = F + 1: fits
    assert!(evaluate(&enough, culture("laitue"), today()).is_some());
    let too_short = creneau(d(2027, 8, 2), d(2027, 9, 29)); // F + 1 = 09-30 < r + 10
    assert_eq!(evaluate(&too_short, culture("laitue"), today()), None);
}

#[test]
fn planting_out_is_searched_in_the_first_28_days() {
    // Tomato window opens 05-01: reachable from 04-04 (D + 27), not from 04-03.
    let ok = creneau(d(2027, 4, 4), d(2027, 11, 15));
    assert_eq!(
        evaluate(&ok, culture("tomate"), d(2027, 1, 1))
            .unwrap()
            .planting_out,
        d(2027, 5, 1)
    );
    let too_early = creneau(d(2027, 4, 3), d(2027, 11, 15));
    assert_eq!(evaluate(&too_early, culture("tomate"), d(2027, 1, 1)), None);
}

#[test]
fn sowing_under_cover_today_is_still_possible() {
    let c = creneau(d(2027, 5, 1), d(2027, 11, 15));
    // Tomato sowing would be 03-06.
    assert_eq!(
        evaluate(&c, culture("tomate"), d(2027, 3, 6)).unwrap().mode,
        Mode::Abri
    );
    assert_eq!(
        evaluate(&c, culture("tomate"), d(2027, 3, 7)).unwrap().mode,
        Mode::Plants
    );
}

#[test]
fn crop_without_pots_is_sown_in_pleine_terre() {
    let c = creneau(d(2027, 5, 10), d(2027, 11, 15));
    let haricot = evaluate(&c, culture("haricot-vert"), today()).unwrap();
    assert_eq!(
        (haricot.mode, haricot.date),
        (Mode::PleineTerre, d(2027, 5, 10))
    );
}

#[test]
fn at_most_3_suggestions_ranked_by_filling_then_name() {
    let (plan, planche) = plan_with_planche();
    let c = Creneau {
        planche_id: planche,
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 5, 1),
        end: d(2027, 11, 15),
    };
    let suggestions = plan.suggestions(&c, Catalogue::reference(), d(2027, 4, 20));
    assert_eq!(suggestions.len(), 3);
    let ids: Vec<&str> = suggestions.iter().map(|s| s.culture_id.as_str()).collect();
    // Leek fills the slot (score 1.0, end cut to the slot); then the 0.7 group by name:
    // Carotte, Courgette, Tomate (cut off by the limit of 3).
    assert_eq!(ids, ["poireau", "carotte", "courgette"]);
    assert_eq!(suggestions[0].fin_manuelle, Some(d(2027, 11, 16)));
    assert_eq!(suggestions[0].score_dixiemes(), 10);
}

#[test]
fn on_equal_score_a_different_family_from_the_last_serie_comes_first() {
    let ids = |plan: &Plan, planche| -> Vec<String> {
        let c = Creneau {
            planche_id: planche,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 5, 1),
            end: d(2027, 11, 15),
        };
        plan.suggestions(&c, Catalogue::reference(), d(2027, 4, 20))
            .into_iter()
            .map(|s| s.culture_id)
            .collect()
    };
    // The last serie before the slot is a tomato (Solanacées): in the 0.7 group
    // (carotte, courgette, tomate) it was already last by name.
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        occupying("tomate", planche, 1, 6, d(2027, 3, 1), d(2027, 5, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, planche), ["poireau", "carotte", "courgette"]);

    // It is a carrot (Apiacées): the carrot goes behind the other crops of its score.
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        occupying("carotte", planche, 1, 6, d(2027, 3, 1), d(2027, 5, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, planche), ["poireau", "courgette", "tomate"]);

    // An earlier serie does not count once a later one has ended.
    plan.add_serie(
        serie_id(2),
        occupying("tomate", planche, 1, 6, d(2027, 1, 1), d(2027, 3, 1)),
    )
    .unwrap();
    assert_eq!(ids(&plan, planche), ["poireau", "courgette", "tomate"]);
}

#[test]
fn suggestion_into_serie_prefills_the_creation() {
    let c = creneau(d(2027, 8, 2), d(2027, 11, 15));
    let suggestion = evaluate(&c, culture("laitue"), today()).unwrap();
    let serie = suggestion.into_serie(culture("laitue"), &c).unwrap();
    assert_eq!(
        (serie.mode, serie.date, serie.zone()),
        (Mode::Abri, d(2027, 7, 5), zone(1, 6))
    );
    assert_eq!(serie.dates().planting_out, suggestion.planting_out);
    assert_eq!(serie.dates().end, suggestion.end);
}

// ---- §11 tasks -------------------------------------------------------------

#[test]
fn tasks_by_mode() {
    let types = |mode| -> Vec<TaskType> {
        serie("tomate", planche_id(100), 1, 1, mode, d(2027, 3, 10))
            .tasks(serie_id(1))
            .into_iter()
            .map(|t| t.task_type)
            .collect()
    };
    assert_eq!(
        types(Mode::Abri),
        [
            TaskType::SowUnderCover,
            TaskType::Transplant,
            TaskType::Harvest
        ]
    );
    assert_eq!(types(Mode::Plants), [TaskType::Plant, TaskType::Harvest]);
    assert_eq!(
        types(Mode::PleineTerre),
        [TaskType::SowInGround, TaskType::Harvest]
    );
}

#[test]
fn task_dates_of_an_abri_serie() {
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    let dates: Vec<_> = s.tasks(serie_id(1)).into_iter().map(|t| t.date).collect();
    assert_eq!(dates, [d(2027, 3, 10), d(2027, 5, 5), d(2027, 7, 19)]);
}

#[test]
fn task_id_is_stable_when_the_date_moves() {
    let id = serie_id(7);
    let s = serie("tomate", planche_id(100), 1, 1, Mode::Abri, d(2027, 3, 10));
    let before = s.tasks(id);
    let after = s.shifted(10).tasks(id);
    let ids = |t: &[plan::Task]| t.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&before), ids(&after));
    assert_ne!(before[0].date, after[0].date);
    assert_eq!(before[0].id, format!("{id}/semer-abri"));
    assert_eq!(before[2].id, format!("{id}/recolter"));
}

#[test]
fn task_calendar_by_month() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10)),
    )
    .unwrap();
    let calendrier = plan.calendar_by_month();
    let month: Vec<_> = calendrier.keys().copied().collect();
    assert_eq!(month, [(2027, 3), (2027, 5), (2027, 7)]);
    assert_eq!(calendrier[&(2027, 5)][0].task_type, TaskType::Transplant);
}

#[test]
fn families_of_the_two_previous_seasons() {
    let (mut plan, planche) = plan_with_planche();
    // 2027: tomato (Solanacées) and lettuce; 2028: carrot; 2026 would be Y−2 of 2028.
    plan.add_serie(
        serie_id(1),
        serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10)),
    )
    .unwrap();
    plan.add_serie(
        serie_id(2),
        serie("carotte", planche, 4, 6, Mode::PleineTerre, d(2028, 3, 15)),
    )
    .unwrap();
    plan.add_serie(
        serie_id(3),
        serie("laitue", planche, 4, 6, Mode::Plants, d(2026, 4, 1)),
    )
    .unwrap();
    let families = |year| plan.previous_families(planche, year, Catalogue::reference());
    assert_eq!(families(2028), ["Astéracées", "Solanacées"]);
    assert_eq!(families(2029), ["Apiacées", "Solanacées"]);
    assert_eq!(families(2030), ["Apiacées"]);
    assert!(families(2031).is_empty());
}

// ---- §1 and S05 §5: model --------------------------------------------------------

#[test]
fn plan_serialises_without_derived_data_and_reads_back() {
    let (mut plan, planche) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10)),
    )
    .unwrap();
    plan.set_disposition(planche, 2028, Disposition::Absent, today())
        .unwrap();

    let json = serde_json::to_value(&plan).unwrap();
    // Collections are maps indexed by id, never lists.
    assert!(json["planches"].is_object());
    assert!(json["series"].is_object());
    assert!(json["annotations"].is_object());
    assert!(json["planches"][planche.to_string()]["dispositions"]["2026"].is_object());
    // Only entered values are stored: no m, r, f, quantity, slot or task.
    let serie_json = &json["series"][serie_id(1).to_string()];
    let mut keys: Vec<_> = serie_json.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "abri_jours",
            "apres_jours",
            "culture_id",
            "date",
            "duree_jours",
            "mode",
            "planche_id",
            "troncon_debut",
            "troncon_fin"
        ]
    );
    assert_eq!(serie_json["mode"], "abri");
    assert_eq!(serie_json["date"], "2027-03-10");

    let reread: Plan = serde_json::from_value(json).unwrap();
    assert_eq!(reread, plan);
}

#[test]
fn technical_timestamp_in_utc() {
    let (mut plan, _) = plan_with_planche();
    assert_eq!(plan.version, 1);
    plan.touch(now() + chrono::Duration::hours(1));
    assert_eq!(plan.version, 2);
    let json = serde_json::to_value(&plan).unwrap();
    assert!(json["mis_a_jour_le"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn generated_ids_are_distinct_uuid_v7() {
    let (a, b) = (plan::SerieId::generate(), plan::SerieId::generate());
    assert_ne!(a, b);
    assert_eq!(a.uuid().get_version_num(), 7);
}

#[test]
fn stable_error_codes_for_the_ui() {
    assert_eq!(PlanError::PotDaysRequired.code(), "pot_days_required");
    assert_eq!(
        PlanError::TooManyPlanches {
            year: 2027,
            max: 10
        }
        .code(),
        "too_many_planches"
    );
}

#[test]
fn zone_type_is_reexported() {
    assert_eq!(Zone::new(1, 2).unwrap().troncon_count(), 2);
}
