# T23 · Plan editor: resize handles easy to grab and precise

- Status: Running
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

- [ ] Handles have a hit area of at least 40 px on touch screens, without hiding the bed on a phone.
- [ ] Unit tests of `resize` for each handle (including the opposite edges and the minimum sizes); e2e tests with the mouse and with touch: the edge ends under the pointer, the panel shows the same size.
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
