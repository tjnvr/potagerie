//! Notes and periods, validation of a received plan, "clear le plan".

mod common;

use chrono::NaiveDate;
use common::*;
use plan::{Annotation, AnnotationId, BedId, Incoherence, Layout, Mode, PlanError, TypeAnnotation};
use uuid::Uuid;

fn aid(n: u128) -> AnnotationId {
    AnnotationId::from_uuid(Uuid::from_u128(n))
}

fn note(date: NaiveDate, bed: Option<BedId>, text: &str) -> Annotation {
    Annotation {
        type_annotation: TypeAnnotation::Note,
        date_start: date,
        date_end: None,
        bed_id: bed,
        text: text.into(),
        hidden: false,
    }
}

fn period(a: NaiveDate, b: Option<NaiveDate>) -> Annotation {
    Annotation {
        type_annotation: TypeAnnotation::Period,
        date_end: b,
        ..note(a, None, "Holidays")
    }
}

#[test]
fn a_note_and_a_period_are_added_edited_hidden_and_deleted() {
    let (mut plan, p) = plan_with_bed();
    plan.add_annotation(
        aid(1),
        2026,
        note(d(2026, 12, 1), Some(p), "  Mulch  "),
        today(),
    )
    .unwrap();
    assert_eq!(plan.annotations[&aid(1)].text, "Mulch");
    plan.add_annotation(
        aid(2),
        2027,
        period(d(2027, 8, 1), Some(d(2027, 8, 21))),
        today(),
    )
    .unwrap();

    let mut hidden = plan.annotations[&aid(1)].clone();
    hidden.hidden = true;
    plan.replace_annotation(aid(1), 2026, hidden, today())
        .unwrap();
    assert!(plan.annotations[&aid(1)].hidden);

    plan.delete_annotation(aid(2), 2027, today()).unwrap();
    assert!(!plan.annotations.contains_key(&aid(2)));
    assert_eq!(
        plan.delete_annotation(aid(2), 2027, today()),
        Err(PlanError::UnknownAnnotation(aid(2)))
    );
}

#[test]
fn a_note_ignores_an_end_date() {
    let (mut plan, _) = plan_with_bed();
    let mut n = note(d(2026, 12, 1), None, "x");
    n.date_end = Some(d(2020, 1, 1));
    plan.add_annotation(aid(1), 2026, n, today()).unwrap();
    assert_eq!(plan.annotations[&aid(1)].date_end, None);
}

#[test]
fn past_years_are_read_only() {
    let (mut plan, _) = plan_with_bed();
    // Displayed year in the past.
    assert!(matches!(
        plan.add_annotation(aid(1), 2025, note(d(2026, 12, 1), None, "x"), today()),
        Err(PlanError::PastYear { year: 2025, .. })
    ));
    // Annotation in the past.
    assert!(matches!(
        plan.add_annotation(aid(1), 2026, note(d(2025, 6, 1), None, "x"), today()),
        Err(PlanError::PastYear { year: 2025, .. })
    ));
    // A period that ends this year is allowed even if it starts earlier.
    plan.add_annotation(
        aid(2),
        2026,
        period(d(2025, 12, 20), Some(d(2026, 1, 5))),
        today(),
    )
    .unwrap();
    // A stored past annotation is neither edited nor deleted.
    plan.annotations
        .insert(aid(3), note(d(2025, 3, 1), None, "old"));
    assert!(matches!(
        plan.replace_annotation(aid(3), 2026, note(d(2026, 3, 1), None, "x"), today()),
        Err(PlanError::PastYear { .. })
    ));
    assert!(matches!(
        plan.delete_annotation(aid(3), 2026, today()),
        Err(PlanError::PastYear { .. })
    ));
    assert!(plan.annotations.contains_key(&aid(3)));
}

