# T27 · Plan editor: Delete in the bar under the selected bed

- Status: Ready
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
