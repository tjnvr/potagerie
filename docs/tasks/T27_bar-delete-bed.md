# T27 · Plan editor: Delete in the bar under the selected bed

- Status: Running
- Owner: agent
- Depends on: T24
- Refs: S01 F-PLN-6 (rotate, duplicate, delete "from the bar that appears under the selected bed, or from the panel") · D01 E03 · S05 §20
- From: T24 (agent)

## Context

T24 added the dark bar under the selected bed in the plan editor (D01 E03), with only "Duplicate" in it (`front/src/lib/editor/PlanViewTop.svelte`, `.bar`). S01 F-PLN-6 also puts "Delete" in that bar; today Delete is only in the bed panel (`PanelBed.svelte`, with its confirmation listing the plantings that go with the bed). Rotate stays out: it is after lot 1 (T07, Draft).

## Goal

The selected bed can be deleted from the bar under it, with the same confirmation as the panel's button.

## Acceptance

- [ ] "Delete" in the bar (D01: trash icon, after Duplicate), opening the same confirmation as the panel, in en and fr.
- [ ] e2e: delete from the bar, cancel keeps the bed, confirm removes it and its plantings; Undo brings it back.
- [ ] Matrix row, id free on `origin/main`.
- [ ] Full local gate green (`skills/spec-driven-delivery`, step 5).

## Scope

- `front/src/lib/editor/`
- `front/e2e/`
- `front/messages/`
- `docs/POT-00_*`

## Questions

## Log

- 2026-11-23: S05 v0.26 (§20, the bar holds Duplicate then Delete); e2e UC-47 written (en, fr), failing; then Delete in the bar, passing. Extra paths: `docs/specs/` (S05 v0.26, links in S01 v1.12, S03 v0.21), `README.md` (link to S05).
- 2026-11-23: local gate: fmt, clippy, cargo test, check, unit (261), build green; UC-47 and the delete / duplicate e2e passed in Chromium and WebKit. The full e2e could not finish locally: after the WebKit runs that hung (bound bar width, since removed), the machine stayed loaded (every page load timed out, even UC-01 on a fresh server) and the leftover processes could not be stopped from this session. Pushed `task/T27-bar-delete-bed` to run the full gate in CI instead.

### Decisions

- The bar's Delete calls the panel's `askDelete`: one confirmation, one code path, as the Delete key (T25) already does.
- No new message: the bar reuses `delete` ("Delete" / "Supprimer") with its key in the tooltip, like the panel's button.
- The bar is kept inside the plan with a fixed half-width (115 px, the bar in French) instead of a measured one: binding its width made WebKit loop (all editor e2e timed out). `width: max-content` keeps it on one line near the right edge.
- S01 and S03 keep their versions: only their link to S05 changes, not their content.
- UC-21, UC-25 and UC-36 now click the panel's Delete explicitly (`aside`): with two Delete buttons on screen, `.first()` would have picked the bar's.
