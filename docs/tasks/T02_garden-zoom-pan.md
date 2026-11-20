# T02 · Zoom and pan the garden view

- Status: Ready
- Owner: agent
- Depends on: —
- Refs: S01 F-VIS-1 · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-VIS-1 · lot1-coverage.

## Goal

The diorama zooms with the wheel or trackpad and the pinch, and pans by dragging the background.

## Acceptance

- [ ] Wheel / trackpad and pinch zoom the diorama, with sensible bounds; no zoom button (S01).
- [ ] Dragging the background pans; dragging still works on beds and the time cursor as before.
- [ ] Season playback keeps its smoothness (P01 criterion, measured as in the review).
- [ ] `UC-xx` e2e test (wheel zoom and background drag) and matrix row.
- [ ] `POT-00_lot1-coverage.md`: F-VIS-1 → Done.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/diorama/`
- `front/e2e/`
- `docs/POT-00_*`

## Questions

## Log
