# T10 · A3 paper size in the timeline export

- Status: Draft
- Owner: agent
- Depends on: —
- Refs: S01 F-EXP-1 (after lot 1)
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-EXP-1 (after lot 1).

## Goal

The PDF export offers A3 landscape next to A4.

## Acceptance

- [ ] A3 choice in the dialog; pagination tested in `crates/export/tests/`.
- [ ] `UC-10` extended.

## Scope

- `crates/export/`
- `src-tauri/src/`
- `front/src/lib/settings/`
- `front/messages/`
- `front/e2e/`
- `docs/POT-00_*`

## Questions

1. [functional] Offer A3 paper in the timeline export now? S01 lists it for after lot 1.
   (a) Yes, now. (b) Later.
   Recommendation: (b), unless you print in A3.
   Answer: b

## Log

- 2026-11-21 (chat): back to Draft: the answer is "b" (later), it had been set Ready by mistake.
