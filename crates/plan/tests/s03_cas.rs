//! The test cases T1 to T13 of S03 §12. Expected values are copied as they are
//! in the spec.

mod common;

use catalogue::Catalogue;
use chrono::NaiveDate;
use common::*;
use plan::{Creneau, Mode, Occupation, PlanError, Quantity, Serie, evaluate, quantity};

fn dates_of(serie: &Serie) -> (NaiveDate, NaiveDate, NaiveDate) {
    let dates = serie.dates();
    (dates.planting_out, dates.harvest_start, dates.end)
}

/// T1 · Tomato, sown under cover on 2027-03-10
/// → `m` 2027-05-05 · `r` 2027-07-19 · `f` 2027-09-27
#[test]
fn t01_tomato_sown_under_cover() {
    let s = serie("tomate", planche_id(100), 1, 3, Mode::Abri, d(2027, 3, 10));
    assert_eq!(
        dates_of(&s),
        (d(2027, 5, 5), d(2027, 7, 19), d(2027, 9, 27))
    );
}

/// T2 · Lettuce, pleine terre on 2027-04-01
/// → `m` 2027-04-01 · `r` 2027-06-18 (50 + 28 j) · `f` 2027-07-08
#[test]
fn t02_lettuce_pleine_terre() {
    let s = serie(
        "laitue",
        planche_id(100),
        1,
        1,
        Mode::PleineTerre,
        d(2027, 4, 1),
    );
    assert_eq!(dates_of(&s), (d(2027, 4, 1), d(2027, 6, 18), d(2027, 7, 8)));
}

/// T3 · Carrot, pleine terre on 2027-03-15
/// → `r` 2027-06-23 · `f` 2027-08-02
#[test]
fn t03_carrot_pleine_terre() {
    let s = serie(
        "carotte",
        planche_id(100),
        1,
        1,
        Mode::PleineTerre,
        d(2027, 3, 15),
    );
    let (_, r, f) = dates_of(&s);
    assert_eq!((r, f), (d(2027, 6, 23), d(2027, 8, 2)));
}

/// T4 · Carrot, zone of 2 troncons → 4 rows × 25 plants = ≈ 100
#[test]
fn t04_carrot_quantity() {
    let q = quantity(120, 2, plan::CaseCm::C50, culture("carotte"));
    assert_eq!(
        q,
        Quantity {
            rows: 4,
            plants_per_row: 25
        }
    );
    assert_eq!(q.total(), 100);
}

/// T5 · Tomato, zone of 3 troncons → 1 row × 2 plants = ≈ 2
#[test]
fn t05_tomato_quantity() {
    let q = quantity(120, 3, plan::CaseCm::C50, culture("tomate"));
    assert_eq!(
        q,
        Quantity {
            rows: 1,
            plants_per_row: 2
        }
    );
    assert_eq!(q.total(), 2);
}

/// T6 · Tomato T1 on [1,3], lettuce as plants on 2027-09-27 on [1,2]
/// → No conflict: the tomato ends the day the lettuce starts
fn plan_t6() -> (plan::Plan, plan::PlancheId, plan::SerieId, plan::SerieId) {
    let (mut plan, planche) = plan_with_planche();
    let tomate = serie("tomate", planche, 1, 3, Mode::Abri, d(2027, 3, 10));
    let laitue = serie("laitue", planche, 1, 2, Mode::Plants, d(2027, 9, 27));
    let (tomate_id, laitue_id) = (serie_id(1), serie_id(2));
    plan.add_serie(tomate_id, tomate).unwrap();
    plan.add_serie(laitue_id, laitue).unwrap();
    (plan, planche, tomate_id, laitue_id)
}

#[test]
fn t06_no_conflict_on_the_day_the_other_ends() {
    let (plan, _, tomate, laitue) = plan_t6();
    assert_eq!(plan.series[&tomate].dates().end, d(2027, 9, 27));
    assert_eq!(plan.series[&laitue].dates().planting_out, d(2027, 9, 27));
    assert!(plan.conflicts().is_empty());
}

