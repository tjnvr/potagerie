# T05 · Resize a bed from the keyboard

- Status: Done
- Owner: agent
- Depends on: —
- Refs: POT-00_review-2026-11 (seen, not fixed) · S01 §5
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: POT-00_review-2026-11 (seen, not fixed) · S01 §5.

## Goal

Decide whether the plan resize handles must be reachable from the keyboard, since the panel inputs already resize.

## Acceptance

- [x] No keyboard resizing on the plan: the 8 handles stay mouse and touch only (answer (b), S01 §5 "no action requires the keyboard", S01 F-PLN-6 "or by entering length and width in the panel").
- [x] From the keyboard, a bed is still resized through the length and width fields of its panel (already covered by UC-27).
- [x] The review doc no longer lists the handles under "Seen, not fixed".

## Scope

- `docs/POT-00_review-2026-11.md`

## Questions

1. [functional] Should the resize handles on the plan also work from the keyboard (Tab to a handle, arrow keys to resize)? Today the bed panel already lets you type a size, and S01 says the keyboard is never required.
   (a) Yes, add keyboard resizing on the plan. (b) No, the panel fields are enough.
   Recommendation: (b).
   Answer: keyboard feature not needed let's focus on the mouse or tactile

## Log

- Decisions
  - No spec change: S01 §5 and F-PLN-6 already say what the answer says (handles by mouse or touch, panel fields for typed sizes), so no spec version moves.
  - No version bump and no tag: only `docs/` changed, nothing that ships with the app (`spec-driven-delivery`, versioning).
  - Gate not run again after the rebase: `origin/main` brought only task and index docs, the code is identical to the one the gate ran on.
- 2026-11-21 · Done, no new version (app stays 0.7.1; no spec version).
  - Changed: `docs/POT-00_review-2026-11.md`, the plan resize handles move from "Seen, not fixed" to "Decided, no change (T05)".
  - Checked: full local gate green before the rebase (fmt, clippy, cargo test, front check, test 215, build, e2e 319 passed, 3 skipped).
  - NOT checked: nothing run in the app, as no behaviour changed.
