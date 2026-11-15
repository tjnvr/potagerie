//! Notes and periods, validation of a received plan, "vider le plan".

mod common;

use chrono::NaiveDate;
use common::*;
use plan::{
    Annotation, AnnotationId, Disposition, Incoherence, Mode, PlanError, PlancheId, TypeAnnotation,
};
use uuid::Uuid;

fn aid(n: u128) -> AnnotationId {
    AnnotationId::from_uuid(Uuid::from_u128(n))
}

fn note(date: NaiveDate, planche: Option<PlancheId>, texte: &str) -> Annotation {
    Annotation {
        type_annotation: TypeAnnotation::Note,
        date_debut: date,
        date_fin: None,
        planche_id: planche,
        texte: texte.into(),
        masquee: false,
    }
}

fn period(a: NaiveDate, b: Option<NaiveDate>) -> Annotation {
    Annotation {
        type_annotation: TypeAnnotation::Period,
        date_fin: b,
        ..note(a, None, "Vacances")
    }
}

#[test]
fn a_note_and_a_period_are_added_edited_hidden_and_deleted() {
    let (mut plan, p) = plan_with_planche();
    plan.add_annotation(
        aid(1),
        2026,
        note(d(2026, 12, 1), Some(p), "  Pailler  "),
        today(),
    )
    .unwrap();
    assert_eq!(plan.annotations[&aid(1)].texte, "Pailler");
    plan.add_annotation(
        aid(2),
        2027,
        period(d(2027, 8, 1), Some(d(2027, 8, 21))),
        today(),
    )
    .unwrap();

    let mut hidden = plan.annotations[&aid(1)].clone();
    hidden.masquee = true;
    plan.replace_annotation(aid(1), 2026, hidden, today())
        .unwrap();
    assert!(plan.annotations[&aid(1)].masquee);

    plan.delete_annotation(aid(2), 2027, today()).unwrap();
    assert!(!plan.annotations.contains_key(&aid(2)));
    assert_eq!(
        plan.delete_annotation(aid(2), 2027, today()),
        Err(PlanError::UnknownAnnotation(aid(2)))
    );
}

#[test]
fn a_note_ignores_an_end_date() {
    let (mut plan, _) = plan_with_planche();
    let mut n = note(d(2026, 12, 1), None, "x");
    n.date_fin = Some(d(2020, 1, 1));
    plan.add_annotation(aid(1), 2026, n, today()).unwrap();
    assert_eq!(plan.annotations[&aid(1)].date_fin, None);
}

