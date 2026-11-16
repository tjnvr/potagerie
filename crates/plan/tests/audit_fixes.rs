//! Cases found by the audit of the specs against the code (November 2026).

mod common;

use common::*;
use plan::{Incoherence, Layout, PlanError};

#[test]
fn a_plot_that_breaks_a_past_layout_is_refused() {
    // The bed is at the origin, 3 m long. In 2027 the 2026 layout is past: it
    // cannot be repaired, so the plot may not shrink below it.
    let (mut plan, _) = plan_with_bed();
    let before = plan.plot.clone();
    let result = plan.set_plot(200, 1000, 50, d(2027, 6, 1));
    assert!(
        matches!(result, Err(PlanError::OutsidePlot { .. })),
        "{result:?}"
    );
    assert_eq!(plan.plot, before, "the plan is untouched");
    // A plot that still holds it is accepted.
    plan.set_plot(1500, 800, 50, d(2027, 6, 1)).unwrap();
}

#[test]
fn clearing_the_plan_also_removes_the_beds_created_for_a_later_year() {
    let (mut plan, first) = plan_with_bed();
    let later = bed_id(200);
    plan.add_bed(later, "Later".into(), 2028, geometry(4), today())
        .unwrap();
    assert!(plan.beds_in(2026).iter().all(|(id, _, _)| *id == first));
    let cleared = plan.clear_from(2026, today()).unwrap();
    assert!(cleared.beds.contains(&first) && cleared.beds.contains(&later));
    for year in 2026..=2030 {
        assert!(plan.beds_in(year).is_empty(), "{year}");
    }
}

#[test]
fn the_to_resolve_list_shows_every_placement_problem() {
    let (mut plan, a) = plan_with_bed();
    let b = bed_id(300);
    let c = bed_id(400);
    // Two beds on the same spot, and a third one outside the plot (a file can
    // hold what the operations refuse).
    for (id, name, x) in [(b, "B", 0), (c, "C", 5000)] {
        plan.beds.insert(
            id,
            plan::Bed {
                name: name.into(),
                layouts: [(2026, Layout::Present(plan::Geometry { x, ..geometry(6) }))].into(),
            },
        );
    }
    let placements = plan
        .incoherences()
        .into_iter()
        .filter(|i| matches!(i, Incoherence::Placement(_)))
        .count();
    assert!(placements >= 2, "{placements} placement problems for {a:?}");
}

#[test]
fn deleting_or_clearing_in_an_absurd_year_is_refused() {
    let (mut plan, bed) = plan_with_bed();
    let far = 2_000_000_000;
    let today = d(2027, 1, 1);
    assert!(matches!(
        plan.delete_bed(bed, far, today),
        Err(PlanError::OutOfRange { .. })
    ));
    assert!(matches!(
        plan.clear_from(far, today),
        Err(PlanError::OutOfRange { .. })
    ));
    // Nothing was stored, so the later operations still run at once.
    plan.incoherences();
}