#[test]
fn unknown_bed_bad_dates_and_bad_text_are_refused() {
    let (mut plan, _) = plan_with_bed();
    let ghost = bed_id(999);
    assert_eq!(
        plan.add_annotation(
            aid(1),
            2026,
            note(d(2026, 12, 1), Some(ghost), "x"),
            today()
        ),
        Err(PlanError::UnknownBed(ghost))
    );
    for p in [
        period(d(2026, 12, 5), Some(d(2026, 12, 4))),
        period(d(2026, 12, 5), None),
    ] {
        assert_eq!(
            plan.add_annotation(aid(1), 2026, p, today()),
            Err(PlanError::InvalidAnnotationDates)
        );
    }
    // Same day is fine.
    plan.add_annotation(
        aid(4),
        2026,
        period(d(2026, 12, 5), Some(d(2026, 12, 5))),
        today(),
    )
    .unwrap();
    for t in ["", "   ", &"é".repeat(81)] {
        assert_eq!(
            plan.add_annotation(aid(1), 2026, note(d(2026, 12, 1), None, t), today()),
            Err(PlanError::InvalidAnnotationText)
        );
    }
    plan.add_annotation(
        aid(5),
        2026,
        note(d(2026, 12, 1), None, &"é".repeat(80)),
        today(),
    )
    .unwrap();
    assert_eq!(plan.annotations.len(), 2);
}

#[test]
fn replacing_an_unknown_annotation_is_refused() {
    let (mut plan, _) = plan_with_bed();
    assert_eq!(
        plan.replace_annotation(aid(9), 2026, note(d(2026, 12, 1), None, "x"), today()),
        Err(PlanError::UnknownAnnotation(aid(9)))
    );
}

// ---------- Received plans (S03 §13) ----------

#[test]
fn a_valid_plan_has_no_incoherence() {
    let (mut plan, p) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("tomato", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    )
    .unwrap();
    assert!(plan.check_structure().is_ok());
    assert!(plan.incoherences().is_empty());
}

