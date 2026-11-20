# T01 · Aisle bubble "Keep here" / "Go back"

- Status: Running
- Owner: agent
- Depends on: —
- Refs: S01 F-PLN-2 · S03 (plot and placement of the beds) · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-PLN-2 · S03 (plot and placement of the beds) · lot1-coverage.

## Goal

Dropping a bed closer than the minimum aisle offers "Keep here" or "Go back" instead of refusing it with an error.

## Acceptance

- [ ] Dropping or resizing a bed closer than the aisle width to another bed shows the bubble; overlap and leaving the plot are still refused.
- [ ] "Keep here" keeps the position; "Go back" restores the previous one.
- [ ] If S03 forbids a closer bed today, a new S03 version allows it (spec first) with a §12 case.
- [ ] New row and `UC-xx` e2e test in `POT-00_test-matrix.md`; messages in `en.json` and `fr.json`.
- [ ] `POT-00_lot1-coverage.md`: F-PLN-2 → Done.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `crates/plan/`
- `src-tauri/src/`
- `front/src/lib/editor/`
- `front/messages/`
- `front/e2e/`
- `docs/specs/`
- `docs/POT-00_*`

## Questions

## Log