#[test]
fn past_years_are_read_only() {
    let (mut plan, _) = plan_with_planche();
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
        .insert(aid(3), note(d(2025, 3, 1), None, "vieux"));
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
fn unknown_planche_bad_dates_and_bad_text_are_refused() {
    let (mut plan, _) = plan_with_planche();
    let ghost = planche_id(999);
    assert_eq!(
        plan.add_annotation(
            aid(1),
            2026,
            note(d(2026, 12, 1), Some(ghost), "x"),
            today()
        ),
        Err(PlanError::UnknownPlanche(ghost))
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
    let (mut plan, _) = plan_with_planche();
    assert_eq!(
        plan.replace_annotation(aid(9), 2026, note(d(2026, 12, 1), None, "x"), today()),
        Err(PlanError::UnknownAnnotation(aid(9)))
    );
}

// ---------- Received plans (S03 §13) ----------

#[test]
fn a_valid_plan_has_no_incoherence() {
    let (mut plan, p) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("tomate", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    )
    .unwrap();
    assert!(plan.check_structure().is_ok());
    assert!(plan.incoherences().is_empty());
}

#[test]
fn an_orphan_serie_is_kept_and_listed() {
    let (mut plan, p) = plan_with_planche();
    plan.delete_planche(p, 2026, today()).unwrap();
    plan.series.insert(
        serie_id(1),
        serie("tomate", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    );
    plan.series.insert(
        serie_id(2),
        serie("tomate", planche_id(77), 1, 3, Mode::Plants, d(2026, 5, 20)),
    );
    let list = plan.incoherences();
    assert!(list.iter().any(|i| matches!(i,
        Incoherence::OrphanSerie { serie_id: s, year: Some(2026), .. } if *s == serie_id(1))));
    assert!(list.iter().any(|i| matches!(i,
        Incoherence::OrphanSerie { serie_id: s, year: None, .. } if *s == serie_id(2))));
    assert_eq!(plan.series.len(), 2, "nothing is deleted");
}

#[test]
fn overlapping_series_and_zones_out_of_the_planche_are_listed() {
    let (mut plan, p) = plan_with_planche();
    plan.series.insert(
        serie_id(1),
        serie("tomate", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    );
    plan.series.insert(
        serie_id(2),
        serie("tomate", p, 3, 4, Mode::Plants, d(2026, 5, 25)),
    );
    plan.series.insert(
        serie_id(3),
        serie("laitue", p, 5, 9, Mode::Plants, d(2026, 4, 1)),
    );
    let list = plan.incoherences();
    let conflict = list
        .iter()
        .find(|i| matches!(i, Incoherence::Conflict { .. }))
        .unwrap();
    assert_eq!(conflict.code(), "chevauchement");
    assert!(list.iter().any(|i| matches!(i,
        Incoherence::ZoneOutOfPlanche { serie_id: s, troncons: 6, .. } if *s == serie_id(3))));
}

#[test]
fn more_than_ten_planches_in_a_year_are_listed() {
    let (mut plan, _) = plan_with_planche();
    for n in 0..10u32 {
        plan.planches.insert(
            planche_id(200 + u128::from(n)),
            plan::Planche {
                nom: format!("P{n}"),
                dispositions: [(2026, Disposition::Present(geometry(2)))].into(),
            },
        );
    }
    let list = plan.incoherences();
    assert!(list.iter().any(|i| matches!(
        i,
        Incoherence::TooManyPlanches {
            year: 2026,
            count: 11
        }
    )));
    // They all sit at the same place: placement is reported too.
    assert!(list.iter().any(|i| matches!(i, Incoherence::Placement(_))));
}

#[test]
fn placement_rules_apply_to_a_received_plan() {
    let (mut plan, p) = plan_with_planche();
    let mut g = geometry(2);
    g.x = 1950; // sticks out of the 20 m terrain
    plan.planches
        .get_mut(&p)
        .unwrap()
        .dispositions
        .insert(2027, Disposition::Present(g));
    assert!(plan.incoherences().iter().any(|i| matches!(
        i,
        Incoherence::Placement(PlanError::OutsideTerrain { year: 2027, .. })
    )));
}

#[test]
fn bad_annotations_and_unknown_crops_are_listed() {
    let (mut plan, p) = plan_with_planche();
    plan.annotations
        .insert(aid(1), note(d(2026, 12, 1), Some(planche_id(5)), "x"));
    plan.annotations
        .insert(aid(2), note(d(2026, 12, 1), Some(p), ""));
    let mut s = serie("tomate", p, 1, 1, Mode::Plants, d(2026, 5, 20));
    s.culture_id = "mangue".into();
    plan.series.insert(serie_id(1), s);
    let list = plan.incoherences();
    let codes: Vec<_> = list.iter().map(Incoherence::code).collect();
    assert!(codes.contains(&"note_planche_inconnue"));
    assert!(codes.contains(&"note_invalide"));
    assert!(codes.contains(&"culture_inconnue"));
}

#[test]
fn a_broken_structure_is_refused() {
    let (mut plan, p) = plan_with_planche();
    plan.planches
        .get_mut(&p)
        .unwrap()
        .dispositions
        .insert(2026, Disposition::Present(geometry(0)));
    assert!(matches!(
        plan.check_structure(),
        Err(PlanError::InvalidLength { .. })
    ));

    let (mut plan, p) = plan_with_planche();
    let mut s = serie("tomate", p, 1, 2, Mode::Plants, d(2026, 5, 20));
    s.troncon_debut = 5;
    s.troncon_fin = 2;
    plan.series.insert(serie_id(1), s);
    assert!(matches!(
        plan.check_structure(),
        Err(PlanError::InvalidZone { .. })
    ));

    let (mut plan, _) = plan_with_planche();
    plan.terrain.allee_cm = 7;
    assert!(matches!(
        plan.check_structure(),
        Err(PlanError::InvalidAllee(7))
    ));
}

// ---------- Vider le plan ----------

#[test]
fn clearing_removes_planches_series_and_their_notes_from_the_year_on() {
    let (mut plan, p) = plan_with_planche();
    plan.add_serie(
        serie_id(1),
        serie("tomate", p, 1, 3, Mode::Plants, d(2026, 5, 20)),
    )
    .unwrap();
    plan.add_annotation(
        aid(1),
        2026,
        note(d(2026, 6, 1), Some(p), "planche"),
        today(),
    )
    .unwrap();
    plan.add_annotation(aid(2), 2026, note(d(2026, 6, 1), None, "potager"), today())
        .unwrap();

    let cleared = plan.clear_from(2026, today()).unwrap();
    assert_eq!(cleared.planches, vec![p]);
    assert_eq!(cleared.series, vec![serie_id(1)]);
    assert_eq!(cleared.annotations, vec![aid(1)]);
    assert!(plan.planches_in(2026).is_empty());
    assert!(plan.series.is_empty());
    assert!(
        plan.annotations.contains_key(&aid(2)),
        "whole-potager notes stay"
    );
    assert!(plan.incoherences().is_empty());
    // Nothing to clear is not an error.
    assert_eq!(plan.clear_from(2026, today()).unwrap(), Default::default());
}

#[test]
fn clearing_a_past_year_is_refused() {
    let (mut plan, _) = plan_with_planche();
    assert!(matches!(
        plan.clear_from(2025, today()),
        Err(PlanError::PastYear { .. })
    ));
    assert_eq!(plan.planches_in(2026).len(), 1);
}
