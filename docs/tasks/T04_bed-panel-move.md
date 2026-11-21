# T04 · Bed panel: "Move" action

- Status: Done
- Owner: agent
- Depends on: —
- Refs: S01 F-VIS-7 · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-VIS-7 · lot1-coverage.

## Goal

"Move" in the bed panel opens the plan editor with this bed already selected.

## Acceptance

- [x] From the Garden tab, "Move" switches to Plan with the bed selected (handles visible), in the cursor's year.
- [x] Absent or disabled in a read-only past year, with the reason.
- [x] `UC-xx` e2e test and matrix row; messages in en and fr.
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/`
- `front/messages/`
- `front/e2e/`
- `docs/POT-00_*`

## Questions

## Log

- 2026-11-21 · Done, v0.7.0 (minor: new visible action). No spec change: S01 v1.7 F-VIS-7 already lists "move (shortcut to the plan editor, bed already selected)".
  - Front: the bed panel opened from the garden has a "Move" button (`PanelBed` `onmove`, given by `PanelDiorama` only; the editor's own panel does not show it, the bed already moves by dragging there). `app.moveBed(id)` switches to the Plan tab without touching the year (the timeline keeps `app.year` on the cursor's year), and the editor takes the bed once (`app.takeBedToSelect()`) as its initial selection. In a past season the button is disabled, described by "Past season: the bed cannot be moved any more." (`panel_move`, `panel_move_past`, en and fr).
  - Tests: unit `state.test.ts` (tab, year kept, bed given once); e2e **UC-32** (T03 took UC-29 and UC-31 on main meanwhile, renumbered at rebase): Plan tab current, same year, Roots pressed with handles and its panel, no "Move" in the editor, disabled with its description in the previous year. Matrix and lot 1 coverage updated (F-VIS-7 stays Partial for the families reminder, after lot 1).
  - Checked: full local gate (fmt, clippy, cargo test, check, 215 unit tests, build, 319 e2e in Chromium and WebKit) before and after the rebase on T03; screenshots in WebKit at 1440 and 360 px, en and fr (button in the garden panel, plan opened on the bed).
  - Not checked: real Tauri window, touch, installed build; `site/` not updated (outside Scope; a small panel button).
