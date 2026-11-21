# T21 · Plan editor: the bed lands exactly under the pointer

- Status: Ready
- Owner: agent
- Depends on: —
- Refs: S01 F-PLN-6 (plan editor), §5 (mouse and touch first) · D01 E03 · S05 §4 (drag in the webview) · `front/src/lib/editor/drag.ts`
- From: Tim (chat)

## Context

Tim, 2026-11-21: the plan editor is often imprecise. When drawing or dragging, the bed shows up far from where the pointer is, and with a finger the bed does not really follow the touch. He asked for a rework of the editor's precision, split in as many tasks as possible (T23 to T25, with T08 for undo).

This task is the root of it: the conversion from the pointer to plot centimetres. Likely suspects to check, not assumptions: the offset between the pointer and the grabbed point of the bed (`shift` in `drag.ts`), an item dropped from the tray being centred instead of held where it was grabbed, the SVG scale and its viewBox when the window is resized or the editor scrolled, `devicePixelRatio`, rounding to cells done on the wrong corner.

## Goal

Whatever the window size, zoom, scroll and screen density, a bed dragged or dropped from the tray sits under the pointer at the place it was grabbed, at every moment of the drag, and lands on the cells shown during the drag.

## Acceptance

- [ ] The cause is found and written in `Log`, with a unit test in `drag.test.ts` that fails before the fix.
- [ ] e2e tests (Chromium and WebKit) at 1440 and 360 px wide and with a device scale factor of 2: drop an item from the tray on a given cell, drag a bed by a corner and by its middle; the bed ends where the pointer was, to the cell.
- [ ] The preview during the drag and the final position are the same cells (no jump on release).
- [ ] Matrix rows (`POT-00_test-matrix.md`), ids free on `origin/main`.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/editor/`
- `front/src/lib/`
- `front/e2e/`
- `front/messages/`
- `docs/specs/`
- `docs/POT-00_*`

## Questions

## Log
