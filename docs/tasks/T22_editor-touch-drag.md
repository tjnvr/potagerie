# T22 · Plan editor: dragging with a finger follows it

- Status: Done
- Owner: agent
- Depends on: T21
- Refs: S01 F-PLN-6 (plan editor), §5 (mouse and touch first) · D01 E03 · S05 §4 (drag in the webview)
- From: Tim (chat)

## Context

Tim, 2026-11-21: the plan editor is often imprecise. When drawing or dragging, the bed shows up far from where the pointer is, and with a finger the bed does not really follow the touch. He asked for a rework of the editor's precision, split in as many tasks as possible (T23 to T25, with T08 for undo).

This task is the touch part: on a touch screen the bed lags or does not follow the finger. Things to check: `touch-action` on the plan (the page may scroll or zoom instead), pointer capture, the long press before a move (S01 F-PLN-6) cancelling or delaying the drag, events throttled per frame, the finger hiding the bed (an offset is a functional choice: keep the bed under the finger unless the spec says otherwise).

## Goal

On a phone or a touch screen, after the long press, the bed follows the finger without lag or jump and lands where the finger is lifted; the page neither scrolls nor zooms meanwhile.

## Acceptance

- [x] e2e tests with touch emulation (`hasTouch`, touch pointer events): long press then drag a bed; drop an item from the tray; the bed follows each move and ends under the finger.
- [x] A drag that starts on the grass still pans or does what it does today; a short tap still selects.
- [x] Matrix rows, ids free on `origin/main`.
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/editor/`
- `front/src/lib/`
- `front/e2e/`
- `front/messages/`
- `docs/specs/`
- `docs/POT-00_*`

## Questions

## Log

- 2026-11-22 · Checked with real touches (Chromium `Input.dispatchTouchEvent`, 360 × 800 and 1440 × 900): after the long press the bed already follows each move and lands under the finger, the page does not scroll and no `pointercancel` comes. `touch-action: none` on the beds is ignored by Chromium (an SVG child): a swipe on a bed before the long press scrolls the page, which is what S01 and `ControllerDrag` want, and the `touchmove` stopped once armed is what keeps the page still. Two defects found: (1) a second finger put on the plan during a drag starts a new gesture and drops the one under way (the bed goes back, measured); (2) the context menu that a long press sends (Windows touch screens, Android) is not prevented, on the plan nor on the tray.
- 2026-11-22 · Tests written: `front/e2e/touch.spec.ts` TCH-01..05 (phone 360 × 800 and touch screen 1440 × 900, density 2). Before the fix: TCH-02 and TCH-03 fail (menu not prevented, second finger), TCH-01, TCH-04, TCH-05 pass.
- 2026-11-22 · Code passing: the plan and the tray keep the pointer that started the gesture and ignore the others (move, release, cancel, and a second press while a bed is held); a long press with a finger or a pen no longer opens the context menu on the plan or the tray (the mouse keeps its menu); the tray items get `-webkit-touch-callout: none` like the plan. Touch suite 35 passed, 9 skipped (WebKit moves). Full local gate green; EXP-02 (export form, WebKit fr) failed once under load, then passed alone 3/3 and in a second full run (364 passed, 12 skipped).
- 2026-11-22 · Done, v0.11.2 (patch: a fix). No spec changed (S01 v1.10 and S05 v0.22 already say the bed follows the finger after the long press and the page stays still; these were defects). The rebase on `origin/main` brought nothing. Checked: full local gate green (fmt, clippy, cargo test, check, unit tests, build, e2e 364 passed, 12 skipped), real touches in Chromium at 360 × 800 and 1440 × 900. NOT checked: touch on a real device (iOS, Android, Windows touch screen, UC-30), touch moves in WebKit (Playwright cannot drive them), pinch zoom, the real Tauri window and an installed build. Extra paths: none outside `Scope`.
- 2026-11-22 · CI green: `main` (run 45065686947), tag `v0.11.2` (45065694651), release (45065694612, pre-release with the dmg, exe and msi). Landing page unchanged: nothing new to show.

### Decisions

- Real touches through CDP, Chromium only, for the moves: synthetic pointer events would bypass `touch-action`, the scrolling and the pointer capture, which are what this task is about. WebKit only gets the tap (TCH-05), the only touch Playwright drives there.
- Test ids use the prefix `TCH-` (free on `origin/main`), as T21 did with `PTR-`.