#[test]
fn an_orphan_planting_is_kept_and_listed() {
    let (mut plan, p) = plan_with_bed();
    plan.delete_bed(p, 2026, today()).unwrap();
    plan.plantings.insert(
        planting_id(1),
        planting("tomato", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    );
    plan.plantings.insert(
        planting_id(2),
        planting("tomato", bed_id(77), 1, 3, Mode::Plants, d(2026, 5, 20)),
    );
    let list = plan.incoherences();
    assert!(list.iter().any(|i| matches!(i,
        Incoherence::OrphanPlanting { planting_id: s, year: Some(2026), .. } if *s == planting_id(1))));
    assert!(list.iter().any(|i| matches!(i,
        Incoherence::OrphanPlanting { planting_id: s, year: None, .. } if *s == planting_id(2))));
    assert_eq!(plan.plantings.len(), 2, "nothing is deleted");
}

#[test]
fn overlapping_plantings_and_zones_out_of_the_bed_are_listed() {
    let (mut plan, p) = plan_with_bed();
    plan.plantings.insert(
        planting_id(1),
        planting("tomato", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    );
    plan.plantings.insert(
        planting_id(2),
        planting("tomato", p, 3, 4, Mode::Plants, d(2026, 5, 25)),
    );
    plan.plantings.insert(
        planting_id(3),
        planting("lettuce", p, 5, 9, Mode::Plants, d(2026, 4, 1)),
    );
    let list = plan.incoherences();
    let conflict = list
        .iter()
        .find(|i| matches!(i, Incoherence::Conflict { .. }))
        .unwrap();
    assert_eq!(conflict.code(), "overlap");
    assert!(list.iter().any(|i| matches!(i,
        Incoherence::ZoneOutOfBed { planting_id: s, segments: 6, .. } if *s == planting_id(3))));
}

#[test]
fn more_than_ten_beds_in_a_year_are_listed() {
    let (mut plan, _) = plan_with_bed();
    for n in 0..10u32 {
        plan.beds.insert(
            bed_id(200 + u128::from(n)),
            plan::Bed {
                name: format!("P{n}"),
                layouts: [(2026, Layout::Present(geometry(2)))].into(),
            },
        );
    }
    let list = plan.incoherences();
    assert!(list.iter().any(|i| matches!(
        i,
        Incoherence::TooManyBeds {
            year: 2026,
            count: 11
        }
    )));
    // They all sit at the same place: placement is reported too.
    assert!(list.iter().any(|i| matches!(i, Incoherence::Placement(_))));
}

#[test]
fn placement_rules_apply_to_a_received_plan() {
    let (mut plan, p) = plan_with_bed();
    let mut g = geometry(2);
    g.x = 1950; // sticks out of the 20 m plot
    plan.beds
        .get_mut(&p)
        .unwrap()
        .layouts
        .insert(2027, Layout::Present(g));
    assert!(plan.incoherences().iter().any(|i| matches!(
        i,
        Incoherence::Placement(PlanError::OutsidePlot { year: 2027, .. })
    )));
}

#[test]
fn bad_annotations_and_unknown_crops_are_listed() {
    let (mut plan, p) = plan_with_bed();
    plan.annotations
        .insert(aid(1), note(d(2026, 12, 1), Some(bed_id(5)), "x"));
    plan.annotations
        .insert(aid(2), note(d(2026, 12, 1), Some(p), ""));
    let mut s = planting("tomato", p, 1, 1, Mode::Plants, d(2026, 5, 20));
    s.crop_id = "mangue".into();
    plan.plantings.insert(planting_id(1), s);
    let list = plan.incoherences();
    let codes: Vec<_> = list.iter().map(Incoherence::code).collect();
    assert!(codes.contains(&"note_bed_unknown"));
    assert!(codes.contains(&"note_invalid"));
    assert!(codes.contains(&"crop_unknown"));
}

#[test]
fn a_broken_structure_is_refused() {
    let (mut plan, p) = plan_with_bed();
    plan.beds
        .get_mut(&p)
        .unwrap()
        .layouts
        .insert(2026, Layout::Present(geometry(0)));
    assert!(matches!(
        plan.check_structure(),
        Err(PlanError::InvalidLength { .. })
    ));

    let (mut plan, p) = plan_with_bed();
    let mut s = planting("tomato", p, 1, 2, Mode::Plants, d(2026, 5, 20));
    s.segment_start = 5;
    s.segment_end = 2;
    plan.plantings.insert(planting_id(1), s);
    assert!(matches!(
        plan.check_structure(),
        Err(PlanError::InvalidZone { .. })
    ));

    let (mut plan, _) = plan_with_bed();
    plan.plot.aisle_cm = 7;
    assert!(matches!(
        plan.check_structure(),
        Err(PlanError::InvalidAisle(7))
    ));
}

// ---------- Clear le plan ----------

#[test]
fn clearing_removes_beds_plantings_and_their_notes_from_the_year_on() {
    let (mut plan, p) = plan_with_bed();
    plan.add_planting(
        planting_id(1),
        planting("tomato", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    )
    .unwrap();
    plan.add_annotation(aid(1), 2026, note(d(2026, 6, 1), Some(p), "bed"), today())
        .unwrap();
    plan.add_annotation(aid(2), 2026, note(d(2026, 6, 1), None, "garden"), today())
        .unwrap();

    let cleared = plan.clear_from(2026, today()).unwrap();
    assert_eq!(cleared.beds, vec![p]);
    assert_eq!(cleared.plantings, vec![planting_id(1)]);
    assert_eq!(cleared.annotations, vec![aid(1)]);
    assert!(plan.beds_in(2026).is_empty());
    assert!(plan.plantings.is_empty());
    assert!(
        plan.annotations.contains_key(&aid(2)),
        "whole-garden notes stay"
    );
    assert!(plan.incoherences().is_empty());
    // Nothing to clear is not an error.
    assert_eq!(plan.clear_from(2026, today()).unwrap(), Default::default());
}

#[test]
fn clearing_a_past_year_is_refused() {
    let (mut plan, _) = plan_with_bed();
    assert!(matches!(
        plan.clear_from(2025, today()),
        Err(PlanError::PastYear { .. })
    ));
    assert_eq!(plan.beds_in(2026).len(), 1);
}
