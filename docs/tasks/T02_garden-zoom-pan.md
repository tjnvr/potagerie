# T02 · Zoom and pan the garden view

- Status: Done
- Owner: agent
- Depends on: —
- Refs: S01 F-VIS-1 · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-VIS-1 · lot1-coverage.

## Goal

The diorama zooms with the wheel or trackpad and the pinch, and pans by dragging the background.

## Acceptance

- [x] Wheel / trackpad and pinch zoom the diorama, with sensible bounds; no zoom button (S01).
- [x] Dragging the background pans; dragging still works on beds and the time cursor as before.
- [x] Season playback keeps its smoothness (P01 criterion, measured as in the review).
- [x] `UC-xx` e2e test (wheel zoom and background drag) and matrix row.
- [x] `POT-00_lot1-coverage.md`: F-VIS-1 → Done.
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/diorama/`
- `front/e2e/`
- `docs/POT-00_*`

## Questions

## Log

- 2026-11-20 · Done, v0.5.0 (minor: new visible behaviour). No spec change: S01 F-VIS-1 already asks for wheel / pinch zoom and background pan without a button.
  - Changed: `front/src/lib/diorama/camera.ts` (zoom ×1 to ×4 around the pointer, pan kept inside the frame, wheel steps capped at ×2); `Diorama.svelte` only moves the SVG view box (wheel, Ctrl + wheel trackpad pinch, Safari `gesture*`, two-pointer pinch, one-pointer drag on the background). Zoomed all the way out, the wheel scrolls the page; at full view one finger still scrolls the page (`touch-action: pan-y`). A drag never opens a bed; the time cursor is outside the SVG and untouched.
  - Checked: full local gate green (fmt, clippy, cargo test, front check / test / build, e2e 310 passed, 2 skipped = WebKit frame timings). Unit `camera.test.ts`; e2e UC-28 (wheel zoom, background drag, wheel back to the page, bed click) in Chromium and WebKit; DIO-02 zoomed playback p95 8.9 ms vs DIO-01 8.7 ms (limit 50 ms). Matrix row UC-28, F-VIS-1 Done in `POT-00_lot1-coverage.md`.
  - NOT checked: real touch pinch and the Safari trackpad gesture (not automated), the real Tauri window, the installed build; no 1440 / 360 px screenshots (running a Node script was not allowed in this session). The e2e suite with the default worker count failed in bulk while other task worktrees ran theirs (shared port 5320, `reuseExistingServer`); every failing test passed alone, and the full run with `--workers=2` (as in CI) was green. Follow-up created: T16 (one e2e server per task worktree).
