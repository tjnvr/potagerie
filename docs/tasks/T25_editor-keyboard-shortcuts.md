# T25 · Plan editor: keyboard shortcuts

- Status: Ready
- Owner: agent
- Depends on: T08
- Refs: S01 F-PLN-6 (plan editor), §5 (mouse and touch first) · D01 E03 · S05 §4 (drag in the webview)
- From: Tim (chat)

## Context

Tim, 2026-11-21: wants keyboard actions in the plan editor (cancel, copy, paste, etc.). S01 §5 says no action may require the keyboard: these are shortcuts for actions that also exist with the mouse or a finger. Undo / Redo shortcuts are in T08, copy / paste / duplicate in T24; this task does the rest.

As decided with Tim: Escape cancels a drag or a resize in progress (the bed goes back) and otherwise deselects; Delete / Backspace deletes the selected bed (same confirmation as the button); the arrow keys move the selected bed by one cell, Shift + arrow by 1 m, with the same refusals as a drag (overlap, leaving the plot, aisle bubble of T01); Tab / Shift+Tab select the next / previous bed. Tooltips of the buttons show their shortcut. Nothing fires while typing in a field. Keys follow the platform (Cmd on macOS, Ctrl elsewhere).

## Goal

The plan editor can be driven from the keyboard for the common actions, without getting in the way of typing.

## Acceptance

- [ ] Each shortcut above, with e2e tests (macOS and other platforms' modifier), including: ignored in an input, arrow move refused on overlap, Escape during a drag restores the bed.
- [ ] Moves made with the arrows are undoable like a drag (T08).
- [ ] Tooltips show the shortcuts (en and fr).
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
