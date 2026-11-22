//! Undo / Redo of the plan editor (S01 F-PLN-6, S05 §18). The store is
//! played by `Stored`: it checks and bumps `version` like `Store::save`.

mod common;

use common::*;
use plan::{History, Layout, MAX_UNDO_STEPS, Plan};

/// The stored document, saved the way the store does.
struct Stored(Plan);

impl Stored {
    fn save(&mut self, plan: &mut Plan) -> Result<(), String> {
        if plan.version != self.0.version {
            return Err(format!("stale: {} over {}", plan.version, self.0.version));
        }
        plan.touch(now());
        self.0 = plan.clone();
        Ok(())
    }

    /// A change of the editor: `change` on the stored plan, saved, recorded.
    fn edit(&mut self, history: &mut History, change: impl FnOnce(&mut Plan)) {
        let before = self.0.clone();
        let mut plan = before.clone();
        change(&mut plan);
        self.save(&mut plan).unwrap();
        history.record(before, plan.version, today());
    }

    fn undo(&mut self, history: &mut History) -> Option<Plan> {
        let current = self.0.clone();
        history
            .undo(current, today(), |plan| self.save(plan))
            .unwrap()
    }

    fn redo(&mut self, history: &mut History) -> Option<Plan> {
        let current = self.0.clone();
        history
            .redo(current, today(), |plan| self.save(plan))
            .unwrap()
    }

    fn x(&self) -> u32 {
        self.0.beds[&bed_id(100)].layout_in(2026).unwrap().x
    }
}

fn move_to(x: u32) -> impl FnOnce(&mut Plan) {
    move |plan| {
        let mut g = plan.beds[&bed_id(100)].layout_in(2026).unwrap().clone();
        g.x = x;
        plan.set_layout(bed_id(100), 2026, Layout::Present(g), today())
            .unwrap();
    }
}

fn stored() -> Stored {
    let (mut plan, _) = plan_with_bed();
    plan.version = 3;
    Stored(plan)
}

#[test]
fn an_empty_history_has_nothing_to_undo_or_redo() {
    let mut store = stored();
    let mut history = History::default();
    assert!(!history.can_undo(3, today()));
    assert!(!history.can_redo(3, today()));
    assert_eq!(store.undo(&mut history), None);
    assert_eq!(store.redo(&mut history), None);
    assert_eq!(store.0.version, 3, "nothing was saved");
}

#[test]
fn undo_puts_back_the_plan_before_the_change_and_redo_the_one_after() {
    let mut store = stored();
    let mut history = History::default();
    store.edit(&mut history, move_to(100));
    store.edit(&mut history, move_to(200));
    assert_eq!(store.x(), 200);
    assert!(history.can_undo(store.0.version, today()));
    assert!(!history.can_redo(store.0.version, today()));

    let undone = store.undo(&mut history).unwrap();
    assert_eq!(store.x(), 100);
    assert_eq!(undone.version, 6, "an undo is a save: version + 1");
    assert!(history.can_redo(6, today()));

    store.undo(&mut history).unwrap();
    assert_eq!(store.x(), 0);
    assert!(!history.can_undo(store.0.version, today()));
    assert_eq!(store.undo(&mut history), None);

    store.redo(&mut history).unwrap();
    assert_eq!(store.x(), 100);
    store.redo(&mut history).unwrap();
    assert_eq!(store.x(), 200);
    assert_eq!(store.redo(&mut history), None);
    assert!(history.can_undo(store.0.version, today()));
}

#[test]
fn a_deleted_bed_comes_back_with_its_plantings() {
    let mut store = stored();
    let id = bed_id(100);
    store
        .0
        .add_planting(
            planting_id(1),
            planting("lettuce", id, 1, 3, plan::Mode::OpenGround, d(2027, 4, 1)),
        )
        .unwrap();
    let mut history = History::default();
    store.edit(&mut history, |plan| {
        plan.delete_bed(id, 2026, today()).unwrap();
    });
    assert!(store.0.plantings.is_empty());

    store.undo(&mut history).unwrap();
    assert!(store.0.beds.contains_key(&id));
    assert!(store.0.plantings.contains_key(&planting_id(1)));
}

#[test]
fn a_new_change_after_an_undo_empties_redo() {
    let mut store = stored();
    let mut history = History::default();
    store.edit(&mut history, move_to(100));
    store.undo(&mut history).unwrap();
    store.edit(&mut history, move_to(300));
    assert!(!history.can_redo(store.0.version, today()));
    assert_eq!(store.redo(&mut history), None);
    store.undo(&mut history).unwrap();
    assert_eq!(store.x(), 0);
}

#[test]
fn a_change_made_elsewhere_empties_the_history() {
    let mut store = stored();
    let mut history = History::default();
    store.edit(&mut history, move_to(100));
    // A planting, a note, another instance of the app: written without the history.
    let mut other = store.0.clone();
    store.save(&mut other).unwrap();
    assert!(!history.can_undo(store.0.version, today()));
    assert_eq!(store.undo(&mut history), None);
    assert_eq!(store.x(), 100, "the other change is not taken back");

    // Even once the store is back on a version the history knew.
    store.edit(&mut history, move_to(200));
    assert_eq!(store.undo(&mut history).map(|_| store.x()), Some(100));
    assert_eq!(store.undo(&mut history), None);
}

#[test]
fn clear_empties_both_lists() {
    let mut store = stored();
    let mut history = History::default();
    store.edit(&mut history, move_to(100));
    store.edit(&mut history, move_to(200));
    store.undo(&mut history).unwrap();
    history.clear();
    assert!(!history.can_undo(store.0.version, today()));
    assert!(!history.can_redo(store.0.version, today()));
}

#[test]
fn a_new_calendar_year_empties_the_history() {
    let mut store = stored();
    let mut history = History::default();
    store.edit(&mut history, move_to(100));
    let next_year = d(2027, 1, 1);
    assert!(!history.can_undo(store.0.version, next_year));
    let current = store.0.clone();
    let undone = history
        .undo(current, next_year, |plan| store.save(plan))
        .unwrap();
    assert_eq!(undone, None);
    assert_eq!(store.x(), 100, "2026 is past: never changed back");
}

#[test]
fn a_failed_save_keeps_the_history_as_it_was() {
    let mut store = stored();
    let mut history = History::default();
    store.edit(&mut history, move_to(100));
    let current = store.0.clone();
    let failed = history.undo(current, today(), |_| Err::<(), _>("disk full"));
    assert_eq!(failed, Err("disk full"));
    assert!(history.can_undo(store.0.version, today()));
    assert!(!history.can_redo(store.0.version, today()));
    store.undo(&mut history).unwrap();
    assert_eq!(store.x(), 0, "Retry undoes the same step");
}

#[test]
fn the_oldest_steps_go_beyond_the_limit() {
    let mut store = stored();
    let mut history = History::default();
    for i in 1..=MAX_UNDO_STEPS as u32 + 5 {
        store.edit(&mut history, move_to(i * 10));
    }
    let mut undone = 0;
    while store.undo(&mut history).is_some() {
        undone += 1;
    }
    assert_eq!(undone, MAX_UNDO_STEPS);
    assert_eq!(store.x(), 50, "the 5 oldest steps are gone");
}
