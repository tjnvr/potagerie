//! Placement rules of the editor (S03 §15).

mod common;

use common::*;
use plan::{CellCm, Geometry, Layout, Plan, PlanError, PlanId, Plot, TypeBed};
use uuid::Uuid;

fn at(x: u32, y: u32, length_cm: u32, width_cm: u32) -> Geometry {
    Geometry {
        bed_type: TypeBed::Bed,
        x,
        y,
        length_cm,
        width_cm,
    }
}

/// An empty plan on a 20 × 10 m plot, 50 cm cases and aisles.
fn empty() -> Plan {
    Plan::new(PlanId::from_uuid(Uuid::from_u128(1)), plot(), now())
}

fn put(plan: &mut Plan, n: u128, year: i32, g: Geometry) {
    plan.add_bed(bed_id(n), format!("P{n}"), year, g, today())
        .unwrap();
}

#[test]
fn plot_dimensions_and_aisle_are_checked() {
    let ok = |length_cm, width_cm, cell_cm, aisle_cm| {
        Plot {
            length_cm,
            width_cm,
            cell_cm,
            aisle_cm,
        }
        .validate()
    };
    assert!(ok(1100, 650, CellCm::C50, 50).is_ok());
    assert!(ok(100, 10_000, CellCm::C100, 30).is_ok());
    assert!(matches!(
        ok(99, 650, CellCm::C50, 50),
        Err(PlanError::InvalidPlot { .. })
    ));
    assert!(matches!(
        ok(1100, 10_001, CellCm::C50, 50),
        Err(PlanError::InvalidPlot { .. })
    ));
    assert!(matches!(
        ok(1100, 0, CellCm::C50, 50),
        Err(PlanError::InvalidPlot { .. })
    ));
    assert_eq!(
        ok(1100, 650, CellCm::C50, 40),
        Err(PlanError::InvalidAisle(40))
    );
}

#[test]
fn a_bed_may_touch_the_edges_of_the_plot() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 2000, 1000));
    assert_eq!(plan.check_placements(2026), Ok(()));
}

#[test]
fn a_bed_outside_the_plot_is_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(1750, 0, 300, 120));
    assert_eq!(
        plan.check_placements(2026),
        Err(PlanError::OutsidePlot {
            bed_id: bed_id(1),
            year: 2026
        })
    );
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 900, 300, 120));
    assert!(matches!(
        plan.check_placements(2026),
        Err(PlanError::OutsidePlot { .. })
    ));
}

#[test]
fn overlapping_beds_are_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(50, 50, 300, 120));
    put(&mut plan, 2, 2026, at(300, 100, 300, 120));
    assert!(matches!(
        plan.check_placements(2026),
        Err(PlanError::BedsOverlap { year: 2026, .. })
    ));
}

#[test]
fn the_aisle_width_is_asked_for_between_beds() {
    let mut start = empty();
    put(&mut start, 1, 2026, at(0, 0, 300, 120));
    // Bed 2 placed at `g`, as the commands check it: against the plan before.
    let asked = |g: Geometry| {
        let mut plan = start.clone();
        put(&mut plan, 2, 2026, g);
        plan.check_placement_change(&start, 2026, false)
    };
    // 49 cm to the right: too close; exactly 50 cm: fine.
    assert!(matches!(
        asked(at(349, 0, 300, 120)),
        Err(PlanError::AisleTooNarrow {
            aisle_cm: 50,
            year: 2026,
            ..
        })
    ));
    assert_eq!(asked(at(350, 0, 300, 120)), Ok(()));
    // Below, 49 cm: too close; diagonal corners far enough on one axis: fine.
    assert!(matches!(
        asked(at(0, 169, 300, 120)),
        Err(PlanError::AisleTooNarrow { .. })
    ));
    assert_eq!(asked(at(350, 100, 300, 120)), Ok(()));
    // Too close is not a placement error: only the change asks about it.
    let mut close = start.clone();
    put(&mut close, 2, 2026, at(349, 0, 300, 120));
    assert_eq!(close.check_placements(2026), Ok(()));
    assert!(close.placement_errors(2026).is_empty());
}

#[test]
fn a_pair_made_too_close_in_a_following_year_is_asked_for() {
    let mut start = empty();
    put(&mut start, 1, 2026, at(0, 0, 300, 120));
    put(&mut start, 2, 2028, at(400, 0, 300, 120));
    // Bed 1 moved in 2027, 40 cm from bed 2 of 2028: too close in 2028 only.
    let mut plan = start.clone();
    plan.set_layout(
        bed_id(1),
        2027,
        Layout::Present(at(60, 0, 300, 120)),
        today(),
    )
    .unwrap();
    assert!(matches!(
        plan.check_placement_change(&start, 2027, false),
        Err(PlanError::AisleTooNarrow { year: 2028, .. })
    ));
    assert_eq!(plan.check_placement_change(&start, 2027, true), Ok(()));
}

