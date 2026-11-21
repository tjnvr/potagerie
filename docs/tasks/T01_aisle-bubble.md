# T01 · Aisle bubble "Keep here" / "Go back"

- Status: Done
- Owner: agent
- Depends on: —
- Refs: S01 F-PLN-2 · S03 (plot and placement of the beds) · lot1-coverage
- From: chat

## Context

Created on 2026-11-20 when the task system was set up, from what was still open: S01 F-PLN-2 · S03 (plot and placement of the beds) · lot1-coverage.

## Goal

Dropping a bed closer than the minimum aisle offers "Keep here" or "Go back" instead of refusing it with an error.

## Acceptance

- [x] Dropping or resizing a bed closer than the aisle width to another bed shows the bubble; overlap and leaving the plot are still refused.
- [x] "Keep here" keeps the position; "Go back" restores the previous one.
- [x] If S03 forbids a closer bed today, a new S03 version allows it (spec first) with a §12 case.
- [x] New row and `UC-xx` e2e test in `POT-00_test-matrix.md`; messages in `en.json` and `fr.json`.
- [x] `POT-00_lot1-coverage.md`: F-PLN-2 → Done.
- [x] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `crates/plan/`
- `src-tauri/src/`
- `front/src/lib/editor/`
- `front/messages/`
- `front/e2e/`
- `docs/specs/`
- `docs/POT-00_*`

## Questions

1. **Paths outside `Scope`, and a few spec choices.** S03 v0.17 §15 refuses a narrow aisle (`aisle_too_narrow`) and puts the bubble out of lot 1, so S03 needs a new version. Any design for the bubble also has to touch files that `Scope` leaves out:
   - `front/src/lib/bindings.ts`: regenerated, because `create_bed`, `move_bed` and `modify_bed` get a new argument (spec-driven-delivery step 4).
   - `front/src/lib/simulation.ts`: the fake of the commands that the e2e tests run against. It refuses `aisle_too_narrow` today (line 192), and its `?example` plan lists an `aisle_too_narrow` incoherence (line 264).
   - `front/src/lib/diorama/PanelDiorama.svelte`: the diorama view also drags beds through `moveBed` and `modifyBed`, so it needs the same bubble or it keeps the old refusal.
   - Maybe `front/src/lib/state.svelte.ts` (`app.modify` shows the error alert) and `front/src/lib/settings/incoherences.ts`.

   Proposed design, to write in S03 v0.18 §15, §13, §14 and §12:
   - `create_bed`, `move_bed` and `modify_bed` take `keep_close: bool`. If it is false, and the changed bed would end up closer than the aisle to another bed, the command refuses with `aisle_too_narrow` as today, and the front shows the bubble. "Keep here" sends the same call again with `true`. "Go back" sends nothing, so the bed stays where it was and the drag preview snaps back. Overlap, leaving the plot and a width that is too small are still refused, whatever the flag.
   - Only the pairs that include the changed bed are checked against the aisle. A pair that was kept close earlier never blocks a later action.
   - A narrow aisle is no longer an incoherence: it leaves `placement_errors`, the "To resolve" list and the import check. The plan stores nothing about the choice (S05 §5), so a pair kept on purpose would otherwise stay "to resolve" forever.
   - `set_plot` (`modify_plot`) refuses only when a pair that kept the old aisle would no longer keep the new one. Pairs that were already closer do not count, otherwise a kept pair would block every later change of the plot. SET-01 still passes, with "80 cm refused".
   - A new §12 case: two beds 30 cm apart with a 50 cm aisle are refused without the flag and accepted with it; overlapping beds are refused even with the flag.

   Options:
   - (a) Widen `Scope` to the files above and accept the design. **Recommended.**
   - (b) The same, but drop the flag: Rust accepts narrow aisles and the front decides when to show the bubble. This duplicates the rule in the front, against "Rust owns the rules".
   - (c) Keep the diorama out (it keeps the error message there) and log a follow-up task for it.
   Answer: This is a technical choice: under the new rule (skill run-task, "Decide, do not ask"), it is yours. Go with (a), as recommended; the bubble itself is what S01 F-PLN-2 describes.

## Log

- 2026-11-20: Blocked on question 1. S03 v0.17 refuses the narrow aisle, and the bubble needs `bindings.ts`, `simulation.ts` (the e2e fake) and the diorama drag, which are outside `Scope`. No code or spec changed yet.
- 2026-11-21: Resumed with option (a).

### Decisions

- `create_bed`, `move_bed` and `modify_bed` take `keep_close`; Rust refuses with `aisle_too_narrow` without it and the front shows the bubble, so the rule stays in Rust (S05 v0.18 §7).
- Only the pairs a change makes too close are asked about (`Plan::check_placement_change` compares with the plan before the change): a pair kept earlier never asks again.
- A narrow aisle is no longer a placement error nor a "to resolve" item, since the plan stores nothing about the choice (S03 v0.18 §13, §15).
- `set_plot` refuses only for pairs that kept the old aisle and would not keep the new one; no bubble for the plot, the refusal message is kept (SET-01 unchanged).
- "Go back" sends nothing; the bed snaps back like a cancelled drag. Escape answers "Go back"; focus goes to "Keep here". One bubble for the plan editor and the diorama, in `App.svelte`, driven by `app.place`.
- A change of type or name (`modifyBed` from the bed panel) sends `keep_close: false` and never asks: it does not move the bed.
- The `?example` fake now lists a `bed_overlaps` incoherence instead of `aisle_too_narrow`.
- The landing page said "aisles are kept for you": reworded in English and French to say the app asks.

- 2026-11-21: Done. v0.8.0. Specs: S03 v0.18 (§12 T14, §13, §15), S05 v0.18 (§7). Rust: `placement.rs` (`check_placement_change`), commands with `keep_close`, `tests.rs` `a_narrow_aisle_is_kept_only_when_asked`. Front: `AisleBubble.svelte`, `app.place`, editor and diorama drags, simulation, messages EN/FR, e2e UC-33. Docs: test matrix (UC-33, F-PLN-2), lot 1 coverage (F-PLN-2 Done, 22 / 7), `site/index.html`. Paths outside `Scope`: `front/src/App.svelte`, `front/src/lib/bindings.ts`, `front/src/lib/simulation.ts`, `front/src/lib/state.svelte.ts`, `front/src/lib/diorama/PanelDiorama.svelte`, `front/src/lib/settings/incoherences.ts`, `site/index.html`, version files. Checked: full local gate green (fmt, clippy, cargo test, check, 215 unit, build, 321 e2e) before and after the rebase; screenshots of the bubble at 1440 and 360 px, English and French, in headless WebKit. NOT checked: the real Tauri window, touch (long press then the bubble), the installed build, the diorama drag by hand (same `app.place` path, no dedicated e2e).
