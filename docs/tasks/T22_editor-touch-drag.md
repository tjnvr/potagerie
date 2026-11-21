# T22 · Plan editor: dragging with a finger follows it

- Status: Ready
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
