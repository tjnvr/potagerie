# T08 · Undo / Redo

- Status: Running
- Owner: agent
- Depends on: —
- Refs: S01 F-PLN-6 (after lot 1)
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-PLN-6 (after lot 1).

## Goal

Each action of the editor can be undone and redone from the header.

## Acceptance

- [x] Undo and Redo buttons in the header of the Plan tab, greyed out when there is nothing to undo or redo (D01 E03).
- [x] Each change to the beds is one step: add (from the tray, a click or a drawn zone), move, resize, rename, change of type, delete (the bed comes back with its plantings). Undone and redone in order; a new change empties Redo.
- [x] Any other change (planting, note, settings, import, clear, erase) or a new calendar year empties the history: undoing never takes back what was done elsewhere.
- [x] Shortcuts: Cmd+Z / Ctrl+Z undoes, Shift+Cmd+Z / Ctrl+Y redoes, in the plan editor, ignored while typing in a field (Tim, 2026-11-21, see T25).
- [x] Spec first: S01 brings Undo / Redo into lot 1, S05 states the commands and the history; matrix rows UC-36, UC-37; coverage updated.
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `docs/specs/` (S01, S05), `docs/POT-00_*`
- `crates/plan/` (`history.rs`, `view.rs`, `tests/history.rs`)
- `src-tauri/` (`commands.rs`, `error.rs`, `lib.rs`, `command_names.rs`, `capabilities/main.json`, `tests.rs`)
- `front/src/lib/` (`state.svelte.ts`, `shell/Header.svelte`, `shell/Icon.svelte`, `editor/Editor.svelte`, `editor/shortcuts.ts`, `simulation.ts`, `errors.ts`, `bindings.ts`), `front/messages/`, `front/e2e/`

## Questions

1. [functional] Build Undo / Redo now? S01 lists it for after lot 1, and it is the largest of these.
   (a) Yes, for the plan editor only (beds). (b) Yes, for every change (beds, plantings, notes). (c) Later.
   Recommendation: (a), after T07.
   Answer: a

## Log

- 2026-11-21 (chat): keyboard shortcuts added to the acceptance, at Tim's request; T24 and T25 depend on this task. No longer waits for T07 (rotation postponed).
- 2026-11-22: Started (answer a). Specs S01 v1.10 (F-PLN-6, out of the "after lot 1" list) and S05 v0.22 (§5, §7, §18) committed.
- 2026-11-22: Tests written and passing: Rust `crates/plan/tests/history.rs` (9 cases), app `the_changes_of_the_editor_are_undone_and_redone`, `undoing_a_deletion_brings_the_bed_back_with_its_plantings`, `other_changes_empty_the_history`, `a_drawn_bed_is_one_step`; unit `shortcuts.test.ts`; e2e UC-36, UC-37 (Chromium, WebKit). Extra paths outside the first scope: `README.md`, the D01 / D02 READMEs and S03 (spec links), `timeline/geometry.test.ts` (fixture).

### Decisions

- History on the Rust side, of whole documents (`plan::History`), not inverse operations: an undo puts back exactly what the change touched (a deleted bed with its plantings) and copies no rule. It lives in memory in `AppState`, never saved: it starts empty at each launch.
- Scope (a) is enforced by emptying the history on every other write (plot, plantings, notes, import, clear, erase), plus a `version` check against the database: another instance of the app writing in between empties it too. So an undo never reverts a planting or a note.
- Also emptied when the calendar year changes, so an undo never rewrites a year that has become past.
- An undo is a normal save (`version + 1`, `version_stale` check); a failed save (`saving`) leaves the history untouched, so "Retry" undoes the same step.
- A bed drawn on the grass used to be two commands (create on a free spot, then resize): `create_bed` gains an optional `size` so it is one command and one step.
- `can_undo` / `can_redo` travel in the year view (false from `YearView::build`, filled by the app), so the buttons follow every command without an extra call.
- 50 steps at most; `history_empty` (translated) when there is nothing to undo or redo; the interface stays silent about it for a shortcut pressed with nothing left.
- Shortcuts accept Cmd or Ctrl on every platform (T25 may narrow it to the platform's key); ignored in inputs, text areas, selects, editable elements, and while a window (settings, catalogue, forms) or a question (shortening, aisle bubble) is open. The buttons show in the Plan tab only, as on D01 E03, between the season controls and the save indicator; no shortcut in tooltips yet (T25).
- The buttons also work on a past season shown read-only: the steps undone were made on the current or a future year.