#[test]
fn keep_close_never_lets_an_overlap_or_a_bed_outside_the_plot_through() {
    let mut start = empty();
    put(&mut start, 1, 2026, at(0, 0, 300, 120));
    let mut over = start.clone();
    put(&mut over, 2, 2026, at(250, 0, 300, 120));
    assert!(matches!(
        over.check_placement_change(&start, 2026, true),
        Err(PlanError::BedsOverlap { .. })
    ));
    let mut outside = start.clone();
    put(&mut outside, 2, 2026, at(1800, 0, 300, 120));
    assert!(matches!(
        outside.check_placement_change(&start, 2026, true),
        Err(PlanError::OutsidePlot { .. })
    ));
}

#[test]
fn a_move_is_checked_in_the_following_years_too() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 300, 120));
    put(&mut plan, 2, 2026, at(400, 0, 300, 120));
    // Bed 2 moves in 2028 onto bed 1: refused for 2028, not for 2027.
    plan.set_layout(
        bed_id(2),
        2028,
        Layout::Present(at(100, 0, 300, 120)),
        today(),
    )
    .unwrap();
    assert!(plan.check_placements(2026).is_err());
    // Moving bed 1 in 2027 collides with the 2028 layout of bed 2.
    let mut other = empty();
    put(&mut other, 1, 2026, at(0, 0, 300, 120));
    put(&mut other, 2, 2028, at(400, 0, 300, 120));
    other
        .set_layout(
            bed_id(1),
            2027,
            Layout::Present(at(200, 0, 300, 120)),
            today(),
        )
        .unwrap();
    assert!(matches!(
        other.check_placements(2027),
        Err(PlanError::BedsOverlap { year: 2028, .. })
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
    assert!(plan::check_name("The tomatoes").is_ok());
    assert!(plan::check_name(&"é".repeat(30)).is_ok());
    assert_eq!(plan::check_name("  "), Err(PlanError::InvalidName));
    assert_eq!(
        plan::check_name(&"é".repeat(31)),
        Err(PlanError::InvalidName)
    );
}

#[test]
fn a_plot_that_no_longer_holds_its_beds_is_refused() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(1000, 0, 300, 120));
    let before = plan.clone();
    assert!(matches!(
        plan.set_plot(1200, 1000, 50, today()),
        Err(PlanError::OutsidePlot { .. })
    ));
    assert_eq!(plan, before);
    assert!(matches!(
        plan.set_plot(2000, 1000, 40, today()),
        Err(PlanError::InvalidAisle(40))
    ));
    plan.set_plot(1300, 800, 80, today()).unwrap();
    assert_eq!(plan.plot.aisle_cm, 80);
    assert_eq!(plan.plot.cell_cm, CellCm::C50);
}

/// A copy off the grid, next to a bed that is not on it either, and the
/// largest plot read cell by cell when it holds no spot (S03 §15).
#[test]
fn a_copy_off_the_grid_and_a_full_large_plot() {
    let mut plan = empty();
    put(&mut plan, 1, 2027, at(13, 7, 300, 95));
    let g = plan
        .paste_bed(bed_id(2), "C".into(), 2027, &at(13, 7, 300, 95), today())
        .unwrap();
    assert_eq!((g.x, g.y, g.length_cm, g.width_cm), (363, 7, 300, 95));

    // Left of the original fits when right and below do not.
    let mut plan = empty();
    put(&mut plan, 1, 2027, at(1700, 880, 300, 120));
    let g = plan
        .paste_bed(
            bed_id(2),
            "C".into(),
            2027,
            &at(1700, 880, 300, 120),
            today(),
        )
        .unwrap();
    assert_eq!((g.x, g.y), (1350, 880));

    let mut plan = Plan::new(
        PlanId::from_uuid(Uuid::from_u128(1)),
        Plot {
            length_cm: 10_000,
            width_cm: 10_000,
            cell_cm: CellCm::C25,
            aisle_cm: 30,
        },
        now(),
    );
    put(&mut plan, 1, 2027, at(0, 0, 10_000, 9_900));
    // In 2027 the strip under the first bed holds the copy, but a bed comes
    // there in 2030 and the copy would still be on the plan then.
    put(&mut plan, 2, 2030, at(0, 9_930, 10_000, 70));
    let before = plan.clone();
    assert_eq!(
        plan.paste_bed(bed_id(3), "C".into(), 2027, &at(0, 9_930, 100, 70), today()),
        Err(PlanError::NoFreeSpot { year: 2027 })
    );
    assert_eq!(plan, before);
}

#[test]
fn a_larger_aisle_can_make_the_plan_invalid() {
    let mut plan = empty();
    put(&mut plan, 1, 2026, at(0, 0, 300, 120));
    put(&mut plan, 2, 2026, at(360, 0, 300, 120));
    assert!(matches!(
        plan.set_plot(2000, 1000, 80, today()),
        Err(PlanError::AisleTooNarrow { aisle_cm: 80, .. })
    ));
}
