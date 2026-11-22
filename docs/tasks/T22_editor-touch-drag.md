# T22 · Plan editor: dragging with a finger follows it

- Status: Running
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

- [ ] e2e tests with touch emulation (`hasTouch`, touch pointer events): long press then drag a bed; drop an item from the tray; the bed follows each move and ends under the finger.
- [ ] A drag that starts on the grass still pans or does what it does today; a short tap still selects.
- [ ] Matrix rows, ids free on `origin/main`.
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

- 2026-11-22 · Checked with real touches (Chromium `Input.dispatchTouchEvent`, 360 × 800 and 1440 × 900): after the long press the bed already follows each move and lands under the finger, the page does not scroll and no `pointercancel` comes. `touch-action: none` on the beds is ignored by Chromium (an SVG child): a swipe on a bed before the long press scrolls the page, which is what S01 and `ControllerDrag` want, and the `touchmove` stopped once armed is what keeps the page still. Two defects found: (1) a second finger put on the plan during a drag starts a new gesture and drops the one under way (the bed goes back, measured); (2) the context menu that a long press sends (Windows touch screens, Android) is not prevented, on the plan nor on the tray.
- 2026-11-22 · Tests written: `front/e2e/touch.spec.ts` TCH-01..05 (phone 360 × 800 and touch screen 1440 × 900, density 2). Before the fix: TCH-02 and TCH-03 fail (menu not prevented, second finger), TCH-01, TCH-04, TCH-05 pass.

### Decisions

- Real touches through CDP, Chromium only, for the moves: synthetic pointer events would bypass `touch-action`, the scrolling and the pointer capture, which are what this task is about. WebKit only gets the tap (TCH-05), the only touch Playwright drives there.
- Test ids use the prefix `TCH-` (free on `origin/main`), as T21 did with `PTR-`.
