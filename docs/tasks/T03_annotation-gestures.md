# T03 · Annotations: create on a row, move by dragging

- Status: Ready
- Owner: agent
- Depends on: —
- Refs: S01 F-VIS-5 · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-VIS-5 · lot1-coverage.

## Goal

A note is created by a double click (computer) or a long press (phone) on a timeline row at a date, and an annotation is moved by dragging it.

## Acceptance

- [ ] Double click on a row opens note creation for that bed and that date; same with a long press on touch.
- [ ] Dragging a note or a period changes its dates (and its bed for a note when dropped on another row); refusals go back with the reason.
- [ ] `UC-xx` e2e tests and matrix rows; Rust tests if a command changes.
- [ ] `POT-00_lot1-coverage.md`: F-VIS-5 → Done.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

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