/// T7 · Planche of 6 troncons, with only the carrot T3 on [1,6], year 2027
/// → Creneaux: [1,6] from 03-01 to 03-14 dropped (< 35 days);
/// **[1,6] from 2027-08-02 to 2027-11-15** (106 days)
fn plan_t7() -> (plan::Plan, plan::PlancheId) {
    let (mut plan, planche) = plan_with_planche();
    let carotte = serie("carotte", planche, 1, 6, Mode::PleineTerre, d(2027, 3, 15));
    plan.add_serie(serie_id(1), carotte).unwrap();
    (plan, planche)
}

#[test]
fn t07_creneaux_after_the_carrot() {
    let (plan, planche) = plan_t7();
    let creneaux = plan.creneaux(2027);
    assert_eq!(
        creneaux,
        vec![Creneau {
            planche_id: planche,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 8, 2),
            end: d(2027, 11, 15),
        }]
    );
    assert_eq!(creneaux[0].length_days(), 106);
}

/// T8 · Suggestions for the creneau of T7
/// → Lettuce only: sown under cover on 2027-07-05, `m` 2027-08-02, `r` 2027-09-21,
/// `f` 2027-10-11; the 5 other crops are out of their planting window
#[test]
fn t08_creneau_suggestions() {
    let (plan, _) = plan_t7();
    let creneau = plan.creneaux(2027)[0];
    let suggestions = plan.suggestions(&creneau, Catalogue::reference(), today());
    assert_eq!(suggestions.len(), 1);
    let laitue = &suggestions[0];
    assert_eq!(laitue.culture_id, "laitue");
    assert_eq!(laitue.mode, Mode::Abri);
    assert_eq!(laitue.date, d(2027, 7, 5));
    assert_eq!(laitue.planting_out, d(2027, 8, 2));
    assert_eq!(laitue.harvest_start, d(2027, 9, 21));
    assert_eq!(laitue.end, d(2027, 10, 11));
    assert_eq!(laitue.fin_manuelle, None);
    // The 5 other crops are out of their planting window.
    for id in ["tomate", "carotte", "haricot-vert", "courgette", "poireau"] {
        assert_eq!(evaluate(&creneau, culture(id), today()), None, "{id}");
    }
}

/// T9 · Carrot, sown under cover on 2027-03-01, 21 days in pots entered
/// → `m` 2027-03-22 · `r` 2027-06-30 · `f` 2027-08-09
#[test]
fn t09_carrot_under_cover_with_entered_pot_days() {
    let s = Serie::new(
        culture("carotte"),
        planche_id(100),
        zone(1, 1),
        Mode::Abri,
        d(2027, 3, 1),
        Some(21),
        None,
    )
    .unwrap();
    assert_eq!(
        dates_of(&s),
        (d(2027, 3, 22), d(2027, 6, 30), d(2027, 8, 9))
    );
}

/// T10 · Creneau [1,6] from 2027-05-01 to 2027-11-15, today = 2027-04-20
/// → The tomato is proposed **as plants** (its sowing under cover would fall on
/// 2027-03-06, in the past): `m` 2027-05-01 · `r` 2027-07-15 · `f` 2027-09-23
#[test]
fn t10_tomato_as_plants_when_sowing_would_be_in_the_past() {
    let creneau = Creneau {
        planche_id: planche_id(100),
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 5, 1),
        end: d(2027, 11, 15),
    };
    let tomate = evaluate(&creneau, culture("tomate"), d(2027, 4, 20)).unwrap();
    assert_eq!(tomate.mode, Mode::Plants);
    assert_eq!(tomate.date, d(2027, 5, 1));
    assert_eq!(tomate.planting_out, d(2027, 5, 1));
    assert_eq!(tomate.harvest_start, d(2027, 7, 15));
    assert_eq!(tomate.end, d(2027, 9, 23));
    assert_eq!(tomate.fin_manuelle, None);

    // Without the cut-off, the same slot would have given a sowing under cover.
    let pot_days = evaluate(&creneau, culture("tomate"), d(2027, 3, 6)).unwrap();
    assert_eq!((pot_days.mode, pot_days.date), (Mode::Abri, d(2027, 3, 6)));
}

