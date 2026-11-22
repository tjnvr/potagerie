# T23 · Plan editor: resize handles easy to grab and precise

- Status: Done
- Owner: agent
- Depends on: T21
- Refs: S01 F-PLN-6 (plan editor), §5 (mouse and touch first) · D01 E03 · S05 §4 (drag in the webview) · `resize` in `drag.ts`
- From: Tim (chat)

## Context

Tim, 2026-11-21: the plan editor is often imprecise. When drawing or dragging, the bed shows up far from where the pointer is, and with a finger the bed does not really follow the touch. He asked for a rework of the editor's precision, split in as many tasks as possible (T23 to T25, with T08 for undo).

This task is about the 8 resize handles: they must be easy to grab with a mouse and with a finger (touch targets of 40 px at least, as done for the timeline in v0.4.10), and the edge must follow the pointer, cell by cell for the length and to the centimetre for the width (S01 F-PLN-6).

## Goal

Resizing a bed with its handles is precise with a mouse and with a finger: the edge stays under the pointer and the size shown is the size saved.

## Acceptance

- [x] Handles have a hit area of at least 40 px on touch screens, without hiding the bed on a phone.
- [x] Unit tests of `resize` for each handle (including the opposite edges and the minimum sizes); e2e tests with the mouse and with touch: the edge ends under the pointer, the panel shows the same size.
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

### Decisions
- Hit circles of 44 px on a touch screen (`any-pointer: coarse`, or once a finger touched the plan), 36 px with a mouse (as before): the 40 px of S01 §5 with a margin, the mouse unchanged.
- "Without hiding the bed": on a bed small on the screen, the circles are pushed outwards so they reach at most a quarter of the bed into it, and the middle handles of a side too short are left out (the corners resize both ways): the middle of the bed stays grabbable to move it.
- The gap between the press and the handle is kept for the whole gesture: with a 44 px target, jumping the edge to the pointer would be up to 22 px off; the edge now moves with the pointer, and ends under it when pressed on the dot.
- A finger on a handle drags at once, with no long press: S01 asks for the long press only to move; a handle is only on the selected bed, so it cannot be a scroll.
- A no-op `mousedown` listener on each hit circle: found by PTR-07, Chromium's touch adjustment gave the press to the bed under the handle because Svelte delegates its listeners to the document.
- S05 v0.23 (section 4 and new section 19) records these choices; S01 is unchanged (no behaviour of S01 changes).

### Progress
- 2026-11-22: unit tests of `resize` (8 handles, opposite edges, minimum sizes, rounding, kept gap), `handlePoint`, `handleZones`, the controller without long press; e2e PTR-05 to PTR-07 (mouse in 3 windows on Chromium and WebKit, finger on Chromium). Code passing; rebased on T22 (touch drag, merged the `onpointerdowncapture` of both). Extra path: `README.md` (S05 link).
- 2026-11-22: done, v0.12.0 (minor: a finger resizes without a long press, larger targets). Specs: S05 v0.23; S01 v1.10 and S03 v0.20 unchanged (links only). Checked: full local gate green (fmt, clippy, cargo test, check, 250 unit tests, build, e2e 377 passed / 13 skipped); screenshots of the editor at 1440 (English) and 360 px (English, French) with a bed selected. The first full e2e run had TCH-02 and TCH-03 (T22, tray and second finger, code not touched here) fail once under load; green alone and on the next full run: follow-up T26. NOT checked: a real touch device (iOS, Android, Windows touch screen), the real Tauri window, finger e2e on WebKit (no touch protocol), the installed build. `site/` not updated: the handles are not shown on the landing page.
