//! Placement rules of the atelier (S03 §15).

mod common;

use common::*;
use plan::{CaseCm, Disposition, Geometry, Plan, PlanError, PlanId, Terrain, TypePlanche};
use uuid::Uuid;

fn at(x: u32, y: u32, longueur_cm: u32, largeur_cm: u32) -> Geometry {
    Geometry {
        planche_type: TypePlanche::Planche,
        x,
        y,
        longueur_cm,
        largeur_cm,
    }
}

/// An empty plan on a 20 × 10 m terrain, 50 cm cases and allées.
fn empty() -> Plan {
    Plan::new(PlanId::from_uuid(Uuid::from_u128(1)), terrain(), now())
}

fn put(plan: &mut Plan, n: u128, year: i32, g: Geometry) {
    plan.add_planche(planche_id(n), format!("P{n}"), year, g, today())
        .unwrap();
}

#[test]
fn terrain_dimensions_and_allee_are_checked() {
    let ok = |longueur_cm, largeur_cm, case_cm, allee_cm| {
        Terrain {
            longueur_cm,
            largeur_cm,
            case_cm,
            allee_cm,
        }
        .validate()
    };
    assert!(ok(1100, 650, CaseCm::C50, 50).is_ok());
    assert!(ok(100, 10_000, CaseCm::C100, 30).is_ok());
    assert!(matches!(
        ok(99, 650, CaseCm::C50, 50),
        Err(PlanError::InvalidTerrain { .. })
    ));
    assert!(matches!(
        ok(1100, 10_001, CaseCm::C50, 50),
        Err(PlanError::InvalidTerrain { .. })
    ));
    assert!(matches!(
        ok(1100, 0, CaseCm::C50, 50),
        Err(PlanError::InvalidTerrain { .. })
    ));
    assert_eq!(
        ok(1100, 650, CaseCm::C50, 40),
        Err(PlanError::InvalidAllee(40))
    );
}

#[test]
fn a_planche_may_touch_the_edges_of_the_terrain() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 2000, 1000));
    assert_eq!(plan.check_placements(2026), Ok(()));
}

#[test]
fn a_planche_outside_the_terrain_is_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(1750, 0, 300, 120));
    assert_eq!(
        plan.check_placements(2026),
        Err(PlanError::OutsideTerrain {
            planche_id: planche_id(1),
            year: 2026
        })
    );
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 900, 300, 120));
    assert!(matches!(
        plan.check_placements(2026),
        Err(PlanError::OutsideTerrain { .. })
    ));
}

#[test]
fn overlapping_planches_are_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(50, 50, 300, 120));
    put(&mut plan, 2, 2026, at(300, 100, 300, 120));
    assert!(matches!(
        plan.check_placements(2026),
        Err(PlanError::PlanchesOverlap { year: 2026, .. })
    ));
}

#[test]
fn the_allee_width_is_kept_between_planches() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 300, 120));
    // 49 cm to the right: too close; exactly 50 cm: fine.
    put(&mut plan, 2, 2026, at(349, 0, 300, 120));
    assert!(matches!(
        plan.check_placements(2026),
        Err(PlanError::AlleeTooNarrow {
            allee_cm: 50,
            year: 2026,
            ..
        })
    ));
    plan.set_disposition(
        planche_id(2),
        2026,
        Disposition::Present(at(350, 0, 300, 120)),
        today(),
    )
    .unwrap();
    assert_eq!(plan.check_placements(2026), Ok(()));
    // Below, 49 cm: too close; diagonal corners far enough on one axis: fine.
    plan.set_disposition(
        planche_id(2),
        2026,
        Disposition::Present(at(0, 169, 300, 120)),
        today(),
    )
    .unwrap();
    assert!(matches!(
        plan.check_placements(2026),
        Err(PlanError::AlleeTooNarrow { .. })
    ));
    plan.set_disposition(
        planche_id(2),
        2026,
        Disposition::Present(at(350, 100, 300, 120)),
        today(),
    )
    .unwrap();
    assert_eq!(plan.check_placements(2026), Ok(()));
}

#[test]
fn a_move_is_checked_in_the_following_years_too() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 300, 120));
    put(&mut plan, 2, 2026, at(400, 0, 300, 120));
    // Planche 2 moves in 2028 onto planche 1: refused for 2028, not for 2027.
    plan.set_disposition(
        planche_id(2),
        2028,
        Disposition::Present(at(100, 0, 300, 120)),
        today(),
    )
    .unwrap();
    assert!(plan.check_placements(2026).is_err());
    // Moving planche 1 in 2027 collides with the 2028 layout of planche 2.
    let mut other = empty();
    put(&mut other, 1, 2026, at(0, 0, 300, 120));
    put(&mut other, 2, 2028, at(400, 0, 300, 120));
    other
        .set_disposition(
            planche_id(1),
            2027,
            Disposition::Present(at(200, 0, 300, 120)),
            today(),
        )
        .unwrap();
    assert!(matches!(
        other.check_placements(2027),
        Err(PlanError::PlanchesOverlap { year: 2028, .. })
    ));
}

#[test]
fn a_width_under_10_cm_is_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 300, 9));
    assert_eq!(plan.check_placements(2026), Err(PlanError::InvalidWidth(9)));
}

#[test]
fn names_have_1_to_30_characters() {
    assert!(plan::check_name("Les tomates").is_ok());
    assert!(plan::check_name(&"é".repeat(30)).is_ok());
    assert_eq!(plan::check_name("  "), Err(PlanError::InvalidName));
    assert_eq!(
        plan::check_name(&"é".repeat(31)),
        Err(PlanError::InvalidName)
    );
}

#[test]
fn a_terrain_that_no_longer_holds_its_planches_is_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(1000, 0, 300, 120));
    let before = plan.clone();
    assert!(matches!(
        plan.set_terrain(1200, 1000, 50, today()),
        Err(PlanError::OutsideTerrain { .. })
    ));
    assert_eq!(plan, before);
    assert!(matches!(
        plan.set_terrain(2000, 1000, 40, today()),
        Err(PlanError::InvalidAllee(40))
    ));
    plan.set_terrain(1300, 800, 80, today()).unwrap();
    assert_eq!(plan.terrain.allee_cm, 80);
    assert_eq!(plan.terrain.case_cm, CaseCm::C50);
}

#[test]
fn a_larger_allee_can_make_the_plan_invalid() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 300, 120));
    put(&mut plan, 2, 2026, at(360, 0, 300, 120));
    assert!(matches!(
        plan.set_terrain(2000, 1000, 80, today()),
        Err(PlanError::AlleeTooNarrow { allee_cm: 80, .. })
    ));
}
