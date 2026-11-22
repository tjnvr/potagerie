//! The test cases T1 to T14, T20 and T21 of S03 §12 (T15 to T19 are in
//! `perennials.rs`). Expected values are copied as they are
//! in the spec.

mod common;

use catalogue::Catalogue;
use chrono::NaiveDate;
use common::*;
use plan::{Mode, Occupation, PlanError, Planting, Quantity, Slot, evaluate, quantity};

fn dates_of(planting: &Planting) -> (NaiveDate, NaiveDate, NaiveDate) {
    let dates = planting.dates();
    (dates.planting_out, dates.harvest_start, dates.end)
}

/// T1 · Tomato, sown under cover on 2027-03-10
/// → `m` 2027-05-05 · `r` 2027-07-19 · `f` 2027-09-27
#[test]
fn t01_tomato_sown_under_cover() {
    let s = planting("tomato", bed_id(100), 1, 3, Mode::Indoor, d(2027, 3, 10));
    assert_eq!(
        dates_of(&s),
        (d(2027, 5, 5), d(2027, 7, 19), d(2027, 9, 27))
    );
}

/// T2 · Lettuce, open ground on 2027-04-01
/// → `m` 2027-04-01 · `r` 2027-06-18 (50 + 28 j) · `f` 2027-07-08
#[test]
fn t02_lettuce_open_ground() {
    let s = planting(
        "lettuce",
        bed_id(100),
        1,
        1,
        Mode::OpenGround,
        d(2027, 4, 1),
    );
    assert_eq!(dates_of(&s), (d(2027, 4, 1), d(2027, 6, 18), d(2027, 7, 8)));
}

/// T3 · Carrot, open ground on 2027-03-15
/// → `r` 2027-06-23 · `f` 2027-08-02
#[test]
fn t03_carrot_open_ground() {
    let s = planting(
        "carrot",
        bed_id(100),
        1,
        1,
        Mode::OpenGround,
        d(2027, 3, 15),
    );
    let (_, r, f) = dates_of(&s);
    assert_eq!((r, f), (d(2027, 6, 23), d(2027, 8, 2)));
}

/// T4 · Carrot, zone of 2 segments → 4 rows × 25 plants = ≈ 100
#[test]
fn t04_carrot_quantity() {
    let q = quantity(120, 2, plan::CellCm::C50, crop("carrot"));
    assert_eq!(
        q,
        Quantity {
            rows: 4,
            plants_per_row: 25
        }
    );
    assert_eq!(q.total(), 100);
}