/// T11 · Planche of 6 troncons: carrot T3 on [1,3], leek sown under cover on
/// 2027-05-01 on [4,6]; planche deleted from 2028
/// → Leek: `m` 2027-07-10 · `r` 2027-11-07 · `f` 2028-02-05 → **deleted**
/// (spans 2028). Carrot kept
#[test]
fn t11_planche_deleted_from_2028() {
    let (mut plan, planche) = plan_with_planche();
    let carotte = serie("carotte", planche, 1, 3, Mode::PleineTerre, d(2027, 3, 15));
    let poireau = serie("poireau", planche, 4, 6, Mode::Abri, d(2027, 5, 1));
    assert_eq!(
        dates_of(&poireau),
        (d(2027, 7, 10), d(2027, 11, 7), d(2028, 2, 5))
    );
    plan.add_serie(serie_id(1), carotte).unwrap();
    plan.add_serie(serie_id(2), poireau).unwrap();

    assert_eq!(plan.series_deleted_by(planche, 2028), vec![serie_id(2)]);
    let deleted = plan.delete_planche(planche, 2028, today()).unwrap();

    assert_eq!(deleted, vec![serie_id(2)]);
    assert!(plan.series.contains_key(&serie_id(1)));
    assert!(!plan.series.contains_key(&serie_id(2)));
}

/// T12 · Series of T6; the lettuce is dragged by −1 day
/// → Refused: the lettuce would start on 2027-09-26, in conflict with the tomato on [1,2]
#[test]
fn t12_move_refused_on_conflict() {
    let (mut plan, _, tomate, laitue) = plan_t6();
    let before = plan.clone();

    let error = plan.shift_serie(laitue, -1).unwrap_err();

    assert_eq!(
        error,
        PlanError::Conflict {
            with: tomate,
            troncons: zone(1, 2),
            period: Occupation {
                start: d(2027, 9, 26),
                end: d(2027, 9, 27),
            },
        }
    );
    assert_eq!(error.code(), "conflict");
    assert_eq!(plan, before, "a refused move changes nothing");
    assert_eq!(plan.series[&laitue].date, d(2027, 9, 27));
}

/// T13 · Creneau [1,6] from 2027-08-02 to 2027-10-05 (65 days)
/// → Lettuce: `m` 2027-08-02 · `r` 2027-09-21 · `f` proposed 2027-10-06 instead of
/// 2027-10-11, recorded as a **manually modified end**
#[test]
fn t13_shortened_end_recorded_as_manual_end() {
    let creneau = Creneau {
        planche_id: planche_id(100),
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 8, 2),
        end: d(2027, 10, 5),
    };
    assert_eq!(creneau.length_days(), 65);

    let laitue = evaluate(&creneau, culture("laitue"), today()).unwrap();
    assert_eq!(laitue.planting_out, d(2027, 8, 2));
    assert_eq!(laitue.harvest_start, d(2027, 9, 21));
    assert_eq!(laitue.end, d(2027, 10, 6));
    assert_eq!(laitue.fin_manuelle, Some(d(2027, 10, 6)));

    // Touching the suggestion creates a serie whose end no longer follows `r + duration`.
    let serie = laitue.into_serie(culture("laitue"), &creneau).unwrap();
    assert_eq!(serie.fin_manuelle, Some(d(2027, 10, 6)));
    assert_eq!(serie.dates().end, d(2027, 10, 6));
}
