# T06 · Flag two beds with the same name

- Status: Running
- Owner: agent
- Depends on: —
- Refs: S01 F-PLN-1 (after lot 1)
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-PLN-1 (after lot 1).

## Goal

Two beds with the same name are allowed but flagged.

## Acceptance

- [ ] Two beds of the same season with the same name (case aside) are accepted, never refused.
- [ ] The panel of each of them says that another bed of the season has the same name, in English and French; renamed apart, the warning goes.
- [ ] Spec first: S01 moves the warning into lot 1, S03 states the rule with a §12 case, S05 the field of the year view.
- [ ] New row and `UC-xx` e2e test in `POT-00_test-matrix.md`; `POT-00_lot1-coverage.md`: F-PLN-1 → Done.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `docs/specs/` (S01, S03, S05), `docs/POT-00_*`
- `crates/plan/` (`layouts.rs`, `view.rs`, `tests/s03_cas.rs`)
- `src-tauri/src/tests.rs`
- `front/src/lib/` (`bindings.ts`, `simulation.ts`, `editor/PanelBed.svelte`, test fixtures), `front/messages/`, `front/e2e/`

## Questions

1. [functional] Build it now? S01 lists the warning for two beds with the same name for after lot 1.
   (a) Yes, now. (b) Later.
   Recommendation: (a), it is small.
   Answer: a

## Log

- 2026-11-21: Started (answer a). Specs S01 v1.9, S03 v0.20 (§15, case T20), S05 v0.20 committed.
- 2026-11-21: Tests written and passing: Rust `t20_two_beds_with_the_same_name_are_flagged`, `two_beds_with_the_same_name_are_flagged_not_refused`; e2e UC-35 (Chromium, WebKit). Extra paths outside the first scope: `README.md` and the D01 / D02 READMEs (spec links), the timeline test fixtures.

### Decisions

- The rule lives in Rust: `Plan::name_shared` and a `name_shared` flag on each bed of the year view; the front only shows it. The e2e fake (`simulation.ts`) mirrors it.
- Names are compared trimmed and without regard to case: "Tomatoes" and "tomatoes" read as the same bed, which is what the warning is for.
- Only the beds that exist in the viewed year count, as the view only shows those; a bed deleted earlier or created later does not flag.
- The warning is a line in the bed panel (editor and garden, past seasons included), for each of the two beds; no marker on the plan labels or the timeline, to keep the change small. S01 F-PLN-1 now says where it is shown.
