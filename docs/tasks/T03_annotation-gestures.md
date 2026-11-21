# T03 · Annotations: create on a row, move by dragging

- Status: Done
- Owner: agent
- Depends on: —
- Refs: S01 F-VIS-5 · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-VIS-5 · lot1-coverage.

## Goal

A note is created by a double click (computer) or a long press (phone) on a timeline row at a date, and an annotation is moved by dragging it.

## Acceptance

- [x] Double click on a row opens note creation for that bed and that date; same with a long press on touch.
- [x] Dragging a note or a period changes its dates (and its bed for a note when dropped on another row); refusals go back with the reason.
- [x] `UC-xx` e2e tests and matrix rows; Rust tests if a command changes.
- [x] `POT-00_lot1-coverage.md`: F-VIS-5 → Done.
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/`
- `crates/plan/src/annotations.rs`
- `crates/plan/tests/`
- `src-tauri/src/`
- `front/messages/`
- `front/e2e/`
- `docs/POT-00_*`

## Questions

## Log

- 2026-11-21 · Done, v0.6.0 (minor: new visible behaviour). No spec change: S01 v1.7 F-VIS-5 already asks for double click / long press creation and drag; the refusals are the existing S03 v0.17 §16 rules of `modify_annotation` (no Rust or command change, so no new Rust test).
  - Changed: `Timeline.svelte`: a double click on an empty point of a row (not a bar, slot or note) opens "New note" on that bed and day; a touch long press does the same (a finger that moves first scrolls). A note or a period is dragged with the bar drag controller, in whole days, drawn live where it would land; a note of a bed follows the row it is dropped on, a period or a whole-garden note keeps its bed; a click still opens it. A refusal shows "Move refused. {reason} The annotation goes back to its place." Helpers `movedInput` / `isMove` (`note.ts`) and `rowAt` (`geometry.ts`). The `?example` fake now refuses, like Rust, an annotation that would end before the year of today. Timeline help line and both messages in `en.json` and `fr.json`.
  - Checked: full local gate green after the rebase on T02 (fmt, clippy, cargo test, front check / test 214 / build, e2e 317 passed, 3 skipped = WebKit frame timings and the touch test outside Chromium). Unit `note.test.ts` (moved input across the year, bed rules, no-op drag), `geometry.test.ts` (row under the pointer). e2e UC-29 (double click; long press through Chromium CDP touch events) and UC-31 (note dragged to another row and date; period dragged into 2026 refused and back), in Chromium and WebKit. Matrix rows UC-29 and UC-31 (UC-28 was taken by T02 meanwhile), F-VIS-5 Done in `POT-00_lot1-coverage.md`.
  - NOT checked: a long press and a drag on a real phone, the real Tauri window, the installed build; no 1440 / 360 px screenshots in English and French (the layout tests at 360 / 390 / 1440 px pass with the longer help line).
