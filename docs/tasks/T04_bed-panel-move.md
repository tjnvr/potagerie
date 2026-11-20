# T04 · Bed panel: "Move" action

- Status: Ready
- Owner: agent
- Depends on: —
- Refs: S01 F-VIS-7 · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-VIS-7 · lot1-coverage.

## Goal

"Move" in the bed panel opens the plan editor with this bed already selected.

## Acceptance

- [ ] From the Garden tab, "Move" switches to Plan with the bed selected (handles visible), in the cursor's year.
- [ ] Absent or disabled in a read-only past year, with the reason.
- [ ] `UC-xx` e2e test and matrix row; messages in en and fr.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/`
- `front/messages/`
- `front/e2e/`
- `docs/POT-00_*`

## Questions

## Log
