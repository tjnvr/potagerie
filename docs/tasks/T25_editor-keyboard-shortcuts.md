# T25 · Plan editor: keyboard shortcuts

- Status: Running
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

- 2026-11-22: Started. Specs S01 v1.11 (F-PLN-6 keyboard shortcuts, undo / redo keys per platform) and S05 v0.24 (§4, new §20) committed.
- 2026-11-22: Tests written: unit `shortcuts.test.ts` (platform, every key, fields, tooltip keys); e2e `front/e2e/shortcuts.spec.ts` UC-37 (moved from `use-cases.spec.ts`, now per platform), UC-38 to UC-42. Code passing; full local gate green (fmt, clippy, cargo test, check, 257 unit tests, build, 397 e2e passed, 13 skipped). Extra paths outside the first scope: `front/src/lib/shell/Header.svelte` (tooltips), `site/index.html` (landing line, en / fr), `README.md`, the D01 / D02 READMEs and S03 (spec links).

### Decisions

- Undo / Redo keys narrowed to the platform (T08 accepted both): Cmd on macOS / iOS, Ctrl elsewhere, from `navigator.platform`; the other modifier does nothing. Redo: Shift+Cmd+Z on macOS, Ctrl+Y and Shift+Ctrl+Z elsewhere; the tooltip shows Ctrl+Y.
- One window listener in the editor runs every shortcut; the per-bed handler keeps only Enter / Space. Ignored in fields, while a window, a confirmation (`dialog`, `role=dialog`) or the aisle bubble is open, and when the key was already taken (`defaultPrevented`): Escape on the aisle bubble now takes the key, so it goes back without also deselecting.
- Escape cancels every gesture in progress: bed move, resize, zone drawn on the grass and drag from the tray; the bed stays selected. Not while Rust is already judging a drop.
- Arrow moves go through `deposer` and `move_bed` like a drop (refusal flashes pale red, before-plot refused without a call, aisle bubble); a key pressed while a move waits for Rust is ignored. Shift = 100 cm, always a whole number of cells (25, 50, 100 cm).
- Tab / Shift+Tab act only when the focus is on the plan or on nothing, in the order of the year view (the focus order); past the ends the key is the browser's, so the focus is never trapped (WCAG 2.1.2). With nothing selected, Tab takes the first bed, Shift+Tab the last.
- Delete / Backspace open the panel's own confirmation (`PanelBed.askDelete`); nothing in a past season.
- Tooltips: Undo, Redo (header), Delete and Deselect (panel, editor only: the garden's panel has no shortcuts). Format "{label} ({keys})"; macOS symbols ⌘Z, ⇧⌘Z, ⌫; elsewhere Ctrl+Z, Ctrl+Y, Delete / Suppr; Esc / Échap.
- No visible help text for the arrows and Tab beyond the landing page: Tim asked for tooltips only.
- e2e set `navigator.platform` in the page so both platforms are tested on any machine; UC-37 sets the name by script, because on a macOS host Playwright plays Cmd+Z as the browser's own undo of the last typing.
