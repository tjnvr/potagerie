//! Undo / Redo of the plan editor (S01 F-PLN-6, S05 §18).
//!
//! The history keeps the previous documents, not inverse operations: an undo
//! puts back exactly what a change touched and adds no rule. It lives in
//! memory, next to the store, and is never saved.

use chrono::{Datelike, NaiveDate};

use crate::model::Plan;

/// Steps kept at most: the oldest one goes.
pub const MAX_UNDO_STEPS: usize = 50;

/// The documents before the changes of the editor (Undo) and after the undone
/// ones (Redo).
#[derive(Debug, Clone, Default)]
pub struct History {
    undo: Vec<Plan>,
    redo: Vec<Plan>,
    /// The stored `version` after the last write the history knows, and the
    /// calendar year of that day. A store at another version was written
    /// elsewhere; a new year may have made the changed years past.
    written: Option<(u64, i32)>,
}

enum Direction {
    Undo,
    Redo,
}

impl History {
    /// Whether the history still describes the stored plan at `version`.
    fn matches(&self, version: u64, today: NaiveDate) -> bool {
        self.written == Some((version, today.year()))
    }

    /// Records a change of the editor: `before` is the stored document it
    /// started from, `version` the stored one once it is saved. Empties Redo.
    pub fn record(&mut self, before: Plan, version: u64, today: NaiveDate) {
        if !self.matches(before.version, today) {
            self.clear();
        }
        self.undo.push(before);
        if self.undo.len() > MAX_UNDO_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.written = Some((version, today.year()));
    }

    /// Forgets everything: a change made outside the editor, an import.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.written = None;
    }

    /// A step waits to be undone on the stored plan at `version`.
    pub fn can_undo(&self, version: u64, today: NaiveDate) -> bool {
        self.matches(version, today) && !self.undo.is_empty()
    }

    /// A step waits to be redone on the stored plan at `version`.
    pub fn can_redo(&self, version: u64, today: NaiveDate) -> bool {
        self.matches(version, today) && !self.redo.is_empty()
    }

    /// Undoes the last change: `save` writes the document from before it,
    /// carrying the version of `current` (the stored plan). `None` when there
    /// is nothing to undo. A failed save leaves the history as it was.
    pub fn undo<E>(
        &mut self,
        current: Plan,
        today: NaiveDate,
        save: impl FnOnce(&mut Plan) -> Result<(), E>,
    ) -> Result<Option<Plan>, E> {
        self.step(Direction::Undo, current, today, save)
    }

    /// Redoes the last undone change, as [`History::undo`].
    pub fn redo<E>(
        &mut self,
        current: Plan,
        today: NaiveDate,
        save: impl FnOnce(&mut Plan) -> Result<(), E>,
    ) -> Result<Option<Plan>, E> {
        self.step(Direction::Redo, current, today, save)
    }

    fn step<E>(
        &mut self,
        direction: Direction,
        current: Plan,
        today: NaiveDate,
        save: impl FnOnce(&mut Plan) -> Result<(), E>,
    ) -> Result<Option<Plan>, E> {
        if !self.matches(current.version, today) {
            self.clear();
            return Ok(None);
        }
        let (from, to) = match direction {
            Direction::Undo => (&mut self.undo, &mut self.redo),
            Direction::Redo => (&mut self.redo, &mut self.undo),
        };
        let Some(mut plan) = from.last().cloned() else {
            return Ok(None);
        };
        plan.version = current.version;
        save(&mut plan)?;
        from.pop();
        to.push(current);
        self.written = Some((plan.version, today.year()));
        Ok(Some(plan))
    }
}