/// T5 · Tomato, zone of 3 segments → 1 row × 2 plants = ≈ 2
#[test]
fn t05_tomato_quantity() {
    let q = quantity(120, 3, plan::CellCm::C50, crop("tomato"));
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
fn plan_t6() -> (plan::Plan, plan::BedId, plan::PlantingId, plan::PlantingId) {
    let (mut plan, bed) = plan_with_bed();
    let tomato = planting("tomato", bed, 1, 3, Mode::Indoor, d(2027, 3, 10));
    let lettuce = planting("lettuce", bed, 1, 2, Mode::Plants, d(2027, 9, 27));
    let (tomato_id, lettuce_id) = (planting_id(1), planting_id(2));
    plan.add_planting(tomato_id, tomato).unwrap();
    plan.add_planting(lettuce_id, lettuce).unwrap();
    (plan, bed, tomato_id, lettuce_id)
}

#[test]
fn t06_no_conflict_on_the_day_the_other_ends() {
    let (plan, _, tomato, lettuce) = plan_t6();
    assert_eq!(plan.plantings[&tomato].dates().end, d(2027, 9, 27));
    assert_eq!(
        plan.plantings[&lettuce].dates().planting_out,
        d(2027, 9, 27)
    );
    assert!(plan.conflicts().is_empty());
}

/// T7 · Bed of 6 segments, with only the carrot T3 on [1,6], year 2027
/// → Slots: [1,6] from 03-01 to 03-14 dropped (< 35 days);
/// **[1,6] from 2027-08-02 to 2027-11-15** (106 days)
fn plan_t7() -> (plan::Plan, plan::BedId) {
    let (mut plan, bed) = plan_with_bed();
    let carrot = planting("carrot", bed, 1, 6, Mode::OpenGround, d(2027, 3, 15));
    plan.add_planting(planting_id(1), carrot).unwrap();
    (plan, bed)
}

#[test]
fn t07_slots_after_the_carrot() {
    let (plan, bed) = plan_t7();
    let slots = plan.slots(2027);
    assert_eq!(
        slots,
        vec![Slot {
            bed_id: bed,
            year: 2027,
            zone: zone(1, 6),
            start: d(2027, 8, 2),
            end: d(2027, 11, 15),
        }]
    );
    assert_eq!(slots[0].length_days(), 106);
}

/// T8 · Suggestions for the slot of T7
/// → Lettuce only: sown under cover on 2027-07-05, `m` 2027-08-02, `r` 2027-09-21,
/// `f` 2027-10-11; the 5 other crops are out of their planting window
#[test]
fn t08_slot_suggestions() {
    let (plan, _) = plan_t7();
    let slot = plan.slots(2027)[0];
    let suggestions = plan.suggestions(&slot, Catalogue::reference(), today());
    assert_eq!(suggestions.len(), 1);
    let lettuce = &suggestions[0];
    assert_eq!(lettuce.crop_id, "lettuce");
    assert_eq!(lettuce.mode, Mode::Indoor);
    assert_eq!(lettuce.date, d(2027, 7, 5));
    assert_eq!(lettuce.planting_out, d(2027, 8, 2));
    assert_eq!(lettuce.harvest_start, d(2027, 9, 21));
    assert_eq!(lettuce.end, d(2027, 10, 11));
    assert_eq!(lettuce.manual_end, None);
    // The 5 other crops are out of their planting window.
    for id in ["tomato", "carrot", "green-bean", "zucchini", "leek"] {
        assert_eq!(evaluate(&slot, crop(id), today()), None, "{id}");
    }
}

/// T9 · Carrot, sown under cover on 2027-03-01, 21 days in pots entered
/// → `m` 2027-03-22 · `r` 2027-06-30 · `f` 2027-08-09
#[test]
fn t09_carrot_under_cover_with_entered_pot_days() {
    let s = Planting::new(
        crop("carrot"),
        bed_id(100),
        zone(1, 1),
        Mode::Indoor,
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

/// T10 · Slot [1,6] from 2027-05-01 to 2027-11-15, today = 2027-04-20
/// → The tomato is proposed **as plants** (its sowing under cover would fall on
/// 2027-03-06, in the past): `m` 2027-05-01 · `r` 2027-07-15 · `f` 2027-09-23
#[test]
fn t10_tomato_as_plants_when_sowing_would_be_in_the_past() {
    let slot = Slot {
        bed_id: bed_id(100),
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 5, 1),
        end: d(2027, 11, 15),
    };
    let tomato = evaluate(&slot, crop("tomato"), d(2027, 4, 20)).unwrap();
    assert_eq!(tomato.mode, Mode::Plants);
    assert_eq!(tomato.date, d(2027, 5, 1));
    assert_eq!(tomato.planting_out, d(2027, 5, 1));
    assert_eq!(tomato.harvest_start, d(2027, 7, 15));
    assert_eq!(tomato.end, d(2027, 9, 23));
    assert_eq!(tomato.manual_end, None);

    // Without the cut-off, the same slot would have given a sowing under cover.
    let pot_days = evaluate(&slot, crop("tomato"), d(2027, 3, 6)).unwrap();
    assert_eq!(
        (pot_days.mode, pot_days.date),
        (Mode::Indoor, d(2027, 3, 6))
    );
}

/// T11 · Bed of 6 segments: carrot T3 on [1,3], leek sown under cover on
/// 2027-05-01 on [4,6]; bed deleted from 2028
/// → Leek: `m` 2027-07-10 · `r` 2027-11-07 · `f` 2028-02-05 → **deleted**
/// (spans 2028). Carrot kept
#[test]
fn t11_bed_deleted_from_2028() {
    let (mut plan, bed) = plan_with_bed();
    let carrot = planting("carrot", bed, 1, 3, Mode::OpenGround, d(2027, 3, 15));
    let leek = planting("leek", bed, 4, 6, Mode::Indoor, d(2027, 5, 1));
    assert_eq!(
        dates_of(&leek),
        (d(2027, 7, 10), d(2027, 11, 7), d(2028, 2, 5))
    );
    plan.add_planting(planting_id(1), carrot).unwrap();
    plan.add_planting(planting_id(2), leek).unwrap();

    assert_eq!(plan.plantings_deleted_by(bed, 2028), vec![planting_id(2)]);
    let deleted = plan.delete_bed(bed, 2028, today()).unwrap();

    assert_eq!(deleted, vec![planting_id(2)]);
    assert!(plan.plantings.contains_key(&planting_id(1)));
    assert!(!plan.plantings.contains_key(&planting_id(2)));
}

/// T12 · Plantings of T6; the lettuce is dragged by −1 day
/// → Refused: the lettuce would start on 2027-09-26, in conflict with the tomato on [1,2]
#[test]
fn t12_move_refused_on_conflict() {
    let (mut plan, _, tomato, lettuce) = plan_t6();
    let before = plan.clone();

    let error = plan.shift_planting(lettuce, -1).unwrap_err();

    assert_eq!(
        error,
        PlanError::Conflict {
            with: tomato,
            segments: zone(1, 2),
            period: Occupation {
                start: d(2027, 9, 26),
                end: d(2027, 9, 27),
            },
        }
    );
    assert_eq!(error.code(), "conflict");
    assert_eq!(plan, before, "a refused move changes nothing");
    assert_eq!(plan.plantings[&lettuce].date, d(2027, 9, 27));
}

/// T13 · Slot [1,6] from 2027-08-02 to 2027-10-05 (65 days)
/// → Lettuce: `m` 2027-08-02 · `r` 2027-09-21 · `f` proposed 2027-10-06 instead of
/// 2027-10-11, recorded as a **manually modified end**
#[test]
fn t13_shortened_end_recorded_as_manual_end() {
    let slot = Slot {
        bed_id: bed_id(100),
        year: 2027,
        zone: zone(1, 6),
        start: d(2027, 8, 2),
        end: d(2027, 10, 5),
    };
    assert_eq!(slot.length_days(), 65);

    let lettuce = evaluate(&slot, crop("lettuce"), today()).unwrap();
    assert_eq!(lettuce.planting_out, d(2027, 8, 2));
    assert_eq!(lettuce.harvest_start, d(2027, 9, 21));
    assert_eq!(lettuce.end, d(2027, 10, 6));
    assert_eq!(lettuce.manual_end, Some(d(2027, 10, 6)));

    // Touching the suggestion creates a planting whose end no longer follows `r + duration`.
    let planting = lettuce.into_planting(crop("lettuce"), &slot).unwrap();
    assert_eq!(planting.manual_end, Some(d(2027, 10, 6)));
    assert_eq!(planting.dates().end, d(2027, 10, 6));
}

/// T14 · Plot 20 × 10 m, aisle 50 cm; bed A 3 × 1.2 m at (0, 0); bed B of the
/// same size placed at (330, 0), 30 cm from A (S03 §15)
/// → Without "Keep here": refused, `aisle_too_narrow`, nothing changes. With
/// "Keep here": accepted, and the pair is not "to resolve". Moving B afterwards
/// to (340, 0) does not ask again. B at (200, 0), over A: refused even with
/// "Keep here". The aisle can then go to 80 cm; after a bed C 60 cm below A,
/// it can no longer go to 80 cm, but it can go to 30 cm.
#[test]
fn t14_keep_a_bed_closer_than_the_aisle() {
    use plan::{Geometry, Layout, Plan, TypeBed};
    let at = |x, y| Geometry {
        bed_type: TypeBed::Bed,
        x,
        y,
        length_cm: 300,
        width_cm: 120,
    };
    // As the commands do: the change, then the placement check against the plan before it.
    let place = |plan: &mut Plan, keep_close: bool, change: &dyn Fn(&mut Plan)| {
        let before = plan.clone();
        change(plan);
        let checked = plan.check_placement_change(&before, 2026, keep_close);
        if checked.is_err() {
            *plan = before;
        }
        checked
    };
    let (a, b, c) = (bed_id(1), bed_id(2), bed_id(3));
    let mut plan = Plan::new(
        plan::PlanId::from_uuid(uuid::Uuid::from_u128(1)),
        plot(),
        now(),
    );
    plan.add_bed(a, "A".into(), 2026, at(0, 0), today())
        .unwrap();
    let add_b = |p: &mut Plan| p.add_bed(b, "B".into(), 2026, at(330, 0), today()).unwrap();

    let before = plan.clone();
    assert!(matches!(
        place(&mut plan, false, &add_b),
        Err(PlanError::AisleTooNarrow { aisle_cm: 50, .. })
    ));
    assert_eq!(plan, before, "a refusal changes nothing");

    assert_eq!(place(&mut plan, true, &add_b), Ok(()));
    assert!(plan.beds.contains_key(&b));
    assert!(
        plan.incoherences().is_empty(),
        "a kept pair is not to resolve"
    );

    let move_b = |x| {
        move |p: &mut Plan| {
            p.set_layout(b, 2026, Layout::Present(at(x, 0)), today())
                .unwrap()
        }
    };
    assert_eq!(place(&mut plan, false, &move_b(340)), Ok(()));
    assert!(matches!(
        place(&mut plan, true, &move_b(200)),
        Err(PlanError::BedsOverlap { .. })
    ));
    assert_eq!(plan.beds[&b].layout_in(2026), Some(&at(340, 0)));

    let mut wider = plan.clone();
    assert_eq!(wider.set_plot(2000, 1000, 80, today()), Ok(()));

    let add_c = |p: &mut Plan| p.add_bed(c, "C".into(), 2026, at(0, 180), today()).unwrap();
    assert_eq!(place(&mut plan, false, &add_c), Ok(()));
    assert!(matches!(
        plan.set_plot(2000, 1000, 80, today()),
        Err(PlanError::AisleTooNarrow { aisle_cm: 80, .. })
    ));
    assert_eq!(plan.set_plot(2000, 1000, 30, today()), Ok(()));
}

/// T20 · Plot 20 × 10 m, created in 2026: bed A "Tomatoes", bed B named
/// " tomatoes ", bed C "Shed side", bed D "Tomatoes" deleted from 2027
/// (S03 §15)
/// → Accepted. Year 2027: A and B flagged `name_shared`, C not, D absent.
/// Year 2026: A, B and D flagged, C not. B renamed "Leeks": in 2027 no bed is
/// flagged, in 2026 A and D still are. B is saved trimmed by the commands
/// (`bed_names_are_trimmed_when_saved`).
#[test]
fn t20_two_beds_with_the_same_name_are_flagged() {
    use plan::{Geometry, Plan, TypeBed, YearView};
    let at = |x, y| Geometry {
        bed_type: TypeBed::Bed,
        x,
        y,
        length_cm: 300,
        width_cm: 120,
    };
    let (a, b, c, dd) = (bed_id(1), bed_id(2), bed_id(3), bed_id(4));
    let mut plan = Plan::new(
        plan::PlanId::from_uuid(uuid::Uuid::from_u128(1)),
        plot(),
        now(),
    );
    for (id, name, x) in [
        (a, "Tomatoes", 0),
        (b, " tomatoes ", 400),
        (c, "Shed side", 800),
        (dd, "Tomatoes", 1200),
    ] {
        plan.add_bed(id, name.into(), 2026, at(x, 0), today())
            .unwrap();
    }
    plan.delete_bed(dd, 2027, today()).unwrap();

    let flagged = |plan: &Plan, year| {
        let mut ids: Vec<_> = YearView::build(Some(plan), year, today(), Catalogue::reference())
            .beds
            .into_iter()
            .filter(|bed| bed.name_shared)
            .map(|bed| bed.id)
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(flagged(&plan, 2027), vec![a, b]);
    assert_eq!(flagged(&plan, 2026), vec![a, b, dd]);

    plan.beds.get_mut(&b).unwrap().name = "Leeks".into();
    assert_eq!(flagged(&plan, 2027), Vec::<plan::BedId>::new());
    assert_eq!(flagged(&plan, 2026), vec![a, dd]);
}

/// T21 · Plot 20 × 10 m; bed A 3 × 1.2 m at (0, 0) in 2027, with a planting.
/// A duplicated three times in 2027 (S03 §15, "Copy of a bed")
/// → 1st copy at (350, 0), 2nd at (0, 170), 3rd at (700, 0); each 3 × 1.2 m,
/// of the type of A, with no planting. A copied in 2027, then moved in 2028
/// to (1000, 500), and pasted in 2028 on a plot with only A: (350, 0). With a
/// bed B added in 2029 at (350, 0), the same paste lands at (0, 170). Plot
/// 4 × 2 m with A alone: `no_free_spot`. A year with 10 beds: `too_many_beds`.
/// A refusal changes nothing.
#[test]
fn t21_a_copied_bed_lands_on_the_first_free_spot() {
    use plan::{Geometry, Layout, Plan, PlanId, Plot, TypeBed};
    let at = |x, y| Geometry {
        bed_type: TypeBed::Mound,
        x,
        y,
        length_cm: 300,
        width_cm: 120,
    };
    let empty = |plot: Plot| Plan::new(PlanId::from_uuid(uuid::Uuid::from_u128(1)), plot, now());
    let with_a = |plot: Plot| {
        let mut plan = empty(plot);
        plan.add_bed(bed_id(1), "A".into(), 2027, at(0, 0), today())
            .unwrap();
        plan
    };
    let a = at(0, 0);

    // Duplicated three times in 2027.
    let mut plan = with_a(plot());
    plan.add_planting(
        planting_id(1),
        planting("lettuce", bed_id(1), 1, 2, Mode::OpenGround, d(2027, 4, 1)),
    )
    .unwrap();
    let mut placed = Vec::new();
    for n in 2..=4 {
        let g = plan
            .paste_bed(bed_id(n), format!("C{n}"), 2027, &a, today())
            .unwrap();
        assert_eq!(plan.beds[&bed_id(n)].layout_in(2027), Some(&g));
        placed.push((g.x, g.y, g.length_cm, g.width_cm, g.bed_type));
    }
    assert_eq!(
        placed,
        vec![
            (350, 0, 300, 120, TypeBed::Mound),
            (0, 170, 300, 120, TypeBed::Mound),
            (700, 0, 300, 120, TypeBed::Mound),
        ]
    );
    assert!(plan.plantings.values().all(|p| p.bed_id == bed_id(1)));

    // Copied in 2027, A moved in 2028, pasted in 2028.
    let mut plan = with_a(plot());
    plan.set_layout(bed_id(1), 2028, Layout::Present(at(1000, 500)), today())
        .unwrap();
    let mut pasted = plan.clone();
    let g = pasted
        .paste_bed(bed_id(2), "C".into(), 2028, &a, today())
        .unwrap();
    assert_eq!((g.x, g.y), (350, 0));
    assert_eq!(pasted.beds[&bed_id(2)].layout_in(2027), None);

    // A bed B in 2029 where the copy would go: the copy keeps the aisle in 2029 too.
    plan.add_bed(bed_id(3), "B".into(), 2029, at(350, 0), today())
        .unwrap();
    let g = plan
        .paste_bed(bed_id(2), "C".into(), 2028, &a, today())
        .unwrap();
    assert_eq!((g.x, g.y), (0, 170));

    // No free spot.
    let small = Plot {
        length_cm: 400,
        width_cm: 200,
        ..plot()
    };
    let mut plan = with_a(small);
    let before = plan.clone();
    assert_eq!(
        plan.paste_bed(bed_id(2), "C".into(), 2027, &a, today()),
        Err(PlanError::NoFreeSpot { year: 2027 })
    );
    assert_eq!(plan, before);
    assert_eq!(PlanError::NoFreeSpot { year: 2027 }.code(), "no_free_spot");

    // Ten beds.
    let mut plan = empty(plot());
    for n in 1..=10u32 {
        let (col, row) = ((n - 1) % 5, (n - 1) / 5);
        plan.add_bed(
            bed_id(u128::from(n)),
            format!("B{n}"),
            2027,
            at(col * 350, row * 170),
            today(),
        )
        .unwrap();
    }
    let before = plan.clone();
    assert!(matches!(
        plan.paste_bed(bed_id(11), "C".into(), 2027, &a, today()),
        Err(PlanError::TooManyBeds { year: 2027, .. })
    ));
    assert_eq!(plan, before);
}
