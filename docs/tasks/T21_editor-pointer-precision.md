# T21 · Plan editor: the bed lands exactly under the pointer

- Status: Done
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

- 2026-11-22 · Cause found. The plan is `width: 100%; max-height: 70vh`: when the window is low, or the plot more square than the space, the SVG element is wider (or taller) than its drawing, and the SVG centres the drawing with empty bands (`preserveAspectRatio` `xMidYMid meet`, the default). `pixelsToCm` stretched the pointer over the whole element, so the pointer was off by up to 137 cm (3 cells) near the edges of the plot, growing away from the centre (measured: 1440 × 640, element 972 px wide, drawing 738.5 px). The handles were sized with the same wrong scale. The other suspects were checked and are not the cause: the grab offset (`shift`) is kept during a move; `devicePixelRatio`, scroll and page zoom do not matter (`clientX` and `getBoundingClientRect` are both CSS pixels of the viewport); the preview and the drop use the same geometry, so there is no jump on release.
- 2026-11-22 · Tests written: `drag.test.ts` (letterbox wide and tall, `cmPerPixel`), failing before the fix (137 cm off). e2e `front/e2e/pointer.spec.ts` PTR-01..04 at 1440 × 900, 1440 × 640 and 360 × 800, density 2, Chromium and WebKit: 8 failed before the fix (all at 1440 × 640), 24 passed after.
- 2026-11-22 · Code passing: `pixelsToCm` maps to the centred drawing with one scale for both axes (`cmPerPixel`), the handles use that scale, `versCm` returns `null` outside the drawing (the empty bands are no longer "on the plan").
- 2026-11-22 · Done, v0.11.1 (patch: a fix). No spec changed (S01 v1.9, S05 v0.21 already say the bed follows the pointer; this was a defect). Rebased on T08 (v0.11.0), matrix conflict resolved (UC-36, UC-37 kept, PTR rows added). Checked: full local gate green after the rebase (fmt, clippy, cargo test, check, 226 unit tests, build, e2e 353 passed, 3 skipped as before). NOT checked: touch (T22), the real Tauri window and an installed build, browser zoom other than 100 %, screenshots of the functional check (the e2e tests read the positions in the browser instead). Extra paths: none outside `Scope`.
- 2026-11-22 · CI green: `main` (run 45020582389, the first run was cancelled by the next push), tag `v0.11.1` (45020474313), release (45020474378, pre-release with the dmg, exe and msi). Landing page unchanged: nothing new to show.

### Decisions

- The fix is in the conversion (`pixelsToCm`), not in the CSS: it holds for any element size, and it keeps the layout as it is.
- An item of the tray stays held by its centre: the tray button is not the bed at scale, so its centre is the only point that matches; PTR-01 asserts it lands centred on the cells under the pointer.
- New test ids use the prefix `PTR-` (free on `origin/main`) rather than `UC-38+`, which the other editor tasks running at the same time may take.
- Touch (T22) and the handles' hit areas (T23) are left to their tasks; the e2e tests here use the mouse.
